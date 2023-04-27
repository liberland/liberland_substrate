use super::*;

use crate::{
	bridge_abi::BridgeABI,
	sync_managers::{Substrate as SubstrateSyncManager, SubstrateSyncTarget},
	utils::try_to_decode_err,
};
use ethers::{
	middleware::SignerMiddleware,
	prelude::LocalWallet,
	signers::Signer,
	types::Eip1559TransactionRequest,
	utils::{parse_ether, to_checksum},
};
use subxt::events::EventDetails;

use liberland::{
	lld_bridge::events::OutgoingReceipt as LLDEvent,
	llm_bridge::events::OutgoingReceipt as LLMEvent,
};

use tokio::sync::mpsc;

#[derive(Debug)]
pub struct SubstrateToEthereum {
	id: RelayId,
	db: Arc<SqlitePool>,
	eth_signer: Arc<SignerMiddleware<Arc<Provider<Ws>>, LocalWallet>>,
	sub_api: Arc<OnlineClient<SubstrateConfig>>,
	llm_bridge_contract: EthAddress,
	lld_bridge_contract: EthAddress,
	tx_manager_send: mpsc::Sender<(CallId, Eip1559TransactionRequest)>,
	minimum_balance: Amount,
	claim_rewards_threshold: Amount,
	withdraw_rewards_threshold: Amount,
	rewards_address: EthAddress,
}

impl SubstrateToEthereum {
	pub async fn new(
		id: RelayId,
		db: Arc<SqlitePool>,
		eth_provider: Arc<Provider<Ws>>,
		eth_wallet: LocalWallet,
		sub_api: Arc<OnlineClient<SubstrateConfig>>,
		llm_bridge_contract: EthAddress,
		lld_bridge_contract: EthAddress,
		minimum_balance: &str,
		claim_rewards_threshold: &str,
		withdraw_rewards_threshold: &str,
		rewards_address: EthAddress,
		tx_manager_send: mpsc::Sender<(CallId, Eip1559TransactionRequest)>,
	) -> Result<Self> {
		let eth_wallet = eth_wallet.with_chain_id(eth_provider.get_chainid().await?.as_u64());
		let eth_signer = Arc::new(SignerMiddleware::new(eth_provider, eth_wallet));
		let minimum_balance = parse_ether(minimum_balance)?;
		let claim_rewards_threshold = parse_ether(claim_rewards_threshold)?;
		let withdraw_rewards_threshold = parse_ether(withdraw_rewards_threshold)?;

		Ok(Self {
			id,
			db,
			eth_signer,
			sub_api,
			llm_bridge_contract,
			lld_bridge_contract,
			minimum_balance,
			claim_rewards_threshold,
			withdraw_rewards_threshold,
			rewards_address,
			tx_manager_send,
		})
	}

	#[tracing::instrument(skip_all,fields(id = ?self.id))]
	pub async fn run(self) -> Result<()> {
		let address = to_checksum(&self.eth_signer.address(), None);
		tracing::info!(address, "Started Substrate -> Ethereum relay");
		tokio::select! {
			r = self.sync() => r?,
			r = self.claim_rewards(self.llm_bridge_contract) => r?,
			r = self.claim_rewards(self.lld_bridge_contract) => r?,
			r = self.withdraw_rewards() => r?,
		};
		Err(eyre::eyre!("Relay task finished early!"))
	}

	#[tracing::instrument(skip(self))]
	pub async fn withdraw_rewards(&self) -> Result<()> {
		let relay_address = self.eth_signer.address();

		let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600 * 1));
		interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

		let pretty_minimum_balance = format_units(self.minimum_balance, "ether")?;
		let pretty_withdraw_rewards_threshold =
			format_units(self.withdraw_rewards_threshold, "ether")?;

		loop {
			interval.tick().await;

			let account_balance = self.eth_signer.get_balance(relay_address, None).await?;
			if account_balance >= self.withdraw_rewards_threshold {
				let withdraw_amount = account_balance.saturating_sub(self.minimum_balance);
				if !withdraw_amount.is_zero() {
					tracing::info!(
						"Accumulated rewards is {} ETH, withdrawing {} ETH.",
						format_units(account_balance, "ether")?,
						format_units(withdraw_amount, "ether")?
					);
					let tx = Eip1559TransactionRequest::new()
						.to(self.rewards_address)
						.value(withdraw_amount);
					self.tx_manager_send.send((CallId::random(), tx.into())).await?;
				} else {
					tracing::debug!(
						"Accumulated {} ETH of rewards, but it's less than the minimum of {} ETH - not withdrawing",
						format_units(account_balance, "ether")?,
						pretty_minimum_balance,
					);
				}
			} else {
				tracing::debug!(
					"Accumulated {} ETH of rewards, but the threshold is {} ETH - not withdrawing",
					format_units(account_balance, "ether")?,
					pretty_withdraw_rewards_threshold,
				);
			}
		}
	}

	#[tracing::instrument(skip(self))]
	pub async fn claim_rewards(&self, bridge_address: EthAddress) -> Result<()> {
		let bridge = BridgeABI::new(bridge_address, self.eth_signer.clone());
		let relay_address = self.eth_signer.address();

		let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600 * 1));
		interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

		let pretty_claim_rewards_threshold = format_units(self.claim_rewards_threshold, "ether")?;

		loop {
			interval.tick().await;

			let pending_rewards = bridge.pending_rewards(relay_address).call().await?;
			let pretty_pending = format_units(pending_rewards, "ether")?;
			if pending_rewards >= self.claim_rewards_threshold {
				tracing::info!("Pending rewards is {pretty_pending} ETH - claiming.");
				let tx = bridge.claim_reward();
				self.tx_manager_send.send((CallId::random(), tx.tx.into())).await?;
			} else {
				tracing::debug!("Pending rewards is {pretty_pending} ETH, while claim threshold is set to {pretty_claim_rewards_threshold} ETH - not claiming yet.");
			}
		}
	}

	pub async fn sync(&self) -> Result<()> {
		let sync_manager = SubstrateSyncManager::new(
			(&self.id).into(),
			self.db.clone(),
			self.sub_api.clone(),
			SubstrateSyncTarget::Finalized,
		);
		let subscription = sync_manager.sync();
		pin_mut!(subscription);
		while let Some(event) = subscription.next().await {
			let (block_number, block_hash, event) = event?;
			self.process_raw_event(block_number, block_hash, event).await?;
		}

		Ok(())
	}

	async fn process_raw_event(
		&self,
		block_number: BlockNumber,
		block_hash: H256,
		event: EventDetails,
	) -> Result<()> {
		let idx = event.index();
		if let Ok(Some(event)) = event.as_event::<LLDEvent>() {
			self.process_event(
				self.lld_bridge_contract,
				block_number,
				block_hash,
				idx,
				event.amount.into(),
				event.eth_recipient.into(),
			)
			.await?;
		} else if let Ok(Some(event)) = event.as_event::<LLMEvent>() {
			self.process_event(
				self.llm_bridge_contract,
				block_number,
				block_hash,
				idx,
				event.amount.into(),
				event.eth_recipient.into(),
			)
			.await?;
		} else {
			tracing::trace!("Non-bridge event skipped: {:?}", event);
		}
		Ok(())
	}

	async fn process_event(
		&self,
		bridge: EthAddress,
		block_number: BlockNumber,
		block_hash: <SubstrateConfig as subxt::config::Config>::Hash,
		idx: u32,
		amount: Amount,
		eth_recipient: EthAddress,
	) -> Result<()> {
		let receipt_id = self.receipt_id(block_hash, idx, amount, &eth_recipient);
		if self.should_vote(bridge, &receipt_id).await? {
			self.vote(bridge, &receipt_id, block_number, amount, eth_recipient).await?;
		} else {
			tracing::info!(
				"Skipping vote on {receipt_id:?} as it's already approved or we've already voted!"
			);
		}
		Ok(())
	}

	async fn should_vote(&self, contract: EthAddress, receipt_id: &ReceiptId) -> Result<bool> {
		let bridge = BridgeABI::new(contract, self.eth_signer.clone());

		if bridge.voted(receipt_id.into(), self.eth_signer.address()).call().await? {
			// We've already voted on this receipt - probably sqlite db got wiped and we're
			// resyncing
			return Ok(false)
		}

		let (_substrate_block_number, _recipient, _amount, _approved_on, processed_on) =
			bridge.incoming_receipts(receipt_id.into()).call().await?;

		// skip voting if other receipt already processed
		Ok(processed_on.is_zero())
	}

	#[tracing::instrument(skip(self))]
	async fn vote(
		&self,
		contract: EthAddress,
		receipt_id: &ReceiptId,
		block_number: BlockNumber,
		amount: Amount,
		recipient: EthAddress,
	) -> Result<()> {
		let bridge = BridgeABI::new(contract, self.eth_signer.clone());
		let tx = bridge.vote_mint(receipt_id.into(), block_number, amount, recipient);
		tracing::info!("Submitting vote",);
		tracing::debug!("Simulating tx...");

		tx.call().await.map_err(try_to_decode_err)?;

		tracing::debug!("Queuing tx...");
		self.tx_manager_send.send((receipt_id.clone().into(), tx.tx.into())).await?;
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
}
