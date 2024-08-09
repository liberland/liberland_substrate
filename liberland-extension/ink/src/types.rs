use ink::env::Environment;

pub type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
pub type Balance = <ink::env::DefaultEnvironment as Environment>::Balance;
pub type AssetsBalance = u128;
pub type AssetId = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
pub enum LLMAccount<AccountId> {
	Liquid(AccountId),
	Locked(AccountId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
pub struct LLMForceTransferArguments {
	pub from: LLMAccount<AccountId>,
	pub to: LLMAccount<AccountId>,
	pub amount: Balance,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
	Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(TypeInfo)]
pub enum LiberlandEnvironment {}
