use crate::Result;
use std::sync::Arc;

use futures::{stream::FuturesUnordered, StreamExt};
use sp_core::sr25519::Pair as SubstratePair;
use subxt::{
	config::Hasher,
	tx::{PairSigner, TxPayload, TxProgress},
	utils::Encoded,
	OnlineClient, SubstrateConfig,
};
use tokio::sync::{mpsc, Mutex as TokioMutex};

pub struct Substrate {
	sub_api: Arc<OnlineClient<SubstrateConfig>>,
	sub_signer: PairSigner<SubstrateConfig, SubstratePair>,
	new_tx_recv: TokioMutex<mpsc::Receiver<Vec<u8>>>,
	pub new_tx_send: mpsc::Sender<Vec<u8>>,
}

impl Substrate {
	pub async fn new(
		sub_api: Arc<OnlineClient<SubstrateConfig>>,
		sub_signer: PairSigner<SubstrateConfig, SubstratePair>,
	) -> Result<Self> {
		let (new_tx_send, new_tx_recv) = mpsc::channel(11);
		Ok(Self { sub_api, sub_signer, new_tx_send, new_tx_recv: new_tx_recv.into() })
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

	#[tracing::instrument(skip(self, call))]
	pub async fn add<Call: TxPayload>(&self, call: &Call) -> Result<()> {
		tracing::info!("Submitted transaction");
		let signed_tx = self
			.sub_api
			.tx()
			.create_signed(call, &self.sub_signer, Default::default())
			.await?
			.into_encoded();

		self.new_tx_send.send(signed_tx).await?;
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
