use super::*;

use crate::{
	bridge_abi::{BridgeABI, VoteFilter},
	liberland::{
		lld_bridge::events::OutgoingReceipt as LLDOutgoingReceipt,
		llm_bridge::events::OutgoingReceipt as LLMOutgoingReceipt,
	},
	sync_managers::{Ethereum as EthereumSyncManager, EthereumSyncTarget},
};
use ethers::{
	abi::Address,
	contract::EthEvent,
	prelude::SignerMiddleware,
	signers::{LocalWallet, Signer},
	types::Log,
};
use primitive_types::H160;
use sp_core::sr25519::Pair as SubstratePair;
use subxt::{
	config::Hasher as HasherT,
	events::{EventDetails, Events},
	tx::PairSigner,
};

pub struct SubstrateToEthereum {
	id: WatcherId,
	db: Arc<SqlitePool>,
	sub_api: Arc<OnlineClient<SubstrateConfig>>,
	eth_provider: Arc<Provider<Ws>>,
	llm_bridge_contract: EthAddress,
	lld_bridge_contract: EthAddress,
	eth_signer: Arc<SignerMiddleware<Arc<Provider<Ws>>, LocalWallet>>,
	sub_signer: PairSigner<SubstrateConfig, SubstratePair>,
}

impl SubstrateToEthereum {
	pub async fn new(
		id: WatcherId,
		db: Arc<SqlitePool>,
		eth_provider: Arc<Provider<Ws>>,
		eth_wallet: LocalWallet,
		sub_signer: PairSigner<SubstrateConfig, SubstratePair>,
		sub_api: Arc<OnlineClient<SubstrateConfig>>,
		llm_bridge_contract: EthAddress,
		lld_bridge_contract: EthAddress,
	) -> Result<Self> {
		let eth_wallet = eth_wallet.with_chain_id(eth_provider.get_chainid().await?.as_u64());
		let eth_signer = Arc::new(SignerMiddleware::new(eth_provider.clone(), eth_wallet));
		Ok(Self {
			id,
			db,
			eth_signer,
			sub_api,
			eth_provider,
			llm_bridge_contract,
			lld_bridge_contract,
			sub_signer,
		})
	}

	#[tracing::instrument(skip_all,fields(id = ?self.id))]
	pub async fn run(self) -> Result<()> {
		let address = format!("{}", self.sub_signer.account_id());
		tracing::info!(address, "Started Substrate -> Ethereum watcher");
		tokio::select! {
			r = self.sync() => r?,
		};
		Err(eyre::eyre!("Watcher task finished early!"))
	}

	pub async fn process_log(&self, log: Log) -> Result<()> {
		let contract = log.address;
		let event = VoteFilter::decode_log(&log.into())?;

		let receipt_id = ReceiptId::from(H256::from(event.receipt_id));
		let relay: Address = event.relay;
		let receipt_substrate_block_number = event.substrate_block_number;

		let latest_hash = self.sub_api.rpc().block_hash(None).await?.unwrap();
		let latest_substrate_block: u64 = self
			.sub_api
			.rpc()
			.header(Some(latest_hash))
			.await?
			.expect("Block doesn't have a header?!")
			.number
			.into();

		tracing::info!("Relay {relay} voted for {receipt_id:?} at {receipt_substrate_block_number} now is {latest_substrate_block}");

		let bridge_id = self.get_bridge_id(contract);

		// TODO: WEE NEED TO CHECK HOW FAR AWAY IS LIBERLAND NODE BEHIND
		if receipt_substrate_block_number > latest_substrate_block + 50 {
			tracing::error!("Receipt comes from a non-existent block");
			return self.emergency_stop_bridges(bridge_id).await
		}
		let receipt_block_hash = self
			.sub_api
			.rpc()
			.block_hash(Some(receipt_substrate_block_number.into()))
			.await?
			.expect("Block doesn't exist");

		let block_events = self.sub_api.events().at(receipt_block_hash).await?;
		let is_receipt_valid = self.block_has_valid_receipt_event(
			bridge_id,
			block_events,
			receipt_id,
			receipt_block_hash,
		);
		if !is_receipt_valid? {
			tracing::error!("Receipt is not valid");
			return self.emergency_stop_bridges(bridge_id).await
		}
		tracing::info!("Receipt is valid");

		Ok(())
	}

	fn block_has_valid_receipt_event(
		&self,
		bridge_id: BridgeId,
		block_events: Events<SubstrateConfig>,
		receipt_id: ReceiptId,
		block: <SubstrateConfig as subxt::config::Config>::Hash,
	) -> Result<bool> {
		match bridge_id {
			BridgeId::LLD => {
				let block_events: Result<Vec<EventDetails>, subxt::Error> =
					block_events.iter().collect();
				Ok(block_events?.iter().any(|event| {
					self.block_has_valid_lld_receipt(event.clone(), block, receipt_id.clone())
				}))
			},
			BridgeId::LLM => {
				let block_events: Result<Vec<EventDetails>, subxt::Error> =
					block_events.iter().collect();
				Ok(block_events?.iter().any(|event| {
					self.block_has_valid_llm_receipt(event.clone(), block, receipt_id.clone())
				}))
			},
		}
	}

	fn block_has_valid_llm_receipt(
		&self,
		event: EventDetails,
		block: <SubstrateConfig as subxt::config::Config>::Hash,
		incoming_receipt: ReceiptId,
	) -> bool {
		let idx = event.index();
		if let Ok(Some(event)) = event.as_event::<LLMOutgoingReceipt>() {
			let receipt_id =
				self.receipt_id(block, idx, event.amount.into(), &event.eth_recipient.into());
			return receipt_id == incoming_receipt
		}
		return false
	}

	fn block_has_valid_lld_receipt(
		&self,
		event: EventDetails,
		block: <SubstrateConfig as subxt::config::Config>::Hash,
		incoming_receipt: ReceiptId,
	) -> bool {
		let idx = event.index();
		if let Ok(Some(event)) = event.as_event::<LLDOutgoingReceipt>() {
			let receipt_id =
				self.receipt_id(block, idx, event.amount.into(), &event.eth_recipient.into());
			return receipt_id == incoming_receipt
		}
		return false
	}

	fn get_bridge_id(&self, contract: H160) -> BridgeId {
		match contract {
			x if x == self.lld_bridge_contract => BridgeId::LLD,
			x if x == self.llm_bridge_contract => BridgeId::LLM,
			_ => {
				tracing::error!(
					contract = hex::encode(contract),
					"Unexpected log received from subscription"
				);
				panic!("Unexpected log received");
			},
		}
	}

	async fn emergency_stop_bridges(&self, bridge_id: BridgeId) -> Result<()> {
		match bridge_id {
			BridgeId::LLD => self.emergency_stop_lld_bridge().await,
			BridgeId::LLM => self.emergency_stop_llm_bridge().await,
		}
	}

	async fn emergency_stop_lld_bridge(&self) -> Result<()> {
		let stop_tx = liberland::tx().lld_bridge().emergency_stop();
		let sub = self
			.sub_api
			.tx()
			.sign_and_submit_then_watch_default(&stop_tx, &self.sub_signer)
			.await?;

		tracing::info!("Substrate emergency LLD bridge stop transactions were sent!");
		let stop_sub_tx = sub.wait_for_finalized_success();
		tracing::info!("Substrate LLD bridge successfully stopped!");

		let bridge = BridgeABI::new(self.lld_bridge_contract, self.eth_signer.clone());
		tracing::info!("Ethereum emergency LLD bridge stop transactions were sent!");
		let stop_tx = bridge.emergency_stop();
		stop_tx.call().await?;
		tracing::info!("Ethereum LLD bridge successfully stopped!");
		stop_sub_tx.await?;
		tracing::info!("Substrate LLD bridge successfully stopped!");
		Ok(())
	}

	async fn emergency_stop_llm_bridge(&self) -> Result<()> {
		let stop_tx = liberland::tx().llm_bridge().emergency_stop();
		let sub = self
			.sub_api
			.tx()
			.sign_and_submit_then_watch_default(&stop_tx, &self.sub_signer)
			.await?;

		tracing::info!("Substrate emergency LLM bridge stop transactions were sent!");
		sub.wait_for_finalized_success().await?;
		tracing::info!("Substrate LLM bridge successfully stopped!");

		let bridge = BridgeABI::new(self.llm_bridge_contract, self.eth_signer.clone());
		tracing::info!("Ethereum emergency LLM bridge stop transactions were sent!");
		let stop_tx = bridge.emergency_stop();
		stop_tx.send().await?;
		tracing::info!("Ethereum LLM bridge successfully stopped!");
		Ok(())
	}

	fn receipt_id(
		&self,
		block: <SubstrateConfig as subxt::config::Config>::Hash,
		idx: u32,
		amount: Amount,
		recipient: &EthAddress,
	) -> ReceiptId {
		let mut amount_bytes: [u8; 32] = Default::default();
		amount.to_big_endian(&mut amount_bytes);

		BlakeTwo256::hash(
			&[block.as_bytes(), &idx.to_be_bytes(), &amount_bytes, recipient.as_bytes()].concat(),
		)
		.into()
	}

	pub async fn sync(&self) -> Result<()> {
		let sync_manager = EthereumSyncManager::new(
			(&self.id).into(),
			self.db.clone(),
			self.eth_provider.clone(),
			self.llm_bridge_contract,
			self.lld_bridge_contract,
			EthereumSyncTarget::Latest,
			&VoteFilter::abi_signature(),
		);

		let subscription = sync_manager.sync();
		pin_mut!(subscription);

		tracing::info!("Synced, starting realtime monitoring");
		while let Some(log) = subscription.next().await {
			let log = log?;
			self.process_log(log).await?;
		}
		Err(eyre::eyre!("Block subscription ended - connection to Ethereum failed?"))
	}
}
