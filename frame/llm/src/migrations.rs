use super::*;
use frame_support::{pallet_prelude::*, storage_alias, traits::OnRuntimeUpgrade};
use liberland_traits::CitizenshipChecker;
use pallet_identity::Registration;
use super::pallet::Config;

/// The log target.
const TARGET: &'static str = "runtime::llm::migration::v1";

pub mod v1 {
	use super::*;
    use sp_std::vec::Vec;

	#[storage_alias]
	pub type IdentityOf<T: Config> = StorageMap<
		pallet_identity::Pallet<T>,
        Twox64Concat,
        <T as frame_system::Config>::AccountId,
        Registration<BalanceOf<T>, <T as pallet_identity::Config>::MaxRegistrars, <T as pallet_identity::Config>::MaxAdditionalFields>,
	>;

	/// Migration for adding origin type to proposals and referendums.
	pub struct Migration<T>(sp_std::marker::PhantomData<T>);

	impl<T: Config> OnRuntimeUpgrade for Migration<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
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
				return weight
			}

            IdentityOf::<T>::translate(|_, reg: Registration<BalanceOf<T>, <T as pallet_identity::Config>::MaxRegistrars, <T as pallet_identity::Config>::MaxAdditionalFields>| {
                let mut additional = Vec::new();
                let mut v = Vec::new();
                v.push(0);
                additional.push(
                    (
                        pallet_identity::Data::Raw(b"eligible_on".to_vec().try_into().unwrap()),
                        pallet_identity::Data::Raw(v.try_into().unwrap())
                    )
                );
                Some(pallet_identity::Registration {
                    info: pallet_identity::IdentityInfo {
                        additional: additional.try_into().unwrap(),
                        ..reg.info
                    },
                    ..reg
                })
            });

            let citizens_count = IdentityOf::<T>::iter_keys().filter(|id| Pallet::<T>::is_citizen(id)).count();
            let citizens_count: u64 = citizens_count.try_into().unwrap();
			Citizens::<T>::put(citizens_count);
			weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

			StorageVersion::new(1).put::<Pallet<T>>();
			weight.saturating_add(T::DbWeight::get().reads_writes(1, 1))
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 1, "must upgrade");
			log::info!(
				target: TARGET,
				"Counted {} citizens",
				Citizens::<T>::get(),
			);
			Ok(())
		}
	}
}