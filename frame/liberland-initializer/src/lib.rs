#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use pallet_identity::{Data, IdentityInfo, RegistrarIndex};
	use sp_runtime::traits::{Hash, StaticLookup};
	use sp_std::prelude::*;

	type IdentityPallet<T> = pallet_identity::Pallet<T>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_identity::Config + pallet_llm::Config {}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub citizenship_registrar: Option<T::AccountId>,
		pub initial_citizens: Vec<(T::AccountId, T::Balance, T::Balance)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { citizenship_registrar: None, initial_citizens: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			if let Some(registrar_account) = &self.citizenship_registrar {
				let registrar_idx = Pallet::<T>::add_registrar(registrar_account.clone());
				for citizen in &self.initial_citizens {
					Pallet::<T>::give_citizenship(
						registrar_account.clone(),
						registrar_idx,
						citizen.0.clone(),
					);
				}
			}

			for citizen in &self.initial_citizens {
				Pallet::<T>::give_llm(citizen.0.clone(), citizen.1.clone());
				Pallet::<T>::politics_lock_llm(citizen.0.clone(), citizen.2.clone());
			}
		}
	}

	impl<T: Config> Pallet<T> {
		fn add_registrar(registrar: T::AccountId) -> RegistrarIndex {
			let root = frame_system::RawOrigin::Root;
			IdentityPallet::<T>::add_registrar(root.into(), T::Lookup::unlookup(registrar))
				.unwrap();
			let registrars_count = pallet_identity::Pallet::<T>::registrars().len();
			assert!(registrars_count > 0);
			(registrars_count - 1).try_into().unwrap()
		}

		fn get_citizen_identity_info() -> IdentityInfo<T::MaxAdditionalFields> {
			let data = Data::Raw(b"1".to_vec().try_into().unwrap());
			IdentityInfo {
				citizen: data.clone(),
				additional: vec![].try_into().unwrap(),
				display: data.clone(),
				legal: data.clone(),
				web: data.clone(),
				riot: data.clone(),
				email: data.clone(),
				pgp_fingerprint: Some([0; 20]),
				image: data,
			}
		}

		fn give_citizenship(
			registrar: T::AccountId,
			registrar_idx: RegistrarIndex,
			citizen: T::AccountId,
		) {
			let registrar_origin = frame_system::RawOrigin::Signed(registrar).into();
			let citizen_origin = frame_system::RawOrigin::Signed(citizen.clone()).into();
			let info = Self::get_citizen_identity_info();

			IdentityPallet::<T>::set_identity(citizen_origin, Box::new(info.clone())).unwrap();
			IdentityPallet::<T>::provide_judgement(
				registrar_origin,
				registrar_idx,
				T::Lookup::unlookup(citizen),
				pallet_identity::Judgement::KnownGood,
				T::Hashing::hash_of(&info),
			)
			.unwrap();
		}

		fn give_llm(citizen: T::AccountId, amount: T::Balance) {
			let origin = frame_system::RawOrigin::Signed(citizen.clone()).into();
			pallet_llm::Pallet::<T>::fake_send(origin, citizen, amount).unwrap();
		}

		fn politics_lock_llm(citizen: T::AccountId, amount: T::Balance) {
			let origin = frame_system::RawOrigin::Signed(citizen.clone()).into();
			pallet_llm::Pallet::<T>::politics_lock(origin, amount).unwrap();
		}
	}
}
