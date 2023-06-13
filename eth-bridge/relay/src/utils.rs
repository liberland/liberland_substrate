use crate::{
	bridge_abi::BridgeABIErrors,
	types::{Amount, ReceiptId},
	Result,
};
use ethers::{
	abi::AbiDecode,
	prelude::{k256::ecdsa::SigningKey, ContractError, SignerMiddleware},
	providers::{Provider, Ws},
	signers::Wallet,
};
use primitive_types::{H160, H256};
use std::{fmt::Debug, future::Future, sync::Arc};
use subxt::{
	config::{substrate::BlakeTwo256, Hasher},
	SubstrateConfig,
};
use tokio::time::{interval, Duration};

type AccountId = <SubstrateConfig as subxt::config::Config>::AccountId;

pub async fn wait_for<T, F, R, E>(err_msg: &str, f: F) -> T
where
	F: Fn() -> R,
	R: Future<Output = Result<T, E>>,
	E: Debug,
{
	let mut interval = interval(Duration::from_secs(5));
	loop {
		interval.tick().await;
		match f().await {
			Ok(res) => return res,
			Err(e) => {
				tracing::warn!("{err_msg} Error: {e:?}");
			},
		}
	}
}

pub fn try_to_decode_err(
	err: ContractError<SignerMiddleware<Arc<Provider<Ws>>, Wallet<SigningKey>>>,
) -> eyre::Report {
	if let Some(bytes) = err.as_revert() {
		let abi_err = match BridgeABIErrors::decode(bytes) {
			Ok(data) => data,
			Err(err) => return err.into(),
		};
		return eyre::eyre!(format!("Call reverted with: {:?}", abi_err))
	}
	return err.into()
}

pub fn substrate_receipt_id(
	block: <SubstrateConfig as subxt::config::Config>::Hash,
	idx: u32,
	amount: Amount,
	recipient: &H160,
) -> ReceiptId {
	let mut amount_bytes: [u8; 32] = Default::default();
	amount.to_big_endian(&mut amount_bytes);

	BlakeTwo256::hash(
		&[block.as_bytes(), &idx.to_be_bytes(), &amount_bytes, recipient.as_bytes()].concat(),
	)
	.into()
}

pub fn eth_receipt_id(
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
