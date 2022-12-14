//! # Liberland Initializer Pallet
//!
//! ## Overview
//!
//! The Liberland Initializer pallet handles setting up citizenships and LLM
//! balances in genesis block. Especially useful for setting up dev testnets and
//! state for unit tests.
//!
//! ## Usage:
//!
//! Add the `liberland-initializer` pallet to the runtime's `Cargo.toml`. Use `[dev-dependencies]`
//! if it's only for unit tests:
//!
//! ```
//! pallet-liberland-initializer = { path = "../../../frame/liberland-initializer", default-features = false }
//! ```
//!
//! Make it a part of the runtime. No parameters needed for the `Config` trait:
//!
//! ```
//! construct_runtime!(
//!     pub enum Runtime where
//!         [...]
//!     {
//!         [...]
//! 		LiberlandInitializer: pallet_liberland_initializer,
//!     }
//! )
//!
//! impl pallet_liberland_initializer::Config for Runtime {}
//! ```
//!
//! Add the `LiberlandInitializerConfig` to your `GenesisConfig`:
//! ```
//! 	GenesisConfig {
//!         [...]
//! 		liberland_initializer: LiberlandInitializerConfig {
//! 			citizenship_registrar, initial_citizens
//! 		},
//!     }
//! ```
//!
//! * `citizenship_registrar: Option<AccountId>`: AccountID of account that should be used as an
//!   identity registrar for providing citizenship judgements
//! * `initial_citizens: Vec<(AccountId, Balance, Balance)>`: Vector of `(account: AccountId,
//!   total_llm: Balance, politipooled_llm: Balance)` - specifies accounts that should get
//!   citizenships together with amount of LLM sent to them and amount of LLM that should be
//!   politipooled. Note that politipooled LLM will be taked from the `total_llm`, so `(0, 6000,
//!   5000)` will result in account `0` having `5000` politipooled LLM and `1000` free LLM.
//!
//! License: MIT

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub mod traits;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
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
		/// Adds `registrar` as an identity registrar in identity pallet.
		///
		/// Returns index of the added registrar.
		fn add_registrar(registrar: T::AccountId) -> RegistrarIndex {
			let root = frame_system::RawOrigin::Root;
			IdentityPallet::<T>::add_registrar(root.into(), T::Lookup::unlookup(registrar))
				.unwrap();
			let registrars_count = pallet_identity::Pallet::<T>::registrars().len();
			assert!(registrars_count > 0);
			(registrars_count - 1).try_into().unwrap()
		}

		/// Returns an IdentityInfo with placeholder values and citizen field
		/// set.
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

		/// Sets identity of `citizen` to IdentityInfo returned by
		/// `get_citizen_identity_info()` and provides `KnownGood` judgement
		/// using provided registrar.
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

		/// Sends `amount` of LLM to `citizen`.
		fn give_llm(citizen: T::AccountId, amount: T::Balance) {
			let origin = frame_system::RawOrigin::Signed(citizen.clone()).into();
			pallet_llm::Pallet::<T>::fake_send(origin, citizen, amount).unwrap();
		}

		/// Politipools `amount` of `citizen`'s LLM.
		fn politics_lock_llm(citizen: T::AccountId, amount: T::Balance) {
			let origin = frame_system::RawOrigin::Signed(citizen.clone()).into();
			pallet_llm::Pallet::<T>::politics_lock(origin, amount).unwrap();
		}
	}

	impl<T: Config> traits::LLInitializer<T::AccountId, T::Balance> for Pallet<T> {
		#[cfg(feature = "runtime-benchmarks")]
		fn make_citizen(account: &T::AccountId, amount: T::Balance) {
			if pallet_identity::Pallet::<T>::registrars().len() == 0 {
				let registrar: T::AccountId = frame_benchmarking::account("liberland_registrar", 0u32, 0u32);
				Self::add_registrar(registrar);
			}
			let registrar = pallet_identity::Pallet::<T>::registrars()[0].clone().unwrap().account;

			Self::give_citizenship(registrar, 0, account.clone());
			Self::give_llm(account.clone(), amount);
			Self::politics_lock_llm(account.clone(), amount);
		}
	}
}
