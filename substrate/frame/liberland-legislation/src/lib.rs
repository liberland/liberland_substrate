//! # Liberland Legislation Pallet
//!
//! ## Overview
//!
//! The Liberland Legislation pallet handles adding and removing legislations.
//!
//! ### Terminology
//!
//! - **Tier:** Lower tier legislations are more important then higher tiers.
//! - **Id:** Unique identifier of a legislation inside a tier. Composed of:
//!     - **Year**
//!     - **Index**
//! - **Section:** Part of legislation that can be amended, repealed or referenced directly.
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
//! - `add_legislation` - Adds a new legislation.
//! - `amend_legislation` - Change existing section or add a new section to existing legislation.
//! - `repeal_legislation` - Repeals whole legislation (all sections).
//! - `repeal_legislation_section` - Repeals single legislation.
//! - `submit_veto` - Registers veto for given legislation (or its specific section) for the signer.
//! - `revert_veto` - Removes veto for given legislation (or its specific section) for the signer.
//! - `trigger_headcount_veto` - Repeals legislation (all sections) if veto count requirements are met for it.
//! - `trigger_section_headcount_veto` - Repeals legislation section if veto count requirements are met for it.
//!
//!
//! License: MIT

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod benchmarking;
pub mod migrations;
mod mock;
mod tests;
pub mod types;
pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, Blake2_128Concat};
	use frame_system::pallet_prelude::*;
	use liberland_traits::CitizenshipChecker;
	use types::{LegislationContent, LegislationId, LegislationSection, LegislationTier};
	use LegislationTier::*;

	type Citizenship<T> = <T as Config>::Citizenship;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
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
		type LowTierDeleteOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A legislation was added.
		LegislationAdded { tier: LegislationTier, id: LegislationId },
		/// A legislation was amended.
		LegislationAmended { tier: LegislationTier, id: LegislationId, section: LegislationSection },
		/// A legislation was removed.
		LegislationRepealed {
			tier: LegislationTier,
			id: LegislationId,
			section: Option<LegislationSection>,
		},
		/// Citizen submitted a veto for a legislation.
		VetoSubmitted {
			tier: LegislationTier,
			id: LegislationId,
			section: Option<LegislationSection>,
			account: T::AccountId,
		},
		/// Citizen reverted their veto for a legislation.
		VetoReverted {
			tier: LegislationTier,
			id: LegislationId,
			section: Option<LegislationSection>,
			account: T::AccountId,
		},
		/// Legislation was removed by headcount veto process.
		LegislationRepealedByHeadcountVeto {
			tier: LegislationTier,
			section: Option<LegislationSection>,
			id: LegislationId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Legislation with given Tier and Id already exists.
		LegislationAlreadyExists,
		/// Legislation with given Tier and Id doesn't exist.
		InvalidLegislation,
		/// Invalid witness data - maybe legislation changed in the meantie
		InvalidWitness,
		/// Signer isn't a valid citizen.
		NonCitizen,
		/// Invalid tier was requested.
		InvalidTier,
		/// Number of vetos collected didn't meet the requirements for given
		/// tier.
		InsufficientVetoCount,
		/// Given action cannot be done on given (tier, id).
		ProtectedLegislation,
		/// Internal error, for example related to incompatible types
		InternalError,
		/// Trying to add legislation with no sections
		EmptyLegislation,
	}

	/// Registered legislations.
	///
	/// If it doesn't exist, then it never existed.
	/// If it exists but is None, then it was repealed.
	/// If it exists and is Some, it's in legal force.
	#[pallet::storage]
	#[pallet::getter(fn legislation)]
	pub(super) type Legislation<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, LegislationTier>,
			NMapKey<Blake2_128Concat, LegislationId>,
			NMapKey<Blake2_128Concat, LegislationSection>,
		),
		Option<LegislationContent>,
		OptionQuery,
	>;

	/// Legislation versions - used as witness data
	#[pallet::storage]
	#[pallet::getter(fn legislation_versions)]
	pub(super) type LegislationVersion<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, LegislationTier>,
			NMapKey<Blake2_128Concat, LegislationId>,
			NMapKey<Blake2_128Concat, Option<LegislationSection>>,
		),
		u64,
		ValueQuery,
	>;

	/// Registered vetos per legislation.
	#[pallet::storage]
	#[pallet::getter(fn vetos)]
	pub(super) type Vetos<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, LegislationTier>,
			NMapKey<Blake2_128Concat, LegislationId>,
			NMapKey<Blake2_128Concat, Option<LegislationSection>>, // None is a veto for whole Id
			NMapKey<Blake2_128Concat, T::AccountId>,
		),
		bool,
	>;

	/// VetosCount
	#[pallet::storage]
	#[pallet::getter(fn vetos_count)]
	pub(super) type VetosCount<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, LegislationTier>,
			NMapKey<Blake2_128Concat, LegislationId>,
			NMapKey<Blake2_128Concat, Option<LegislationSection>>,
		),
		u64,
		ValueQuery,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add completely new legislation.
		///
		/// The dispatch origin of this call must be:
		/// * _ConstitutionOrigin_ if _tier_ is _Constitution_,
		/// * _InternationalTreatyOrigin_ if _tier_ is _InternationalTreaty_,
		/// * _Root_ otherwise.
		///
		/// - `tier`: Tier of the legislation.
		/// - `id`: Id of the legislation.
		/// - `sections`: content of the legislation - by section
		///
		/// Will fail with:
		/// * `InvalidTier` if `tier` is invalid
		/// * `EmptyLegislation` if there are no sections passed
		/// * `BadOrigin` if `origin` is invalid for given `tier`
		/// * `LegislationAlreadyExists` if legislation with this `tier` and `id` exists
		///
		/// Emits `LegislationAdded`.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::add_legislation(sections.len() as u32))]
		pub fn add_legislation(
			origin: OriginFor<T>,
			tier: LegislationTier,
			id: LegislationId,
			sections: BoundedVec<LegislationContent, ConstU32<1024>>,
		) -> DispatchResult {
			ensure!(tier < InvalidTier, Error::<T>::InvalidTier);
			ensure!(!sections.is_empty(), Error::<T>::EmptyLegislation);

			match tier {
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

			ensure!(
				!Legislation::<T>::contains_key((&tier, &id, 0)),
				Error::<T>::LegislationAlreadyExists
			);

			for (idx, content) in sections.iter().enumerate() {
				let idx = idx as LegislationSection;
				Legislation::<T>::insert((&tier, &id, &idx), Some(content));
				LegislationVersion::<T>::insert((&tier, &id, Some(idx)), 1);
			}
			LegislationVersion::<T>::insert((&tier, &id, None::<LegislationSection>), 1);

			Self::deposit_event(Event::LegislationAdded { tier, id });

			Ok(())
		}

		/// Repeal whole legislation (all sections). Doesn't remove keys, only
		/// marks as None.
		///
		/// The dispatch origin of this call must be:
		/// * _ConstitutionOrigin_ if _tier_ is _Constitution_,
		/// * _InternationalTreatyOrigin_ if _tier_ is _InternationalTreaty_,
		/// * _Root_ if _tier_ is _Law_.
		/// * _LowTierDeleteOrigin_ otherwise.
		///
		/// - `tier`: Tier of the legislation.
		/// - `id`: Id of the legislation.
		/// - `witness`: Current version of legislation.
		///
		/// Will fail with:
		/// * `InvalidTier` if `tier` is invalid,
		/// * `ProtectedLegislation` if trying to repeal Constitution Year 0 Index 0,
		/// * `BadOrigin` if `origin` is invalid for given `tier`,
		/// * `InvalidWitness` if `witness` doesn't match current legislation version,
		/// * `InvalidLegislation` if legislation with this `tier` and `id` doesn't exist,
		///
		/// Emits `LegislationRepealed`.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::repeal_legislation(
			Legislation::<T>::iter_key_prefix((tier, id)).count() as u32
		))]
		pub fn repeal_legislation(
			origin: OriginFor<T>,
			tier: LegislationTier,
			id: LegislationId,
			witness: u64,
		) -> DispatchResult {
			ensure!(tier < InvalidTier, Error::<T>::InvalidTier);

			match tier {
				Constitution => {
					T::ConstitutionOrigin::ensure_origin(origin)?;
					if id.year == 0 && id.index == 0 {
						return Err(Error::<T>::ProtectedLegislation.into());
					}
				},
				InternationalTreaty => {
					T::InternationalTreatyOrigin::ensure_origin(origin)?;
				},
				Law => {
					ensure_root(origin)?;
				},
				_ => {
					T::LowTierDeleteOrigin::ensure_origin(origin)?;
				},
			}

			let current_version =
				LegislationVersion::<T>::get((tier, id, None::<LegislationSection>));
			ensure!(current_version == witness, Error::<T>::InvalidWitness);

			ensure!(Legislation::<T>::contains_key((tier, id, 0)), Error::<T>::InvalidLegislation,);

			Self::do_repeal(tier, id, None);
			Self::deposit_event(Event::LegislationRepealed { tier, id, section: None });

			Ok(())
		}

		/// Repeal single section of legislation. Doesn't remove keys, only
		/// marks as None.
		///
		/// The dispatch origin of this call must be:
		/// * _ConstitutionOrigin_ if _tier_ is _Constitution_,
		/// * _InternationalTreatyOrigin_ if _tier_ is _InternationalTreaty_,
		/// * _Root_ if _tier_ is _Law_.
		/// * _LowTierDeleteOrigin_ otherwise.
		///
		/// - `tier`: Tier of the legislation.
		/// - `id`: Id of the legislation.
		/// - `section`: Index of section to remove
		/// - `witness`: Current version of legislation.
		///
		/// Will fail with:
		/// * `InvalidTier` if `tier` is invalid,
		/// * `ProtectedLegislation` if trying to repeal Constitution Year 0 Index 0,
		/// * `BadOrigin` if `origin` is invalid for given `tier`,
		/// * `InvalidWitness` if `witness` doesn't match current section version,
		/// * `InvalidLegislation` if section with this `tier`, `id` and `section` doesn't exist,
		///
		/// Emits `LegislationRepealed`.
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::repeal_legislation_section())]
		pub fn repeal_legislation_section(
			origin: OriginFor<T>,
			tier: LegislationTier,
			id: LegislationId,
			section: LegislationSection,
			witness: u64,
		) -> DispatchResult {
			ensure!(tier < InvalidTier, Error::<T>::InvalidTier);

			match tier {
				Constitution => {
					T::ConstitutionOrigin::ensure_origin(origin)?;
					if id.year == 0 && id.index == 0 {
						return Err(Error::<T>::ProtectedLegislation.into());
					}
				},
				InternationalTreaty => {
					T::InternationalTreatyOrigin::ensure_origin(origin)?;
				},
				Law => {
					ensure_root(origin)?;
				},
				_ => {
					T::LowTierDeleteOrigin::ensure_origin(origin)?;
				},
			}

			let current_version = LegislationVersion::<T>::get((tier, id, Some(section)));
			ensure!(current_version == witness, Error::<T>::InvalidWitness);

			ensure!(
				Legislation::<T>::contains_key((tier, id, section)),
				Error::<T>::InvalidLegislation,
			);

			Self::do_repeal(tier, id, Some(section));
			Self::deposit_event(Event::LegislationRepealed { tier, id, section: Some(section) });

			Ok(())
		}

		/// Add a veto to whole legislation or single section
		///
		/// The dispatch origin of this call must be _Signed_.
		///
		/// - `tier`: Tier of the legislation.
		/// - `id`: Id of the legislation.
		/// - `section`: Section to repeal (None to apply to whole legislation)
		///
		/// Will fail with:
		/// - `BadOrigin` if called by origin other than _Signed_
		/// - `NonCitizen` if caller isn't a valid citizen.
		/// - `InvalidTier` if called for Constitution
		/// - `InvalidTier` if called for invalid tier
		///
		/// Emits `VetoSubmitted`.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::submit_veto())]
		pub fn submit_veto(
			origin: OriginFor<T>,
			tier: LegislationTier,
			id: LegislationId,
			section: Option<LegislationSection>,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;
			ensure!(tier != Constitution, Error::<T>::InvalidTier);
			ensure!(tier < InvalidTier, Error::<T>::InvalidTier);
			ensure!(Citizenship::<T>::is_citizen(&account), Error::<T>::NonCitizen);
			let key = (tier, id, section, &account);
			if !Vetos::<T>::contains_key(key) {
				VetosCount::<T>::mutate((tier, id, section), |x| *x += 1);
				Vetos::<T>::insert(key, true);
				Self::deposit_event(Event::<T>::VetoSubmitted { tier, id, section, account });
			}
			Ok(())
		}

		/// Revert a veto to whole legislation or single section
		///
		/// The dispatch origin of this call must be _Signed_.
		///
		/// - `tier`: Tier of the legislation.
		/// - `id`: Id of the legislation.
		/// - `section`: Section to repeal (None to apply to whole legislation)
		///
		/// Will fail with:
		/// - `BadOrigin` if called by origin other than _Signed_
		///
		/// Emits `VetoReverted`.
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::revert_veto())]
		pub fn revert_veto(
			origin: OriginFor<T>,
			tier: LegislationTier,
			id: LegislationId,
			section: Option<LegislationSection>,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;
			let key = (&tier, &id, &section, &account);
			if Vetos::<T>::contains_key(key) {
				VetosCount::<T>::mutate((tier, id, section), |x| *x -= 1);
				Vetos::<T>::remove(key);
				Self::deposit_event(Event::<T>::VetoReverted { tier, id, section, account });
			}
			Ok(())
		}

		/// Trigger a headcount veto, which removes a whole legislation.
		/// Registered vetos are verified (to make sure citizenships are still
		/// valid) before counting.
		///
		/// The dispatch origin of this call must be _Signed_.
		///
		/// - `tier`: Tier of the legislation.
		/// - `id`: Id of the legislation.
		///
		/// Will fail with `InsufficientVetoCount` if number of valid vetos
		/// doesn't meet requirements for given Tier.
		///
		/// Emits `LegislationRepealedByHeadcountVeto`.
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::trigger_headcount_veto(Citizenship::<T>::citizens_count() as u32))]
		pub fn trigger_headcount_veto(
			origin: OriginFor<T>,
			tier: LegislationTier,
			id: LegislationId,
		) -> DispatchResult {
			ensure_signed(origin)?;
			Self::do_headcount_veto(tier, id, None)
		}

		/// Trigger a headcount veto for single legislation section. Registered
		/// vetos are verified (to make sure citizenships are still valid)
		/// before counting.
		///
		/// The dispatch origin of this call must be _Signed_.
		///
		/// - `tier`: Tier of the legislation.
		/// - `id`: Id of the legislation.
		/// - `section`: Index of section
		///
		/// Will fail with `InsufficientVetoCount` if number of valid vetos
		/// doesn't meet requirements for given Tier.
		///
		/// Emits `LegislationRepealedByHeadcountVeto`.
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::trigger_section_headcount_veto(Citizenship::<T>::citizens_count() as u32))]
		pub fn trigger_section_headcount_veto(
			origin: OriginFor<T>,
			tier: LegislationTier,
			id: LegislationId,
			section: LegislationSection,
		) -> DispatchResult {
			ensure_signed(origin)?;
			Self::do_headcount_veto(tier, id, Some(section))
		}

		/// Amend legislation section. Can be used to add completely new section.
		///
		/// The dispatch origin of this call must be:
		/// * _ConstitutionOrigin_ if _tier_ is _Constitution_,
		/// * _InternationalTreatyOrigin_ if _tier_ is _InternationalTreaty_,
		/// * _Root_ otherwise.
		///
		/// - `tier`: Tier of the legislation.
		/// - `id`: Id of the legislation.
		/// - `section`: Index of section to amend
		/// - `new_content`: New content of section
		/// - `witness`: Current version of section
		///
		/// Will fail with:
		/// * `InvalidTier` if `tier` is invalid,
		/// * `InvalidLegislation` if legislation with this `tier` and `id` doesn't exist,
		/// * `BadOrigin` if `origin` is invalid for given `tier`,
		/// * `ProtectedLegislation` if trying to repeal Constitution Year 0 Index 0,
		/// * `InvalidWitness` if `witness` doesn't match current legislation version,
		///
		/// Emits `LegislationAmended`.
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::amend_legislation(new_content.len() as u32))]
		pub fn amend_legislation(
			origin: OriginFor<T>,
			tier: LegislationTier,
			id: LegislationId,
			section: LegislationSection,
			new_content: LegislationContent,
			witness: u64,
		) -> DispatchResult {
			ensure!(tier < InvalidTier, Error::<T>::InvalidTier);

			ensure!(Legislation::<T>::contains_key((tier, id, 0)), Error::<T>::InvalidLegislation,);

			match tier {
				Constitution => {
					T::ConstitutionOrigin::ensure_origin(origin)?;
					if id.year == 0 && id.index == 0 {
						return Err(Error::<T>::ProtectedLegislation.into());
					}
				},
				InternationalTreaty => {
					T::InternationalTreatyOrigin::ensure_origin(origin)?;
				},
				_ => {
					ensure_root(origin)?;
				},
			}

			let current_version = LegislationVersion::<T>::get((tier, id, Some(section)));
			ensure!(current_version == witness, Error::<T>::InvalidWitness);

			Legislation::<T>::insert((&tier, &id, &section), Some(new_content));
			LegislationVersion::<T>::mutate((&tier, &id, Some(section)), |v| *v += 1);

			Self::deposit_event(Event::LegislationAmended { tier, id, section });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn do_headcount_veto(
			tier: LegislationTier,
			id: LegislationId,
			section: Option<LegislationSection>,
		) -> DispatchResult {
			ensure!(tier != Constitution, Error::<T>::InvalidTier);

			let citizens = Citizenship::<T>::citizens_count();
			let required = match tier {
				InternationalTreaty => citizens / 2 + 1,
				Law => citizens / 2 + 1,
				Tier3 => citizens / 2 + 1,
				Tier4 => citizens / 2 + 1,
				Tier5 => citizens / 2 + 1,
				Decision => citizens / 2 + 1,
				_ => return Err(Error::<T>::InvalidTier.into()),
			};

			let valid_vetos: u64 = Vetos::<T>::iter_key_prefix((tier, id, section))
				.filter(|sender| Citizenship::<T>::is_citizen(sender))
				.count()
				.try_into()
				.map_err(|_| Error::<T>::InternalError)?;

			ensure!(valid_vetos >= required, Error::<T>::InsufficientVetoCount);

			Self::do_repeal(tier, id, section);
			Self::deposit_event(Event::LegislationRepealedByHeadcountVeto { tier, id, section });

			Ok(())
		}

		fn do_repeal(
			tier: LegislationTier,
			id: LegislationId,
			section: Option<LegislationSection>,
		) {
			if let Some(section) = section {
				Legislation::<T>::insert((&tier, &id, &section), None::<LegislationContent>);
				LegislationVersion::<T>::mutate((&tier, &id, Some(section)), |v| *v += 1);
			} else {
				for idx in Legislation::<T>::iter_key_prefix((&tier, &id)) {
					let idx = idx as LegislationSection;
					Legislation::<T>::insert((&tier, &id, &idx), None::<LegislationContent>);
					LegislationVersion::<T>::mutate((&tier, &id, Some(idx)), |v| *v += 1);
				}
			}
		}
	}
}
