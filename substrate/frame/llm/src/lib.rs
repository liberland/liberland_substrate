#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

mod benchmarking;
mod mock;
mod tests;
pub mod weights;
pub use weights::WeightInfo;

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
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum LLMAccount<AccountId> {
	Liquid(AccountId),
	Locked(AccountId),
}

#[frame_support::pallet(dev_mode)] // FIXME
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
		AccountId32, Perbill, Permill,
	};
	use sp_std::vec::Vec;

	pub type RemarkData = BoundedVec<u8, ConstU32<256>>;

	/// block number of last LLM release event (transfer from **Vault** to **Treasury**)
	#[pallet::storage]
	#[pallet::getter(fn last_release)]
	pub(super) type LastRelease<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>; // ValueQuery ,  OnEmpty = 0

	/// amount of LLM each account has allocated into politics
	#[pallet::storage]
	#[pallet::getter(fn llm_politics)]
	pub(super) type LLMPolitics<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

	/// block number until which account can't do another `politics_unlock`
	#[pallet::storage]
	#[pallet::getter(fn withdraw_lock)]
	pub(super) type Withdrawlock<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumberFor<T>, ValueQuery>; // account and blocknumber

	#[pallet::type_value]
	pub fn WithdrawlockDurationOnEmpty<T: Config>() -> BlockNumberFor<T> {
		120u8.into()
	}

	#[pallet::storage]
	#[pallet::getter(fn withdraw_lock_duration)]
	pub(super) type WithdrawlockDuration<T: Config> =
		StorageValue<_, BlockNumberFor<T>, ValueQuery, WithdrawlockDurationOnEmpty<T>>; // seconds

	/// block number until which account can't participate in politics directly
	#[pallet::storage]
	#[pallet::getter(fn election_lock)]
	pub(super) type Electionlock<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumberFor<T>, ValueQuery>; // account and blocknumber

	#[pallet::type_value]
	pub fn ElectionlockDurationOnEmpty<T: Config>() -> BlockNumberFor<T> {
		120u8.into()
	}
	#[pallet::storage]
	#[pallet::getter(fn election_lock_duration)]
	pub(super) type ElectionlockDuration<T: Config> =
		StorageValue<_, BlockNumberFor<T>, ValueQuery, ElectionlockDurationOnEmpty<T>>; // seconds

	#[pallet::storage]
	#[pallet::getter(fn citizens)]
	pub(super) type Citizens<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn courts)]
	pub(super) type Courts<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxCourts>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// duration, in blocks, for which additional unlocks should be locked
		/// after `politics_unlock`
		pub unpooling_withdrawlock_duration: BlockNumberFor<T>,
		/// duration, in blocks, for which politics rights should be suspended
		/// after `politics_unlock`
		pub unpooling_electionlock_duration: BlockNumberFor<T>,
		pub _phantom: PhantomData<T>,
	}

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
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
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
		#[pallet::constant]
		type CitizenshipMinimumPooledLLM: Get<u128>; // Pre defined the total supply in runtime

		/// How much funds unlock on politics_unlock
		#[pallet::constant]
		type UnlockFactor: Get<Permill>;

		/// Senate origin - can transfer from treasury
		type SenateOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		#[pallet::constant]
		type AssetId: Get<<Self as pallet_assets::Config>::AssetId>;
		type AssetName: Get<Vec<u8>>;
		type AssetSymbol: Get<Vec<u8>>;

		#[pallet::constant]
		type InflationEventInterval: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type InflationEventReleaseFactor: Get<Perbill>;

		type OnLLMPoliticsUnlock: OnLLMPoliticsUnlock<Self::AccountId>;
		type WeightInfo: WeightInfo;
		type MaxCourts: Get<u32>;
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
		/// Caller isn't an authorized court
		NotCourt,
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(4);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(b: BlockNumberFor<T>) -> Weight {
			if let Err(e) = Self::maybe_release(b) {
				log::error!("LLM maybe_release failure: {e:?}");
			};
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
		#[pallet::weight(<T as Config>::WeightInfo::politics_lock())]
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
		/// * `LLMPoliticsUnlocked`
		/// * `Transferred` from `pallet-assets`
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::politics_unlock())]
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
		#[pallet::weight(<T as Config>::WeightInfo::treasury_llm_transfer())]
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
		#[pallet::weight(<T as Config>::WeightInfo::treasury_llm_transfer_to_politipool())]
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
		#[pallet::weight(<T as Config>::WeightInfo::send_llm_to_politipool())]
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
		#[pallet::weight(<T as Config>::WeightInfo::send_llm())]
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
		#[pallet::weight(<T as Config>::WeightInfo::treasury_lld_transfer())]
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

		/// Emit Remarked event. Used by Liberland tooling to annotate transfers.
		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::remark(data.len() as u32))]
		pub fn remark(_origin: OriginFor<T>, data: RemarkData) -> DispatchResult {
			Self::deposit_event(Event::<T>::Remarked(data));
			Ok(())
		}

		/// Force transfer LLM. Can only be called by Courts.
		///
		/// - `from`: Account to transfer from.
		/// - `to`: Account to transfer to.
		/// - `amount`: Amount to transfer.
		///
		/// Emits:
		/// * `LLMPoliticsLocked`
		/// * `LLMPoliticsUnlocked`
		#[pallet::call_index(8)]
		#[pallet::weight(<T as Config>::WeightInfo::force_transfer())]
		pub fn force_transfer(
			origin: OriginFor<T>,
			from: LLMAccount<T::AccountId>,
			to: LLMAccount<T::AccountId>,
			amount: T::Balance,
		) -> DispatchResult {
			let caller: T::AccountId = ensure_signed(origin)?;
			ensure!(Courts::<T>::get().contains(&caller), Error::<T>::NotCourt);

			let politipool_account = Self::get_llm_politipool_account();
			let transfer_from = match from {
				LLMAccount::Liquid(_) => return Err(Error::<T>::InvalidAccount.into()),
				LLMAccount::Locked(account) => {
					let politics_balance = LLMPolitics::<T>::get(account.clone());
					ensure!(politics_balance >= amount, Error::<T>::LowBalance);

					LLMPolitics::<T>::mutate(account.clone(), |b| *b -= amount);
					Self::deposit_event(Event::<T>::LLMPoliticsUnlocked(account, amount));
					politipool_account.clone()
				},
			};
			let transfer_to = match to {
				LLMAccount::Liquid(account) => account,
				LLMAccount::Locked(account) => {
					LLMPolitics::<T>::mutate(account.clone(), |b| *b += amount);
					Self::deposit_event(Event::<T>::LLMPoliticsLocked(account, amount));
					politipool_account
				},
			};
			if transfer_from != transfer_to {
				Self::transfer(transfer_from, transfer_to, amount)?;
			}
			Ok(())
		}

		/// Set Courts. Can only be called by Root
		///
		/// - `courts`: New set of authorized courts
		#[pallet::call_index(9)]
		#[pallet::weight(<T as Config>::WeightInfo::set_courts(courts.len() as u32))]
		pub fn set_courts(
			origin: OriginFor<T>,
			courts: BoundedVec<T::AccountId, T::MaxCourts>,
		) -> DispatchResult {
			ensure_root(origin)?;
			Courts::<T>::set(courts);
			Ok(())
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
		/// Remark
		Remarked(RemarkData),
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
			LastRelease::<T>::put(frame_system::Pallet::<T>::block_number());

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

		fn get_release_amount() -> Result<T::Balance, Error<T>> {
			let asset_id = Self::llm_id().into();
			let vault_account = Self::get_llm_vault_account();
			let unreleased_amount = Assets::<T>::balance(asset_id, vault_account);

			let factor = T::InflationEventReleaseFactor::get();
			let release_amount = factor.mul_floor(unreleased_amount);
			release_amount.try_into().map_err(|_| Error::<T>::InvalidAmount)
		}

		fn maybe_release(block: BlockNumberFor<T>) -> DispatchResult {
			let next_release = LastRelease::<T>::get() + T::InflationEventInterval::get();
			if block < next_release {
				return Ok(());
			}

			LastRelease::<T>::put(next_release);
			let release_amount = Self::get_release_amount()?;
			if release_amount > 0u8.into() {
				log::info!("LLM - releasing {release_amount:?} from vault");
				Self::release_tokens_from_vault(release_amount)?;
			}

			Ok(())
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
				return current_block_number > unlocked_on_block;
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
			let eligible_on: Result<BlockNumberFor<T>, _> = eligible_on.try_into();

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

		fn ensure_validate_allowed(account: &T::AccountId) -> Result<(), DispatchError> {
			ensure!(Self::is_known_good(account), Error::<T>::NonCitizen);
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
