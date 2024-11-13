#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[cfg(test)]
mod mock;

#[ink::contract(env = liberland_extension::LiberlandEnvironment)]
mod asset_timelock {
	use ink::storage::Mapping;
	use liberland_extension::{AssetId, AssetsBalance};

	pub type DepositId = u32;

	#[derive(Debug, PartialEq, Eq, Clone)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
	pub struct Deposit {
		timestamp: Timestamp,
		asset_id: AssetId,
		recipient: AccountId,
		amount: AssetsBalance,
	}

	#[derive(Debug, PartialEq, Eq, Clone)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	pub enum Error {
		/// Funds not unlocked yet
		TooEarly,
		/// No such deposit found
		NotFound,
		/// Runtime call failed
		CallFailed,
		/// Arithmetic overflow
		Overflow,
		/// Unlock timestamp is before current timestamp
		TimestampFromThePast,
		/// Invalid amount
		InvalidAmount,
	}

	impl From<liberland_extension::Error> for Error {
		fn from(_: liberland_extension::Error) -> Self {
			Self::CallFailed
		}
	}

	pub type Result<T> = core::result::Result<T, Error>;

	#[ink(storage)]
	#[derive(Default)]
	pub struct AssetTimelock {
		next_deposit_id: DepositId,
		deposits: Mapping<DepositId, Deposit>,
	}

	#[ink(event)]
	pub struct Deposited {
		#[ink(topic)]
		depositor: AccountId,
		#[ink(topic)]
		recipient: AccountId,
		deposit_id: DepositId,
		asset_id: AssetId,
		timestamp: Timestamp,
		amount: AssetsBalance,
	}

	#[ink(event)]
	pub struct Withdrawn {
		#[ink(topic)]
		recipient: AccountId,
		deposit_id: DepositId,
		asset_id: AssetId,
		amount: AssetsBalance,
	}

	impl AssetTimelock {
		#[ink(constructor)]
		pub fn new() -> Self {
			Default::default()
		}

		#[ink(message)]
		pub fn get_deposit(&self, deposit_id: DepositId) -> Option<Deposit> {
			self.deposits.get(deposit_id)
		}

		#[ink(message)]
		pub fn deposit(
			&mut self,
			asset_id: AssetId,
			recipient: AccountId,
			amount: AssetsBalance,
			timestamp: Timestamp,
		) -> Result<DepositId> {
			if timestamp < self.env().block_timestamp() {
				return Err(Error::TimestampFromThePast);
			}
			if amount <= 0u128 {
				return Err(Error::InvalidAmount);
			}

			let depositor = self.env().caller();
			let contract_account = self.env().account_id();
			let deposit_id = self.next_deposit_id;
			self.next_deposit_id = self.next_deposit_id.checked_add(1).ok_or(Error::Overflow)?;
			self.env().extension().asset_transfer_approved(
				asset_id,
				depositor,
				contract_account,
				amount,
			)?;
			self.deposits
				.insert(deposit_id, &Deposit { timestamp, asset_id, recipient, amount });
			self.env().emit_event(Deposited {
				depositor,
				deposit_id,
				asset_id,
				recipient,
				timestamp,
				amount,
			});
			Ok(deposit_id)
		}

		#[ink(message)]
		pub fn withdraw(&mut self, deposit_id: DepositId) -> Result<()> {
			let Deposit { asset_id, recipient, amount, timestamp } =
				self.deposits.take(&deposit_id).ok_or(Error::NotFound)?;
			if timestamp > self.env().block_timestamp() {
				return Err(Error::TooEarly);
			}
			self.env().extension().asset_transfer(asset_id, recipient, amount)?;
			self.env().emit_event(Withdrawn { deposit_id, asset_id, recipient, amount });
			Ok(())
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;
		use crate::mock::*;

		fn ts(ts: u64) {
			ink::env::test::set_block_timestamp::<Environment>(ts);
		}

		fn init() -> AssetTimelock {
			ts(0);
			AssetTimelock::new()
		}

		fn alice() -> AccountId {
			ink::env::test::default_accounts::<Environment>().alice
		}

		fn bob() -> AccountId {
			ink::env::test::default_accounts::<Environment>().bob
		}

		fn set_next_caller(caller: AccountId) {
			ink::env::test::set_caller::<Environment>(caller);
		}

		fn assert_deposited_event(
			event: &ink::env::test::EmittedEvent,
			expected_depositor: AccountId,
			expected_recipient: AccountId,
			expected_deposit_id: DepositId,
			expected_asset_id: AssetId,
			expected_timestamp: Timestamp,
			expected_amount: AssetsBalance,
		) {
			let decoded_event = <Deposited as ink::scale::Decode>::decode(&mut &event.data[..])
				.expect("encountered invalid contract event data buffer");
			let Deposited { depositor, recipient, deposit_id, asset_id, timestamp, amount } =
				decoded_event;
			assert_eq!(depositor, expected_depositor);
			assert_eq!(recipient, expected_recipient);
			assert_eq!(deposit_id, expected_deposit_id);
			assert_eq!(asset_id, expected_asset_id);
			assert_eq!(timestamp, expected_timestamp);
			assert_eq!(amount, expected_amount);
		}

		fn assert_withdrawn_event(
			event: &ink::env::test::EmittedEvent,
			expected_recipient: AccountId,
			expected_deposit_id: DepositId,
			expected_asset_id: AssetId,
			expected_amount: AssetsBalance,
		) {
			let decoded_event = <Withdrawn as ink::scale::Decode>::decode(&mut &event.data[..])
				.expect("encountered invalid contract event data buffer");
			let Withdrawn { recipient, deposit_id, asset_id, amount } = decoded_event;
			assert_eq!(recipient, expected_recipient);
			assert_eq!(deposit_id, expected_deposit_id);
			assert_eq!(asset_id, expected_asset_id);
			assert_eq!(amount, expected_amount);
		}

		#[ink::test]
		fn new_works() {
			let timelock = init();
			assert_eq!(timelock.next_deposit_id, 0);
			assert_eq!(timelock.deposits.get(0), None);
		}

		#[ink::test]
		fn deposit_works() {
			ink::env::test::register_chain_extension(MockedLiberlandExtensionSuccess);

			let mut timelock = init();
			set_next_caller(bob());
			let deposit_id = timelock.deposit(5, alice(), 2u128, 1).unwrap();
			assert_eq!(deposit_id, 0);

			let Deposit { timestamp, asset_id, recipient, amount } =
				timelock.get_deposit(0).unwrap();
			assert_eq!(timestamp, 1);
			assert_eq!(asset_id, 5);
			assert_eq!(recipient, alice());
			assert_eq!(amount, 2u128);

			let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
			assert_eq!(emitted_events.len(), 1);
			assert_deposited_event(&emitted_events[0], bob(), alice(), 0, 5, 1, 2u128);
		}

		#[ink::test]
		fn cant_deposit_past_for_past_ts() {
			let mut timelock = init();
			ts(10);
			let res = timelock.deposit(5, alice(), 2u128, 1);
			assert_eq!(res, Err(Error::TimestampFromThePast));
		}

		#[ink::test]
		fn cant_deposit_nothing() {
			let mut timelock = init();
			let res = timelock.deposit(5, alice(), 0u128, 1);
			assert_eq!(res, Err(Error::InvalidAmount));
		}

		#[ink::test]
		fn deposit_fails_if_transfer_fails() {
			ink::env::test::register_chain_extension(MockedLiberlandExtensionFail);
			let mut timelock = init();
			let res = timelock.deposit(5, alice(), 1u128, 1);
			assert_eq!(res, Err(Error::CallFailed));
		}

		#[ink::test]
		fn deposit_increments_ids() {
			ink::env::test::register_chain_extension(MockedLiberlandExtensionSuccess);

			let mut timelock = init();
			let deposit_id = timelock.deposit(5, alice(), 2u128, 1).unwrap();
			assert_eq!(deposit_id, 0);

			let deposit_id = timelock.deposit(5, alice(), 2u128, 1).unwrap();
			assert_eq!(deposit_id, 1);

			let deposit_id = timelock.deposit(5, alice(), 2u128, 1).unwrap();
			assert_eq!(deposit_id, 2);

			assert!(timelock.get_deposit(0).is_some());
			assert!(timelock.get_deposit(1).is_some());
			assert!(timelock.get_deposit(2).is_some());
			assert!(timelock.get_deposit(3).is_none());
		}

		#[ink::test]
		fn withdraw_works() {
			ink::env::test::register_chain_extension(MockedLiberlandExtensionSuccess);

			let mut timelock = init();
			set_next_caller(bob());
			let deposit_id = timelock.deposit(5, alice(), 2u128, 1).unwrap();
			ts(2);
			timelock.withdraw(deposit_id).unwrap();
			assert!(timelock.get_deposit(0).is_none());
			let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
			assert_eq!(emitted_events.len(), 2);
			assert_withdrawn_event(&emitted_events[1], alice(), 0, 5, 2u128);
		}

		#[ink::test]
		fn withdraw_fails_if_too_early() {
			ink::env::test::register_chain_extension(MockedLiberlandExtensionSuccess);

			let mut timelock = init();
			set_next_caller(bob());
			let deposit_id = timelock.deposit(5, alice(), 2u128, 1).unwrap();
			let res = timelock.withdraw(deposit_id);
			assert_eq!(res, Err(Error::TooEarly));
		}
	}
}
