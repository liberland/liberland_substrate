use super::*;

use crate::{
	bridge_abi::OutgoingReceiptFilter,
	liberland_utils,
	sync_managers::{Ethereum as EthereumSyncManager, EthereumSyncTarget},
	tx_managers,
	utils::eth_receipt_id,
};
use ethers::{contract::EthEvent, types::Log};
use liberland::runtime_types::pallet_federated_bridge::IncomingReceiptStatus;
use sp_core::sr25519::Pair as SubstratePair;
use subxt::{tx::PairSigner, utils::AccountId32};
use tracing::{span, Instrument, Level};

pub struct EthereumToSubstrate {
	id: RelayId,
	db: Arc<SqlitePool>,
	sub_signer: PairSigner<SubstrateConfig, SubstratePair>,
	sub_api: Arc<OnlineClient<SubstrateConfig>>,
	eth_provider: Arc<Provider<Ws>>,
	llm_bridge_contract: EthAddress,
	lld_bridge_contract: EthAddress,
	sub_tx_manager: Arc<tx_managers::Substrate>,
	unsafe_fast_mode: bool,
}

impl EthereumToSubstrate {
	pub async fn new(
		id: RelayId,
		db: Arc<SqlitePool>,
		eth_provider: Arc<Provider<Ws>>,
		sub_signer: PairSigner<SubstrateConfig, SubstratePair>,
		sub_api: Arc<OnlineClient<SubstrateConfig>>,
		llm_bridge_contract: EthAddress,
		lld_bridge_contract: EthAddress,
		sub_tx_manager: &Arc<tx_managers::Substrate>,
	) -> Result<Self> {
		let unsafe_fast_mode = cfg!(debug_assertions);

		if unsafe_fast_mode {
			tracing::error!("Running relay with UNSAFE fast mode enabled! Transactions will be voted on ASAP, without waiting for finalization. DO NOT RUN THIS IN REAL SYSTEM!");
		}
		let sub_tx_manager = sub_tx_manager.clone();

		Ok(Self {
			id,
			db,
			sub_signer,
			sub_api,
			eth_provider,
			llm_bridge_contract,
			lld_bridge_contract,
			unsafe_fast_mode,
			sub_tx_manager,
		})
	}

	#[tracing::instrument(skip_all,fields(id = ?self.id))]
	pub async fn run(self) -> Result<()> {
		let address = format!("{}", self.sub_signer.account_id());
		tracing::info!(address, "Started Ethereum -> Substrate relay");
		self.sync().await?;
		Err(eyre::eyre!("Relay task finished unexpectedly!"))
	}

	pub async fn sync_db_calls(&self) -> Result<()> {
		let mut conn = self.db.acquire().await?;
		let calls = db::get_unfinished_sub_calls(&mut conn, &self.id).await?;

		for call in calls {
			let span = span!(Level::INFO, "reprocess_receipt", ?call.call_id, ?call.bridge_id,);
			let receipt = EthReceipt::new(
				call.bridge_id,
				call.call_id.into(),
				call.eth_block_number,
				call.amount,
				call.substrate_recipient,
			);

			self.process_receipt(receipt).instrument(span).await?;
		}
		Ok(())
	}

	pub async fn sync(&self) -> Result<()> {
		self.sync_db_calls().await?;

		let target = if self.unsafe_fast_mode {
			EthereumSyncTarget::Latest
		} else {
			EthereumSyncTarget::Finalized
		};

		let sync_manager = EthereumSyncManager::new(
			(&self.id).into(),
			self.db.clone(),
			self.eth_provider.clone(),
			self.llm_bridge_contract,
			self.lld_bridge_contract,
			target,
			&OutgoingReceiptFilter::abi_signature(),
		);
		let subscription = sync_manager.sync();
		pin_mut!(subscription);

		while let Some(log) = subscription.next().await {
			let log = log?;
			self.process_log(log).await?;
		}
		Err(eyre::eyre!("Block subscription ended - connection to Ethereum failed?"))
	}

	pub async fn process_log(&self, log: Log) -> Result<()> {
		let block_number = log.block_number.expect("Log without block number").as_u64();
		let block_hash = log.block_hash.expect("Log without block hash");
		let eth_tx_hash =
			format!("0x{}", hex::encode(log.transaction_hash.expect("Log without tx hash")));
		let index = log.log_index.expect("Log without index").as_u64();
		let contract = log.address;

		let event = OutgoingReceiptFilter::decode_log(&log.into())?;
		let amount = event.amount;
		let substrate_recipient: AccountId32 = event.substrate_recipient.into();

		let receipt_id = eth_receipt_id(&block_hash, &index, &amount, &substrate_recipient);
		let bridge_id = match contract {
			x if x == self.lld_bridge_contract => BridgeId::LLD,
			x if x == self.llm_bridge_contract => BridgeId::LLM,
			_ => {
				tracing::error!(
					contract = hex::encode(contract),
					"Unexpected log received from subscription"
				);
				panic!("Unexpected log received");
			},
		};

		let span = span!(Level::INFO, "process_receipt", ?receipt_id, eth_tx_hash, ?bridge_id,);
		let receipt =
			EthReceipt::new(bridge_id, receipt_id, block_number, amount, substrate_recipient);

		self.process_receipt(receipt).instrument(span).await?;
		Ok(())
	}

	async fn process_receipt(&self, receipt: EthReceipt) -> Result<()> {
		if self.should_vote(&receipt).await? {
			self.vote_on_receipt(&receipt).await?;
		}
		let mut conn = self.db.acquire().await?;
		db::finish_sub_call(&mut conn, receipt.data().id.clone()).await?;
		Ok(())
	}

	async fn should_vote(&self, receipt: &EthReceipt) -> Result<bool> {
		let status = liberland_utils::storage::status_of(receipt);
		let voting = liberland_utils::storage::voting(receipt);

		let status = self.sub_api.storage().at_latest().await?.fetch(&status).await?;
		if let Some(status) = status {
			if !matches!(status, IncomingReceiptStatus::Processed(_)) {
				tracing::info!("Skipping vote as it's already processed!");
				return Ok(false)
			}
		}

		let voters = self.sub_api.storage().at_latest().await?.fetch(&voting).await?;
		if let Some(voters) = voters {
			if voters.0.contains(self.sub_signer.account_id()) {
				tracing::info!("Skipping vote as we've already voted!");
				return Ok(false)
			}
		}

		Ok(true)
	}

	async fn vote_on_receipt(&self, receipt: &EthReceipt) -> Result<()> {
		let tx = liberland_utils::tx::vote_withdraw(&self.sub_api, &self.sub_signer, receipt)
			.await?;

		tracing::info!("Submitting vote");
		let mut conn = self.db.acquire().await?;
		let data = receipt.data();

		// FIXME do we still need this...?
		db::insert_sub_call(
			&mut conn,
			data.id.clone(),
			&self.id,
			receipt.bridge_id(),
			data.eth_block_number,
			data.amount,
			data.substrate_recipient.clone(),
		)
		.await?;

		self.sub_tx_manager.add(tx).await?;
		tracing::info!("Submitted vote, waiting for finalization");
		Ok(())
	}
}
