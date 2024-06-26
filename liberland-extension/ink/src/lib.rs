#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::env::Environment;
mod types;

pub use types::*;

#[ink::chain_extension(extension = 0)]
pub trait Liberland {
	type ErrorCode = Error;

	#[ink(function = 1)]
	fn llm_force_transfer(args: LLMForceTransferArguments);

	#[ink(function = 2)]
	fn asset_balance_of(asset_id: AssetId, account: AccountId) -> AssetsBalance;

	#[ink(function = 3)]
	fn asset_total_supply_of(asset_id: AssetId) -> AssetsBalance;

	#[ink(function = 4)]
	fn asset_approve_transfer(asset_id: AssetId, delegate: AccountId, amount: Balance);

	#[ink(function = 5)]
	fn asset_cancel_approval(asset_id: AssetId, delegate: AccountId);

	#[ink(function = 6)]
	fn asset_transfer(asset_id: AssetId, target: AccountId, amount: Balance);
	#[ink(function = 7)]
	fn asset_transfer_approved(
		asset_id: AssetId,
		owner: AccountId,
		destination: AccountId,
		amount: Balance,
	);
	#[ink(function = 8)]
	fn asset_transfer_keep_alive(asset_id: AssetId, target: AccountId, amount: Balance);
}

impl ink::env::chain_extension::FromStatusCode for Error {
	fn from_status_code(status_code: u32) -> Result<(), Self> {
		match status_code {
			0 => Ok(()),
			1 => Err(Self::Failed),
			_ => panic!("encountered unknown status code"),
		}
	}
}

impl Environment for LiberlandEnvironment {
	const MAX_EVENT_TOPICS: usize = <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

	type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
	type Balance = <ink::env::DefaultEnvironment as Environment>::Balance;
	type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
	type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
	type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;

	type ChainExtension = Liberland;
}
