use super::{pallet::Config, *};
use frame_support::{pallet_prelude::*, storage_alias, traits::OnRuntimeUpgrade};

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

use crate::types::*;

/// The log target.
const TARGET: &'static str = "runtime::liberland_legislation::migration";

pub mod v0 {
	use super::*;

	#[storage_alias]
	pub type Laws<T: Config> = StorageDoubleMap<
		Pallet<T>,
		Blake2_128Concat,
		u32,
		Blake2_128Concat,
		u32,
		BoundedVec<u8, ConstU32<65536>>,
		ValueQuery,
	>;

	#[storage_alias]
	pub type Vetos<T: Config> = StorageNMap<
		Pallet<T>,
		(
			NMapKey<Blake2_128Concat, u32>,
			NMapKey<Blake2_128Concat, u32>,
			NMapKey<Blake2_128Concat, <T as frame_system::Config>::AccountId>,
		),
		bool,
	>;

	#[storage_alias]
	pub type VetosCount<T: Config> =
		StorageDoubleMap<Pallet<T>, Blake2_128Concat, u32, Blake2_128Concat, u32, u64, ValueQuery>;
}

pub mod v1 {
	use super::*;
	use sp_std::vec::Vec;

	#[storage_alias]
	pub type Laws<T: Config> = StorageDoubleMap<
		Pallet<T>,
		Blake2_128Concat,
		LegislationTier,
		Blake2_128Concat,
		LegislationId,
		BoundedVec<u8, ConstU32<65536>>,
		ValueQuery,
	>;

	#[storage_alias]
	pub type Vetos<T: Config> = StorageNMap<
		Pallet<T>,
		(
			NMapKey<Blake2_128Concat, LegislationTier>,
			NMapKey<Blake2_128Concat, LegislationId>,
			NMapKey<Blake2_128Concat, <T as frame_system::Config>::AccountId>,
		),
		bool,
	>;

	#[storage_alias]
	pub type VetosCount<T: Config> = StorageDoubleMap<
		Pallet<T>,
		Blake2_128Concat,
		LegislationTier,
		Blake2_128Concat,
		LegislationId,
		u64,
		ValueQuery,
	>;

	/// Migration for adding origin type to proposals and referendums.
	pub struct Migration<T>(sp_std::marker::PhantomData<T>);

	fn update_tier(old: u32) -> LegislationTier {
		use LegislationTier::*;
		match old {
			0 => Constitution,
			1 => InternationalTreaty,
			2 => Law,
			3 => Tier3,
			4 => Tier4,
			5 => Tier5,
			_ => Decision,
		}
	}

	impl<T: Config> OnRuntimeUpgrade for Migration<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 0, "can only upgrade from version 0");
			Ok(().encode())
		}

		fn on_runtime_upgrade() -> Weight {
			let weight = T::DbWeight::get().reads(1);
			if StorageVersion::get::<Pallet<T>>() != 0 {
				log::warn!(
					target: TARGET,
					"skipping on_runtime_upgrade: executed on wrong storage version.\
				Expected version 0"
				);
				return weight
			}

			let old_keys: Vec<_> = v0::Laws::<T>::iter_keys().collect();
			for (old_tier, old_index) in old_keys {
				let value = v0::Laws::<T>::take(old_tier, old_index);
				Laws::<T>::insert(
					update_tier(old_tier),
					LegislationId { year: 2023, index: old_index },
					value,
				);
			}

			let old_keys: Vec<_> = v0::Vetos::<T>::iter_keys().collect();
			for old_key in old_keys {
				let new_key = (
					update_tier(old_key.0),
					LegislationId { year: 2023, index: old_key.1 },
					old_key.2.clone(),
				);
				let value = v0::Vetos::<T>::take(old_key).unwrap_or(false);
				Vetos::<T>::insert(new_key, value);
			}

			let old_keys: Vec<_> = v0::VetosCount::<T>::iter_keys().collect();
			for (old_tier, old_index) in old_keys {
				let value = v0::VetosCount::<T>::take(old_tier, old_index);
				VetosCount::<T>::insert(
					update_tier(old_tier),
					LegislationId { year: 2023, index: old_index },
					value,
				);
			}

			StorageVersion::new(1).put::<Pallet<T>>();
			weight.saturating_add(T::DbWeight::get().reads_writes(1, 1))
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 1, "must upgrade");
			Ok(())
		}
	}
}
