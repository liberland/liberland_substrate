//! # Liberland Merit(LLM) Pallet
//!
//! ## Overview
//!
//! Liberland Merit is a Liberland currency that gives political power to citizens.
//!
//! The LLM pallet handles:
//!
//! * creating LLM asset in `pallet-assets` on genesis
//! * LLM inflation to treasury
//! * locking, a.k.a. politipooling the LLM for use in politics
//! * veryfing citizenship status
//!
//! ## LLM lifecycle
//!
//! On Genesis (see `fn create_llm`):
//!
//! * LLM is created in `pallet-assets`
//! * configured `TotalSupply` amount of LLM is created and transferred to **Vault**
//! * configured `PreMintedAmount` is transferred from **Vault** to **Treasury**
//!
//! On yearly basis (see `fn try_mint`):
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
//!     * gets preminted and inflation LLM
//!     * derived from PalletID `py/trsry`: `5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z`
//!
//! * **Vault**:
//!     * gets initial supply of LLM on genesis
//!     * releases 90% of it's balance to **Trasury** on inflation event (yearly)
//!     * derived from PalletID `llm/safe`: `5EYCAe5hvejUE1BUTDSnxDfCqVkADRicSKqbcJrduV1KCDmk`
//!
//! * **Politipool**,
//!     * gets LLM locked in politics by other accounts (`politics_lock`)
//!     * releases locked LLM back on `politics_unlock`
//!     * derived from PalletID `politilock`: `5EYCAe5ijGqt3WEM9aKUBdth51NEBNz9P84NaUMWZazzWt7c`
//!
//! ## Internal Storage:
//!
//! * `NextMint`: block number for next LLM inflation mint (transfer of 90% from **Vault** to
//!   **Treasury**)
//! * `LLMPolitics`: amount of LLM each account has allocated into politics
//! * `Withdrawlock`: block number until which account can't do another `politics_unlock`
//! * `Electionlock`: block number until which account can't participate in politics directly
//!
//! ## Runtime config
//!
//! * `RuntimeEvent`: Event type to use.
//! * `AssetId`: Type of AssetId.
//! * `TotalSupply`: Total amount of LLM to be created on genesis. That's all LLM that will ever
//!   exit. It will be stored in **Vault**.
//! * `PreMintedAmount`: Amount of LLM that should be minted (a.k.a. transferred from **Vault** to
//!   **Treasury**) on genesis.
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
//! * `fake_send`: Mint LLM to account. Development only, to be removed/restricted.
//! * `send_llm`: Transfer LLM. Wrapper over `pallet-assets`' `transfer`.
//! * `politics_lock`: Lock LLM into politics pool, a.k.a. politipool.
//! * `politics_unlock`: Unlock 10% of locked LLM. Can't be called again for a WithdrawalLock
//!   period. Affects political rights for an ElectionLock period.
//! * `createllm`: Creates LLM asset in `pallet-assets`.
//! * `approve_transfer`: As an assembly member you can approve a transfer of LLM. Not implemented.
//!
//! #### Restricted
//!
//! * `treasury_llm_transfer`: Transfer LLM from treasury to specified account. Can only be called
//!   by selected accounts and Senate.
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
//! * `ensure_democracy_allowed`: Checks if given account can participate in democracy actions. It
//!   verifies that it's a valid citizen, doesn't have election rights locked and has some LLM
//!   locked in politics.
//! * `ensure_elections_allowed`: Checks if given account can participate in election actions. It
//!   verifies that it's a valid citizen, doesn't have election rights locked.
//!
//! License: MIT

#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;
pub mod traits;

/// Liberland Merit Pallet
/*
decrease the total supply with 0.9 % per year
mint 0.9% per year to the treasury with pallet assets

Author: <filip@rustsyndi.cat>
Copyright © 2022 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

type Assets<T> = pallet_assets::Pallet<T>;

#[frame_support::pallet]
pub mod pallet {
	// Import various types used to declare pallet in scope.
	use super::{traits::LLM, *};
	use frame_support::{
		pallet_prelude::{DispatchResult, *},
		traits::{fungibles::Mutate, Currency},
		PalletId,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use hex_literal::hex;
	use scale_info::prelude::vec;
	use sp_runtime::{
		traits::{AccountIdConversion, StaticLookup},
		AccountId32, SaturatedConversion,
	};
	use sp_std::vec::Vec;

	type BalanceOf<T> = <<T as pallet_identity::Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	/// block number for next LLM inflation mint (transfer of 90% from **Vault** to **Treasury**)
	#[pallet::storage]
	#[pallet::getter(fn next_mint)]
	pub(super) type NextMint<T: Config> = StorageValue<_, u64, ValueQuery>; // ValueQuery ,  OnEmpty = 0

	/// amount of LLM each account has allocated into politics
	#[pallet::storage]
	#[pallet::getter(fn llm_politics)]
	pub(super) type LLMPolitics<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

	/// block number until which account can't do another `politics_unlock`
	#[pallet::storage]
	#[pallet::getter(fn withdraw_lock)]
	pub(super) type Withdrawlock<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>; // account and blocknumber

	#[pallet::type_value]
	pub fn WithdrawlockDurationOnEmpty<T: Config>() -> u64 {
		120u64
	}

	#[pallet::storage]
	#[pallet::getter(fn withdraw_lock_duration)]
	pub(super) type WithdrawlockDuration<T: Config> =
		StorageValue<_, u64, ValueQuery, WithdrawlockDurationOnEmpty<T>>; // seconds

	/// block number until which account can't participate in politics directly
	#[pallet::storage]
	#[pallet::getter(fn election_lock)]
	pub(super) type Electionlock<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>; // account and blocknumber

	#[pallet::type_value]
	pub fn ElectionlockDurationOnEmpty<T: Config>() -> u64 {
		120u64
	}
	#[pallet::storage]
	#[pallet::getter(fn election_lock_duration)]
	pub(super) type ElectionlockDuration<T: Config> =
		StorageValue<_, u64, ValueQuery, ElectionlockDurationOnEmpty<T>>; // seconds

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// duration, in seconds, for which additional unlocks should be locked
		/// after `politics_unlock`
		pub unpooling_withdrawlock_duration: u64,
		/// duration, in seconds, for which politics rights should be suspended
		/// after `politics_unlock`
		pub unpooling_electionlock_duration: u64,
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

		/// Total amount of LLM to be created on genesis. That's all LLM that
		/// will ever exit. It will be stored in **Vault**.
		type TotalSupply: Get<u64>;

		/// Amount of LLM that should be minted (a.k.a. transferred from
		/// **Vault** to **Treasury**) on genesis.
		type PreMintedAmount: Get<u64>; // Pre defined the total supply in runtime

		type AssetId: IsType<<Self as pallet_assets::Config>::AssetId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy;
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

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(b: T::BlockNumber) -> Weight {
			let blocknumber = b.saturated_into::<u64>();
			let _didmint = Self::try_mint(blocknumber);
			Weight::zero()
		}

		fn on_finalize(_n: T::BlockNumber) {}

		fn offchain_worker(_n: T::BlockNumber) {}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Mint LLM to account by transferring LLM from **Vault**. Development
		/// only, to be removed/restricted.  DO NOT USE IN PROD.
		///
		/// - `to_account`: Account to mint LLM to
		/// - `amount`: Amount of LLM to mint
		///
		/// Emits: `Transferred` from `pallet-assets`
		#[pallet::weight(10_000)]
		pub fn fake_send(
			origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			ensure_signed(origin)?;
			let balance = amount.try_into().map_err(|_| Error::<T>::InvalidAmount)?;
			Self::transfer_from_vault(to_account, balance)
		}

		/// Wrapper over `pallet-assets` `transfer`. Transfers LLM from sender
		/// to specified account.
		///
		/// - `to_account`: Account to transfer LLM to
		/// - `amount`: Amount of LLM to transfer
		///
		/// Emits: `Transferred` from `pallet-assets`
		#[pallet::weight(10_000)]
		pub fn send_llm(
			origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			Self::transfer(sender, to_account, amount)
		}

		/// Lock LLM into politics pool, a.k.a. politipool.  Internally it
		/// transfers LLM to **Politipool** account.
		///
		/// * `amount`: Amount of LLM to lock.
		///
		/// Emits:
		/// * `LLMPoliticsLocked`
		/// * `Transferred` from `pallet-assets`
		#[pallet::weight(10_000)]
		pub fn politics_lock(origin: OriginFor<T>, amount: T::Balance) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			Self::transfer_to_politipool(sender.clone(), amount)?;
			LLMPolitics::<T>::mutate(sender.clone(), |b| *b += amount);

			let amount_u64 = amount.try_into().map_err(|_| Error::<T>::InvalidAmount)?;
			Self::deposit_event(Event::<T>::LLMPoliticsLocked(sender, amount_u64));
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
		#[pallet::weight(10_000)]
		pub fn politics_unlock(origin: OriginFor<T>) -> DispatchResult {
			let sender: T::AccountId = ensure_signed(origin.clone())?;
			// check if we have political locked LLM
			log::info!("unlock called");

			let politics_balance = LLMPolitics::<T>::get(sender.clone());
			ensure!(politics_balance > 0u8.into(), Error::<T>::InvalidAccount);

			let current_block_number: u64 =
				<frame_system::Pallet<T>>::block_number().try_into().unwrap_or(0u64);
			ensure!(current_block_number > Withdrawlock::<T>::get(&sender), Error::<T>::Gottawait);

			let ten_percent: T::Balance = Self::get_10_percent(politics_balance);
			log::info!("releasing 10% {:?}", ten_percent.clone());

			Self::transfer_from_politipool(sender.clone(), ten_percent)?;
			LLMPolitics::<T>::mutate(&sender, |b| *b -= ten_percent);

			let withdraw_lock_end =
				Self::get_future_block_with_seconds(Self::withdraw_lock_duration());
			let election_lock_end =
				Self::get_future_block_with_seconds(Self::election_lock_duration());
			Withdrawlock::<T>::insert(&sender, withdraw_lock_end);
			Electionlock::<T>::insert(&sender, election_lock_end);

			let amount = ten_percent.try_into().map_err(|_| Error::<T>::InvalidAmount)?;
			Self::deposit_event(Event::<T>::LLMPoliticsUnlocked(sender, amount));
			Ok(())
		}

		/// Transfer LLM from treasury to specified account. Can only be called
		/// by selected accounts and Senate.
		///
		/// - `to_account`: Account to transfer to.
		/// - `amount`: Amount to transfer.
		///
		/// Emits: `Transferred` from `pallet-assets`
		#[pallet::weight(10_000)]
		pub fn treasury_llm_transfer(
			origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let account_map: Vec<T::AccountId> = vec![
				Self::account_id32_to_accountid(
					hex!["91c7c2ea588cc63a45a540d4f2dbbae7967d415d0daec3d6a5a0641e969c635c"].into(), /* test senate */
				),
				Self::account_id32_to_accountid(
					hex!["9b1e9c82659816b21042772690aafdc58e784aa69eeefdb68fa1e86a036ff634"].into(),
				), // V + DEVKEY + N + M
			];
			let sender: T::AccountId = ensure_signed(origin)?;

			ensure!(account_map.contains(&sender), Error::<T>::InvalidAccount);

			Self::transfer_from_treasury(to_account, amount)
		}

		/// Create LLM manually plus mint it to the treasury
		#[pallet::weight(10_000)] // change me
		pub fn createllm(origin: OriginFor<T>) -> DispatchResult {
			ensure_signed(origin.clone())?;
			Self::create_llm(origin)?;

			Ok(())
		}

		/// Allow the senate to approve transfers
		#[pallet::weight(10_000)]
		pub fn approve_transfer(
			_origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let _x = amount;
			let _y = to_account;
			todo!("approve_transfer");
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New llm has been minted
		MintedLLM(T::AccountId, T::Balance),
		/// New LLM has been created
		LLMCreated(T::AccountId, T::Balance), // acountid, amount
		/// X amount of LLM has been unlocked
		LLMPoliticsLocked(T::AccountId, T::Balance),
		/// sent to user account, amount
		LLMPoliticsUnlocked(T::AccountId, T::Balance),
	}

	impl<T: Config> Pallet<T> {
		fn account_id32_to_accountid(accountid32: AccountId32) -> T::AccountId {
			let mut init_account32 = AccountId32::as_ref(&accountid32);
			let init_account: T::AccountId = T::AccountId::decode(&mut init_account32).unwrap();
			init_account
		}

		pub fn has_llm_politics(sender: T::AccountId) -> bool {
			LLMPolitics::<T>::get(sender) > 0u8.into()
		}

		fn balance(account: T::AccountId) -> T::Balance {
			Assets::<T>::balance(Self::llm_id().into(), account)
		}

		// get 10% of the users balance
		fn get_10_percent(balance: T::Balance) -> T::Balance {
			balance / 10u8.into()
		}

		fn transfer(
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

		fn transfer_to_politipool(
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

		fn transfer_from_vault(to_account: T::AccountId, amount: T::Balance) -> DispatchResult {
			let vault = Self::get_llm_vault_account();
			Self::transfer(vault, to_account, amount)
		}

		fn transfer_from_treasury(to_account: T::AccountId, amount: T::Balance) -> DispatchResult {
			let treasury = Self::get_llm_treasury_account();
			Self::transfer(treasury, to_account, amount)
		}

		// could do like a OriginFor<SenateGroup> or X(Tech) committee
		fn create_llm(origin: OriginFor<T>) -> DispatchResult {
			let assetid = Self::llm_id().into();
			let treasury = Self::get_llm_treasury_account();
			let challenger_lookup: <T::Lookup as StaticLookup>::Source =
				T::Lookup::unlookup(treasury.clone());
			let asset_supply = Assets::<T>::total_supply(assetid.into());
			ensure!(asset_supply == 0u8.into(), Error::<T>::AssetExists); // if the asset supply is zero == that means it is not been created and we can create

			let min_balance: T::Balance = 1u8.into();
			let name: Vec<u8> = "Liberland Merit".into();
			let symbol: Vec<u8> = "LLM".into();
			let decimals: u8 = 0u8;
			Assets::<T>::force_create(
				origin.clone(),
				assetid.into(),
				challenger_lookup,
				true,
				min_balance,
			)?;
			Assets::<T>::force_set_metadata(
				origin.clone(),
				assetid.into(),
				name,
				symbol,
				decimals,
				false,
			)?;
			Self::deposit_event(Event::<T>::LLMCreated(treasury.clone(), 0u8.into()));

			// Mint tokens into the llm/vault
			let vaultac: T::AccountId = Self::get_llm_vault_account();
			let supply = T::TotalSupply::get().try_into().map_err(|_| Error::<T>::InvalidAmount)?;
			Assets::<T>::mint_into(assetid, &vaultac, supply)?;

			let nextblock = Self::get_future_block();
			NextMint::<T>::put(nextblock);

			// Mint a.k.a. transfer to treasury
			let preminted =
				T::PreMintedAmount::get().try_into().map_err(|_| Error::<T>::InvalidAmount)?;
			Self::mint_tokens(assetid.into(), preminted)
		}

		fn llm_id() -> AssetId<T> {
			1u32.into()
		}

		fn get_llm_vault_account() -> T::AccountId {
			PalletId(*b"llm/safe").into_account_truncating()
		}

		fn get_llm_treasury_account() -> T::AccountId {
			PalletId(*b"py/trsry").into_account_truncating()
		}

		fn get_llm_politipool_account() -> T::AccountId {
			PalletId(*b"polilock").into_account_truncating()
		}

		fn get_future_block_with_seconds(seconds: u64) -> u64 {
			let current_block_number: u64 =
				<frame_system::Pallet<T>>::block_number().try_into().unwrap_or(0u64);
			let seconds_per_block: u64 = 6u64; // 6 seconds per block
			let block = current_block_number + seconds / seconds_per_block;
			block
		}

		fn get_future_block() -> u64 {
			let current_block_number: u64 =
				<frame_system::Pallet<T>>::block_number().try_into().unwrap_or(0u64);
			let blocks_per_second: u64 = 6u64; // 6 seconds per block
			let one_minute: u64 = 60u64 / blocks_per_second;
			let one_day: u64 = one_minute * 60u64 * 24u64;
			let _one_year: u64 = one_day * 365u64; //365.24
			let block = current_block_number + 2u64 * one_minute; // 2 minutes
			block
		}

		// each time we mint (should be each year), release 10% from vault
		fn get_allowed_spending() -> T::Balance {
			let asset_id = Self::llm_id().into();
			let vault_account = Self::get_llm_vault_account();
			let unminted_amount = Assets::<T>::balance(asset_id, vault_account);
			let release_amount = unminted_amount / 10u8.into();
			release_amount
		}

		fn try_mint(block: u64) -> bool {
			if block < NextMint::<T>::get() {
				return false
			}
			NextMint::<T>::put(Self::get_future_block());

			let mint_amount = Self::get_allowed_spending();
			Self::mint_tokens(0.into(), mint_amount).unwrap();

			log::info!("try_mint ran all the way");
			true
		}

		/// Mint tokens to the treasury account. Sends tokens from the llm/vault to the treasury
		fn mint_tokens(_assetid: AssetId<T>, amount: T::Balance) -> DispatchResult {
			let treasury = Self::get_llm_treasury_account();

			Self::transfer_from_vault(treasury.clone(), amount)?;
			Self::deposit_event(Event::<T>::MintedLLM(treasury, amount));

			Ok(())
		}
	}

	impl<T: Config> traits::LLM<T::AccountId, T::Balance> for Pallet<T> {
		fn check_pooled_llm(account: &T::AccountId) -> bool {
			LLMPolitics::<T>::get(account) > 0u8.into()
		}

		fn is_election_unlocked(account: &T::AccountId) -> bool {
			if Electionlock::<T>::contains_key(account) {
				let current_block_number: u64 =
					<frame_system::Pallet<T>>::block_number().try_into().unwrap_or(0u64);
				let unlocked_on_block = Electionlock::<T>::get(account);
				return current_block_number >= unlocked_on_block
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
			reg.info.citizen != pallet_identity::Data::None &&
				reg.judgements.contains(&(0u32, pallet_identity::Judgement::KnownGood))
		}

		fn is_known_good(account: &T::AccountId) -> bool {
			match pallet_identity::Pallet::<T>::identity(account) {
				Some(reg) => Self::is_known_good_identity(&reg),
				None => false,
			}
		}
	}

	impl<T: Config> traits::CitizenshipChecker<T::AccountId> for Pallet<T> {
		fn ensure_democracy_allowed(account: &T::AccountId) -> Result<(), DispatchError> {
			ensure!(Self::is_known_good(account), Error::<T>::NonCitizen);
			ensure!(Self::is_election_unlocked(account), Error::<T>::Locked);
			ensure!(Self::check_pooled_llm(account), Error::<T>::NoPolLLM);
			Ok(())
		}

		fn ensure_elections_allowed(account: &T::AccountId) -> Result<(), DispatchError> {
			ensure!(Self::is_known_good(account), Error::<T>::NonCitizen);
			ensure!(Self::is_election_unlocked(account), Error::<T>::Locked);
			Ok(())
		}

		fn is_citizen(account: &T::AccountId) -> bool {
			Self::is_known_good(account)
		}

		fn citizens_count() -> usize {
			pallet_identity::Pallet::<T>::identities_iter()
				.filter(|(_account, registration)| Self::is_known_good_identity(registration))
				.count()
		}
	}
}
