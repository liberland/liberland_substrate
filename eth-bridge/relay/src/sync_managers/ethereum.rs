use super::*;

use crate::settings::EthAddress;
use ethers::{
	prelude::{Provider, Ws},
	providers::Middleware,
	types::{BlockNumber, Filter, Log, U64},
};

#[derive(Debug)]
pub enum EthereumSyncTarget {
	Latest,
	Finalized,
}

pub struct Ethereum {
	id: TaskId,
	db: Arc<SqlitePool>,
	eth_provider: Arc<Provider<Ws>>,
	llm_bridge_contract: EthAddress,
	lld_bridge_contract: EthAddress,
	target: EthereumSyncTarget,
	signature: String,
}

impl Ethereum {
	pub fn new(
		id: TaskId,
		db: Arc<SqlitePool>,
		eth_provider: Arc<Provider<Ws>>,
		llm_bridge_contract: EthAddress,
		lld_bridge_contract: EthAddress,
		target: EthereumSyncTarget,
		signature: &str,
	) -> Self {
		let signature = signature.to_string();
		Self { id, db, eth_provider, llm_bridge_contract, lld_bridge_contract, target, signature }
	}

	pub fn sync(&self) -> impl Stream<Item = Result<Log>> + '_ {
		try_stream! {
			let mut conn = self.db.acquire().await?;
			let target = self.get_target_block_number().await?;
			let from_block = db::get_synced_block(&mut conn, &self.id).await?;

			tracing::info!("Starting batch sync of blocks {}..{}", from_block, target,);

			let mut filter = Filter::new()
				.event(&self.signature)
				.address(vec![self.lld_bridge_contract, self.llm_bridge_contract])
				.from_block(from_block)
				.to_block(target);

			for await log in self.batch_sync(&filter) {
				let log = log?;
				yield log;
			};

			tracing::info!("Synced, starting realtime monitoring");
			let mut last_synced = target;
			let mut subscription = self.eth_provider.subscribe_blocks().await?;
			while subscription.next().await.is_some() {
				let target = self.get_target_block_number().await?;
				if target > last_synced {
					tracing::debug!("Received block {target}");
					filter = filter.from_block(last_synced + 1).to_block(target);
					for log in self.eth_provider.get_logs(&filter).await? {
						yield log;
					}
					db::set_synced_block(&mut conn, &self.id, target.as_u64()).await?;
					last_synced = target;
				} else {
					tracing::debug!("Received new block, current target is {target}. Ignoring as we're already synced up to {last_synced}");
				}
			}
		}
	}

	async fn get_target_block_number(&self) -> Result<U64> {
		let target = match self.target {
			EthereumSyncTarget::Latest => BlockNumber::Latest,
			EthereumSyncTarget::Finalized => BlockNumber::Finalized,
		};

		Ok(self
			.eth_provider
			.get_block(target)
			.await?
			.expect("No sync target block?")
			.number
			.expect("Sync target block with no number?"))
	}

	fn batch_sync<'a>(&'a self, filter: &'a Filter) -> impl Stream<Item = Result<Log>> + 'a {
		try_stream! {
			let mut conn = self.db.acquire().await?;
			// FIXME we should fetch logs in chunks
			let logs = self.eth_provider.get_logs(filter).await?;
			for log in logs {
				let block_number = log.block_number.expect("Missing block number");
				yield log;
				db::set_synced_block(&mut conn, &self.id, block_number.as_u64()).await?;
			}
			let synced_block_number = filter
				.get_to_block()
				.expect("ToBlock should always be set on batch_sync filter");
			db::set_synced_block(&mut conn, &self.id, synced_block_number.as_u64()).await?;
		}
	}
}
