//! # Liberland Legislation Pallet
//!
//! ## Overview
//!
//! The Liberland Legislation pallet handles adding and removing legislations.
//!
//! ### Terminology
//!
//! - **Tier:** Lower tier legislations are more important then higher tiers.
//! - **Index:** Unique identifier of a legislation inside a tier.
//! - **Headcount veto:** Process of legislation repeal driven by citizens.
//!
//! ### Headcount Veto
//!
//! Legislation pallet allows citizens to submit their veto for given legislation.
//! After the required percentage of vetos (different for each tier) of vetos is
//! collected, it's possible to trigger the headcount veto which removes given
//! legislation.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### Signed Origin
//!
//! Basic actions:
//! - `submit_veto` - Registers veto for given legislation for the signer.
//! - `revert_veto` - Removes veto for given legislation for the signer.
//! - `trigger_headcount_veto` - Removes legislation if veto count requirements are met for it.
//!
//! #### Root origin
//!
//! - `add_law` - Adds a new legislation.
//! - `repeal_law` - Removes legislation.
//!
//! License: MIT

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod benchmarking;
mod mock;
mod tests;
pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, Blake2_128Concat};
	use frame_system::pallet_prelude::*;
	use liberland_traits::CitizenshipChecker;

	type Citizenship<T> = <T as Config>::Citizenship;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Citizenship provider used to check for citizenship and number of citizens (for headcount
		/// veto).
		type Citizenship: CitizenshipChecker<Self::AccountId>;
		type LLInitializer: liberland_traits::LLInitializer<Self::AccountId>;

		type ConstitutionOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type InternationalTreatyOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A legislation was added.
		LawAdded { tier: u32, index: u32 },
		/// A legislation was removed.
		LawRepealed { tier: u32, index: u32 },
		/// Citizen submitted a veto for a legislation.
		VetoSubmitted { tier: u32, index: u32, account: T::AccountId },
		/// Citizen reverted their veto for a legislation.
		VetoReverted { tier: u32, index: u32, account: T::AccountId },
		/// Legislation was removed by headcount veto process.
		LawRepealedByHeadcountVeto { tier: u32, index: u32 },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Legislation with given Tier and Index already exists.
		LawAlreadyExists,
		/// Signer isn't a valid citizen.
		NonCitizen,
		/// Invalid tier was requested.
		InvalidTier,
		/// Number of vetos collected didn't meet the requirements for given
		/// tier.
		InsufficientVetoCount,
		/// Given action cannot be done on given (tier, index).
		ProtectedLegislation,
		/// Internal error, for example related to incompatible types
		InternalError,
	}

	#[derive(Clone, Copy)]
	pub enum LegislationTier {
		Constitution = 0,
		InternationalTreaty,
		Tier2, // FIXME proper names
		Tier3,
		Tier4,
		Tier5,
		Decision,
		// If adding anything, update From trait implementation
		InvalidTier, // keep this last
	}

	use LegislationTier::*;

	impl From<u32> for LegislationTier {
		fn from(v: u32) -> Self {
			static VALUES: [LegislationTier; 7] =
				[Constitution, InternationalTreaty, Tier2, Tier3, Tier4, Tier5, Decision];
			for i in VALUES {
				if v == i as u32 {
					return i
				}
			}
			InvalidTier
		}
	}

	/// Registered legislations.
	#[pallet::storage]
	#[pallet::getter(fn laws)]
	pub(super) type Laws<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Blake2_128Concat,
		u32,
		BoundedVec<u8, ConstU32<65536>>,
		ValueQuery,
	>;

	/// Registered vetos per legislation.
	#[pallet::storage]
	#[pallet::getter(fn vetos)]
	pub(super) type Vetos<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, u32>,
			NMapKey<Blake2_128Concat, u32>,
			NMapKey<Blake2_128Concat, T::AccountId>,
		),
		bool,
	>;

	/// VetosCount
	#[pallet::storage]
	#[pallet::getter(fn vetos_count)]
	pub(super) type VetosCount<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, u32, u64, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add a new legislation.
		///
		/// The dispatch origin of this call must be _Root_.
		///
		/// - `tier`: Tier of the legislation.
		/// - `index`: Index of the legislation.
		/// - `law_content`: Content of the legislation.
		///
		/// Will fail with `LawAlreadyExists` if legislation with given `tier` and
		/// `index` already exists.
		///
		/// Emits `LawAdded`.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::add_law(law_content.len() as u32))]
		pub fn add_law(
			origin: OriginFor<T>,
			tier: u32,
			index: u32,
			law_content: BoundedVec<u8, ConstU32<65536>>,
		) -> DispatchResult {
			ensure!(tier < InvalidTier as u32, Error::<T>::InvalidTier);

			match tier.into() {
				Constitution => {
					T::ConstitutionOrigin::ensure_origin(origin)?;
				},
				InternationalTreaty => {
					T::InternationalTreatyOrigin::ensure_origin(origin)?;
				},
				_ => {
					ensure_root(origin)?;
				},
			}

			ensure!(!Laws::<T>::contains_key(&tier, &index), Error::<T>::LawAlreadyExists);

			Laws::<T>::insert(&tier, &index, &law_content);

			Self::deposit_event(Event::LawAdded { tier, index });

			Ok(())
		}

		/// Remove legislation. The result is as if the legislation never existed,
		/// so the `tier` and `index` can be reused to add a new legislation with
		/// `add_law` in the future.
		///
		/// The dispatch origin of this call must be _Root_.
		///
		/// - `tier`: Tier of the legislation.
		/// - `index`: Index of the legislation.
		///
		/// Emits `LawRepealed`.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::repeal_law())]
		pub fn repeal_law(origin: OriginFor<T>, tier: u32, index: u32) -> DispatchResult {
			ensure!(tier < InvalidTier as u32, Error::<T>::InvalidTier);

			match tier.into() {
				Constitution => {
					T::ConstitutionOrigin::ensure_origin(origin)?;
					if index == 0 {
						return Err(Error::<T>::ProtectedLegislation.into())
					}
				},
				InternationalTreaty => {
					T::InternationalTreatyOrigin::ensure_origin(origin)?;
				},
				_ => {
					ensure_root(origin)?;
				},
			}

			Laws::<T>::remove(&tier, &index);

			Self::deposit_event(Event::LawRepealed { tier, index });

			Ok(())
		}

		/// Add a veto.
		///
		/// The dispatch origin of this call must be _Signed_.
		///
		/// - `tier`: Tier of the legislation.
		/// - `index`: Index of the legislation.
		///
		/// Will fail with `NonCitizen` if caller isn't a valid citizen.
		///
		/// Emits `VetoSubmitted`.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::submit_veto())]
		pub fn submit_veto(origin: OriginFor<T>, tier: u32, index: u32) -> DispatchResult {
			let account = ensure_signed(origin)?;
			ensure!(tier != Constitution as u32, Error::<T>::InvalidTier);
			ensure!(tier < InvalidTier as u32, Error::<T>::InvalidTier);
			ensure!(Citizenship::<T>::is_citizen(&account), Error::<T>::NonCitizen);
			let key = (tier, index, &account);
			if !Vetos::<T>::contains_key(key) {
				VetosCount::<T>::mutate(tier, index, |x| *x = *x + 1);
				Vetos::<T>::insert(key, true);
				Self::deposit_event(Event::<T>::VetoSubmitted { tier, index, account });
			}
			Ok(())
		}

		/// Remove a veto.
		///
		/// The dispatch origin of this call must be _Signed_.
		///
		/// - `tier`: Tier of the legislation.
		/// - `index`: Index of the legislation.
		///
		/// Emits `VetoReverted`.
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::revert_veto())]
		pub fn revert_veto(origin: OriginFor<T>, tier: u32, index: u32) -> DispatchResult {
			let account = ensure_signed(origin)?;
			let key = (&tier, &index, &account);
			if Vetos::<T>::contains_key(key) {
				VetosCount::<T>::mutate(tier, index, |x| *x = *x - 1);
				Vetos::<T>::remove(key);
				Self::deposit_event(Event::<T>::VetoReverted { tier, index, account });
			}
			Ok(())
		}

		/// Trigger a headcount veto, which removes a legislation. Registered
		/// vetos are verified (to make sure citizenships are still valid)
		/// before counting.
		///
		/// The dispatch origin of this call must be _Signed_.
		///
		/// - `tier`: Tier of the legislation.
		/// - `index`: Index of the legislation.
		///
		/// Will fail with `InsufficientVetoCount` if number of valid vetos
		/// doesn't meet requirements for given Tier.
		///
		/// Emits `LawRepealedByHeadcountVeto`.
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::trigger_headcount_veto(Citizenship::<T>::citizens_count() as u32))]
		pub fn trigger_headcount_veto(
			origin: OriginFor<T>,
			tier: u32,
			index: u32,
		) -> DispatchResult {
			ensure_signed(origin)?;

			ensure!(tier != Constitution as u32, Error::<T>::InvalidTier);

			let citizens = Citizenship::<T>::citizens_count();
			let required = match tier.into() {
				InternationalTreaty => citizens / 2 + 1,
				Tier2 => citizens / 2 + 1,
				Tier3 => citizens / 2 + 1,
				Tier4 => citizens / 2 + 1,
				Tier5 => citizens / 2 + 1,
				Decision => citizens / 2 + 1,
				_ => return Err(Error::<T>::InvalidTier.into()),
			};

			let valid_vetos: u64 = Vetos::<T>::iter_key_prefix((tier, index))
				.filter(|sender| Citizenship::<T>::is_citizen(sender))
				.count()
				.try_into()
				.map_err(|_| Error::<T>::InternalError)?;

			ensure!(valid_vetos >= required, Error::<T>::InsufficientVetoCount);

			Laws::<T>::remove(&tier, &index);
			Self::deposit_event(Event::LawRepealedByHeadcountVeto { tier, index });

			// FIXME we should allow doing this over multiple transactions by saving cursor in
			// storage. See example: https://github.com/paritytech/substrate/blob/70351393fd632317124f35ab8b24ef7134e08864/frame/ranked-collective/src/lib.rs#L622
			// We could skip clearing if instead we prevent reusing indexes of repealed laws
			let mut res = Vetos::<T>::clear_prefix((tier, index), u32::MAX, None);
			while let Some(cursor) = res.maybe_cursor {
				res = Vetos::<T>::clear_prefix((tier, index), u32::MAX, Some(&cursor));
			}
			VetosCount::<T>::remove(tier, index);

			Ok(())
		}
	}
}
