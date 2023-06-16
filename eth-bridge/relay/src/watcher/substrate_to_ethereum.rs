use super::*;

use crate::{
	bridge_abi::{BridgeABI, VoteFilter},
	liberland::{
		lld_bridge::events::OutgoingReceipt as LLDOutgoingReceipt,
		llm_bridge::events::OutgoingReceipt as LLMOutgoingReceipt,
	},
	liberland_utils,
	sync_managers::{Ethereum as EthereumSyncManager, EthereumSyncTarget},
	tx_managers,
	utils::{substrate_receipt_id, try_to_decode_err},
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
	tx_manager_send: mpsc::Sender<(CallId, Eip1559TransactionRequest)>,
	sub_tx_manager: Arc<tx_managers::Substrate>,
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
		tx_manager_send: mpsc::Sender<(CallId, Eip1559TransactionRequest)>,
		sub_tx_manager: &Arc<tx_managers::Substrate>,
	) -> Result<Self> {
		let eth_wallet = eth_wallet.with_chain_id(eth_provider.get_chainid().await?.as_u64());
		let eth_signer = Arc::new(SignerMiddleware::new(eth_provider.clone(), eth_wallet));
		let sub_tx_manager = sub_tx_manager.clone();
		Ok(Self {
			id,
			db,
			eth_signer,
			sub_api,
			eth_provider,
			llm_bridge_contract,
			lld_bridge_contract,
			sub_signer,
			tx_manager_send,
			sub_tx_manager,
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

		tracing::info!(%relay, ?receipt_id, receipt_substrate_block_number, latest_substrate_block, "Relay voted");

		let bridge_id = self.get_bridge_id(contract);

		if receipt_substrate_block_number > latest_substrate_block + 50 {
			tracing::error!("Receipt comes from a non-existent block");
			return self.emergency_stop(bridge_id).await
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
			receipt_id.clone(),
			receipt_block_hash,
		);
		if !is_receipt_valid? {
			tracing::error!(?receipt_id, "Receipt is not valid");
			return self.emergency_stop(bridge_id).await
		}
		tracing::info!(?receipt_id, "Receipt is valid");

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
				substrate_receipt_id(block, idx, event.amount.into(), &event.eth_recipient.into());
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
				substrate_receipt_id(block, idx, event.amount.into(), &event.eth_recipient.into());
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

	#[tracing::instrument(skip(self))]
	// this is likely the same for both watchers, extract it somewhere?
	async fn emergency_stop(&self, bridge: BridgeId) -> Result<()> {
		tracing::debug!("Emergency stop contract...");
		let contract = match bridge {
			BridgeId::LLM => self.llm_bridge_contract,
			BridgeId::LLD => self.lld_bridge_contract,
		};
		let contract = BridgeABI::new(contract, self.eth_signer.clone());
		let eth_tx = contract.emergency_stop();
		eth_tx.call().await.map_err(try_to_decode_err)?;
		tracing::debug!("Queuing tx...");
		self.tx_manager_send.send((CallId::random(), eth_tx.tx.into())).await?;

		tracing::debug!("Emergency stop pallet...");
		let sub_tx =
			liberland_utils::tx::emergency_stop(&self.sub_api, &self.sub_signer, bridge).await?;
		self.sub_tx_manager.add(sub_tx).await?;

		Ok(())
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

		while let Some(log) = subscription.next().await {
			let log = log?;
			self.process_log(log).await?;
		}
		Err(eyre::eyre!("Block subscription ended - connection to Ethereum failed?"))
	}
}
