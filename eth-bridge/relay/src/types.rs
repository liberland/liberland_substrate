//! Special types + conversion between them, substrate types and eth types
//! Conversion to DB types is in db module

use crate::{bail, liberland::runtime_types::pallet_federated_bridge::IncomingReceipt, Result};
use ethers::abi::AbiEncode;
pub use primitive_types::{H256, U256};
use rand::{thread_rng, Rng};
use subxt::utils::AccountId32;

// generic block number for both networks
pub type BlockNumber = u64; // FIXME newtype maybe?

// generic amount for both networks
pub type Amount = U256; // FIXME newtype maybe?

#[derive(Clone, Debug, derive_more::From, derive_more::Into)]
pub struct RelayId(String);

#[derive(Clone, Debug, derive_more::From, derive_more::Into)]
pub struct WatcherId(String);

#[derive(Clone, Debug, derive_more::From, derive_more::Into)]
pub struct TaskId(String);

impl From<&RelayId> for TaskId {
	fn from(r: &RelayId) -> Self {
		Self(r.clone().into())
	}
}

impl From<&WatcherId> for TaskId {
	fn from(r: &WatcherId) -> Self {
		Self(r.clone().into())
	}
}

#[derive(PartialEq, Clone, derive_more::From, derive_more::Into)]
pub struct ReceiptId(H256);

impl std::fmt::Debug for ReceiptId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0.encode_hex())
	}
}

impl ReceiptId {
	pub fn encode_hex(&self) -> String {
		self.0.encode_hex()
	}
}

impl From<[u8; 32]> for ReceiptId {
	fn from(d: [u8; 32]) -> Self {
		Self(d.into())
	}
}

impl From<&ReceiptId> for [u8; 32] {
	fn from(d: &ReceiptId) -> Self {
		*d.0.as_fixed_bytes()
	}
}

impl From<ReceiptId> for [u8; 32] {
	fn from(d: ReceiptId) -> Self {
		*d.0.as_fixed_bytes()
	}
}

impl From<ReceiptId> for CallId {
	fn from(d: ReceiptId) -> Self {
		Self(d.0)
	}
}

impl From<CallId> for ReceiptId {
	fn from(d: CallId) -> Self {
		Self(d.0)
	}
}

#[derive(Clone, Debug, derive_more::From, derive_more::Into)]
pub struct CallId(H256);

impl CallId {
	pub fn random() -> Self {
		let bytes: [u8; 32] = thread_rng().gen();
		Self(H256(bytes))
	}

	pub fn encode_hex(&self) -> String {
		self.0.encode_hex()
	}
}

pub struct EthReceiptData {
	pub id: ReceiptId,
	pub eth_block_number: BlockNumber,
	pub amount: Amount,
	pub substrate_recipient: AccountId32,
}

impl From<&EthReceiptData> for IncomingReceipt<AccountId32, u128> {
	fn from(d: &EthReceiptData) -> Self {
		IncomingReceipt {
			eth_block_number: d.eth_block_number,
			substrate_recipient: d.substrate_recipient.clone(),
			amount: d.amount.as_u128(),
		}
	}
}

pub enum EthReceipt {
	LLM(EthReceiptData),
	LLD(EthReceiptData),
}

impl EthReceipt {
	pub fn new(
		bridge_id: BridgeId,
		id: ReceiptId,
		eth_block_number: BlockNumber,
		amount: Amount,
		substrate_recipient: AccountId32,
	) -> EthReceipt {
		let receipt_data = EthReceiptData { id, eth_block_number, amount, substrate_recipient };
		match bridge_id {
			BridgeId::LLM => EthReceipt::LLM(receipt_data),
			BridgeId::LLD => EthReceipt::LLD(receipt_data),
		}
	}

	pub fn data(&self) -> &EthReceiptData {
		match self {
			Self::LLM(data) => data,
			Self::LLD(data) => data,
		}
	}

	pub fn bridge_id(&self) -> BridgeId {
		match self {
			Self::LLM(_) => BridgeId::LLM,
			Self::LLD(_) => BridgeId::LLD,
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub enum BridgeId {
	LLM,
	LLD,
}

impl From<BridgeId> for &str {
	fn from(s: BridgeId) -> &'static str {
		match s {
			BridgeId::LLM => "LLM",
			BridgeId::LLD => "LLD",
		}
	}
}

impl TryFrom<String> for BridgeId {
	type Error = eyre::Report;

	fn try_from(s: String) -> Result<Self> {
		s.as_str().try_into()
	}
}

impl TryFrom<&str> for BridgeId {
	type Error = eyre::Report;

	fn try_from(s: &str) -> Result<Self> {
		match s {
			x if x == "LLM" => Ok(Self::LLM),
			x if x == "LLD" => Ok(Self::LLD),
			_ => bail!("Invalid BridgeId"),
		}
	}
}

#[derive(Clone, Debug)]
pub struct EthTrackedTx {
	pub hash: H256,
	pub max_fee_per_gas: U256,
}
