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
//! ```ignore
//! pallet-liberland-initializer = { path = "../../../frame/liberland-initializer", default-features = false }
//! ```
//!
//! Make it a part of the runtime. No parameters needed for the `Config` trait:
//!
//! ```ignore
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
//! ```ignore
//! 	GenesisConfig {
//!         [...]
//! 		liberland_initializer: LiberlandInitializerConfig {
//! 			citizenship_registrar,
//! 			initial_citizens,
//! 			land_registrar,
//! 			metaverse_land_registrar,
//! 			asset_registrar,
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
//! * `land_registrar: Option<AccountId>`: AccountID of account that should be used as a collection
//!   owner for land NFTs.
//! * `metaverse_land_registrar: Option<AccountId>`: AccountID of account that should be used as a
//!   collection owner for metaverse land NFTs.
//! * `asset_registrar: Option<AccountId>`: AccountID of account that should be used as a collection
//!   owner for asset NFTs.
//!
//! License: MIT

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, traits::tokens::nonfungibles_v2::InspectEnumerable};
	use liberland_traits::LLInitializer;
	use pallet_identity::{Data, IdentityInfo, RegistrarIndex};
	use sp_runtime::traits::{Hash, StaticLookup};
	use sp_std::prelude::*;

	type IdentityPallet<T> = pallet_identity::Pallet<T>;

	#[cfg(any(test, feature = "runtime-benchmarks"))]
	use ::{frame_support::traits::Currency, sp_runtime::traits::Bounded};

	#[cfg(any(test, feature = "runtime-benchmarks"))]
	type BalanceOfIdentity<T> = <<T as pallet_identity::Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	type BalanceOfAssets<T> = <T as pallet_assets::Config>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_identity::Config + pallet_llm::Config + pallet_nfts::Config
	{
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub citizenship_registrar: Option<T::AccountId>,
		pub initial_citizens: Vec<(T::AccountId, BalanceOfAssets<T>, BalanceOfAssets<T>)>,
		pub land_registrar: Option<T::AccountId>,
		pub metaverse_land_registrar: Option<T::AccountId>,
		pub asset_registrar: Option<T::AccountId>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				citizenship_registrar: None,
				initial_citizens: vec![],
				land_registrar: None,
				metaverse_land_registrar: None,
				asset_registrar: None,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			let collection_config = pallet_nfts::CollectionConfig {
				settings: Default::default(),
				max_supply: None,
				mint_settings: Default::default(),
			};
			if let Some(land_registrar) = &self.land_registrar {
				pallet_nfts::Pallet::<T>::force_create(
					frame_system::RawOrigin::Root.into(),
					T::Lookup::unlookup(land_registrar.clone()),
					collection_config.clone(),
				)
				.unwrap();
				let idx = pallet_nfts::Pallet::<T>::collections().last().unwrap();
				pallet_nfts::Pallet::<T>::set_citizenship_required(
					frame_system::RawOrigin::Root.into(),
					idx,
					true,
				)
				.unwrap();
			}

			if let Some(metaverse_land_registrar) = &self.metaverse_land_registrar {
				pallet_nfts::Pallet::<T>::force_create(
					frame_system::RawOrigin::Root.into(),
					T::Lookup::unlookup(metaverse_land_registrar.clone()),
					collection_config.clone(),
				)
				.unwrap();
			}

			if let Some(asset_registrar) = &self.asset_registrar {
				pallet_nfts::Pallet::<T>::force_create(
					frame_system::RawOrigin::Root.into(),
					T::Lookup::unlookup(asset_registrar.clone()),
					collection_config.clone(),
				)
				.unwrap();
			}

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
			let eligible_on = (
				Data::Raw(b"eligible_on".to_vec().try_into().unwrap()),
				Data::Raw(vec![0].try_into().unwrap()),
			);
			let citizen = (Data::Raw(b"citizen".to_vec().try_into().unwrap()), data.clone());

			IdentityInfo {
				twitter: data,
				additional: vec![eligible_on, citizen].try_into().unwrap(),
				display: Data::None,
				legal: Data::None,
				web: Data::None,
				riot: Data::None,
				email: Data::None,
				pgp_fingerprint: None,
				image: Data::None,
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
		fn give_llm(citizen: T::AccountId, amount: BalanceOfAssets<T>) {
			pallet_llm::Pallet::<T>::transfer_from_treasury(citizen, amount).unwrap();
		}

		/// Politipools `amount` of `citizen`'s LLM.
		fn politics_lock_llm(citizen: T::AccountId, amount: BalanceOfAssets<T>) {
			let origin = frame_system::RawOrigin::Signed(citizen.clone()).into();
			pallet_llm::Pallet::<T>::politics_lock(origin, amount).unwrap();
		}
	}

	impl<T: Config> LLInitializer<T::AccountId> for Pallet<T> {
		#[cfg(any(test, feature = "runtime-benchmarks"))]
		fn make_test_citizen(account: &T::AccountId) {
			if pallet_identity::Pallet::<T>::registrars().len() == 0 {
				#[cfg(test)]
				{
					use frame_support::PalletId;
					use sp_runtime::traits::AccountIdConversion;
					let pallet_id = PalletId(*b"llregstr");
					let registrar: T::AccountId = pallet_id.into_account_truncating();
					Self::add_registrar(registrar);
				}
				#[cfg(all(not(test), feature = "runtime-benchmarks"))]
				{
					use frame_benchmarking::account;
					let registrar: T::AccountId = account("liberland_registrar", 0u32, 0u32);
					Self::add_registrar(registrar);
				}
			}
			let registrar = pallet_identity::Pallet::<T>::registrars()[0].clone().unwrap().account;
			let amount = match T::CitizenshipMinimumPooledLLM::get().try_into() {
				Ok(m) => m,
				_ => panic!("Configured Balance type for pallet_assets can't fit values required by pallet_llm!")
			};

			if <T as pallet_identity::Config>::Currency::free_balance(&account) == 0u8.into() {
				let balance = BalanceOfIdentity::<T>::max_value() / 2u8.into();
				<T as pallet_identity::Config>::Currency::make_free_balance_be(&account, balance);
			}

			Self::give_citizenship(registrar, 0, account.clone());
			Self::give_llm(account.clone(), amount);
			Self::politics_lock_llm(account.clone(), amount);
		}
	}
}
