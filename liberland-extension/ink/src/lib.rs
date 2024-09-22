#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::env::Environment;
mod types;

pub use types::*;

#[ink::chain_extension(extension = 0)]
pub trait Liberland {
	type ErrorCode = Error;

	#[ink(function = 1)]
	fn llm_force_transfer(args: LLMForceTransferArguments);
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
