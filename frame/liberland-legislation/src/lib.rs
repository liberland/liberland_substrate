#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, Blake2_128Concat};
	use frame_system::pallet_prelude::*;
	use pallet_llm::traits::CitizenshipChecker;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Citizenship: CitizenshipChecker<Self::AccountId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		LawAdded { tier: u32, index: u32 },
		LawRepealed { tier: u32, index: u32 },
		VetoSubmitted { tier: u32, index: u32, account: T::AccountId },
		VetoReverted { tier: u32, index: u32, account: T::AccountId },
		LawRepealedByHeadcountVeto { tier: u32, index: u32 },
	}

	#[pallet::error]
	pub enum Error<T> {
		LawAlreadyExists,
		NonCitizen,
		InvalidTier,
		InsufficientVetoCount,
	}

	#[derive(Clone, Copy)]
	pub enum LegislationTier {
		Constitution = 0,
		Tier1, // FIXME proper names
		Tier2,
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
				[Constitution, Tier1, Tier2, Tier3, Tier4, Tier5, Decision];
			for i in VALUES {
				if v == i as u32 {
					return i
				}
			}
			InvalidTier
		}
	}

	#[pallet::storage]
	//metadata stored on centralized db in order for it to be available during proposal, referendum
	pub(super) type Laws<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Blake2_128Concat,
		u32,
		BoundedVec<u8, ConstU32<65536>>,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type Vetos<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, u32>,
			NMapKey<Blake2_128Concat, u32>,
			NMapKey<Blake2_128Concat, T::AccountId>,
		),
		bool,
	>;

	#[pallet::storage]
	pub(super) type VetoCleanup<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Blake2_128Concat,
		u32,
		BoundedVec<u8, ConstU32<65536>>,
		ValueQuery,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn add_law(
			origin: OriginFor<T>,
			tier: u32,
			index: u32,
			law_content: BoundedVec<u8, ConstU32<65536>>,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(tier < InvalidTier as u32, Error::<T>::InvalidTier);
			ensure!(!Laws::<T>::contains_key(&tier, &index), Error::<T>::LawAlreadyExists);

			Laws::<T>::insert(&tier, &index, &law_content);

			Self::deposit_event(Event::LawAdded { tier, index });

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn repeal_law(origin: OriginFor<T>, tier: u32, index: u32) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(tier < InvalidTier as u32, Error::<T>::InvalidTier);

			Laws::<T>::remove(&tier, &index);

			Self::deposit_event(Event::LawRepealed { tier, index });

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn submit_veto(origin: OriginFor<T>, tier: u32, index: u32) -> DispatchResult {
			let account = ensure_signed(origin)?;
			ensure!(tier != Constitution as u32, Error::<T>::InvalidTier);
			ensure!(tier < InvalidTier as u32, Error::<T>::InvalidTier);
			ensure!(T::Citizenship::is_citizen(&account), Error::<T>::NonCitizen);
			Vetos::<T>::insert((tier, index, &account), true);
			Self::deposit_event(Event::<T>::VetoSubmitted { tier, index, account });
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn revert_veto(origin: OriginFor<T>, tier: u32, index: u32) -> DispatchResult {
			let account = ensure_signed(origin)?;
			Vetos::<T>::remove((&tier, &index, &account));
			Self::deposit_event(Event::<T>::VetoReverted { tier, index, account });
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn trigger_headcount_veto(
			origin: OriginFor<T>,
			tier: u32,
			index: u32,
		) -> DispatchResult {
			ensure_signed(origin)?;

			ensure!(tier != Constitution as u32, Error::<T>::InvalidTier);

			let citizens = T::Citizenship::citizens_count();
			let required = match tier.into() {
				Tier1 => citizens / 2 + 1,
				Tier2 => citizens / 2 + 1,
				Tier3 => citizens / 2 + 1,
				Tier4 => citizens / 2 + 1,
				Tier5 => citizens / 2 + 1,
				Decision => citizens / 2 + 1,
				_ => return Err(Error::<T>::InvalidTier.into()),
			};

			// FIXME? Calling `is_citizen` here results in a duplicate reads to
			// identity of those citizens (as we've already read it once in
			// citizens_count)
			// We could optimize it and for ex. return list of all citizens when
			// counting them, but that would result in a more confusing code.
			// Let's postpone this optimization until it's confirmed it's needed.
			let valid_vetos = Vetos::<T>::iter_key_prefix((tier, index))
				.filter(|sender| T::Citizenship::is_citizen(sender))
				.count();

			ensure!(valid_vetos >= required, Error::<T>::InsufficientVetoCount);

			Laws::<T>::remove(&tier, &index);
			Self::deposit_event(Event::LawRepealedByHeadcountVeto { tier, index });

			// FIXME we should allow doing this over multiple transactions by saving cursor in
			// storage. See example: https://github.com/paritytech/substrate/blob/70351393fd632317124f35ab8b24ef7134e08864/frame/ranked-collective/src/lib.rs#L622
			let mut res = Vetos::<T>::clear_prefix((tier, index), u32::MAX, None);
			while let Some(cursor) = res.maybe_cursor {
				res = Vetos::<T>::clear_prefix((tier, index), u32::MAX, Some(&cursor));
			}

			Ok(())
		}
	}
}
