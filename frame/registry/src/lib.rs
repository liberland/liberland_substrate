//! # Liberland Registry Pallet
//!
//! ## Overview
//!
//! Registry pallet is a general-purpose pallet for tracking and registering
//! data about abstract entities.
//!
//! In Liberland, it's used to implement Company Registry.
//!
//! ## Terminology
//!
//! * Entity - object, identified with AccountId, that has data attached to it and can be registered
//!   at a registrar
//! * Registrar - AccountId that can register Entities in Registry.
//! * Registry - database of Entities and their data that were registered.
//! * Deposit - amount of Currency that gets reserved when requesting registration - refunded data
//!   is removed
//!
//! ## Entity Lifecycle
//!
//! 1. Entity requests registration at Registry using `request_registration()` call. This creates
//!    and stores a Registration Request and reserves the deposit.
//! 2. If Registrar approves Entity's data, they call `register_entity()` which
//!    moves data from Registration Request to the Registry.
//! 3. To update the data, Entity needs to repeat the same process as for initial registration.
//! 3. Entity can be removed from Registry by the Registrar - deposit will be refunded.
//!
//! ## Deposits
//!
//! Registry pallet requires deposits to cover the cost of storing data in
//! current blockchain state (which isn't pruned). These deposits always matches
//! the amount of stored data:
//! * if there's no prior deposit, total required deposit will be reserved
//! * if there's a deposit, but now we want to store more data, the additional difference will be
//!   reserved
//! * if there's a deposit, but now we want to store less or no data, the excess will be refunded
//!
//! Formula for required deposit to store `N` bytes of data:
//! ```ignore
//! deposit = BaseDeposit + N * ByteDeposit
//! ```
//!
//! Deposits are separate for each registry (as data is stored separately as
//! well).
//!
//! * `request_registration(registry, data, editable)` requires deposit for length of `data`
//!   parameter. Will immediately reserve/refund any difference.
//! * `cancel_request()` will refund complete deposit for given request.
//! * `unregister()` will refund deposit for data at given registrar.
//! * `register_entity()` will refund old deposit, if any.
//!
//! ## Pallet Config
//!
//! * `Currency` - pallet implementing NamedReservableCurrency - it will be used for reserving
//!   deposits
//! * `ReserveIdentified` - will be used in Currency to attach reserves to this pallet
//! * `MaxRegistrars` - max number of registrars that can be added
//! * `BaseDeposit` - see **Deposits** above
//! * `ByteDeposit` - see **Deposits** above
//! * `AddRegistrarOrigin` - origin that's authorized to add new registrars
//! * `RegistrarOrigin` - origin of registrars - must return AccountId on success
//! * `EntityOrigin` - origin of entities - must return AccountId on usccess
//! * `EntityData` - type that will be used to store and process Entities data
//! * `WeightInfo` - see [Substrate docs](https://docs.substrate.io/reference/how-to-guides/weights/use-custom-weights/)
//!
//! ## Genesis Config
//!
//! * `registries`: registries that should be preset on genesis
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### Public
//!
//! These calls can be made from any _Signed_ origin.
//!
//! * `add_registrar`: Adds a new registrar
//! * `request_registration`: Requests a registration of Entity in Registry
//! * `cancel_request`: Cancels registration request
//! * `unregister`: Removes Entity from given Registry
//! * `register_entity`: Adds Entity to the Registry
//! * `set_registered_entity`: Updates Entity data in given Registry
//!
//!
//! License: MIT
/*
Copyright © 2023 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod benchmarking;
mod mock;
mod tests;
pub mod weights;
pub use crate::weights::WeightInfo;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	traits::{Currency, NamedReservableCurrency},
	BoundedVec,
};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

pub type RegistryIndex = u32;

#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
/// Structure for keeping Entity data that want to be registered
pub struct Request<Balance, EntityData> {
	/// Deposit safely reserved - will always match length of data
	pub deposit: Balance,
	/// Entity data
	pub data: EntityData,
	/// Whether Registrar can edit Entity data in their Registry after
	/// registration
	pub editable_by_registrar: bool,
}

#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
/// Structure for keeping Entity data in Registry
pub struct Registration<Balance, EntityData> {
	/// Deposit safely reserved - will always match length of data
	pub deposit: Balance,
	/// Registered Entity data
	pub data: EntityData,
	/// Whether Registrar can edit Entity data
	pub editable_by_registrar: bool,
}

type BalanceOf<T, I> =
	<<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::{DispatchResult, *};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::vec;
	use sp_runtime::{traits::Hash, Saturating};

	type ReserveIdentifierOf<T, I> = <<T as Config<I>>::Currency as NamedReservableCurrency<
		<T as frame_system::Config>::AccountId,
	>>::ReserveIdentifier;

	type RequestOf<T, I> = Request<BalanceOf<T, I>, <T as Config<I>>::EntityData>;
	type RegistrationOf<T, I> = Registration<BalanceOf<T, I>, <T as Config<I>>::EntityData>;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(_);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Currency for Deposits
		type Currency: NamedReservableCurrency<Self::AccountId>;

		#[pallet::constant]
		/// Maximum number of registrars that can be added
		type MaxRegistrars: Get<u32>;

		#[pallet::constant]
		/// Base deposit for storing data
		type BaseDeposit: Get<BalanceOf<Self, I>>;

		#[pallet::constant]
		/// Per Byte deposit for storing data
		type ByteDeposit: Get<BalanceOf<Self, I>>;

		#[pallet::constant]
		/// Identifies reserves in Currency
		type ReserveIdentifier: Get<&'static ReserveIdentifierOf<Self, I>>;

		/// Origin that can add new Registrars
		type AddRegistrarOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Origin used to check Registrars - must return AccountId on success
		type RegistrarOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

		/// Origin used to check Entities - must return AccountId on success
		type EntityOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

		/// Type for storing and processing Entities' Data
		type EntityData: Parameter + Member + MaxEncodedLen;

		/// WeightInfo
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Maximum amount of registrars reached. Cannot add any more.
		TooManyRegistrars,
		/// Trying to register entity that didn't request anything.
		InvalidEntity,
		/// Invalid registry - either doesn't exist or not authorized
		InvalidRegistry,
		/// Trying to register entity with old/wrong data.
		MismatchedData,
		/// Insufficient deposit for data storage
		InsufficientDeposit,
		/// Entity doesn't allow edits by registrars
		NotEditableByRegistrar,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// New Registrar added
		RegistryAdded { registry_index: RegistryIndex },
		/// Entity provided data and deposit for registration
		RegistrationRequested { entity: T::AccountId, registry_index: RegistryIndex },
		/// Entity canceled registration request before registrar registered it
		RegistrationRequestCanceled { entity: T::AccountId, registry_index: RegistryIndex },
		/// Registrar added Entity to Registry
		EntityRegistered { entity: T::AccountId, registry_index: RegistryIndex },
		/// Entity was removed from Registry
		EntityUnregistered { entity: T::AccountId, registry_index: RegistryIndex },
	}

	#[pallet::storage]
	#[pallet::getter(fn requests)]
	/// Registration requests. See `request_registration` and `cancel_request`
	pub(super) type Requests<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		RegistryIndex,
		Blake2_128Concat,
		T::AccountId,
		RequestOf<T, I>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn registries)]
	/// Registered data of Entities in given Registries. See `register_entity`, `unregister` and
	/// `set_registered_data`
	pub(super) type Registries<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		RegistryIndex, // Registry
		Blake2_128Concat,
		T::AccountId, // EntityId
		RegistrationOf<T, I>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn registrars)]
	/// List of registries - order matters, as it's tied to RegistryIndex!
	pub(super) type Registrars<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxRegistrars>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		/// Registries that should be preset on genesis
		pub registries: BoundedVec<T::AccountId, T::MaxRegistrars>,
	}

	#[cfg(feature = "std")]
	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			Self { registries: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
		fn build(&self) {
			Registrars::<T, I>::put(&self.registries);
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Add a new registry.
		///
		/// * `account` - AccountId of Registrar for the new Registry
		///
		/// Emits `RegistryAdded` with the index of new Registry.
		///
		/// Must be called by `AddRegistrarOrigin`
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::add_registry(T::MaxRegistrars::get()))]
		pub fn add_registry(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			T::AddRegistrarOrigin::ensure_origin(origin)?;

			let registry_index = Registrars::<T, I>::try_mutate(
				|registrars| -> Result<RegistryIndex, DispatchError> {
					registrars.try_push(account).map_err(|_| Error::<T, I>::TooManyRegistrars)?;
					Ok((registrars.len() - 1) as RegistryIndex)
				},
			)?;

			Self::deposit_event(Event::RegistryAdded { registry_index });

			Ok(())
		}

		/// Request registration of Entity in Registry
		///
		/// * `registry_index` - Registry to register in
		/// * `data` - data to set
		/// * `editable_by_registrar` - if true, registrar will be able to modify Entity's data in
		///   the Registry
		///
		/// Entity will be stored under AccountId of caller.
		///
		/// Will reserve deposit to cover stored data.
		///
		/// Emits `RegistrationRequested`.
		///
		/// Must be called by `EntityOrigin`
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::request_registration(T::EntityData::max_encoded_len() as u32))]
		pub fn request_registration(
			origin: OriginFor<T>,
			registry_index: RegistryIndex,
			data: T::EntityData,
			editable_by_registrar: bool,
		) -> DispatchResult {
			let entity = T::EntityOrigin::ensure_origin(origin)?;
			let required_deposit = Self::calculate_deposit(&data);
			let old_deposit = Self::requests(&registry_index, &entity)
				.map(|Request { deposit, .. }| deposit)
				.unwrap_or_default();

			// refund old deposit, if any
			T::Currency::unreserve_named(T::ReserveIdentifier::get(), &entity, old_deposit);

			// reserve new deposit
			T::Currency::reserve_named(T::ReserveIdentifier::get(), &entity, required_deposit)?;

			Requests::<T, I>::insert(
				&registry_index,
				&entity,
				Request { deposit: required_deposit, data, editable_by_registrar },
			);

			Self::deposit_event(Event::RegistrationRequested { registry_index, entity });

			Ok(())
		}

		/// Cancel Entity's Registration request for given Registry
		///
		/// * `registry_index` - Registry to register in
		///
		/// Will refund deposit of stored data.
		///
		/// Emits `RegistrationRequestCanceled`.
		///
		/// Must be called by `EntityOrigin`
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::cancel_request())]
		pub fn cancel_request(
			origin: OriginFor<T>,
			registry_index: RegistryIndex,
		) -> DispatchResult {
			let entity = T::EntityOrigin::ensure_origin(origin)?;

			if let Some(Request { deposit, .. }) = Self::requests(&registry_index, &entity) {
				// refund deposit
				T::Currency::unreserve_named(T::ReserveIdentifier::get(), &entity, deposit);
			} else {
				return Err(Error::<T, I>::InvalidEntity.into())
			}
			Requests::<T, I>::remove(&registry_index, &entity);

			Self::deposit_event(Event::RegistrationRequestCanceled { entity, registry_index });

			Ok(())
		}

		/// Remove Entity from given registry.
		///
		/// * `registry_index` - Registry index to remove from
		/// * `entity` - AccountId of entity to unregister
		///
		/// Will refund deposit of stored data.
		///
		/// Emits `EntityUnregistered`.
		///
		/// Must be called by `RegistryOrigin`
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::unregister(T::MaxRegistrars::get()))]
		pub fn unregister(
			origin: OriginFor<T>,
			registry_index: RegistryIndex,
			entity: T::AccountId,
		) -> DispatchResult {
			let sender = T::RegistrarOrigin::ensure_origin(origin)?;

			Self::registrars()
				.get(registry_index as usize)
				.filter(|acc| *acc == &sender)
				.ok_or(Error::<T, I>::InvalidRegistry)?;

			if let Some(Registration { deposit, .. }) = Self::registries(&registry_index, &entity) {
				// refund deposit
				T::Currency::unreserve_named(T::ReserveIdentifier::get(), &entity, deposit);
			} else {
				return Err(Error::<T, I>::InvalidEntity.into())
			}
			Registries::<T, I>::remove(&registry_index, &entity);

			Self::deposit_event(Event::EntityUnregistered { entity, registry_index });

			Ok(())
		}

		/// Add Entity to Registry. Entity data will be copied to the Registry.
		///
		/// * `registry_index` - Registry index to add to
		/// * `entity` - AccountId of Entity to register
		/// * `data` - Hash of data being registered
		///
		/// Will verify that correct deposit for given data size was paid with
		/// `request_registration`.
		///
		/// Emits `EntityRegistered`.
		///
		/// Must be called by `RegistrarOrigin`
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::register_entity(T::MaxRegistrars::get(), T::EntityData::max_encoded_len() as u32))]
		pub fn register_entity(
			origin: OriginFor<T>,
			registry_index: RegistryIndex,
			entity: T::AccountId,
			data: T::Hash,
		) -> DispatchResult {
			// make sure origin is valid for given registry index
			let sender = T::RegistrarOrigin::ensure_origin(origin)?;
			Self::registrars()
				.get(registry_index as usize)
				.filter(|acc| *acc == &sender)
				.ok_or(Error::<T, I>::InvalidRegistry)?;

			// get the request and verify it matches what we want to register
			let Request { data: request_data, deposit: request_deposit, editable_by_registrar } =
				Self::requests(&registry_index, &entity).ok_or(Error::<T, I>::InvalidEntity)?;
			ensure!(T::Hashing::hash_of(&request_data) == data, Error::<T, I>::MismatchedData);

			// ensure deposit is ok - mostly defensive, it should never fail,
			// unless deposits were increased in runtime upgrade
			let required_deposit = Self::calculate_deposit(&request_data);
			ensure!(request_deposit >= required_deposit, Error::<T, I>::InsufficientDeposit);

			// Refund old deposit, if any
			let old_deposit = Self::registries(&registry_index, &entity)
				.map(|Registration { deposit, .. }| deposit)
				.unwrap_or_default();
			T::Currency::unreserve_named(T::ReserveIdentifier::get(), &entity, old_deposit);

			// Move request to registration
			Requests::<T, I>::remove(&registry_index, &entity);
			Registries::<T, I>::insert(
				&registry_index,
				&entity,
				Registration {
					deposit: request_deposit,
					data: request_data,
					editable_by_registrar,
				},
			);

			Self::deposit_event(Event::EntityRegistered { entity, registry_index });

			Ok(())
		}

		/// Sets Entity data in Registry. Entity must've been registered with
		/// `editable_by_registrar = true`.
		///
		/// * `registry_index` - Registry index to set data in
		/// * `entity` - AccountId of the Entity
		/// * `data` - data to set
		///
		/// Emits `EntityRegistered`.
		///
		/// Must be called by `RegistrarOrigin`
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::set_registered_entity(T::MaxRegistrars::get(), T::EntityData::max_encoded_len() as u32))]
		pub fn set_registered_entity(
			origin: OriginFor<T>,
			registry_index: RegistryIndex,
			entity: T::AccountId,
			data: T::EntityData,
		) -> DispatchResult {
			// make sure origin is ok for registry_index
			let sender = T::RegistrarOrigin::ensure_origin(origin)?;
			Self::registrars()
				.get(registry_index as usize)
				.filter(|acc| *acc == &sender)
				.ok_or(Error::<T, I>::InvalidRegistry)?;

			// verify entity is editable by registrar
			let Registration { deposit, editable_by_registrar, .. } =
				Self::registries(&registry_index, &entity).ok_or(Error::<T, I>::InvalidEntity)?;
			ensure!(editable_by_registrar, Error::<T, I>::NotEditableByRegistrar);

			// ensure there's enough deposit to cover for the new data
			let required_deposit = Self::calculate_deposit(&data);
			ensure!(deposit >= required_deposit, Error::<T, I>::InsufficientDeposit);

			Registries::<T, I>::insert(
				&registry_index,
				&entity,
				Registration { deposit, data, editable_by_registrar },
			);

			Self::deposit_event(Event::EntityRegistered { entity, registry_index });

			Ok(())
		}

		/* see https://github.com/liberland/liberland_substrate/issues/250
		/// Remove Entity from given registry.
		///
		/// * `registry_index` - Registry index to remove from
		///
		/// Will refund deposit of stored data.
		///
		/// Emits `EntityUnregistered`.
		///
		/// Must be called by `EntityOrigin`
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::unregister())]
		pub fn unregister(origin: OriginFor<T>, registry_index: RegistryIndex) -> DispatchResult {
			let entity = T::EntityOrigin::ensure_origin(origin)?;
			if let Some(Registration { deposit, .. }) = Self::registries(&entity, registry_index) {
				// refund deposit
				T::Currency::unreserve_named(T::ReserveIdentifier::get(), &entity, deposit);
			} else {
				return Err(Error::<T, I>::InvalidEntity.into())
			}
			EntityRegistries::<T, I>::remove(&entity, registry_index);

			Self::deposit_event(Event::EntityUnregistered { entity, registry_index });

			Ok(())
		}
		*/
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		fn calculate_deposit(data: &T::EntityData) -> BalanceOf<T, I> {
			let data_len = data.encoded_size() as u32;
			let required_deposit = T::BaseDeposit::get()
				.saturating_add(T::ByteDeposit::get().saturating_mul(data_len.into()));
			required_deposit
		}
	}
}
