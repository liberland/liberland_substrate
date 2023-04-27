use ethers::{
	abi::AbiDecode,
	prelude::{k256::ecdsa::SigningKey, ContractError, SignerMiddleware},
	providers::{Provider, Ws},
	signers::Wallet,
};
use std::{fmt::Debug, future::Future, sync::Arc};
use tokio::time::{interval, Duration};

use crate::{bridge_abi::BridgeABIErrors, Result};

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
