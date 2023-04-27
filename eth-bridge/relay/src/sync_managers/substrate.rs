use super::*;

use subxt::{
	events::{EventDetails, Events},
	OnlineClient, SubstrateConfig,
};

#[derive(Debug)]
pub enum SubstrateSyncTarget {
	Finalized,
	Best,
}

#[derive(Debug)]
pub struct Substrate {
	id: TaskId,
	db: Arc<SqlitePool>,
	sub_api: Arc<OnlineClient<SubstrateConfig>>,
	target: SubstrateSyncTarget,
}

type SubstrateHash = <SubstrateConfig as subxt::config::Config>::Hash;
type Log = (BlockNumber, SubstrateHash, EventDetails);

impl Substrate {
	pub fn new(
		id: TaskId,
		db: Arc<SqlitePool>,
		sub_api: Arc<OnlineClient<SubstrateConfig>>,
		target: SubstrateSyncTarget,
	) -> Self {
		Self { id, db, sub_api, target }
	}

	pub fn sync(&self) -> impl Stream<Item = Result<Log>> + '_ {
		try_stream! {
			// get mostly up to date
			tracing::info!("Starting initial sync.");
			let mut synced_block = 0;
			for await log in self.batch_sync() {
				let log = log?;
				let log_block = log.0;
				yield log;
				synced_block = log_block;
			}

			// start collecting all new blocks
			let mut block_sub = match self.target {
				SubstrateSyncTarget::Best => self.sub_api.blocks().subscribe_best().await?,
				SubstrateSyncTarget::Finalized => self.sub_api.blocks().subscribe_finalized().await?,
			};

			// make sure we've synced everything between rough resync and subscription start
			for await log in self.batch_sync() {
				let log = log?;
				let log_block = log.0;
				yield log;
				synced_block = log_block;
			}

			// we're in sync
			tracing::info!("Synced, starting realtime monitoring.");
			let mut conn = self.db.acquire().await?;
			while let Some(block) = block_sub.next().await {
				let block = block?;
				let number: BlockNumber = block.header().number.into();
				let is_fresh = number > synced_block;
				if is_fresh {
					tracing::debug!("Received block {}", number);
					let events = block.events().await?;
					let hash = events.block_hash();
					for log in events.iter() {
						yield (number, hash, log?);
					}
					db::set_synced_block(&mut conn, &self.id, number).await?;
				}
			}
		}
	}

	async fn get_target_block_number(&self) -> Result<BlockNumber> {
		Ok(match self.target {
			SubstrateSyncTarget::Best => self
				.sub_api
				.rpc()
				.header(None)
				.await?
				.expect("Best block doesn't have a header?!")
				.number
				.into(),
			SubstrateSyncTarget::Finalized => self
				.sub_api
				.rpc()
				.header(Some(self.sub_api.rpc().finalized_head().await?))
				.await?
				.expect("Finalized block doesn't have a header?!")
				.number
				.into(),
		})
	}

	fn batch_sync(&self) -> impl Stream<Item = Result<Log>> + '_ {
		try_stream! {
			let mut conn = self.db.acquire().await?;
			let mut synced_block = db::get_synced_block(&mut conn, &self.id).await?;
			let mut target_block = self.get_target_block_number().await?;

			while target_block > synced_block {
				for i in (synced_block..=target_block).step_by(100) {
					let batch_end = std::cmp::min(i + 99, target_block);
					tracing::info!("Syncing blocks {i}..{batch_end}");
					for i in i..=batch_end {
						let events = self.sync_block(i).await?;
						for event in events.iter() {
							yield (i, events.block_hash(), event?);
						}
					}
					db::set_synced_block(&mut conn, &self.id, batch_end).await?;
					synced_block = batch_end;
				}

				target_block = self.get_target_block_number().await?;
			}
		}
	}

	async fn sync_block(&self, number: BlockNumber) -> Result<Events<SubstrateConfig>> {
		let hash = self
			.sub_api
			.rpc()
			.block_hash(Some(number.into()))
			.await?
			.expect("Block number {number} doesn't have a hash?!");
		Ok(self.sub_api.events().at(hash).await?)
	}
}
