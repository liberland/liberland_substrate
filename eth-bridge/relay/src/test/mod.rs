#![cfg(test)]

mod substrate;

use crate::{
	bridge_abi::BridgeABI,
	liberland,
	settings::{Keys, Relay, Settings},
};
use assert_cmd::cargo::cargo_bin;
use ethers::{
	contract::abigen,
	prelude::{LocalWallet, Provider, SignerMiddleware, Ws},
	signers::Signer as EthSigner,
	types::H160,
	utils::{keccak256, Anvil, AnvilInstance},
};
use liberland::runtime_types::pallet_federated_bridge::{BridgeState, IncomingReceipt};
use parity_scale_codec::Encode;
use sp_core::{sr25519::Pair as SubstratePair, Pair};
use std::{fs::File, io::Write, process, sync::Arc};
use subxt::{tx::PairSigner, OnlineClient, SubstrateConfig};
use tempfile::tempdir;
use tokio::{
	io::{AsyncBufRead, AsyncBufReadExt, BufReader, Lines},
	process::Command,
	time::{sleep, timeout, Duration},
};

abigen!(WrappedToken, "contracts/WrappedToken.json");
abigen!(ERC1967Proxy, "contracts/ERC1967Proxy.json");

#[derive(Default)]
struct BridgeInstance {
	pub llm_contract: Option<BridgeABI<SignerMiddleware<Provider<Ws>, LocalWallet>>>,
	pub lld_contract: Option<BridgeABI<SignerMiddleware<Provider<Ws>, LocalWallet>>>,
	pub substrate_ws_url: Option<url::Url>,
	pub anvil: Option<AnvilInstance>,
	pub sub_client: Option<OnlineClient<SubstrateConfig>>,
	pub sub_signer: Option<PairSigner<SubstrateConfig, SubstratePair>>,
	pub relay_cmd: Option<substrate::KillChildOnDrop>,
	substrate_cmd: Option<substrate::KillChildOnDrop>,
	relay_tmpdir: Option<tempfile::TempDir>,
}

fn start_relay(settings: &mut Settings, bridge_instance: &mut BridgeInstance) {
	let tmpdir = tempdir().expect("Cant create tempdir");
	let config = tmpdir.path().join("config.toml");
	settings.db_path = tmpdir.path().join("sqlite.db").into();

	let mut file = File::create(&config).expect("Cant create config file");
	let conf = toml::to_string_pretty(settings).expect("Invalid settings");
	file.write_all(conf.as_bytes()).expect("Cant write config file");
	drop(file);

	let cmd = substrate::KillChildOnDrop(
		Command::new(cargo_bin("relay"))
			.stdout(process::Stdio::piped())
			.stderr(process::Stdio::piped())
			.arg("--config")
			.arg(config)
			.arg("run")
			.arg("-v")
			.spawn()
			.unwrap(),
	);

	bridge_instance.relay_cmd = Some(cmd);
	bridge_instance.relay_tmpdir = Some(tmpdir);
}

async fn start_ethereum(bridge_instance: &mut BridgeInstance) {
	let anvil = Anvil::new().spawn();
	let wallet: LocalWallet = anvil.keys()[0].clone().into();
	let provider = Provider::<Ws>::connect(anvil.ws_endpoint()).await.unwrap();
	let client = Arc::new(SignerMiddleware::new(provider, wallet.with_chain_id(anvil.chain_id())));

	let token = WrappedToken::deploy(client.clone(), ("Test token".to_string(), "TST".to_string()))
		.unwrap()
		.send()
		.await
		.unwrap();

	let bridge_contract = BridgeABI::deploy(client.clone(), ()).unwrap().send().await.unwrap();

	let llm_proxy_contract = ERC1967Proxy::deploy(
		client.clone(),
		(
			bridge_contract.address(),
			bridge_contract
				.initialize(
					token.address(),
					1u32,              // votes required
					0.into(),          // mint delay
					0.into(),          // fee
					10000000.into(),   // counter limit
					10000000.into(),   // decay rate
					1000000000.into(), // supply limit
				)
				.calldata()
				.unwrap(),
		),
	)
	.unwrap()
	.send()
	.await
	.unwrap();

	let lld_proxy_contract = ERC1967Proxy::deploy(
		client.clone(),
		(
			bridge_contract.address(),
			bridge_contract
				.initialize(
					token.address(),
					1u32,              // votes required
					0.into(),          // mint delay
					0.into(),          // fee
					10000000.into(),   // counter limit
					10000000.into(),   // decay rate
					1000000000.into(), // supply limit
				)
				.calldata()
				.unwrap(),
		),
	)
	.unwrap()
	.send()
	.await
	.unwrap();

	let proxy_contract = BridgeABI::new(llm_proxy_contract.address(), client.clone());

	proxy_contract
		.grant_role(keccak256("ADMIN_ROLE"), client.signer().address())
		.send()
		.await
		.unwrap();

	proxy_contract
		.grant_role(keccak256("RELAY_ROLE"), client.signer().address())
		.send()
		.await
		.unwrap();

	proxy_contract
		.grant_role(keccak256("WATCHER_ROLE"), client.signer().address())
		.send()
		.await
		.unwrap();

	token.mint(client.signer().address(), 1000.into()).send().await.unwrap();
	token.approve(proxy_contract.address(), 1000.into()).send().await.unwrap();
	token.transfer_ownership(proxy_contract.address()).send().await.unwrap();

	proxy_contract.set_active(true).send().await.unwrap();

	bridge_instance.llm_contract = Some(proxy_contract);
	bridge_instance.lld_contract =
		Some(BridgeABI::new(lld_proxy_contract.address(), client.clone()));
	bridge_instance.anvil = Some(anvil);
}

async fn start_liberland(bridge_instance: &mut BridgeInstance) {
	let path = std::env::var("RELAY_TEST_SUBSTRATE_PATH")
		.unwrap_or("../../target/release/substrate".into());

	let mut cmd = substrate::KillChildOnDrop(
		Command::new(path)
			.stdout(process::Stdio::piped())
			.stderr(process::Stdio::piped())
			.arg("--dev")
			.arg("--no-hardware-benchmarks")
			.spawn()
			.unwrap(),
	);

	let stderr = cmd.stderr.take().unwrap();
	let ws_url: url::Url = substrate::find_ws_url_from_output(stderr).await.parse().unwrap();

	let (pair, _) = SubstratePair::from_string_with_seed("//Alice", None).unwrap();
	let signer = PairSigner::new(pair.clone());
	let client = OnlineClient::<SubstrateConfig>::from_url(&ws_url).await.unwrap();

	client
		.tx()
		.sign_and_submit_then_watch_default(
			&liberland::tx().llm_bridge().add_watcher(pair.public().into()),
			&signer,
		)
		.await
		.unwrap()
		.wait_for_in_block()
		.await
		.unwrap()
		.wait_for_success()
		.await
		.unwrap();

	client
		.tx()
		.sign_and_submit_then_watch_default(
			&liberland::tx().llm_bridge().add_relay(pair.public().into()),
			&signer,
		)
		.await
		.unwrap()
		.wait_for_in_block()
		.await
		.unwrap()
		.wait_for_success()
		.await
		.unwrap();

	client
		.tx()
		.sign_and_submit_then_watch_default(
			&liberland::tx().llm_bridge().set_state(BridgeState::Active),
			&signer,
		)
		.await
		.unwrap()
		.wait_for_in_block()
		.await
		.unwrap()
		.wait_for_success()
		.await
		.unwrap();

	bridge_instance.substrate_cmd = Some(cmd);
	bridge_instance.substrate_ws_url = Some(ws_url);
	bridge_instance.sub_client = Some(client);
	bridge_instance.sub_signer = Some(signer);
}

async fn start_bridge() -> BridgeInstance {
	let mut bridge_instance: BridgeInstance = Default::default();
	start_liberland(&mut bridge_instance).await;
	start_ethereum(&mut bridge_instance).await;
	let mut settings = Settings {
		ethereum_ws_url: bridge_instance.anvil.as_ref().unwrap().ws_endpoint().parse().unwrap(),
		substrate_ws_url: bridge_instance.substrate_ws_url.clone().unwrap(),
		llm_bridge_contract: bridge_instance.llm_contract.as_ref().unwrap().address(),
		lld_bridge_contract: bridge_instance.lld_contract.as_ref().unwrap().address(),
		relays: vec![Relay {
			keys: Keys {
				substrate_secret_seed: "//Alice".into(),
				ethereum_private_key: hex::encode(
					bridge_instance.anvil.as_ref().unwrap().keys()[0].to_bytes(),
				),
			},
			claim_rewards_threshold: "99999".into(),
			minimum_balance: "99999".into(),
			withdraw_rewards_threshold: "99999".into(),
			rewards_address: Default::default(),
		}],
		watchers: vec![Keys {
			substrate_secret_seed: "//Alice".into(),
			ethereum_private_key: hex::encode(
				bridge_instance.anvil.as_ref().unwrap().keys()[0].to_bytes(),
			),
		}],
		..Default::default()
	};
	start_relay(&mut settings, &mut bridge_instance);
	bridge_instance
}

async fn find_log<T: AsyncBufRead + Unpin>(lines: &mut Lines<T>, text: &str) {
	while let Some(line) = lines.next_line().await.unwrap() {
		println!("{}", line);
		let line = strip_ansi_escapes::strip(&line).unwrap();
		let line = std::str::from_utf8(&line).unwrap();
		if line.contains(text) {
			return
		}
	}

	println!("WTF");
	panic!("EOF reached, log not found");
}

#[tokio::test]
async fn transfer_to_eth_works() {
	let bridge = start_bridge().await;
	let anvil = bridge.anvil.unwrap();
	let sub_signer = bridge.sub_signer.unwrap();
	let sub_client = bridge.sub_client.unwrap();
	let mut relay = bridge.relay_cmd.unwrap();
	let mut relay_stdout = BufReader::new(relay.stdout.take().unwrap()).lines();

	sub_client
		.tx()
		.sign_and_submit_then_watch_default(
			&liberland::tx().llm_bridge().deposit(100, anvil.addresses()[0].0),
			&sub_signer,
		)
		.await
		.unwrap()
		.wait_for_in_block()
		.await
		.unwrap()
		.wait_for_success()
		.await
		.unwrap();

	let r = &mut relay_stdout;
	timeout(
		Duration::from_secs(20),
		find_log(r, "relay::relay::substrate_to_ethereum: Submitting vote"),
	)
	.await
	.unwrap();
	timeout(
		Duration::from_secs(10),
		find_log(r, "relay::tx_managers::ethereum: Successfully finished eth call!"),
	)
	.await
	.unwrap();
	// FIXME check contract for vote
	assert!(relay.try_wait().unwrap().is_none(), "the process should still be running");
}

#[tokio::test]
async fn transfer_to_sub_works() {
	let bridge = start_bridge().await;
	let bridge_contract = bridge.llm_contract.unwrap();
	let mut relay = bridge.relay_cmd.unwrap();
	let mut relay_stdout = BufReader::new(relay.stdout.take().unwrap()).lines();

	bridge_contract
		.burn(
			1000.into(),
			hex::decode("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d") // Alice account id
				.unwrap()
				.try_into()
				.unwrap(),
		)
		.send()
		.await
		.unwrap();
	let r = &mut relay_stdout;
	timeout(
		Duration::from_secs(20),
		find_log(r, "relay::relay::ethereum_to_substrate: Submitting vote"),
	)
	.await
	.unwrap();
	timeout(
		Duration::from_secs(20),
		find_log(r, "relay::relay::ethereum_to_substrate: Vote finalized!"),
	)
	.await
	.unwrap();
	// FIXME check pallet for vote
	assert!(relay.try_wait().unwrap().is_none(), "the process should still be running");
}

#[tokio::test]
async fn watcher_to_eth_works_for_good() {
	let bridge = start_bridge().await;
	let anvil = bridge.anvil.unwrap();
	let sub_signer = bridge.sub_signer.unwrap();
	let sub_client = bridge.sub_client.unwrap();
	let mut relay = bridge.relay_cmd.unwrap();
	let mut relay_stdout = BufReader::new(relay.stdout.take().unwrap()).lines();

	sub_client
		.tx()
		.sign_and_submit_then_watch_default(
			&liberland::tx().llm_bridge().deposit(100, anvil.addresses()[0].0),
			&sub_signer,
		)
		.await
		.unwrap()
		.wait_for_in_block()
		.await
		.unwrap()
		.wait_for_success()
		.await
		.unwrap();

	let r = &mut relay_stdout;
	timeout(
		Duration::from_secs(20),
		find_log(r, "relay::watcher::substrate_to_ethereum: Relay voted"),
	)
	.await
	.unwrap();
	timeout(
		Duration::from_secs(10),
		find_log(r, "relay::watcher::substrate_to_ethereum: Receipt is valid"),
	)
	.await
	.unwrap();
	assert!(relay.try_wait().unwrap().is_none(), "the process should still be running");
}

#[tokio::test]
async fn watcher_to_sub_works_for_good() {
	let bridge = start_bridge().await;
	let bridge_contract = bridge.llm_contract.unwrap();
	let mut relay = bridge.relay_cmd.unwrap();
	let mut relay_stdout = BufReader::new(relay.stdout.take().unwrap()).lines();

	bridge_contract
		.burn(
			1000.into(),
			hex::decode("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d") // Alice account id
				.unwrap()
				.try_into()
				.unwrap(),
		)
		.send()
		.await
		.unwrap();
	let r = &mut relay_stdout;
	timeout(
		Duration::from_secs(20),
		find_log(r, "relay::watcher::ethereum_to_substrate: Checking eth vote for receipt"),
	)
	.await
	.unwrap();
	timeout(
		Duration::from_secs(30),
		find_log(r, "relay::watcher::ethereum_to_substrate: Check passed."),
	)
	.await
	.unwrap();
	assert!(relay.try_wait().unwrap().is_none(), "the process should still be running");
}

#[tokio::test]
async fn watcher_to_eth_works_for_bad() {
	let bridge = start_bridge().await;
	let bridge_contract = bridge.llm_contract.unwrap();
	let sub_client = bridge.sub_client.unwrap();
	let mut relay = bridge.relay_cmd.unwrap();
	let mut relay_stdout = BufReader::new(relay.stdout.take().unwrap()).lines();

	bridge_contract
		.vote_mint(Default::default(), 1, 100.into(), H160::zero())
		.send()
		.await
		.unwrap();

	let r = &mut relay_stdout;
	timeout(
		Duration::from_secs(20),
		find_log(r, "relay::watcher::substrate_to_ethereum: Relay voted"),
	)
	.await
	.unwrap();
	timeout(
		Duration::from_secs(10),
		find_log(r, "relay::watcher::substrate_to_ethereum: Receipt is not valid"),
	)
	.await
	.unwrap();
	sleep(Duration::from_secs(5)).await;
	assert_eq!(
		sub_client
			.storage()
			.at_latest()
			.await
			.unwrap()
			.fetch(&liberland::storage().llm_bridge().state())
			.await
			.unwrap()
			.unwrap()
			.encode(),
		BridgeState::Stopped.encode()
	);
	// FIXME check if eth stopped too
	assert!(relay.try_wait().unwrap().is_none(), "the process should still be running");
}

#[tokio::test]
async fn watcher_to_sub_works_for_bad() {
	let bridge = start_bridge().await;
	let sub_signer = bridge.sub_signer.unwrap();
	let sub_client = bridge.sub_client.unwrap();
	let mut relay = bridge.relay_cmd.unwrap();
	let mut relay_stdout = BufReader::new(relay.stdout.take().unwrap()).lines();

	sub_client
		.tx()
		.sign_and_submit_then_watch_default(
			&liberland::tx().llm_bridge().vote_withdraw(
				Default::default(),
				IncomingReceipt {
					eth_block_number: 1,
					substrate_recipient: sub_signer.account_id().clone(),
					amount: 15,
				},
			),
			&sub_signer,
		)
		.await
		.unwrap()
		.wait_for_in_block()
		.await
		.unwrap()
		.wait_for_success()
		.await
		.unwrap();

	let r = &mut relay_stdout;
	timeout(
		Duration::from_secs(20),
		find_log(r, "relay::watcher::ethereum_to_substrate: Checking eth vote for receipt"),
	)
	.await
	.unwrap();
	timeout(
		Duration::from_secs(30),
		find_log(
			r,
			"relay::watcher::ethereum_to_substrate: Emergency stopping due to invalid vote!",
		),
	)
	.await
	.unwrap();
	sleep(Duration::from_secs(10)).await; // FIXME ugly, check properly using logs
	assert_eq!(
		sub_client
			.storage()
			.at_latest()
			.await
			.unwrap()
			.fetch(&liberland::storage().llm_bridge().state())
			.await
			.unwrap()
			.unwrap()
			.encode(),
		BridgeState::Stopped.encode()
	);
	// FIXME check if eth stopped too
	assert!(relay.try_wait().unwrap().is_none(), "the process should still be running");
}
