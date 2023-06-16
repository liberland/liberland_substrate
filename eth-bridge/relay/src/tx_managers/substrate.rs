use crate::Result;
use std::sync::Arc;

use futures::{stream::FuturesUnordered, StreamExt};
use subxt::{
	config::Hasher,
	tx::{SubmittableExtrinsic, TxProgress},
	utils::Encoded,
	OnlineClient, SubstrateConfig,
};
use tokio::sync::{mpsc, Mutex as TokioMutex};

pub struct Substrate {
	sub_api: Arc<OnlineClient<SubstrateConfig>>,
	new_tx_recv: TokioMutex<mpsc::Receiver<Vec<u8>>>,
	pub new_tx_send: mpsc::Sender<Vec<u8>>,
}

impl Substrate {
	pub async fn new(
		sub_api: Arc<OnlineClient<SubstrateConfig>>,
	) -> Result<Self> {
		let (new_tx_send, new_tx_recv) = mpsc::channel(11);
		Ok(Self { sub_api, new_tx_send, new_tx_recv: new_tx_recv.into() })
	}

	pub async fn execute_and_watch_encoded(
		&self,
		encoded_tx: Encoded,
	) -> Result<TxProgress<SubstrateConfig, OnlineClient<SubstrateConfig>>> {
		let sub_api = self.sub_api.as_ref().clone();
		let tx_hash = <SubstrateConfig as subxt::config::Config>::Hasher::hash_of(&encoded_tx);

		let sub = self.sub_api.rpc().watch_extrinsic(encoded_tx).await?;
		Ok(TxProgress::new(sub, sub_api, tx_hash))
	}

	#[tracing::instrument(skip_all)]
	pub async fn add(
		&self,
		extrinsic: SubmittableExtrinsic<SubstrateConfig, OnlineClient<SubstrateConfig>>,
	) -> Result<()> {
		tracing::info!("Submitted transaction");
		let encoded = extrinsic.into_encoded();
		self.new_tx_send.send(encoded).await?;
		Ok(())
	}

	pub async fn process(&self) -> Result<()> {
		tracing::info!("Substrate tx manager started!");
		let mut futures = FuturesUnordered::new();

		let mut recv = self.new_tx_recv.lock().await;
		loop {
			tokio::select! {
				Some(signed_tx) = recv.recv() => {
					let tx_progress = self.execute_and_watch_encoded(Encoded(signed_tx)).await?;
					futures.push(tx_progress.wait_for_finalized_success());
				},
				Some(future) = futures.next() => {
					let tx_result = future?;
					tracing::info!("Executed tx at: {}", tx_result.block_hash());
				}
			}
		}
	}
}
