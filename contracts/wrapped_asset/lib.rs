#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[cfg(test)]
mod mock;

mod traits;

// Re-export of PSP22 stuff for convenience of cross-contract calls
pub use psp22::{PSP22Error, PSP22};

pub use traits::{WrappedAsset, BASTIAT, MAINNET};

#[ink::contract(env = liberland_extension::LiberlandEnvironment)]
mod wrapped_asset {
	use crate::WrappedAsset;
	use ink::prelude::{string::String, vec::Vec};
	use liberland_extension::AssetId;
	use psp22::{PSP22Data, PSP22Error, PSP22Event, PSP22Metadata, PSP22};

	#[ink(event)]
	pub struct Approval {
		#[ink(topic)]
		owner: AccountId,
		#[ink(topic)]
		spender: AccountId,
		amount: u128,
	}

	#[ink(event)]
	pub struct Transfer {
		#[ink(topic)]
		from: Option<AccountId>,
		#[ink(topic)]
		to: Option<AccountId>,
		value: u128,
	}

	#[ink(storage)]
	#[derive(Default)]
	pub struct Wasset {
		asset_id: AssetId,
		name: String,
		symbol: String,
		decimals: u8,
		data: PSP22Data,
	}

	impl Wasset {
		#[ink(constructor)]
		pub fn new(asset_id: AssetId, name: String, symbol: String, decimals: u8) -> Self {
			Self { asset_id, name, symbol, decimals, ..Default::default() }
		}

		fn emit_events(&self, events: Vec<PSP22Event>) {
			for event in events {
				match event {
					PSP22Event::Transfer(psp22::Transfer { from, to, value }) => {
						self.env().emit_event(Transfer { from, to, value })
					},
					PSP22Event::Approval(psp22::Approval { owner, spender, amount }) => {
						self.env().emit_event(Approval { owner, spender, amount })
					},
				}
			}
		}
	}

	impl WrappedAsset for Wasset {
		#[ink(message, payable)]
		fn deposit(&mut self, value: u128) -> Result<(), PSP22Error> {
			let caller = self.env().caller();
			if value > 0u128 {
				self.env()
					.extension()
					.asset_transfer_approved(self.asset_id, caller, self.env().account_id(), value)
					.map_err(|_| {
						PSP22Error::Custom(String::from("Wrapped Asset: deposit failed"))
					})?;

				let events = self.data.mint(caller, value)?;
				self.emit_events(events);
			}
			Ok(())
		}

		#[ink(message)]
		fn withdraw(&mut self, value: u128) -> Result<(), PSP22Error> {
			let caller = self.env().caller();
			let events = self.data.burn(caller, value)?;
			self.env()
				.extension()
				.asset_transfer(self.asset_id, caller, value)
				.map_err(|_| PSP22Error::Custom(String::from("Wrapped Asset: withdraw failed")))?;
			self.emit_events(events);
			Ok(())
		}
	}

	impl PSP22Metadata for Wasset {
		#[ink(message)]
		fn token_name(&self) -> Option<String> {
			Some(self.name.clone())
		}

		#[ink(message)]
		fn token_symbol(&self) -> Option<String> {
			Some(self.symbol.clone())
		}

		#[ink(message)]
		fn token_decimals(&self) -> u8 {
			self.decimals
		}
	}

	impl PSP22 for Wasset {
		#[ink(message)]
		fn total_supply(&self) -> u128 {
			self.data.total_supply()
		}

		#[ink(message)]
		fn balance_of(&self, owner: AccountId) -> u128 {
			self.data.balance_of(owner)
		}

		#[ink(message)]
		fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
			self.data.allowance(owner, spender)
		}

		#[ink(message)]
		fn transfer(
			&mut self,
			to: AccountId,
			value: u128,
			_data: Vec<u8>,
		) -> Result<(), PSP22Error> {
			let events = self.data.transfer(self.env().caller(), to, value)?;
			self.emit_events(events);
			Ok(())
		}

		#[ink(message)]
		fn transfer_from(
			&mut self,
			from: AccountId,
			to: AccountId,
			value: u128,
			_data: Vec<u8>,
		) -> Result<(), PSP22Error> {
			let events = self.data.transfer_from(self.env().caller(), from, to, value)?;
			self.emit_events(events);
			Ok(())
		}

		#[ink(message)]
		fn approve(&mut self, spender: AccountId, value: u128) -> Result<(), PSP22Error> {
			let events = self.data.approve(self.env().caller(), spender, value)?;
			self.emit_events(events);
			Ok(())
		}

		#[ink(message)]
		fn increase_allowance(
			&mut self,
			spender: AccountId,
			delta_value: u128,
		) -> Result<(), PSP22Error> {
			let events = self.data.increase_allowance(self.env().caller(), spender, delta_value)?;
			self.emit_events(events);
			Ok(())
		}

		#[ink(message)]
		fn decrease_allowance(
			&mut self,
			spender: AccountId,
			delta_value: u128,
		) -> Result<(), PSP22Error> {
			let events = self.data.decrease_allowance(self.env().caller(), spender, delta_value)?;
			self.emit_events(events);
			Ok(())
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;
		use crate::mock::*;
		use ink::env::{test::*, DefaultEnvironment as E};

		psp22::tests!(Wasset, crate::wrapped_asset::tests::init_psp22_supply);

		fn new() -> Wasset {
			ink::env::test::register_chain_extension(MockedLiberlandExtensionSuccess);
			Wasset::new(1, "Wrapped LLM".into(), "WLLM".into(), 12)
		}

		#[ink::test]
		fn constructor_works() {
			let contract = new();
			assert_eq!(contract.total_supply(), 0);
		}

		#[ink::test]
		fn metadata_is_correct() {
			let contract = new();
			assert_eq!(contract.token_name(), Some(String::from("Wrapped LLM")));
			assert_eq!(contract.token_symbol(), Some(String::from("WLLM")));
			assert_eq!(contract.token_decimals(), 12);
		}

		#[ink::test]
		fn deposit_works() {
			let mut contract = new();
			let amount = 100;
			let alice = default_accounts::<E>().alice;
			set_caller::<E>(alice);

			assert_eq!(contract.total_supply(), 0);
			assert_eq!(contract.balance_of(alice), 0);

			assert!(contract.deposit(amount).is_ok());

			assert_eq!(contract.total_supply(), amount);
			assert_eq!(contract.balance_of(alice), amount);
		}

		#[ink::test]
		fn deposit_emits_event() {
			let mut contract = new();
			let amount = 100;
			let alice = default_accounts::<E>().alice;
			set_caller::<E>(alice);

			assert!(contract.deposit(amount).is_ok());

			let events = decode_events();
			assert_eq!(events.len(), 1);
			assert_transfer(&events[0], None, Some(alice), amount);
		}

		#[ink::test]
		fn deposit_of_0_emits_no_event() {
			let mut contract = new();
			let alice = default_accounts::<E>().alice;
			set_caller::<E>(alice);

			contract.deposit(0).unwrap();

			let events = decode_events();
			assert_eq!(events.len(), 0);
		}

		#[ink::test]
		fn multiple_deposit_works_and_emits_events() {
			let mut contract = new();
			let amount = 100;
			let acc = default_accounts::<E>();
			let (alice, bob) = (acc.alice, acc.bob);

			assert_eq!(contract.total_supply(), 0);
			assert_eq!(contract.balance_of(alice), 0);
			assert_eq!(contract.balance_of(bob), 0);

			set_caller::<E>(alice);
			assert!(contract.deposit(amount).is_ok());

			assert_eq!(contract.total_supply(), amount);
			assert_eq!(contract.balance_of(alice), amount);
			assert_eq!(contract.balance_of(bob), 0);

			set_caller::<E>(bob);
			assert!(contract.deposit(2 * amount).is_ok());

			assert_eq!(contract.total_supply(), 3 * amount);
			assert_eq!(contract.balance_of(alice), amount);
			assert_eq!(contract.balance_of(bob), 2 * amount);

			let events = decode_events();
			assert_eq!(events.len(), 2);
			assert_transfer(&events[0], None, Some(alice), amount);
			assert_transfer(&events[1], None, Some(bob), 2 * amount);
		}

		#[ink::test]
		fn withdraw_works() {
			let (supply, amount) = (1000, 100);
			let alice = default_accounts::<E>().alice;
			set_caller::<E>(alice);
			let mut contract = init_psp22_supply(supply);

			assert_eq!(contract.total_supply(), supply);
			assert_eq!(contract.balance_of(alice), supply);

			//let old_native = get_account_balance::<E>(alice).unwrap();
			assert!(contract.withdraw(amount).is_ok());
			//let new_native = get_account_balance::<E>(alice).unwrap();

			assert_eq!(contract.total_supply(), supply - amount);
			assert_eq!(contract.balance_of(alice), supply - amount);
			//assert_eq!(new_native - old_native, amount);
		}

		#[ink::test]
		fn withdraw_emits_event() {
			let (supply, amount) = (1000, 100);
			let alice = default_accounts::<E>().alice;
			set_caller::<E>(alice);
			let mut contract = init_psp22_supply(supply);

			assert!(contract.withdraw(amount).is_ok());

			let events = decode_events();
			assert_eq!(events.len(), 2);
			assert_transfer(&events[0], None, Some(alice), supply);
			assert_transfer(&events[1], Some(alice), None, amount);
		}

		#[ink::test]
		fn withdraw_of_0_emits_no_event() {
			let amount = 100;
			let alice = default_accounts::<E>().alice;
			set_caller::<E>(alice);
			let mut contract = init_psp22_supply(amount);

			assert!(contract.withdraw(0).is_ok());
			let events = decode_events();
			assert_eq!(events.len(), 1);
			assert_transfer(&events[0], None, Some(alice), amount);
		}

		#[ink::test]
		fn withdraw_too_much_fails() {
			let amount = 100;
			let alice = default_accounts::<E>().alice;
			set_caller::<E>(alice);
			let mut contract = init_psp22_supply(amount);
			assert_eq!(contract.withdraw(amount + 1), Err(PSP22Error::InsufficientBalance));
		}

		#[ink::test]
		fn multiple_withdraw_works_and_emits_events() {
			let (initial, a, b) = (1000, 100, 10);
			let alice = default_accounts::<E>().alice;
			let bob = default_accounts::<E>().bob;
			set_caller::<E>(alice);
			set_callee::<E>(default_accounts::<E>().charlie);
			let mut contract = init_psp22_supply(2 * initial);

			assert!(contract.transfer(bob, initial, vec![]).is_ok());

			//let old_alice = get_account_balance::<E>(alice).unwrap();
			//let old_bob = get_account_balance::<E>(bob).unwrap();

			assert!(contract.withdraw(a).is_ok());
			set_caller::<E>(bob);
			assert!(contract.withdraw(b).is_ok());

			//let new_alice = get_account_balance::<E>(alice).unwrap();
			//let new_bob = get_account_balance::<E>(bob).unwrap();

			assert_eq!(contract.total_supply(), 2 * initial - a - b);
			assert_eq!(contract.balance_of(alice), initial - a);
			assert_eq!(contract.balance_of(bob), initial - b);
			//assert_eq!(new_alice - old_alice, a);
			//assert_eq!(new_bob - old_bob, b);

			let events = decode_events();
			assert_eq!(events.len(), 4);
			assert_transfer(&events[0], None, Some(alice), 2 * initial);
			assert_transfer(&events[1], Some(alice), Some(bob), initial);
			assert_transfer(&events[2], Some(alice), None, a);
			assert_transfer(&events[3], Some(bob), None, b);
		}

		// Unit tests helpers

		// Creates a new contract with given total supply
		fn init_psp22_supply(amount: u128) -> Wasset {
			let mut contract = new();
			contract.deposit(amount).unwrap();
			contract
		}

		// Gathers all emitted events into a vector
		fn decode_events() -> Vec<ink::env::test::EmittedEvent> {
			recorded_events().collect()
		}

		// Asserts if the given event is a Transfer with particular from_, to_ and value_
		fn assert_transfer(
			event: &ink::env::test::EmittedEvent,
			from_: Option<AccountId>,
			to_: Option<AccountId>,
			value_: u128,
		) {
			let decoded_event = <Transfer as ink::scale::Decode>::decode(&mut &event.data[..])
				.expect("encountered invalid contract event data buffer");
			let Transfer { from, to, value } = decoded_event;
			assert_eq!(from, from_, "Transfer event: 'from' mismatch");
			assert_eq!(to, to_, "Transfer event: 'to' mismatch");
			assert_eq!(value, value_, "Transfer event: 'value' mismatch");
		}
	}
}
