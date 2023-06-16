use crate::{
	liberland_api::api::{
		self,
		runtime_types::{pallet_federated_bridge::*, sp_core::bounded::bounded_vec::BoundedVec},
	},
	types::*,
	Result,
};
use sp_core::sr25519::Pair;
use std::sync::Arc;
use subxt::{
	storage::address::{StorageAddress, Yes},
	tx::{PairSigner, SubmittableExtrinsic},
	utils::AccountId32,
	OnlineClient, SubstrateConfig,
};

pub mod storage {
	use super::*;

	pub fn status_of(
		receipt: &EthReceipt,
	) -> impl StorageAddress<Target = IncomingReceiptStatus<u32>, IsFetchable = Yes> {
		let receipt_id: [u8; 32] = receipt.data().id.clone().into();
		match receipt.bridge_id() {
			BridgeId::LLM => api::storage().llm_bridge().status_of(receipt_id),
			BridgeId::LLD => api::storage().lld_bridge().status_of(receipt_id),
		}
	}

	pub fn voting(
		receipt: &EthReceipt,
	) -> impl StorageAddress<Target = BoundedVec<AccountId32>, IsFetchable = Yes> {
		let receipt_id: [u8; 32] = receipt.data().id.clone().into();
		match receipt.bridge_id() {
			BridgeId::LLM => api::storage().llm_bridge().voting(receipt_id),
			BridgeId::LLD => api::storage().lld_bridge().voting(receipt_id),
		}
	}

	pub fn incoming_receipts(
		bridge: BridgeId,
		receipt_id: &ReceiptId,
	) -> impl StorageAddress<Target = IncomingReceipt<AccountId32, u128>, IsFetchable = Yes> {
		let receipt_id: [u8; 32] = receipt_id.clone().into();
		match bridge {
			BridgeId::LLM => api::storage().llm_bridge().incoming_receipts(receipt_id),
			BridgeId::LLD => api::storage().lld_bridge().incoming_receipts(receipt_id),
		}
	}
}

pub mod tx {
	use super::*;
	pub async fn vote_withdraw(
		sub_api: &Arc<OnlineClient<SubstrateConfig>>,
		signer: &PairSigner<SubstrateConfig, Pair>,
		receipt: &EthReceipt,
	) -> Result<SubmittableExtrinsic<SubstrateConfig, OnlineClient<SubstrateConfig>>> {
		let receipt_id: [u8; 32] = receipt.data().id.clone().into();
		let data = receipt.data().into();
		Ok(match receipt.bridge_id() {
			BridgeId::LLM =>
				sub_api
					.tx()
					.create_signed(
						&api::tx().llm_bridge().vote_withdraw(receipt_id, data),
						signer,
						Default::default(),
					)
					.await?,
			BridgeId::LLD =>
				sub_api
					.tx()
					.create_signed(
						&api::tx().lld_bridge().vote_withdraw(receipt_id, data),
						signer,
						Default::default(),
					)
					.await?,
		})
	}

	pub async fn emergency_stop(
		sub_api: &Arc<OnlineClient<SubstrateConfig>>,
		signer: &PairSigner<SubstrateConfig, Pair>,
		bridge: BridgeId,
	) -> Result<SubmittableExtrinsic<SubstrateConfig, OnlineClient<SubstrateConfig>>> {
		Ok(match bridge {
			BridgeId::LLM =>
				sub_api
					.tx()
					.create_signed(
						&api::tx().llm_bridge().emergency_stop(),
						signer,
						Default::default(),
					)
					.await?,
			BridgeId::LLD =>
				sub_api
					.tx()
					.create_signed(
						&api::tx().lld_bridge().emergency_stop(),
						signer,
						Default::default(),
					)
					.await?,
		})
	}
}
