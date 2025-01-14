use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{sp_runtime::RuntimeDebug, traits::Get, BoundedVec};
use scale_info::TypeInfo;

pub type ContractIndex = u32;

#[derive(Clone, Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(MaxContractsLen, MaxParties))]
pub struct ContractDataStorage<MaxContractsLen: Get<u32>, MaxParties: Get<u32>, AccountId, Balance>
{
	// Content of contract
	pub data: BoundedVec<u8, MaxContractsLen>,
	// Vec of parties that may sign following contract
	pub parties: Option<BoundedVec<AccountId, MaxParties>>,
	pub creator: AccountId,
	pub deposit: Balance,
}
