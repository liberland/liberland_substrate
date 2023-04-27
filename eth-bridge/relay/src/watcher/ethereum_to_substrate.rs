use super::*;

use crate::{
	bridge_abi::{BridgeABI, OutgoingReceiptFilter},
	sync_managers::{Substrate as SubstrateSyncManager, SubstrateSyncTarget},
	utils::try_to_decode_err,
};
use ethers::{
	contract::EthEvent, middleware::SignerMiddleware, prelude::LocalWallet, signers::Signer,
	types::Eip1559TransactionRequest, utils::to_checksum,
};
use subxt::events::EventDetails;

use liberland::{lld_bridge::events::Vote as LLDEvent, llm_bridge::events::Vote as LLMEvent};

use tokio::sync::mpsc;

pub struct EthereumToSubstrate {
	id: WatcherId,
	db: Arc<SqlitePool>,
	eth_signer: Arc<SignerMiddleware<Arc<Provider<Ws>>, LocalWallet>>,
	sub_signer: PairSigner<SubstrateConfig, SubstratePair>,
	sub_api: Arc<OnlineClient<SubstrateConfig>>,
	llm_bridge_contract: EthAddress,
	lld_bridge_contract: EthAddress,
	tx_manager_send: mpsc::Sender<(CallId, Eip1559TransactionRequest)>,
}

impl EthereumToSubstrate {
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
	) -> Result<Self> {
		let eth_wallet = eth_wallet.with_chain_id(eth_provider.get_chainid().await?.as_u64());
		let eth_signer = Arc::new(SignerMiddleware::new(eth_provider, eth_wallet));

		Ok(Self {
			id,
			db,
			eth_signer,
			sub_signer,
			sub_api,
			llm_bridge_contract,
			lld_bridge_contract,
			tx_manager_send,
		})
	}

	#[tracing::instrument(skip_all,fields(id = ?self.id))]
	pub async fn run(self) -> Result<()> {
		let eth_address = to_checksum(&self.eth_signer.address(), None);
		let sub_address = format!("{}", self.sub_signer.account_id());
		tracing::info!(eth_address, sub_address, "Started Substrate -> Ethereum relay");
		self.sync().await?;
		Err(eyre::eyre!("Relay task finished early!"))
	}

	pub async fn sync(&self) -> Result<()> {
		let sync_manager = SubstrateSyncManager::new(
			(&self.id).into(),
			self.db.clone(),
			self.sub_api.clone(),
			SubstrateSyncTarget::Best,
		);
		let subscription = sync_manager.sync();
		pin_mut!(subscription);
		while let Some(event) = subscription.next().await {
			let (_block_number, _block_hash, event) = event?;
			self.process_raw_event(event).await?;
		}

		Ok(())
	}

	async fn process_raw_event(&self, event: EventDetails) -> Result<()> {
		if let Ok(Some(event)) = event.as_event::<LLDEvent>() {
			self.process_event(BridgeId::LLM, event.relay.into(), event.receipt_id.into())
				.await?;
		} else if let Ok(Some(event)) = event.as_event::<LLMEvent>() {
			self.process_event(BridgeId::LLM, event.relay.into(), event.receipt_id.into())
				.await?;
		} else {
			tracing::trace!("Non-bridge event skipped: {:?}", event);
		}
		Ok(())
	}

	async fn process_event(
		&self,
		bridge: BridgeId,
		relay: AccountId,
		receipt_id: ReceiptId,
	) -> Result<()> {
		tracing::debug!(?receipt_id, "Checking eth vote for receipt");
		if self.should_emergency_stop(bridge, &receipt_id).await? {
			tracing::warn!(
				?receipt_id,
				%relay,
				?bridge,
				"Emergency stopping due to invalid vote!"
			);
			self.emergency_stop(bridge).await?;
		} else {
			tracing::debug!(?receipt_id, "Check passed.");
		}
		Ok(())
	}

	async fn should_emergency_stop(
		&self,
		bridge: BridgeId,
		receipt_id: &ReceiptId,
	) -> Result<bool> {
		// get details of receipt from substrate
		let receipt_id_bytes: [u8; 32] = receipt_id.into();
		let substrate_receipt = match bridge {
			BridgeId::LLM => liberland::storage().llm_bridge().incoming_receipts(receipt_id_bytes),
			BridgeId::LLD => liberland::storage().lld_bridge().incoming_receipts(receipt_id_bytes),
		};

		let substrate_receipt =
			self.sub_api.storage().at_latest().await?.fetch(&substrate_receipt).await?;

		Ok(match substrate_receipt {
			None => true,
			Some(substrate_receipt) => {
				// try to match corresponding receipt on ethereum
				let filter = Filter::new()
					.event(&OutgoingReceiptFilter::abi_signature())
					.address(vec![self.lld_bridge_contract, self.llm_bridge_contract])
					.from_block(substrate_receipt.eth_block_number)
					.to_block(substrate_receipt.eth_block_number);
				let logs = self.eth_signer.get_logs(&filter).await?;

				!logs.iter().any(|log| {
					let block_hash = log.block_hash.expect("Log without block hash");
					let index = log.log_index.expect("Log without index").as_u64();

					let event = OutgoingReceiptFilter::decode_log(&log.clone().into())
						.expect("Only OutgoingReceipt logs should be passed here");
					let amount = event.amount;
					let substrate_recipient: AccountId32 = event.substrate_recipient.into();

					let log_receipt_id =
						Self::receipt_id(&block_hash, &index, &amount, &substrate_recipient);

					&log_receipt_id == receipt_id &&
						amount == substrate_receipt.amount.into() &&
						substrate_recipient == substrate_receipt.substrate_recipient
				})
			},
		})
	}

	#[tracing::instrument(skip(self))]
	// this is likely the same for both watchers, extract it somewhere?
	async fn emergency_stop(&self, bridge: BridgeId) -> Result<()> {
		match bridge {
			BridgeId::LLM =>
				self._emergency_stop(
					self.llm_bridge_contract,
					liberland::tx().llm_bridge().emergency_stop(),
				)
				.await,
			BridgeId::LLD =>
				self._emergency_stop(
					self.lld_bridge_contract,
					liberland::tx().lld_bridge().emergency_stop(),
				)
				.await,
		}
	}

	async fn _emergency_stop<T>(&self, contract: EthAddress, sub_tx: T) -> Result<()>
	where
		T: subxt::tx::TxPayload,
	{
		tracing::debug!("Emergency stop contract...");
		let contract = BridgeABI::new(contract, self.eth_signer.clone());
		let eth_tx = contract.emergency_stop();

		eth_tx.call().await.map_err(try_to_decode_err)?;

		tracing::debug!("Queuing tx...");
		self.tx_manager_send.send((CallId::random(), eth_tx.tx.into())).await?;

		tracing::debug!("Emergency stop pallet...");
		self.sub_api
			.tx()
			.sign_and_submit_then_watch_default(&sub_tx, &self.sub_signer)
			.await?;

		Ok(())
	}

	// FIXME extract this, as this is copied from relay
	fn receipt_id(
		block_hash: &H256,
		log_index: &u64,
		amount: &Amount,
		recipient: &AccountId,
	) -> ReceiptId {
		let log_index: [u8; 8] = log_index.to_be_bytes();
		let mut amount_bytes: [u8; 32] = Default::default();
		amount.to_big_endian(&mut amount_bytes);
		let recipient: &[u8; 32] = recipient.as_ref();
		BlakeTwo256::hash(&[block_hash.as_bytes(), &log_index, &amount_bytes, recipient].concat())
			.into()
	}
}
