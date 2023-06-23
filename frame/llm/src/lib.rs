//! # Liberland Merit(LLM) Pallet
//!
//! ## Overview
//!
//! Liberland Merit is a Liberland currency that gives political power to citizens.
//!
//! The LLM pallet handles:
//!
//! * creating LLM asset in `pallet-assets` on genesis
//! * LLM release from **Vault** to **Treasury**
//! * locking, a.k.a. politipooling the LLM for use in politics
//! * veryfing citizenship status
//!
//! ## LLM lifecycle
//!
//! On Genesis (see `fn create_llm`):
//!
//! * LLM is created in `pallet-assets`
//! * configured `TotalSupply` amount of LLM is created and transferred to **Vault**
//! * configured `PreReleasedAmount` is transferred from **Vault** to **Treasury**
//!
//! On yearly basis (see `fn try_release`):
//!
//! * 90% of **Vault** balance is transferred to **Treasury**
//!
//! Accounts are free to locks in politics, a.k.a. politipool any amount of LLM at any time.
//!
//! Accounts may unlock 10% of locked LLM once every `Withdrawlock` duration (see [Genesis
//! Config](#genesis-config)), but it will suspend their politics rights for `Electionlock`
//! duration.
//!
//! Accounts may freely transfer their not-locked LLM to other accounts.
//!
//! ### Special accounts:
//!
//! * **Treasury**:
//!     * gets `PreReleasedAmount` LLM on genesis and 10% of **Vault** balance periodically (_LLM
//!       Release Event_)
//!     * may hold LLD
//!     * derived from PalletID `py/trsry`: `5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z`
//!
//! * **Vault**:
//!     * gets initial supply of LLM on genesis
//!     * releases 10% of it's balance to **Trasury** on LLM Release Event (yearly)
//!     * derived from PalletID `llm/safe`: `5EYCAe5hvejUE1BUTDSnxDfCqVkADRicSKqbcJrduV1KCDmk`
//!
//! * **Politipool**,
//!     * gets LLM locked in politics by other accounts (`politics_lock`)
//!     * releases locked LLM back on `politics_unlock`
//!     * derived from PalletID `politilock`: `5EYCAe5ijGqt3WEM9aKUBdth51NEBNz9P84NaUMWZazzWt7c`
//!
//! ## Internal Storage:
//!
//! * `NextRelease`: block number for next LLM Release Event (transfer of 10% from **Vault** to
//!   **Treasury**)
//! * `LLMPolitics`: amount of LLM each account has allocated into politics
//! * `Withdrawlock`: block number until which account can't do another `politics_unlock`
//! * `Electionlock`: block number until which account can't participate in politics directly
//! * `Citizens`: number of valid citizens
//!
//! ## Runtime config
//!
//! * `RuntimeEvent`: Event type to use.
//! * `TotalSupply`: Total amount of LLM to be created on genesis. That's all LLM that will ever
//!   exit. It will be stored in **Vault**.
//! * `PreReleasedAmount`: Amount of LLM that should be released (a.k.a. transferred from **Vault**
//!   to **Treasury**) on genesis.
//! * `CitizenshipMinimumPooledLLM`: Minimum amount of pooled LLM for valid citizens.
//! * `UnlockFactor`: How much to unlock on politics_unlock
//! * `AssetId`: LLM AssetId.
//! * `AssetName`: LLM Asset name.
//! * `AssetSymbol`: LLM Asset symbol.
//! * `InflationEventInterval`: How often should 90% of vault be released to trasury.
//! * `OnLLMPoliticsUnlock`: Handler for unlocks - for example to remove votes and delegeations in
//!   democracy.
//!
//! ## Genesis Config
//!
//! * `unpooling_withdrawlock_duration`: duration, in seconds, for which additional unlocks should
//!   be locked after `politics_unlock`
//! * `unpooling_electionlock_duration`: duration, in seconds, for which politics rights should be
//!   suspended after `politics_unlock`
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### Public
//!
//! These calls can be made from any _Signed_ origin.
//!
//! * `send_llm`: Transfer LLM. Wrapper over `pallet-assets`' `transfer`.
//! * `send_llm`: Transfer LLM to another account's politipool.
//! * `politics_lock`: Lock LLM into politics pool, a.k.a. politipool.
//! * `politics_unlock`: Unlock 10% of locked LLM. Can't be called again for a WithdrawalLock
//!   period. Affects political rights for an ElectionLock period.
//! * `approve_transfer`: As an assembly member you can approve a transfer of LLM. Not implemented.
//!
//! #### Restricted
//!
//! * `treasury_llm_transfer`: Transfer LLM from treasury to specified account. Can only be called
//!   by Senate.
//! * `treasury_llm_transfer`: Transfer LLM from treasury to specified account's politipool. Can
//!   only be called by Senate.
//!
//! ### Public functions
//!
//! * `llm_id`: Asset ID of the LLM asset for `pallet-assets`
//! * `get_llm_vault_account`: AccountId of **Vault** account. **Vault** account stores all LLM
//!   created initially on genesis and releases it to treasury on LLM Release Events.
//! * `get_llm_treasury_account`: AccountId of **Treasury** account. **Treasury** accounts receives
//!   prereleased amount of LLM on genesis and part of LLM from **Vault** on LLM Release Events.
//! * `get_llm_politipool_account`: AccountId of **Politipool** account. **Politipool** account
//!   stores LLM locked in politics by all other accounts.
//!
//! ### LLM trait
//!
//! LLM pallet implements LLM trait with following functions available for other pallets:
//!
//! * `check_pooled_llm`: Checks if given account has any LLM locked in politics.
//! * `is_election_unlocked`: Checks if given account has rights to participate in politics
//!   unlocked. They may be locked after `politics_unlock`. This does NOT check if account is a
//!   valid citizen - use `CitizenshipChecker` trait for that.
//! * `get_politi_pooled_amount`: Get total amount of locked LLM across all accounts.
//! * `get_llm_politics`: Get amount of locked LLM for given account.
//!
//! ### CitizenshipChecker trait
//!
//! LLM pallet implements CitizenshipChecker trait with following functions available for other
//! pallets:
//!
//! * `ensure_politics_allowed`: Checks if given account can participate in
//! politics actions. It verifies that it's a valid citizen, doesn't have
//! election rights locked and has 5000 LLM locked in politics.
//!
//! License: MIT

#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod impl_fungible;
pub mod migrations;

/// Liberland Merit Pallet
/*
Author: <filip@rustsyndi.cat>
Copyright © 2022 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/
use frame_support::traits::Currency;
type Assets<T> = pallet_assets::Pallet<T>;
type BalanceOf<T> = <<T as pallet_identity::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	// Import various types used to declare pallet in scope.
	use super::*;
	use frame_support::{
		pallet_prelude::{DispatchResult, *},
		traits::{
			fungibles::Mutate,
			tokens::{currency::Currency, ExistenceRequirement},
			EnsureOrigin,
		},
		PalletId,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use liberland_traits::{CitizenshipChecker, OnLLMPoliticsUnlock, LLM};
	use scale_info::prelude::vec;
	use sp_runtime::{
		traits::{AccountIdConversion, StaticLookup},
		AccountId32, Permill,
	};
	use sp_std::vec::Vec;

	/// block number for next LLM release event (transfer of 10% from **Vault** to **Treasury**)
	#[pallet::storage]
	#[pallet::getter(fn next_release)]
	pub(super) type NextRelease<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>; // ValueQuery ,  OnEmpty = 0

	/// amount of LLM each account has allocated into politics
	#[pallet::storage]
	#[pallet::getter(fn llm_politics)]
	pub(super) type LLMPolitics<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

	/// block number until which account can't do another `politics_unlock`
	#[pallet::storage]
	#[pallet::getter(fn withdraw_lock)]
	pub(super) type Withdrawlock<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::BlockNumber, ValueQuery>; // account and blocknumber

	#[pallet::type_value]
	pub fn WithdrawlockDurationOnEmpty<T: Config>() -> T::BlockNumber {
		120u8.into()
	}

	#[pallet::storage]
	#[pallet::getter(fn withdraw_lock_duration)]
	pub(super) type WithdrawlockDuration<T: Config> =
		StorageValue<_, T::BlockNumber, ValueQuery, WithdrawlockDurationOnEmpty<T>>; // seconds

	/// block number until which account can't participate in politics directly
	#[pallet::storage]
	#[pallet::getter(fn election_lock)]
	pub(super) type Electionlock<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::BlockNumber, ValueQuery>; // account and blocknumber

	#[pallet::type_value]
	pub fn ElectionlockDurationOnEmpty<T: Config>() -> T::BlockNumber {
		120u8.into()
	}
	#[pallet::storage]
	#[pallet::getter(fn election_lock_duration)]
	pub(super) type ElectionlockDuration<T: Config> =
		StorageValue<_, T::BlockNumber, ValueQuery, ElectionlockDurationOnEmpty<T>>; // seconds

	#[pallet::storage]
	#[pallet::getter(fn citizens)]
	pub(super) type Citizens<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// duration, in seconds, for which additional unlocks should be locked
		/// after `politics_unlock`
		pub unpooling_withdrawlock_duration: T::BlockNumber,
		/// duration, in seconds, for which politics rights should be suspended
		/// after `politics_unlock`
		pub unpooling_electionlock_duration: T::BlockNumber,
		pub _phantom: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				unpooling_withdrawlock_duration: WithdrawlockDurationOnEmpty::<T>::get(),
				unpooling_electionlock_duration: ElectionlockDurationOnEmpty::<T>::get(),
				_phantom: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			let rootorg = frame_system::RawOrigin::Root.into();
			Pallet::<T>::create_llm(rootorg).unwrap();

			WithdrawlockDuration::<T>::put(&self.unpooling_withdrawlock_duration);
			ElectionlockDuration::<T>::put(&self.unpooling_electionlock_duration);
		}
	}

	#[pallet::config]
	pub trait Config:
		pallet_assets::Config + frame_system::Config + pallet_identity::Config
	{
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// LLD
		type Currency: Currency<Self::AccountId>;

		/// Total amount of LLM to be created on genesis. That's all LLM that
		/// will ever exit. It will be stored in **Vault**.
		type TotalSupply: Get<u128>;

		/// Amount of LLM that should be released (a.k.a. transferred from
		/// **Vault** to **Treasury**) on genesis.
		type PreReleasedAmount: Get<u128>; // Pre defined the total supply in runtime

		/// Minimum amount of LLM Accounts needs politipooled to have citizenship rights
		type CitizenshipMinimumPooledLLM: Get<u128>; // Pre defined the total supply in runtime

		/// How much funds unlock on politics_unlock
		type UnlockFactor: Get<Permill>;

		/// Senate origin - can transfer from treasury
		type SenateOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		type AssetId: Get<<Self as pallet_assets::Config>::AssetId>;
		type AssetName: Get<Vec<u8>>;
		type AssetSymbol: Get<Vec<u8>>;
		type InflationEventInterval: Get<<Self as frame_system::Config>::BlockNumber>;
		type OnLLMPoliticsUnlock: OnLLMPoliticsUnlock<Self::AccountId>;
	}

	pub type AssetId<T> = <T as Config>::AssetId;

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid amount
		InvalidAmount,
		/// Balance is less then sending amount
		LowBalance,
		/// Asset already created
		AssetExists,
		/// Account can't perform this action now
		InvalidAccount,
		/// Not allowed to withdraw more LLM, wait until Withdrawlock
		Gottawait,
		/// No politcal LLM allocated tokens
		NoPolLLM,
		/// Not a Citizen
		NonCitizen,
		/// Temporary locked after unpooling LLM
		Locked,
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(b: T::BlockNumber) -> Weight {
			Self::maybe_release(b);
			Weight::zero()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Lock LLM into politics pool, a.k.a. politipool.  Internally it
		/// transfers LLM to **Politipool** account.
		///
		/// * `amount`: Amount of LLM to lock.
		///
		/// Emits:
		/// * `LLMPoliticsLocked`
		/// * `Transferred` from `pallet-assets`
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn politics_lock(origin: OriginFor<T>, amount: T::Balance) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			Self::do_politics_lock(sender, amount)?;
			Ok(())
		}

		/// Unlock 10% of account's LLM from politics pool, a.k.a. politipool.
		/// Internally it transfers LLM from **Politipool** account.
		///
		/// Can only be called once per `Withdrawlock` duration, will fail with
		/// `Gottawait` otherwise.
		///
		/// Emits:
		/// * `LLMPoliticsLocked`
		/// * `Transferred` from `pallet-assets`
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn politics_unlock(origin: OriginFor<T>) -> DispatchResult {
			let sender: T::AccountId = ensure_signed(origin.clone())?;
			// check if we have political locked LLM

			let politics_balance = LLMPolitics::<T>::get(sender.clone());
			ensure!(politics_balance > 0u8.into(), Error::<T>::InvalidAccount);

			let current_block_number = frame_system::Pallet::<T>::block_number();
			ensure!(current_block_number > Withdrawlock::<T>::get(&sender), Error::<T>::Gottawait);

			let ten_percent: T::Balance = Self::get_unlock_amount(politics_balance)?;

			Self::transfer_from_politipool(sender.clone(), ten_percent)?;
			LLMPolitics::<T>::mutate(&sender, |b| *b -= ten_percent);

			let withdraw_lock_end = current_block_number + Self::withdraw_lock_duration();
			let election_lock_end = current_block_number + Self::election_lock_duration();
			Withdrawlock::<T>::insert(&sender, withdraw_lock_end);
			Electionlock::<T>::insert(&sender, election_lock_end);

			T::OnLLMPoliticsUnlock::on_llm_politics_unlock(&sender)?;
			Self::deposit_event(Event::<T>::LLMPoliticsUnlocked(sender, ten_percent));
			Ok(())
		}

		/// Transfer LLM from treasury to specified account. Can only be called
		/// by Senate.
		///
		/// - `to_account`: Account to transfer to.
		/// - `amount`: Amount to transfer.
		///
		/// Emits: `Transferred` from `pallet-assets`
		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn treasury_llm_transfer(
			origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			T::SenateOrigin::ensure_origin(origin)?;
			Self::transfer_from_treasury(to_account, amount)
		}

		/// Transfer LLM from treasury to specified account's politipool. Can
		/// only be called by selected accounts and Senate.
		///
		/// - `to_account`: Account to transfer to.
		/// - `amount`: Amount to transfer.
		///
		/// Emits: `Transferred` from `pallet-assets`
		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn treasury_llm_transfer_to_politipool(
			origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			T::SenateOrigin::ensure_origin(origin)?;

			Self::transfer_from_treasury(to_account.clone(), amount)?;
			Self::do_politics_lock(to_account, amount)
		}

		/// Transfer LLM from sender to specified account's politipool.
		///
		/// - `to_account`: Account to transfer to.
		/// - `amount`: Amount to transfer.
		///
		/// Emits: `Transferred` from `pallet-assets`
		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn send_llm_to_politipool(
			origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let sender: T::AccountId = ensure_signed(origin)?;

			Self::transfer(sender, to_account.clone(), amount)?;
			Self::do_politics_lock(to_account, amount)
		}

		/// Wrapper over `pallet-assets` `transfer`. Transfers LLM from sender
		/// to specified account.
		///
		/// - `to_account`: Account to transfer LLM to
		/// - `amount`: Amount of LLM to transfer
		///
		/// Emits: `Transferred` from `pallet-assets`
		#[pallet::call_index(5)]
		#[pallet::weight(10_000)]
		pub fn send_llm(
			origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			Self::transfer(sender, to_account, amount)
		}

		/// Transfer LLD from treasury to specified account. Can only be called
		/// by Senate.
		///
		/// - `to_account`: Account to transfer to.
		/// - `amount`: Amount to transfer.
		///
		/// Emits: `Transfer` from `pallet-balances`
		#[pallet::call_index(6)]
		#[pallet::weight(10_000)]
		pub fn treasury_lld_transfer(
			origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: <<T as Config>::Currency as Currency<T::AccountId>>::Balance,
		) -> DispatchResult {
			T::SenateOrigin::ensure_origin(origin)?;
			<T as Config>::Currency::transfer(
				&Self::get_llm_treasury_account(),
				&to_account,
				amount,
				ExistenceRequirement::KeepAlive,
			)
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New LLM has been released
		ReleasedLLM(T::AccountId, T::Balance),
		/// New LLM has been created
		LLMCreated(T::AccountId, T::Balance), // acountid, amount
		/// X amount of LLM has been unlocked
		LLMPoliticsLocked(T::AccountId, T::Balance),
		/// sent to user account, amount
		LLMPoliticsUnlocked(T::AccountId, T::Balance),
	}

	impl<T: Config> Pallet<T> {
		fn do_politics_lock(account: T::AccountId, amount: T::Balance) -> DispatchResult {
			Self::do_transfer_to_politipool(account.clone(), amount)?;
			LLMPolitics::<T>::mutate(account.clone(), |b| *b += amount);
			Self::deposit_event(Event::<T>::LLMPoliticsLocked(account, amount));
			Ok(())
		}

		pub fn account_id32_to_accountid(accountid32: AccountId32) -> T::AccountId {
			let mut init_account32 = AccountId32::as_ref(&accountid32);
			let init_account: T::AccountId = T::AccountId::decode(&mut init_account32).unwrap();
			init_account
		}

		pub fn balance(account: T::AccountId) -> T::Balance {
			Assets::<T>::balance(Self::llm_id().into(), account)
		}

		fn get_unlock_amount(balance: T::Balance) -> Result<T::Balance, Error<T>> {
			let factor = T::UnlockFactor::get();
			let balance: u64 = balance.try_into().map_err(|_| Error::<T>::InvalidAmount)?;
			let amount = factor.mul_floor(balance);
			amount.try_into().map_err(|_| Error::<T>::InvalidAmount)
		}

		pub(super) fn transfer(
			from_account: T::AccountId,
			to_account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let origin = frame_system::RawOrigin::Signed(from_account.clone()).into();
			Assets::<T>::transfer(
				origin,
				Self::llm_id().into(),
				T::Lookup::unlookup(to_account.clone()),
				amount.clone(),
			)
		}

		fn do_transfer_to_politipool(
			from_account: T::AccountId,
			amount_balance: T::Balance,
		) -> DispatchResult {
			let politipool_account = Self::get_llm_politipool_account();
			Self::transfer(from_account, politipool_account, amount_balance)
		}

		fn transfer_from_politipool(
			to_account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let politipool_account = Self::get_llm_politipool_account();
			Self::transfer(politipool_account, to_account, amount)
		}

		/// Transfer `amount` LLM to `to_account` from vault
		/// Used in tests.
		pub fn transfer_from_vault(to_account: T::AccountId, amount: T::Balance) -> DispatchResult {
			let vault = Self::get_llm_vault_account();
			Self::transfer(vault, to_account, amount)
		}

		/// Transfer `amount` LLM to `to_account` from treasury
		/// Used in liberland-initializer and in tests.
		pub fn transfer_from_treasury(
			to_account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let treasury = Self::get_llm_treasury_account();
			Self::transfer(treasury, to_account, amount)
		}

		fn create_llm(origin: OriginFor<T>) -> DispatchResult {
			let asset_id: <T as pallet_assets::Config>::AssetId = Self::llm_id().into();
			let treasury = Self::get_llm_treasury_account();
			let challenger_lookup: <T::Lookup as StaticLookup>::Source =
				T::Lookup::unlookup(treasury.clone());
			let asset_supply = Assets::<T>::total_supply(asset_id.clone());
			ensure!(asset_supply == 0u8.into(), Error::<T>::AssetExists); // if the asset supply is zero == that means it is not been created and we can create

			let min_balance: T::Balance = 1u8.into();
			let decimals: u8 = 12u8;
			Assets::<T>::force_create(
				origin.clone(),
				asset_id.clone().into(),
				challenger_lookup,
				true,
				min_balance,
			)?;
			Assets::<T>::force_set_metadata(
				origin.clone(),
				asset_id.clone().into(),
				T::AssetName::get(),
				T::AssetSymbol::get(),
				decimals,
				false,
			)?;
			Self::deposit_event(Event::<T>::LLMCreated(treasury.clone(), 0u8.into()));

			// Mint tokens into the llm/vault
			let vaultac: T::AccountId = Self::get_llm_vault_account();
			let supply = T::TotalSupply::get().try_into().map_err(|_| Error::<T>::InvalidAmount)?;
			Assets::<T>::mint_into(asset_id, &vaultac, supply)?;

			let nextblock = Self::get_future_block();
			NextRelease::<T>::put(nextblock);

			// Release a.k.a. transfer to treasury
			let prereleased =
				T::PreReleasedAmount::get().try_into().map_err(|_| Error::<T>::InvalidAmount)?;
			Self::release_tokens_from_vault(prereleased)
		}

		/// Asset ID of the LLM asset for `pallet-assets`
		pub fn llm_id() -> <T as pallet_assets::Config>::AssetId {
			<T as Config>::AssetId::get()
		}

		/// AccountId of **Vault** account. **Vault** account stores all LLM
		/// created initially on genesis and releases it to treasury on LLM
		/// Release Events.
		pub fn get_llm_vault_account() -> T::AccountId {
			PalletId(*b"llm/safe").into_account_truncating()
		}

		/// AccountId of **Treasury** account. **Treasury** accounts receives
		/// prereleased amount of LLM on genesis and part of LLM from **Vault**
		/// on LLM Release Events.
		pub fn get_llm_treasury_account() -> T::AccountId {
			PalletId(*b"lltreasu").into_account_truncating()
		}

		/// AccountId of **Politipool** account. **Politipool** account stores
		/// LLM locked in politics by all other accounts.
		pub fn get_llm_politipool_account() -> T::AccountId {
			PalletId(*b"polilock").into_account_truncating()
		}

		fn get_future_block() -> T::BlockNumber {
			let current_block_number = frame_system::Pallet::<T>::block_number();
			current_block_number + T::InflationEventInterval::get()
		}

		// each time we release (should be each year), release 10% from vault
		fn get_release_amount() -> T::Balance {
			let asset_id = Self::llm_id().into();
			let vault_account = Self::get_llm_vault_account();
			let unreleased_amount = Assets::<T>::balance(asset_id, vault_account);
			let release_amount = unreleased_amount / 10u8.into();
			release_amount
		}

		fn maybe_release(block: T::BlockNumber) -> bool {
			if block < NextRelease::<T>::get() {
				return false
			}
			NextRelease::<T>::put(Self::get_future_block());

			let release_amount = Self::get_release_amount();
			Self::release_tokens_from_vault(release_amount).unwrap();

			true
		}

		/// Release tokens to the treasury account. Sends tokens from the llm/vault to the treasury
		fn release_tokens_from_vault(amount: T::Balance) -> DispatchResult {
			let treasury = Self::get_llm_treasury_account();

			Self::transfer_from_vault(treasury.clone(), amount)?;
			Self::deposit_event(Event::<T>::ReleasedLLM(treasury, amount));

			Ok(())
		}
	}

	impl<T: Config> LLM<T::AccountId, T::Balance> for Pallet<T> {
		fn check_pooled_llm(account: &T::AccountId) -> bool {
			let minimum = match T::CitizenshipMinimumPooledLLM::get().try_into() {
				Ok(m) => m,
				_ => panic!("Configured Balance type for pallet_assets can't fit u64 values required by pallet_llm!")
			};
			LLMPolitics::<T>::get(account) >= minimum
		}

		fn is_election_unlocked(account: &T::AccountId) -> bool {
			if Electionlock::<T>::contains_key(account) {
				let current_block_number = frame_system::Pallet::<T>::block_number();
				let unlocked_on_block = Electionlock::<T>::get(account);
				return current_block_number > unlocked_on_block
			}
			true
		}

		fn get_politi_pooled_amount() -> T::Balance {
			let politipool_account = Self::get_llm_politipool_account();
			Self::balance(politipool_account)
		}

		fn get_llm_politics(account: &T::AccountId) -> T::Balance {
			LLMPolitics::<T>::get(account)
		}
	}

	impl<T: Config> Pallet<T> {
		fn is_known_good_identity(
			reg: &pallet_identity::Registration<
				BalanceOf<T>,
				T::MaxRegistrars,
				T::MaxAdditionalFields,
			>,
		) -> bool {
			use pallet_identity::{Data, Data::Raw, Judgement::KnownGood};

			let citizen_key = Raw(b"citizen".to_vec().try_into().unwrap());
			let is_citizen_field_set = matches!(reg.info.additional.iter().find(|v| v.0 == citizen_key), Some(x) if x.1 != Data::None);

			let current_block_number = frame_system::Pallet::<T>::block_number();
			let eligible_on_key = Raw(b"eligible_on".to_vec().try_into().unwrap());
			let eligible_on = match reg.info.additional.iter().find(|v| v.0 == eligible_on_key) {
				Some((_, Raw(x))) => x,
				_ => return false,
			};
			// little-endian
			// 256 = vec![0x00, 0x01];
			let eligible_on = eligible_on.iter().rfold(0u64, |r, i: &u8| (r << 8) + (*i as u64));
			let eligible_on: Result<T::BlockNumber, _> = eligible_on.try_into();

			let is_eligible =
				matches!(eligible_on, Ok(eligible_on) if eligible_on <= current_block_number);
			let is_known_good_judgment = reg.judgements.contains(&(0u32, KnownGood));

			is_citizen_field_set && is_eligible && is_known_good_judgment
		}

		fn is_known_good(account: &T::AccountId) -> bool {
			match pallet_identity::Pallet::<T>::identity(account) {
				Some(reg) => Self::is_known_good_identity(&reg),
				None => false,
			}
		}
	}

	impl<T: Config> CitizenshipChecker<T::AccountId> for Pallet<T> {
		fn ensure_politics_allowed(account: &T::AccountId) -> Result<(), DispatchError> {
			ensure!(Self::is_known_good(account), Error::<T>::NonCitizen);
			ensure!(Self::is_election_unlocked(account), Error::<T>::Locked);
			ensure!(Self::check_pooled_llm(account), Error::<T>::NoPolLLM);
			Ok(())
		}

		fn ensure_land_nfts_allowed(account: &T::AccountId) -> Result<(), DispatchError> {
			ensure!(Self::is_known_good(account), Error::<T>::NonCitizen);
			ensure!(Self::check_pooled_llm(account), Error::<T>::NoPolLLM);
			Ok(())
		}

		fn is_citizen(account: &T::AccountId) -> bool {
			Self::is_known_good(account)
		}

		fn citizens_count() -> u64 {
			Citizens::<T>::get()
		}

		fn identity_changed(was_citizen_before_change: bool, account: &T::AccountId) {
			let is_citizen_now = Self::is_citizen(account);
			if was_citizen_before_change && !is_citizen_now {
				Citizens::<T>::mutate(|c| *c -= 1);
			} else if !was_citizen_before_change && is_citizen_now {
				Citizens::<T>::mutate(|c| *c += 1);
			}
		}
	}
}
