use super::{pallet::Config, *};
use frame_support::{pallet_prelude::*, storage_alias, traits::OnRuntimeUpgrade};
use liberland_traits::CitizenshipChecker;
use pallet_identity::Registration;
use sp_std::vec::Vec;
use frame_system::pallet_prelude::BlockNumberFor;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;


pub mod v1 {
	use super::*;
	const TARGET: &'static str = "runtime::llm::migration::v1";

	#[storage_alias]
	pub type IdentityOf<T: Config> = StorageMap<
		pallet_identity::Pallet<T>,
		Twox64Concat,
		<T as frame_system::Config>::AccountId,
		Registration<
			BalanceOf<T>,
			<T as pallet_identity::Config>::MaxRegistrars,
			<T as pallet_identity::Config>::MaxAdditionalFields,
		>,
	>;

	pub struct Migration<T>(sp_std::marker::PhantomData<T>);

	impl<T: Config> OnRuntimeUpgrade for Migration<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 0, "can only upgrade from version 0");

			Ok(().encode())
		}

		fn on_runtime_upgrade() -> Weight {
			let mut weight = T::DbWeight::get().reads(1);
			if StorageVersion::get::<Pallet<T>>() != 0 {
				log::warn!(
					target: TARGET,
					"skipping on_runtime_upgrade: executed on wrong storage version.\
				Expected version 0"
				);
				return weight;
			}

			IdentityOf::<T>::translate(
				|_,
				 reg: Registration<
					BalanceOf<T>,
					<T as pallet_identity::Config>::MaxRegistrars,
					<T as pallet_identity::Config>::MaxAdditionalFields,
				>| {
					let mut additional = Vec::new();
					let mut v = Vec::new();
					v.push(0);
					additional.push((
						pallet_identity::Data::Raw(b"eligible_on".to_vec().try_into().unwrap()),
						pallet_identity::Data::Raw(v.try_into().unwrap()),
					));
					Some(pallet_identity::Registration {
						info: pallet_identity::IdentityInfo {
							additional: additional.try_into().unwrap(),
							..reg.info
						},
						..reg
					})
				},
			);

			let citizens_count =
				IdentityOf::<T>::iter_keys().filter(|id| Pallet::<T>::is_citizen(id)).count();
			let citizens_count: u64 = citizens_count.try_into().unwrap();
			Citizens::<T>::put(citizens_count);
			weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

			StorageVersion::new(1).put::<Pallet<T>>();
			weight.saturating_add(T::DbWeight::get().reads_writes(1, 1))
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 1, "must upgrade");
			log::info!(target: TARGET, "Counted {} citizens", Citizens::<T>::get(),);
			Ok(())
		}
	}
}

pub mod ltm_to_lkn {
	use super::*;
	#[cfg(feature = "try-runtime")]
	use sp_std::vec::Vec;

	const TARGET: &'static str = "runtime::llm::migration::ltm_to_lkn";

	pub struct Migration<T>(sp_std::marker::PhantomData<T>);

	impl<T: Config> OnRuntimeUpgrade for Migration<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			Ok(().encode())
		}

		fn on_runtime_upgrade() -> Weight {
			let weight = T::DbWeight::get().reads(1);
			let origin = frame_system::RawOrigin::Root.into();

			let id = <T as Config>::AssetId::get();
			let name = T::AssetName::get();
			let symbol = T::AssetSymbol::get();

			Assets::<T>::force_set_metadata(
				origin,
				id.into(),
				name.into(),
				symbol.into(),
				12,
				false,
			)
			.unwrap();

			log::warn!(target: TARGET, "Reset metadata of LLM asset!");

			weight
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
			Ok(())
		}
	}
}

pub mod v2 {
	use super::*;
	use pallet_identity::Data;

	const TARGET: &'static str = "runtime::llm::migration::v2";

	#[storage_alias]
	pub type IdentityOf<T: Config> = StorageMap<
		pallet_identity::Pallet<T>,
		Twox64Concat,
		<T as frame_system::Config>::AccountId,
		Registration<
			BalanceOf<T>,
			<T as pallet_identity::Config>::MaxRegistrars,
			<T as pallet_identity::Config>::MaxAdditionalFields,
		>,
	>;

	/// Migration for adding origin type to proposals and referendums.
	pub struct Migration<T>(sp_std::marker::PhantomData<T>);

	impl<T: Config> OnRuntimeUpgrade for Migration<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 1, "can only upgrade from version 1");
			Ok(().encode())
		}

		fn on_runtime_upgrade() -> Weight {
			let weight = T::DbWeight::get().reads(1);
			if StorageVersion::get::<Pallet<T>>() != 1 {
				log::warn!(
					target: TARGET,
					"skipping on_runtime_upgrade: executed on wrong storage version.\
				Expected version 1"
				);
				return weight;
			}

			let mut translated = 0;
			let mut skipped = 0;
			let mut errored = 0;
			IdentityOf::<T>::translate(
				|_,
				 reg: Registration<
					BalanceOf<T>,
					<T as pallet_identity::Config>::MaxRegistrars,
					<T as pallet_identity::Config>::MaxAdditionalFields,
				>| {
					let mut additional = reg.info.additional.clone();

					// called twitter now, but this referred to citizen field in previous storage
					// version
					if reg.info.twitter != Data::None {
						if let Err(_) = additional.try_push((
							Data::Raw(b"citizen".to_vec().try_into().unwrap()),
							Data::Raw(b"1".to_vec().try_into().unwrap()),
						)) {
							errored += 1;
							log::error!(
								target: TARGET,
								"Couldnt migrate Identity - exceeded MaxAdditionalFields!"
							);
						} else {
							translated += 1;
						};
					} else {
						skipped += 1;
					}

					Some(pallet_identity::Registration {
						info: pallet_identity::IdentityInfo {
							additional: additional.try_into().unwrap(),
							twitter: Data::None, // this was the citizen field
							..reg.info
						},
						..reg
					})
				},
			);
			log::info!(
				target: TARGET,
				"Translated {} identities, skipped {}, errored on {}",
				translated,
				skipped,
				errored
			);

			StorageVersion::new(2).put::<Pallet<T>>();
			weight.saturating_add(T::DbWeight::get().reads_writes(1, 1))
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 2, "must upgrade");
			Ok(())
		}
	}
}

pub mod v3 {
	use super::*;

	const TARGET: &'static str = "runtime::llm::migration::v3";

	pub struct Migration<T>(sp_std::marker::PhantomData<T>);

	impl<T: Config> OnRuntimeUpgrade for Migration<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			Ok(().encode())
		}

		fn on_runtime_upgrade() -> Weight {
			let weight = T::DbWeight::get().reads(1);
			if StorageVersion::get::<Pallet<T>>() > 2 {
				log::warn!(
					target: TARGET,
					"skipping on_runtime_upgrade: executed on wrong storage version.\
				Expected version 2 or lower"
				);
				return weight;
			}

			let duration: BlockNumberFor<T> = 432000u32.into();
			WithdrawlockDuration::<T>::put(&duration);
			ElectionlockDuration::<T>::put(&duration);

			let _ = Withdrawlock::<T>::clear(u32::MAX, None);
			let _ = Electionlock::<T>::clear(u32::MAX, None);

			StorageVersion::new(3).put::<Pallet<T>>();
			weight.saturating_add(T::DbWeight::get().reads_writes(1, 1))
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 3, "must upgrade");
			log::info!(target: TARGET, "WithdrawlockDuration set to {:?}, ElectionlockDuration set to {:?}", WithdrawlockDuration::<T>::get(), ElectionlockDuration::<T>::get(),);
			Ok(())
		}
	}
}

pub mod v4 {
	use super::*;

	const TARGET: &'static str = "runtime::llm::migration::v4";

	pub struct Migration<T, OldInterval>(sp_std::marker::PhantomData<(T, OldInterval)>);

	#[storage_alias]
	type NextRelease<T: Config> = StorageValue<
		Pallet<T>,
		BlockNumberFor<T>,
		ValueQuery
	>;

	impl<T: Config, OldInterval: Get<BlockNumberFor<T>>> OnRuntimeUpgrade for Migration<T, OldInterval> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			assert!(StorageVersion::get::<Pallet<T>>() == 3, "can only upgrade from version 3");

			Ok(().encode())
		}

		fn on_runtime_upgrade() -> Weight {
			let weight = T::DbWeight::get().reads(1);
			if StorageVersion::get::<Pallet<T>>() != 3 {
				log::warn!(
					target: TARGET,
					"skipping on_runtime_upgrade: executed on wrong storage version.\
				Expected version 3"
				);
				return weight;
			}

			LastRelease::<T>::put(
				NextRelease::<T>::get() - OldInterval::get()
			);

			StorageVersion::new(4).put::<Pallet<T>>();
			weight.saturating_add(T::DbWeight::get().reads_writes(1, 1))
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 4, "must upgrade");
			Ok(())
		}
	}
}