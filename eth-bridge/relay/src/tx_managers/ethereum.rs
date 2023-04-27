use crate::{
	bail,
	bridge_abi::BridgeABIErrors,
	db,
	types::{Amount, CallId, EthTrackedTx},
	Result,
};
use ethers::{
	middleware::MiddlewareBuilder,
	prelude::{
		LocalWallet, Middleware, MiddlewareError, NonceManagerMiddleware, PendingTransaction,
		Provider, SignerMiddleware, Ws,
	},
	types::{
		Address as EthAddress, BlockId, BlockNumber, Eip1559TransactionRequest, TransactionReceipt,
	},
};
use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use primitive_types::{H256, U256};
use sqlx::{Acquire, SqlitePool};
use std::{
	collections::VecDeque,
	sync::{Arc, Mutex},
};
use tokio::{
	sync::{mpsc, Mutex as TokioMutex},
	time::{timeout, Duration},
};

#[derive(Debug)]
struct TrackedCall {
	id: CallId,
	request: Eip1559TransactionRequest,
	pendings: Vec<EthTrackedTx>,
	new_pending_recv: mpsc::Receiver<()>,
	new_pending_send: mpsc::Sender<()>,
}

pub struct Ethereum {
	db: Arc<SqlitePool>,
	eth_signer: NonceManagerMiddleware<Arc<SignerMiddleware<Arc<Provider<Ws>>, LocalWallet>>>,
	calls: Arc<Mutex<VecDeque<TrackedCall>>>,
	max_gas_price: Amount,
	new_call_recv: TokioMutex<mpsc::Receiver<()>>,
	new_call_send: mpsc::Sender<()>,
	external_new_call_recv: TokioMutex<mpsc::Receiver<(CallId, Eip1559TransactionRequest)>>,
	pub external_new_call_send: mpsc::Sender<(CallId, Eip1559TransactionRequest)>,
}

impl Ethereum {
	pub async fn new(
		db: Arc<SqlitePool>,
		eth_signer: Arc<SignerMiddleware<Arc<Provider<Ws>>, LocalWallet>>,
		max_gas_price: Amount,
	) -> Result<Self> {
		let address = eth_signer.address();

		let calls = Arc::new(Mutex::new(Self::read_calls_from_db(&address, &db).await?));

		let eth_signer = eth_signer.nonce_manager(address);
		eth_signer.initialize_nonce(None).await?;
		let (new_call_send, new_call_recv) = mpsc::channel(10);
		let new_call_recv = new_call_recv.into();
		let (external_new_call_send, external_new_call_recv) = mpsc::channel(10);
		let external_new_call_recv = external_new_call_recv.into();
		Ok(Self {
			db,
			eth_signer,
			calls,
			max_gas_price,
			new_call_recv,
			new_call_send,
			external_new_call_recv,
			external_new_call_send,
		})
	}

	async fn read_calls_from_db(
		signer_address: &EthAddress,
		pool: &Arc<SqlitePool>,
	) -> Result<VecDeque<TrackedCall>> {
		let db_calls = db::get_unfinished_eth_calls(pool, signer_address).await?;
		Ok(db_calls
			.into_iter()
			.map(|db_call| {
				let (new_pending_send, new_pending_recv) = mpsc::channel(10);
				TrackedCall {
					id: db_call.call_id.into(),
					request: db_call.request,
					pendings: db_call.pendings,
					new_pending_recv,
					new_pending_send,
				}
			})
			.collect())
	}

	#[tracing::instrument(skip(self, request))]
	async fn add(&self, id: CallId, request: Eip1559TransactionRequest) -> Result<()> {
		let mut conn = self.db.acquire().await?;
		let (max_fee_per_gas, max_priority_fee_per_gas) =
			self.eth_signer.estimate_eip1559_fees(None).await?;
		let request = request
			.max_fee_per_gas(max_fee_per_gas)
			.max_priority_fee_per_gas(max_priority_fee_per_gas)
			.nonce(self.eth_signer.next());
		let pending = self.eth_signer.send_transaction(request.clone(), None).await?;
		let hash = pending.tx_hash();
		let pendings = vec![EthTrackedTx { hash, max_fee_per_gas }];
		let (new_pending_send, new_pending_recv) = mpsc::channel(10);

		let mut db_tx = conn.begin().await?;
		db::insert_eth_call(&mut db_tx, &id, &self.eth_signer.inner().address(), &request).await?;
		db::insert_eth_tx(&mut db_tx, &id, &hash, max_fee_per_gas).await?;
		db_tx.commit().await?;

		let tracked = TrackedCall { id, request, pendings, new_pending_send, new_pending_recv };
		tracing::info!("Submitted transaction");
		self.calls.lock().unwrap().push_back(tracked);
		self.new_call_send.send(()).await?;
		Ok(())
	}

	async fn process_calls(&self) -> Result<()> {
		loop {
			let maybe_call = self.calls.lock().unwrap().pop_front();
			if let Some(call) = maybe_call {
				self.process_call(call).await?;
			} else {
				self.new_call_recv.lock().await.recv().await;
			}
		}
	}

	pub async fn process(&self) -> Result<()> {
		let (call_processor, call_processor_handle) = self.process_calls().remote_handle();
		let call_processor = call_processor.shared();

		// we'll be the only one to hold this
		let mut recv = self.external_new_call_recv.lock().await;

		loop {
			tokio::select! {
				_ = call_processor.clone() => {
					call_processor_handle.await?;
					bail!("Tx manager internal call processor ended unexpectedly!");
				},
				maybe_call = recv.recv() => {
					if let Some((id, request)) = maybe_call {
						self.add(id, request).await?;
					} else {
						bail!("Tx manager external sender dropped unexpectedly!");
					}
				}
			}
		}
	}

	#[tracing::instrument(skip_all,fields(call_id = ?call.id))]
	async fn process_call(&self, call: TrackedCall) -> Result<()> {
		let mut call = call;
		loop {
			let futures = call.pendings.clone().into_iter().map(|ttx| {
				PendingTransaction::new(ttx.hash, self.eth_signer.provider())
					.map(move |r| (ttx.hash, ttx.max_fee_per_gas, r))
			});
			let mut futures: FuturesUnordered<_> = futures.collect();
			tracing::debug!("Monitoring {:?} pendings for call", futures.len());
			tokio::select! {
				_ = call.new_pending_recv.recv() => {
					tracing::debug!("Got new pending, restarting monitoring");
				},
				// FIXME timeout must be higher that PendingTransaction poll interval, enforce that!
				// FIXME configurable timeout (a.k.a. bump interval)
				res = timeout(Duration::from_secs(30), futures.next()) => {
					match res {
						Err(_) => {
							// Timed out, we should bump fee
							self.maybe_bump_call(&mut call).await?;
						},
						Ok(Some((_tx_hash, _max_fee_per_gas, Err(e)))) => {
							// One of pendings finished with provider error
							tracing::error!("Provider error for {:?}", call.id);
							return Err(e.into());
						},
						Ok(Some((_tx_hash, _max_fee_per_gas, Ok(Some(receipt))))) => {
							// Transaction receipt, actually confirmed!
							self.finish_call(&call, receipt).await?;
							return Ok(());
						},
						Ok(Some((tx_hash, max_fee_per_gas, Ok(None)))) => {
							// Tx removed from mempool
							if max_fee_per_gas >= call.request.max_fee_per_gas.expect("Impossible missing max_fee_per_gas") {
								// and it was actually our highest gas tx
								self.maybe_bump_call(&mut call).await?;
							}
							self.drop_hash(&mut call, &tx_hash).await?
						},
						Ok(None) => {
							// all pendings finished. should never happen, as we add new ones till we get a success and we return on success
							tracing::error!("Unexpectedly run out of transactions for call!");
							panic!("Unexpectedly run out of transactions for call!");
						},
					}
				}
			}
		}
	}

	async fn finish_call(&self, call: &TrackedCall, receipt: TransactionReceipt) -> Result<()> {
		db::finish_eth_call(&self.db, &call.id, &receipt.transaction_hash).await?;

		tracing::info!(hash=%receipt.transaction_hash, "Successfully finished eth call!");
		Ok(())
	}

	async fn maybe_bump_call(&self, call: &mut TrackedCall) -> Result<()> {
		let mut conn = self.db.acquire().await?;

		let new_request = call.request.clone();
		let old_max_priority_fee = new_request
			.max_priority_fee_per_gas
			.expect("Tx unexpectedly missing max_priority_fee_per_gas")
			.as_u64() as f64;
		let new_max_priority_fee = (old_max_priority_fee * 1.13).ceil();
		let new_max_priority_fee = U256::from(new_max_priority_fee as u64);
		let new_request = new_request.max_priority_fee_per_gas(new_max_priority_fee);

		let old_max_fee = new_request
			.max_fee_per_gas
			.expect("Tx unexpectedly missing max_fee_per_gas")
			.as_u64() as f64;
		let new_max_fee = U256::from((old_max_fee * 1.13).ceil() as u64);
		let new_request = new_request.max_fee_per_gas(new_max_fee);
		call.request = new_request;

		if new_max_fee + new_max_priority_fee > self.max_gas_price {
			tracing::warn!("Not bumping long-pending tx as we've reached the max gas price limit!");
			return Ok(())
		}

		tracing::info!("Submitting tx with bumped fee per gas - old fee = {old_max_fee}, new fee = {new_max_fee}");

		let pending = self
			.eth_signer
			.send_transaction(call.request.clone(), Some(BlockId::Number(BlockNumber::Latest)))
			.await;

		match pending {
			Err(outer_err) => {
				if let Some(e) = outer_err.as_error_response() {
					if let Some(BridgeABIErrors::AlreadyVoted(_)) =
						e.decode_revert_data::<BridgeABIErrors>()
					{
						// looks like one of previous txses got confirmed and we just didn't notice
						// it yet lets return OK without actual bump, we'll notice it soon
						tracing::debug!("Got AlreadyVoted when trying to bump.");
						Ok(())
					} else {
						Err(e.clone().into())
					}
				} else {
					Err(outer_err.into())
				}
			},
			Ok(pending) => {
				let hash = pending.tx_hash();
				call.pendings.push(EthTrackedTx { hash, max_fee_per_gas: new_max_fee });
				call.new_pending_send.send(()).await?;

				let mut db_tx = conn.begin().await?;
				db::set_eth_call_request(&mut db_tx, &call.id, &call.request).await?;
				db::insert_eth_tx(&mut db_tx, &call.id, &hash, new_max_fee).await?;
				db_tx.commit().await?;

				Ok(())
			},
		}
	}

	async fn drop_hash(&self, call: &mut TrackedCall, hash: &H256) -> Result<()> {
		let idx = call.pendings.iter().position(|ttx| ttx.hash == *hash);
		if let Some(idx) = idx {
			call.pendings.remove(idx);
		}

		let mut conn = self.db.acquire().await?;
		db::drop_eth_tx(&mut conn, hash).await?;

		tracing::debug!(%hash, "Eth tx dropped.");
		Ok(())
	}
}
