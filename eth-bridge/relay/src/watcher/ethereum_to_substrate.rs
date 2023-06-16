use super::*;

use crate::{
	bridge_abi::{BridgeABI, OutgoingReceiptFilter},
	liberland_utils,
	sync_managers::{Substrate as SubstrateSyncManager, SubstrateSyncTarget},
	tx_managers,
	utils::{eth_receipt_id, try_to_decode_err},
};
use ethers::{
	contract::EthEvent, middleware::SignerMiddleware, prelude::LocalWallet, signers::Signer,
	utils::to_checksum,
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
	sub_tx_manager: Arc<tx_managers::Substrate>,
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
		sub_tx_manager: &Arc<tx_managers::Substrate>,
	) -> Result<Self> {
		let eth_wallet = eth_wallet.with_chain_id(eth_provider.get_chainid().await?.as_u64());
		let eth_signer = Arc::new(SignerMiddleware::new(eth_provider, eth_wallet));
		let sub_tx_manager = sub_tx_manager.clone();

		Ok(Self {
			id,
			db,
			eth_signer,
			sub_signer,
			sub_api,
			llm_bridge_contract,
			lld_bridge_contract,
			tx_manager_send,
			sub_tx_manager,
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
		let substrate_receipt = liberland_utils::storage::incoming_receipts(bridge, receipt_id);

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
						eth_receipt_id(&block_hash, &index, &amount, &substrate_recipient);

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
}
