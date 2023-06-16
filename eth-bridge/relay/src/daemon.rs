use crate::{
	bail, cmdline, ensure, eyre, liberland, relay, settings, tx_managers,
	types::{Amount, CallId},
	utils::wait_for,
	watcher, Result,
};
use ethers::{
	prelude::{LocalWallet, Middleware, Provider, SignerMiddleware, Ws},
	signers::Signer,
	types::{Eip1559TransactionRequest, SyncingStatus},
	utils::{parse_units, ParseUnits},
};
use eyre::WrapErr;
use fs4::FileExt;
use futures::{stream::FuturesUnordered, StreamExt};
use sp_core::{sr25519::Pair as SubstratePair, Pair};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::{collections::HashMap, sync::Arc};
use subxt::{
	config::{substrate::BlakeTwo256, Hasher},
	tx::PairSigner,
	OnlineClient, SubstrateConfig,
};
use tokio::{signal, sync::mpsc, task::JoinHandle};

fn get_id(
	prefix: &[u8],
	keys: &settings::Keys,
	lld_contract: &settings::EthAddress,
	llm_contract: &settings::EthAddress,
) -> String {
	let h1 = BlakeTwo256::hash(keys.ethereum_private_key.as_bytes());
	let h2 = BlakeTwo256::hash(keys.substrate_secret_seed.as_bytes());
	let h = BlakeTwo256::hash(
		&[prefix, h1.as_bytes(), h2.as_bytes(), lld_contract.as_bytes(), llm_contract.as_bytes()]
			.concat(),
	);
	let bytes = &h.as_fixed_bytes()[..10];
	hex::encode(bytes)
}

fn get_substrate_signer(seed: &str) -> Result<PairSigner<SubstrateConfig, SubstratePair>> {
	let (pair, _) = SubstratePair::from_string_with_seed(seed, None)?;
	Ok(PairSigner::new(pair))
}

async fn setup_db(
	db: &SqlitePool,
	sub_api: &OnlineClient<SubstrateConfig>,
	eth_provider: &Provider<Ws>,
) -> Result<()> {
	sqlx::migrate!().run(db).await?;

	let mut conn = db.acquire().await?;

	let sub_genesis = hex::encode(sub_api.genesis_hash().as_bytes());
	let eth_block = eth_provider.get_block(0).await?.ok_or(eyre!("No eth genesis block?!"))?;
	let eth_hash = eth_block.hash.ok_or(eyre!("Genesis block with no hash?!"))?;
	let eth_genesis = hex::encode(eth_hash.as_bytes());

	let stored_sub_genesis =
		sqlx::query!(r#"select genesis from networks where network = 'substrate';"#)
			.fetch_optional(&mut conn)
			.await?;

	if let Some(stored) = stored_sub_genesis {
		ensure!(stored.genesis == sub_genesis,
			"Substrate node genesis {sub_genesis} doesn't match genesis stored in database {}! Are you running on a new network?",
			stored.genesis);
	} else {
		sqlx::query!(
			"insert into networks (network, genesis) values ('substrate', ?1);",
			sub_genesis,
		)
		.execute(&mut conn)
		.await?;
	}

	let stored_eth_genesis =
		sqlx::query!(r#"select genesis from networks where network = 'ethereum';"#)
			.fetch_optional(&mut conn)
			.await?;

	if let Some(stored) = stored_eth_genesis {
		ensure!(stored.genesis == eth_genesis,
			"Ethereum node genesis {eth_genesis} doesn't match genesis stored in database {}! Are you running on a new network?",
			stored.genesis);
	} else {
		sqlx::query!(
			"insert into networks (network, genesis) values ('ethereum', ?1);",
			eth_genesis,
		)
		.execute(&mut conn)
		.await?;
	}

	Ok(())
}

async fn get_sub_api(
	url: &url::Url,
) -> Result<(OnlineClient<SubstrateConfig>, JoinHandle<Result<()>>)> {
	let client = OnlineClient::<SubstrateConfig>::from_url(url).await?;
	ensure!(!client.rpc().system_health().await?.is_syncing, "Substrate node is still syncing...");
	liberland::validate_codegen(&client)?;
	let update_client = client.updater();
	let handle = tokio::spawn(async move {
		update_client.perform_runtime_updates().await?;
		Ok::<(), eyre::Report>(())
	});
	Ok((client, handle))
}

async fn init_eth_tx_manager(
	db: Arc<SqlitePool>,
	eth_provider: Arc<Provider<Ws>>,
	eth_wallet: LocalWallet,
	max_gas_price: Amount,
) -> Result<(JoinHandle<Result<()>>, mpsc::Sender<(CallId, Eip1559TransactionRequest)>)> {
	let eth_signer = Arc::new(SignerMiddleware::new(eth_provider, eth_wallet));
	let tx_manager = tx_managers::Ethereum::new(db.clone(), eth_signer, max_gas_price).await?;
	let sender = tx_manager.external_new_call_send.clone();
	let handle = tokio::spawn(async move { tx_manager.process().await });
	Ok((handle, sender))
}

async fn init_sub_tx_manager(
	sub_api: Arc<OnlineClient<SubstrateConfig>>,
	sub_signer: PairSigner<SubstrateConfig, SubstratePair>,
) -> Result<(JoinHandle<Result<()>>, Arc<tx_managers::Substrate>)> {
	let arc_tx_manager = Arc::new(tx_managers::Substrate::new(sub_api, sub_signer).await?);
	let arc_tx_manager_2 = arc_tx_manager.clone();
	let handle = tokio::spawn(async move { arc_tx_manager_2.process().await });
	Ok((handle, Arc::clone(&arc_tx_manager)))
}

pub async fn run(cli: cmdline::Args) -> Result<()> {
	let config = settings::parse(&cli.config)?;
	let mut tasks = FuturesUnordered::new();
	let mut eth_tx_managers = HashMap::new();
	let mut sub_tx_managers = HashMap::new();

	let db_file = std::fs::OpenOptions::new()
		.write(true)
		.create(true)
		.open(config.db_path.as_path())?;
	db_file.try_lock_exclusive().wrap_err_with(|| {
		"Failed to acquire lock on database file - another instance of relay already running?"
	})?;

	let opts = SqliteConnectOptions::new()
		.filename(config.db_path.as_path())
		.create_if_missing(true);

	let db = Arc::new(SqlitePool::connect_with(opts).await?);

	let (sub_api, sub_api_runtime_upgrades_task_join_handle) =
		wait_for("Connection to Liberland node failed, trying again.", || {
			get_sub_api(&config.substrate_ws_url)
		})
		.await;
	let sub_api = Arc::new(sub_api);
	tasks.push(sub_api_runtime_upgrades_task_join_handle);

	let eth_provider = Arc::new(
		wait_for("Connection to Ethereum node failed, trying again.", || async {
			let provider = Provider::<Ws>::connect(&config.ethereum_ws_url).await?;
			ensure!(
				provider.syncing().await? == SyncingStatus::IsFalse,
				"Ethereum node is still syncing..."
			);
			Ok(provider)
		})
		.await,
	);

	setup_db(&db, &sub_api, &eth_provider).await?;

	let lld_bridge = config.lld_bridge_contract;
	let llm_bridge = config.llm_bridge_contract;
	let max_gas_price = match parse_units(config.max_gas_price_in_gwei, "gwei")? {
		ParseUnits::U256(x) => x,
		_ => bail!("Negative max_gas_price_in_gwei!"),
	};
	let eth_chain_id = eth_provider.get_chainid().await?.as_u64();

	for keys in config.relays.iter().map(|x| &x.keys).chain(config.watchers.iter()) {
		let privkey = &keys.ethereum_private_key;
		if !eth_tx_managers.contains_key(privkey) {
			let eth_wallet: LocalWallet = privkey.parse()?;
			let (tx_manager_handle, tx_manager_channel) = init_eth_tx_manager(
				db.clone(),
				eth_provider.clone(),
				eth_wallet.with_chain_id(eth_chain_id),
				max_gas_price,
			)
			.await?;
			tasks.push(tx_manager_handle);
			eth_tx_managers.insert(privkey, tx_manager_channel);
		}
	}

	for keys in config.relays.iter().map(|x| &x.keys).chain(config.watchers.iter()) {
		let sub_api = sub_api.clone();
		let sub_signer = get_substrate_signer(&keys.substrate_secret_seed)?;

		let substrate_secret_seed = &keys.substrate_secret_seed;
		if !sub_tx_managers.contains_key(substrate_secret_seed) {
			let (tx_manager_handle, tx_manager_channel) =
				init_sub_tx_manager(sub_api, sub_signer).await?;
			tasks.push(tx_manager_handle);
			sub_tx_managers.insert(substrate_secret_seed, tx_manager_channel);
		}
	}

	for watcher_keys in config.watchers.iter() {
		let eth_wallet = watcher_keys.ethereum_private_key.parse()?;
		let sub_signer = get_substrate_signer(&watcher_keys.substrate_secret_seed)?;
		let db = db.clone();
		let sub_api = sub_api.clone();
		let eth_provider = eth_provider.clone();
		let id = get_id(b"watcher::sub_to_eth", &watcher_keys, &lld_bridge, &llm_bridge);
		let sub_tx_manager = sub_tx_managers
			.get(&watcher_keys.substrate_secret_seed)
			.expect("Missing sub tx manager");

		let watcher = watcher::SubstrateToEthereum::new(
			id.into(),
			db,
			eth_provider,
			eth_wallet,
			sub_signer,
			sub_api,
			llm_bridge,
			lld_bridge,
			sub_tx_manager,
		)
		.await?;
		tasks.push(tokio::spawn(async move { watcher.run().await }));
	}

	for relay_config in config.relays.iter() {
		let eth_wallet = relay_config.keys.ethereum_private_key.parse()?;
		let db = db.clone();
		let sub_api = sub_api.clone();
		let eth_provider = eth_provider.clone();
		let id = get_id(b"relay::ethereum", &relay_config.keys, &lld_bridge, &llm_bridge);

		let tx_manager_channel = eth_tx_managers
			.get(&relay_config.keys.ethereum_private_key)
			.expect("Missing eth tx manager");

		let relay = relay::SubstrateToEthereum::new(
			id.into(),
			db,
			eth_provider,
			eth_wallet,
			sub_api,
			llm_bridge,
			lld_bridge,
			&relay_config.minimum_balance,
			&relay_config.claim_rewards_threshold,
			&relay_config.withdraw_rewards_threshold,
			relay_config.rewards_address,
			tx_manager_channel.clone(),
		)
		.await?;
		tasks.push(tokio::spawn(async move { relay.run().await }));
	}

	for relay_config in config.relays.iter() {
		let sub_signer = get_substrate_signer(&relay_config.keys.substrate_secret_seed)?;
		let db = db.clone();
		let sub_api = sub_api.clone();
		let eth_provider = eth_provider.clone();
		let id = get_id(b"relay::substrate", &relay_config.keys, &lld_bridge, &llm_bridge);
		let sub_tx_manager = sub_tx_managers
			.get(&relay_config.keys.substrate_secret_seed)
			.expect("Missing sub tx manager");

		let relay = relay::EthereumToSubstrate::new(
			id.into(),
			db,
			eth_provider,
			sub_signer,
			sub_api,
			llm_bridge,
			lld_bridge,
			sub_tx_manager,
		)
		.await?;
		tasks.push(tokio::spawn(async move { relay.run().await }));
	}

	for watcher_keys in config.watchers.iter() {
		let eth_wallet = watcher_keys.ethereum_private_key.parse()?;
		let sub_signer = get_substrate_signer(&watcher_keys.substrate_secret_seed)?;
		let db = db.clone();
		let sub_api = sub_api.clone();
		let eth_provider = eth_provider.clone();
		let id = get_id(b"watcher::ethereum_to_substrate", &watcher_keys, &lld_bridge, &llm_bridge);
		let tx_manager_channel = eth_tx_managers
			.get(&watcher_keys.ethereum_private_key)
			.expect("Missing eth tx manager");
		let sub_tx_manager = sub_tx_managers
			.get(&watcher_keys.substrate_secret_seed)
			.expect("Missing sub tx manager");

		let watcher = watcher::EthereumToSubstrate::new(
			id.into(),
			db,
			eth_provider,
			eth_wallet,
			sub_signer,
			sub_api,
			llm_bridge,
			lld_bridge,
			tx_manager_channel.clone(),
			sub_tx_manager,
		)
		.await?;
		tasks.push(tokio::spawn(async move { watcher.run().await }));
	}

	tokio::select! {
		_ = signal::ctrl_c() => Ok::<(), eyre::Report>(()),
		r = tasks.next() => {
			r.expect("No tasks?!").expect("Failure while joining handle")?;
			Err(eyre::eyre!("Task finished early!"))
		},
	}?;

	Ok(())
}
