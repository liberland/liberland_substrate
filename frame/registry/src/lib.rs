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
//! * Entity - object, identified with EntityId, that has data attached to it and can be registered
//!   at a registrar
//! * Registrar - AccountId that can register Entities in Registry.
//! * Registry - database of Entities and their data that were registered.
//! * Deposit - amount of Currency that gets reserved when requesting registration - refunded data
//!   is removed
//! * Owner - AccountId that can request registrations for Entity
//!
//! ## Entity Lifecycle
//!
//! 1. Entity is created by owner with `request_entity()` call. This assigns an
//!    EntityId and requests first registration.
//! 2. If Registrar approves Entity's data, they call `register_entity()` which
//!    moves data from Registration Request to the Registry.
//! 3. To update the data or register at additional Registry, Entity's owner can
//!    use `request_registration()` call.
//! 4. Entity can be removed from Registry by the Registrar - deposit will be refunded.
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
//! * `request_entity(registry, data, editable)` requires deposit for length of `data` parameter.
//!   Will immediately reserve the required deposit.
//! * `request_registration(registry, data, editable)` requires deposit for length of `data`
//!   parameter. Will refund deposit of previous pending request (if any) and immediately reserve
//!   the new required deposit.
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
//! * `EntityId` - type that will be used to identify Entities - usually `u32` or bigger unsigned
//!   int type
//! * `WeightInfo` - see [Substrate docs](https://docs.substrate.io/reference/how-to-guides/weights/use-custom-weights/)
//!
//! ## Genesis Config
//!
//! * `registries`: registries that should be preset on genesis
//! * `entities`: entities that should exist on genesis - will collect deposits from owners
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `add_registry`: Adds a new registrar
//! * `request_entity`: Creates Entity, assigns EntityId and requests first registration
//! * `request_registration`: Requests an additional or updated registration of Entity in Registry
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
	use sp_runtime::{
		traits::{AtLeast32BitUnsigned, CheckedAdd, Hash, MaybeSerializeDeserialize},
		Saturating,
	};

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
		type EntityData: Parameter + Member + MaybeSerializeDeserialize + MaxEncodedLen;

		/// Type for identifying Entities
		type EntityId: Parameter + Member + MaxEncodedLen + AtLeast32BitUnsigned + Default;

		/// WeightInfo
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Maximum amount of registrars reached. Cannot add any more.
		TooManyRegistrars,
		/// Trying to register entity that didn't request anything or sender not authorized
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
		RegistrationRequested { entity_id: T::EntityId, registry_index: RegistryIndex },
		/// Entity canceled registration request before registrar registered it
		RegistrationRequestCanceled { entity_id: T::EntityId, registry_index: RegistryIndex },
		/// Registrar added Entity to Registry
		EntityRegistered { entity_id: T::EntityId, registry_index: RegistryIndex },
		/// Entity was removed from Registry
		EntityUnregistered { entity_id: T::EntityId, registry_index: RegistryIndex },
	}

	#[pallet::storage]
	#[pallet::getter(fn requests)]
	/// Registration requests. See `request_registration` and `cancel_request`
	pub(super) type Requests<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		RegistryIndex,
		Blake2_128Concat,
		T::EntityId,
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
		RegistryIndex,
		Blake2_128Concat,
		T::EntityId,
		RegistrationOf<T, I>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn registrars)]
	/// List of registries - order matters, as it's tied to RegistryIndex!
	pub(super) type Registrars<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxRegistrars>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn entity_owner)]
	/// Map from EntityId to AccountId - owners of Entity that do actions on it
	pub(super) type EntityOwner<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::EntityId, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner_entities)]
	/// Map from AccountId to EntityId - list of Entities owned by given account
	pub(super) type OwnerEntities<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::EntityId,
		bool,
		ValueQuery,
	>;

	// no getter on purpose - we dont want direct accesses
	// use get_next_entity_id only!
	#[pallet::storage]
	/// Next free EntityId
	pub(super) type NextEntityId<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::EntityId, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		/// Registries that should be preset on genesis
		pub registries: BoundedVec<T::AccountId, T::MaxRegistrars>,

		/// Entities that should be preset on genesis - collects deposits from
		/// owners automatically - will fail if not enough balance or if
		/// registry doesn't exist
		pub entities: Vec<(
			T::AccountId,  // Owner
			RegistryIndex, // Registry
			T::EntityData, // Data
			bool,          // editable_by_registrar?
		)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			Self { registries: Default::default(), entities: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
		fn build(&self) {
			Registrars::<T, I>::put(&self.registries);

			for (owner, reg_idx, data, editable_by_registrar) in &self.entities {
				assert!(*reg_idx < self.registries.len() as u32);
				let entity_id = Pallet::<T, I>::get_next_entity_id().unwrap();
				let deposit = Pallet::<T, I>::calculate_deposit(&data);
				T::Currency::reserve_named(T::ReserveIdentifier::get(), &owner, deposit).unwrap();
				EntityOwner::<T, I>::insert(&entity_id, owner);
				OwnerEntities::<T, I>::insert(owner, &entity_id, true);
				Registries::<T, I>::insert(
					reg_idx,
					&entity_id,
					Registration {
						deposit,
						data: data.clone(),
						editable_by_registrar: *editable_by_registrar,
					},
				);
			}
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

		/// Assign an EntityId and request registration of Entity in Registry.
		/// This will set the caller as the owner of the new Entity. Check the
		/// RegistrationRequested event to fetch assigned EntityId.
		///
		/// * `registry_index` - Registry to register in
		/// * `data` - data to set
		/// * `editable_by_registrar` - if true, registrar will be able to modify Entity's data in
		///   the Registry
		///
		/// Will reserve deposit to cover stored data.
		///
		/// Emits `RegistrationRequested`.
		///
		/// Must be called by `EntityOrigin`
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::request_entity(T::EntityData::max_encoded_len() as u32))]
		pub fn request_entity(
			origin: OriginFor<T>,
			registry_index: RegistryIndex,
			data: T::EntityData,
			editable_by_registrar: bool,
		) -> DispatchResult {
			let owner = T::EntityOrigin::ensure_origin(origin)?;
			let entity_id = Self::get_next_entity_id().ok_or(Error::<T, I>::InvalidEntity)?;
			EntityOwner::<T, I>::insert(&entity_id, &owner);
			OwnerEntities::<T, I>::insert(&owner, &entity_id, true);
			Self::do_request(owner, entity_id, registry_index, data, editable_by_registrar)
		}

		/// Request an additional registration or updated registration of Entity
		/// in Registry. If there already is a pending request, it will be
		/// canceled and it's deposit will be refunded.
		///
		/// Must be called by the Owner of the Entity
		///
		/// * `registry_index` - Registry to register in
		/// * `entity_id` - Entity to register
		/// * `data` - data to set
		/// * `editable_by_registrar` - if true, registrar will be able to modify Entity's data in
		///   the Registry
		///
		/// Will reserve deposit to cover stored data.
		///
		/// Emits `RegistrationRequested`.
		///
		/// Must be called by `EntityOrigin`
		/// Must be called by Owner of the Entity
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::request_registration(T::EntityData::max_encoded_len() as u32))]
		pub fn request_registration(
			origin: OriginFor<T>,
			registry_index: RegistryIndex,
			entity_id: T::EntityId,
			data: T::EntityData,
			editable_by_registrar: bool,
		) -> DispatchResult {
			let owner = T::EntityOrigin::ensure_origin(origin)?;
			Self::ensure_entity_owner(&owner, &entity_id)?;
			Self::do_request(owner, entity_id, registry_index, data, editable_by_registrar)
		}

		/// Cancel Entity's Registration request for given Registry
		///
		/// * `registry_index` - Registry to register in
		/// * `entity_id` - Entity to register
		///
		/// Will refund deposit of stored data.
		///
		/// Emits `RegistrationRequestCanceled`.
		///
		/// Must be called by `EntityOrigin`
		/// Must be called by Owner of the Entity
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::cancel_request())]
		pub fn cancel_request(
			origin: OriginFor<T>,
			registry_index: RegistryIndex,
			entity_id: T::EntityId,
		) -> DispatchResult {
			let owner = T::EntityOrigin::ensure_origin(origin)?;
			Self::ensure_entity_owner(&owner, &entity_id)?;

			if let Some(Request { deposit, .. }) = Self::requests(&registry_index, &entity_id) {
				// refund deposit
				let owner = Self::entity_owner(&entity_id).ok_or(Error::<T, I>::InvalidEntity)?;
				T::Currency::unreserve_named(T::ReserveIdentifier::get(), &owner, deposit);
			} else {
				return Err(Error::<T, I>::InvalidEntity.into())
			}
			Requests::<T, I>::remove(&registry_index, &entity_id);

			Self::deposit_event(Event::RegistrationRequestCanceled { entity_id, registry_index });

			Ok(())
		}

		/// Remove Entity from given registry.
		///
		/// * `registry_index` - Registry index to remove from
		/// * `entity_id` - AccountId of entity to unregister
		///
		/// Will refund deposit of stored data.
		///
		/// Emits `EntityUnregistered`.
		///
		/// Must be called by `RegistryOrigin`
		/// Must be called by Registrar of the Registry
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::unregister(T::MaxRegistrars::get()))]
		pub fn unregister(
			origin: OriginFor<T>,
			registry_index: RegistryIndex,
			entity_id: T::EntityId,
		) -> DispatchResult {
			let sender = T::RegistrarOrigin::ensure_origin(origin)?;
			Self::ensure_registrar(&sender, registry_index)?;

			if let Some(Registration { deposit, .. }) =
				Self::registries(&registry_index, &entity_id)
			{
				// refund deposit
				let owner = Self::entity_owner(&entity_id).ok_or(Error::<T, I>::InvalidEntity)?;
				T::Currency::unreserve_named(T::ReserveIdentifier::get(), &owner, deposit);
			} else {
				return Err(Error::<T, I>::InvalidEntity.into())
			}
			Registries::<T, I>::remove(&registry_index, &entity_id);

			Self::deposit_event(Event::EntityUnregistered { entity_id, registry_index });

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
		/// Must be called by `RegistryOrigin`
		/// Must be called by Registrar of the Registry
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::register_entity(T::MaxRegistrars::get(), T::EntityData::max_encoded_len() as u32))]
		pub fn register_entity(
			origin: OriginFor<T>,
			registry_index: RegistryIndex,
			entity_id: T::EntityId,
			data: T::Hash,
		) -> DispatchResult {
			// make sure origin is valid for given registry index
			let sender = T::RegistrarOrigin::ensure_origin(origin)?;
			Self::ensure_registrar(&sender, registry_index)?;

			// get the request and verify it matches what we want to register
			let Request { data: request_data, deposit: request_deposit, editable_by_registrar } =
				Self::requests(&registry_index, &entity_id).ok_or(Error::<T, I>::InvalidEntity)?;
			ensure!(T::Hashing::hash_of(&request_data) == data, Error::<T, I>::MismatchedData);

			// ensure deposit is ok - mostly defensive, it should never fail,
			// unless deposits were increased in runtime upgrade
			let required_deposit = Self::calculate_deposit(&request_data);
			ensure!(request_deposit >= required_deposit, Error::<T, I>::InsufficientDeposit);

			// Refund old deposit, if any
			let owner = Self::entity_owner(&entity_id).ok_or(Error::<T, I>::InvalidEntity)?;
			let old_deposit = Self::registries(&registry_index, &entity_id)
				.map(|Registration { deposit, .. }| deposit)
				.unwrap_or_default();
			T::Currency::unreserve_named(T::ReserveIdentifier::get(), &owner, old_deposit);

			// Move request to registration
			Requests::<T, I>::remove(&registry_index, &entity_id);
			Registries::<T, I>::insert(
				&registry_index,
				&entity_id,
				Registration {
					deposit: request_deposit,
					data: request_data,
					editable_by_registrar,
				},
			);

			Self::deposit_event(Event::EntityRegistered { entity_id, registry_index });

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
		/// Must be called by `RegistryOrigin`
		/// Must be called by Registrar of the Registry
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::set_registered_entity(T::MaxRegistrars::get(), T::EntityData::max_encoded_len() as u32))]
		pub fn set_registered_entity(
			origin: OriginFor<T>,
			registry_index: RegistryIndex,
			entity_id: T::EntityId,
			data: T::EntityData,
		) -> DispatchResult {
			// make sure origin is ok for registry_index
			let sender = T::RegistrarOrigin::ensure_origin(origin)?;
			Self::ensure_registrar(&sender, registry_index)?;

			// verify entity is editable by registrar
			let Registration { deposit, editable_by_registrar, .. } =
				Self::registries(&registry_index, &entity_id)
					.ok_or(Error::<T, I>::InvalidEntity)?;
			ensure!(editable_by_registrar, Error::<T, I>::NotEditableByRegistrar);

			// ensure there's enough deposit to cover for the new data
			let required_deposit = Self::calculate_deposit(&data);
			ensure!(deposit >= required_deposit, Error::<T, I>::InsufficientDeposit);

			Registries::<T, I>::insert(
				&registry_index,
				&entity_id,
				Registration { deposit, data, editable_by_registrar },
			);

			Self::deposit_event(Event::EntityRegistered { entity_id, registry_index });

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

		fn get_next_entity_id() -> Option<T::EntityId> {
			let next_entity_id = NextEntityId::<T, I>::get();
			NextEntityId::<T, I>::try_mutate(|v| -> Result<(), ()> {
				*v = (*v).checked_add(&1u8.into()).ok_or(())?;
				Ok(())
			})
			.ok()?;
			Some(next_entity_id)
		}

		/// ensures that given AccountId is the Owner of given Entity
		fn ensure_entity_owner(who: &T::AccountId, entity_id: &T::EntityId) -> DispatchResult {
			let owner = Self::entity_owner(entity_id).ok_or(Error::<T, I>::InvalidEntity)?;
			ensure!(owner == *who, Error::<T, I>::InvalidEntity);
			Ok(())
		}

		/// ensures that given AccountId is the Registrar of given Registry
		fn ensure_registrar(who: &T::AccountId, registry_index: RegistryIndex) -> DispatchResult {
			Self::registrars()
				.get(registry_index as usize)
				.filter(|acc| *acc == who)
				.ok_or(Error::<T, I>::InvalidRegistry)?;
			Ok(())
		}

		fn do_request(
			owner: T::AccountId,
			entity_id: T::EntityId,
			registry_index: RegistryIndex,
			data: T::EntityData,
			editable_by_registrar: bool,
		) -> DispatchResult {
			let required_deposit = Self::calculate_deposit(&data);
			let old_deposit = Self::requests(&registry_index, &entity_id)
				.map(|Request { deposit, .. }| deposit)
				.unwrap_or_default();

			// refund old deposit, if any
			T::Currency::unreserve_named(T::ReserveIdentifier::get(), &owner, old_deposit);

			// reserve new deposit
			T::Currency::reserve_named(T::ReserveIdentifier::get(), &owner, required_deposit)?;

			Requests::<T, I>::insert(
				&registry_index,
				&entity_id,
				Request { deposit: required_deposit, data, editable_by_registrar },
			);

			Self::deposit_event(Event::RegistrationRequested { registry_index, entity_id });

			Ok(())
		}
	}
}
