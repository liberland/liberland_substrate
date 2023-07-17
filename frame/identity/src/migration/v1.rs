use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};
use sp_std::prelude::*;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

use crate as pallet_identity;
use super::{Pallet, Data, IdentityInfo, Registration, BalanceOf, IdentityOf, Config, v0};

/// The log target.
const TARGET: &'static str = "runtime::identity::migration::v1";

pub struct Migration<T>(sp_std::marker::PhantomData<T>);

impl From<v0::Data> for Data {
    fn from(old: v0::Data) -> Data {
        match old {
            v0::Data::None => Data::None,
            v0::Data::Raw(v) =>
                Data::Raw(v.into_inner().try_into().expect("Putting data in bigger vec failed")),
            v0::Data::BlakeTwo256(b) => Data::BlakeTwo256(b),
            v0::Data::Sha256(b) => Data::Sha256(b),
            v0::Data::Keccak256(b) => Data::Keccak256(b),
            v0::Data::ShaThree256(b) => Data::ShaThree256(b),
        }
    }
}
impl<T: Config> OnRuntimeUpgrade for Migration<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
        assert_eq!(StorageVersion::get::<Pallet<T>>(), 0, "can only upgrade from version 0");
        let identities = v0::IdentityOf::<T>::iter().count() as u32;
        log::info!(target: TARGET, "{} identities will be migrated", identities);

        Ok(identities.encode())
    }

    fn on_runtime_upgrade() -> Weight {
        let mut weight = T::DbWeight::get().reads(1);
        if StorageVersion::get::<Pallet<T>>() != 0 {
            log::warn!(
                target: TARGET,
                "skipping pallet-identity on_runtime_upgrade: executed on wrong storage version.\
            Expected version 0"
            );
            return weight
        } else {
            log::info!(target: TARGET, "pallet-identity: upgrading to version 1");
        }

        IdentityOf::<T>::translate(
            |_,
                reg: v0::Registration<
                BalanceOf<T>,
                <T as pallet_identity::Config>::MaxRegistrars,
                <T as pallet_identity::Config>::MaxAdditionalFields,
            >| {
                weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
                let additional: Vec<(Data, Data)> = reg.info.additional
                    .into_iter()
                    .map(|(key, value)| (key.into(), value.into()))
                    .collect();
                Some(Registration {
                    judgements: reg.judgements,
                    deposit: reg.deposit,
                    info: IdentityInfo {
                        additional: additional.try_into().expect("Additional vec size can't change"),
                        display: reg.info.display.into(),
                        legal: reg.info.legal.into(),
                        web: reg.info.web.into(),
                        riot: reg.info.riot.into(),
                        email: reg.info.email.into(),
                        pgp_fingerprint: reg.info.pgp_fingerprint,
                        image: reg.info.image.into(),
                        twitter: reg.info.twitter.into(),
                    },
                })
            },
        );

        StorageVersion::new(1).put::<Pallet<T>>();
        weight.saturating_add(T::DbWeight::get().reads_writes(1, 1))
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
        assert_eq!(StorageVersion::get::<Pallet<T>>(), 1, "must upgrade");
        let old_identities_count: u32 =
            Decode::decode(&mut &state[..]).expect("pre_upgrade provides a valid state; qed");
        let new_identities_count = IdentityOf::<T>::iter().count() as u32;
        assert_eq!(old_identities_count, new_identities_count);
        log::info!(target: TARGET, "{} identities migrated", new_identities_count);
        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "try-runtime")]
mod test {
	use super::*;
	use crate::{
		tests::{Test as T, *},
		types::*,
	};

	#[allow(deprecated)]
	#[test]
	fn migration_works() {
		new_test_ext().execute_with(|| {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 0);
            let data = v0::Data::Raw(b"Some bytes here".to_vec().try_into().unwrap());
            let registration = v0::Registration {
                judgements: Default::default(),
                deposit: Default::default(),
                info: v0::IdentityInfo {
                    additional: vec![(data.clone(), data.clone())].try_into().unwrap(),
                    display: data.clone(),
                    legal: data.clone(),
                    web: data.clone(),
                    riot: data.clone(),
                    email: data.clone(),
                    pgp_fingerprint: Some(*b"12345678901234567890"),
                    image: data.clone(),
                    twitter: data.clone(),
                }
            };
            v0::IdentityOf::<T>::insert(0, registration.clone());
            v0::IdentityOf::<T>::insert(1, registration);

			let state = Migration::<T>::pre_upgrade().unwrap();
			let _weight = Migration::<T>::on_runtime_upgrade();
			Migration::<T>::post_upgrade(state).unwrap();

            let v1data = Data::Raw(b"Some bytes here".to_vec().try_into().unwrap());
            let v1registration = Registration {
                judgements: Default::default(),
                deposit: Default::default(),
                info: IdentityInfo {
                    additional: vec![(v1data.clone(), v1data.clone())].try_into().unwrap(),
                    display: v1data.clone(),
                    legal: v1data.clone(),
                    web: v1data.clone(),
                    riot: v1data.clone(),
                    email: v1data.clone(),
                    pgp_fingerprint: Some(*b"12345678901234567890"),
                    image: v1data.clone(),
                    twitter: v1data.clone(),
                }
            };
			assert_eq!(
				IdentityOf::<T>::get(0u64),
                Some(v1registration.clone())
			);
			assert_eq!(
				IdentityOf::<T>::get(1u64),
                Some(v1registration)
			);
		});
	}
}
