//! # Liberland Registry Pallet
//!
//! ## Overview
//!
//! Registry pallet is a general-purpose pallet for tracking and registering
//! data about abstract entities.
//!
//! In Liberland, it's used to implement Company Registry
//!
//! ## Terminology
//!
//! * Entity - object, identified with AccountId, that can have data attached to it and can be
//!   registered at a registrar
//! * Registrar - AccountId that can register Entities in Registry. Each registrar has it's own
//!   Registry.
//! * Registry - database of Entities and their data that were registered at given Registrar.
//! * Deposit - amount of Currency that gets reserved when setting Entity data or requesting
//!   registration - can be withdrawn when data is removed
//!
//! ## Entity Lifecycle
//!
//! 1. Entity must be created using the `set_entity()` call. This creates
//!    entity and assigns requested data to it.
//! 2. If Entity wishes to be registered at given registrar:
//!     1. It must call `request_registration()` to deposit Currency for registration at given
//! registrar and,     2. The registrar must call `register_entity()` to actually add Entity's data
//! to the Registry 3. If Entity doesn't need any new registrations, it can call
//!    `clear_entity()` which will refund the deposit. It doesn't remove the Entity
//!    from Registries
//! 4. When Entity can unregister at any time with `unregister()` call - deposit will be refunded.
//!
//! To update data in Registry, Entity has to follow the same path as on first registration.
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
//! ```
//! deposit = BaseDeposit + N * ByteDeposit
//! ```
//!
//! Deposits are separate for the current data (`set_entity`, `clear_identity` calls) and for each
//! registry (as data is stored separately as well).
//!
//! * `set_entity(data, editable)` requires deposit length of `data` parameter. Will immediately
//!   reserve/refund any difference.
//! * `clear_identity()` will refund complete deposit for current data.
//! * `unregister()`, `force_unregister()` will refund deposit for data at given registrar.
//! * `request_registration()` will calculate required deposit based on maximum of the current data
//!   and data stored at given registrar (may be 0). Will immediately reserve if needed, but will
//!   not refund any excess (see `refund()` for that)
//! * `refund()` will calculate required deposit based on data stored at given registrar and refund
//!   any excess deposit.
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
//! * `registrars`: registrars that should be preset on genesis
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
//! * `set_entity`: Adds Entity if needed and sets its current data
//! * `clear_entity`: Removes current Entity data - doesn't remove Entity from Registries
//! * `unregister`: Removes Entity from given Registry - called by Entity
//! * `force_unregister`: Removes Entity from given Registry - called by Registrar
//! * `request_registration`: Deposits Currency required to register current data (see `set_entity`)
//!   at given Registry
//! * `register_entity`: Adds Entity to the Registry
//! * `refund`: Refunds any excess deposit for given Registry
//! * `set_registered_entity`: Sets Entity data in given Registry
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

pub type RegistrarIndex = u32;

#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
/// Structure for keeping current Entity data that may be added to Registry by
/// Registrar
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
pub struct Registration<Balance, EntityData>
where
	Balance: Default,
{
	/// Deposit for data - will be at least enough to cover for data stored in
	/// Registry, may be bigger if Entity requested registration update with
	/// bigger data and Registrar didn't confirm it yet
	pub deposit: Balance,
	/// Registered Entity data - if None, Entity isn't yet registered by Registrar
	pub data: Option<EntityData>,
	/// Whether Registrar can edit Entity data
	pub editable_by_registrar: bool,
}

impl<Balance, EntityData> Default for Registration<Balance, EntityData>
where
	Balance: Default,
{
	fn default() -> Self {
		Self { deposit: Default::default(), data: None, editable_by_registrar: false }
	}
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
		/// Invalid registrar - either doesn't exist or not authorized
		InvalidRegistrar,
		/// Trying to register entity with old/wrong data.
		MismatchedData,
		/// Trying to register entity that didn't request anything.
		InvalidEntity,
		/// Insufficient deposit for data storage
		InsufficientDeposit,
		/// Entity doesn't allow edits by registrars
		NotEditableByRegistrar,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// New Registrar added
		RegistrarAdded { registrar_index: RegistrarIndex },
		/// Current (unregistered) data of Entity updated
		EntitySet { entity: T::AccountId },
		/// Current (unregistered) data of Entity cleared
		EntityCleared { entity: T::AccountId },
		/// Entity was removed from Registry
		EntityUnregistered { entity: T::AccountId, registrar_index: RegistrarIndex },
		/// Entity paid deposit for registering
		RegistrationRequested { entity: T::AccountId, registrar_index: RegistrarIndex },
		/// Registrar added Entity to Registry
		EntityRegistered { entity: T::AccountId, registrar_index: RegistrarIndex },
		/// Entity got a refund of excess deposit from Registry
		RefundProcessed { entity: T::AccountId, registrar_index: RegistrarIndex },
	}

	#[pallet::storage]
	#[pallet::getter(fn requests)]
	/// Current, unregistered data of Entities. See `set_entity` and `clear_entity`
	pub(super) type EntityRequests<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::AccountId, RequestOf<T, I>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn registries)]
	/// Registered data of Entities in given Registries
	pub(super) type EntityRegistries<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId, // EntityId
		Blake2_128Concat,
		RegistrarIndex, // Registry
		RegistrationOf<T, I>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn registrars)]
	/// List of registrars - order matters!
	pub(super) type Registrars<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxRegistrars>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		/// Registrars that should be preset on genesis
		pub registrars: BoundedVec<T::AccountId, T::MaxRegistrars>,
	}

	#[cfg(feature = "std")]
	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			Self { registrars: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
		fn build(&self) {
			Registrars::<T, I>::put(&self.registrars);
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Add a new registrar.
		///
		/// * `account` - AccountId of new Registrar
		///
		/// Emits `RegistrarAdded` with the index of new Registry.
		///
		/// Must be called by `AddRegistrarOrigin`
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::add_registrar(T::MaxRegistrars::get()))]
		pub fn add_registrar(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			T::AddRegistrarOrigin::ensure_origin(origin)?;

			let registrar_index = Registrars::<T, I>::try_mutate(
				|registrars| -> Result<RegistrarIndex, DispatchError> {
					registrars.try_push(account).map_err(|_| Error::<T, I>::TooManyRegistrars)?;
					Ok((registrars.len() - 1) as RegistrarIndex)
				},
			)?;

			Self::deposit_event(Event::RegistrarAdded { registrar_index });

			Ok(())
		}

		/// Set current, unregistered data of Entity.
		///
		/// * `data` - data to set
		/// * `editable_by_registrar` - if true, registrar will be able to modify Entity's data in
		///   registry
		///
		/// Entity will be stored under AccountId of caller.
		///
		/// Will reserve deposit to cover stored data.
		///
		/// Emits `EntitySet`.
		///
		/// Must be called by `EntityOrigin`
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::set_entity(T::EntityData::max_encoded_len() as u32))]
		pub fn set_entity(
			origin: OriginFor<T>,
			data: T::EntityData,
			editable_by_registrar: bool,
		) -> DispatchResult {
			let entity = T::EntityOrigin::ensure_origin(origin)?;
			let required_deposit = Self::calculate_deposit(&data);
			let old_deposit = Self::requests(&entity)
				.map(|Request { deposit, .. }| deposit)
				.unwrap_or_default();

			if required_deposit > old_deposit {
				// reserve more
				T::Currency::reserve_named(
					T::ReserveIdentifier::get(),
					&entity,
					required_deposit - old_deposit,
				)?;
			} else {
				// refund excess
				T::Currency::unreserve_named(
					T::ReserveIdentifier::get(),
					&entity,
					old_deposit - required_deposit,
				);
			}

			EntityRequests::<T, I>::insert(
				&entity,
				Request { deposit: required_deposit, data, editable_by_registrar },
			);

			Self::deposit_event(Event::EntitySet { entity });

			Ok(())
		}

		/// Clear current, unregistered data of Entity.
		///
		/// Will refund deposit of stored data.
		///
		/// Emits `EntityCleared`.
		///
		/// Must be called by `EntityOrigin`
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::clear_entity())]
		pub fn clear_entity(origin: OriginFor<T>) -> DispatchResult {
			let entity = T::EntityOrigin::ensure_origin(origin)?;

			if let Some(Request { deposit, .. }) = Self::requests(&entity) {
				// refund deposit
				T::Currency::unreserve_named(T::ReserveIdentifier::get(), &entity, deposit);
			} else {
				return Err(Error::<T, I>::InvalidEntity.into())
			}
			EntityRequests::<T, I>::remove(&entity);

			Self::deposit_event(Event::EntityCleared { entity });

			Ok(())
		}

		/// Remove Entity from given registry.
		///
		/// * `registrar_index` - Registry index to remove from
		///
		/// Will refund deposit of stored data.
		///
		/// Emits `EntityUnregistered`.
		///
		/// Must be called by `EntityOrigin`
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::unregister())]
		pub fn unregister(origin: OriginFor<T>, registrar_index: RegistrarIndex) -> DispatchResult {
			let entity = T::EntityOrigin::ensure_origin(origin)?;
			if let Some(Registration { deposit, .. }) = Self::registries(&entity, registrar_index) {
				// refund deposit
				T::Currency::unreserve_named(T::ReserveIdentifier::get(), &entity, deposit);
			} else {
				return Err(Error::<T, I>::InvalidEntity.into())
			}
			EntityRegistries::<T, I>::remove(&entity, registrar_index);

			Self::deposit_event(Event::EntityUnregistered { entity, registrar_index });

			Ok(())
		}

		/// Remove Entity from given registry.
		///
		/// * `registrar_index` - Registry index to remove from
		/// * `entity` - AccountId of entity to unregister
		///
		/// Will refund deposit of stored data.
		///
		/// Emits `EntityUnregistered`.
		///
		/// Must be called by `RegistryOrigin`
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::force_unregister(T::MaxRegistrars::get()))]
		pub fn force_unregister(
			origin: OriginFor<T>,
			registrar_index: RegistrarIndex,
			entity: T::AccountId,
		) -> DispatchResult {
			let sender = T::RegistrarOrigin::ensure_origin(origin)?;

			Self::registrars()
				.get(registrar_index as usize)
				.filter(|acc| *acc == &sender)
				.ok_or(Error::<T, I>::InvalidRegistrar)?;

			if let Some(Registration { deposit, .. }) = Self::registries(&entity, registrar_index) {
				// refund deposit
				T::Currency::unreserve_named(T::ReserveIdentifier::get(), &entity, deposit);
			} else {
				return Err(Error::<T, I>::InvalidEntity.into())
			}
			EntityRegistries::<T, I>::remove(&entity, registrar_index);

			Self::deposit_event(Event::EntityUnregistered { entity, registrar_index });

			Ok(())
		}

		/// Pay deposit for registering Entity in given Registry
		///
		/// * `registrar_index` - Registry index to remove from
		///
		/// Will reserve deposit to cover stored data based on current data (see `set_entity`).
		///
		/// Emits `RegistrationRequested`.
		///
		/// Must be called by `EntityOrigin`
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::request_registration(T::EntityData::max_encoded_len() as u32))]
		pub fn request_registration(
			origin: OriginFor<T>,
			registrar_index: RegistrarIndex,
		) -> DispatchResult {
			let entity = T::EntityOrigin::ensure_origin(origin)?;

			let Request { data, .. } =
				Self::requests(&entity).ok_or(Error::<T, I>::InvalidEntity)?;

			let old_registration = Self::registries(&entity, registrar_index).unwrap_or_default();
			let new_deposit = Self::calculate_deposit(&data);
			let required_deposit = old_registration.deposit.max(new_deposit);

			if required_deposit > old_registration.deposit {
				// reserve more
				T::Currency::reserve_named(
					T::ReserveIdentifier::get(),
					&entity,
					required_deposit - old_registration.deposit,
				)?;
			}

			EntityRegistries::<T, I>::insert(
				&entity,
				registrar_index,
				Registration { deposit: required_deposit, ..old_registration },
			);

			Self::deposit_event(Event::RegistrationRequested { entity, registrar_index });

			Ok(())
		}

		/// Add Entity to registry. Entity data will be copied to the registry.
		///
		/// * `registrar_index` - Registry index to add to
		/// * `entity` - AccountId of Entity to register
		/// * `data` - Hash of data being registered
		///
		/// Will verify that correct deposit for given data size was paid with
		/// `request_registration`.
		///
		/// Emits `EntityRegistered`.
		///
		/// Must be called by `RegistrarOrigin`
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::register_entity(T::MaxRegistrars::get(), T::EntityData::max_encoded_len() as u32))]
		pub fn register_entity(
			origin: OriginFor<T>,
			registrar_index: RegistrarIndex,
			entity: T::AccountId,
			data: T::Hash,
		) -> DispatchResult {
			let sender = T::RegistrarOrigin::ensure_origin(origin)?;

			Self::registrars()
				.get(registrar_index as usize)
				.filter(|acc| *acc == &sender)
				.ok_or(Error::<T, I>::InvalidRegistrar)?;

			let Request { data: request_data, editable_by_registrar, .. } =
				Self::requests(&entity).ok_or(Error::<T, I>::InvalidEntity)?;

			ensure!(T::Hashing::hash_of(&request_data) == data, Error::<T, I>::MismatchedData);

			let deposit = Self::registries(&entity, &registrar_index)
				.map(|Registration { deposit, .. }| deposit)
				.unwrap_or_default();

			let required_deposit = Self::calculate_deposit(&request_data);
			ensure!(deposit >= required_deposit, Error::<T, I>::InsufficientDeposit);

			EntityRegistries::<T, I>::insert(
				&entity,
				registrar_index,
				Registration { deposit, data: Some(request_data), editable_by_registrar },
			);

			Self::deposit_event(Event::EntityRegistered { entity, registrar_index });

			Ok(())
		}

		/// Refunds any excess deposit from given registry. Usually called to
		/// cancel `request_registration` or after size of registered data was
		/// reduced.
		///
		/// * `registrar_index` - Registry index to refund from
		///
		/// Emits `RefundProcessed`.
		///
		/// Must be called by `EntityOrigin`
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::refund(T::EntityData::max_encoded_len() as u32))]
		pub fn refund(origin: OriginFor<T>, registrar_index: RegistrarIndex) -> DispatchResult {
			let entity = T::EntityOrigin::ensure_origin(origin)?;
			let Registration { deposit, data, editable_by_registrar } =
				Self::registries(&entity, &registrar_index).ok_or(Error::<T, I>::InvalidEntity)?;
			let required_deposit =
				data.as_ref().map(|data| Self::calculate_deposit(data)).unwrap_or_default();
			let refund = deposit.saturating_sub(required_deposit); // shouldn't saturate, but lets be defensive

			T::Currency::unreserve_named(T::ReserveIdentifier::get(), &entity, refund);
			EntityRegistries::<T, I>::insert(
				&entity,
				registrar_index,
				Registration { deposit: required_deposit, data, editable_by_registrar },
			);

			Self::deposit_event(Event::RefundProcessed { entity, registrar_index });

			Ok(())
		}

		/// Sets Entity data in Registry. Entity must've been registered with
		/// `editable_by_registrar = true`.
		///
		/// * `registrar_index` - Registry index to set data in
		/// * `entity` - AccountId of the Entity
		/// * `data` - data to set
		///
		/// Emits `EntityRegistered`.
		///
		/// Must be called by `RegistrarOrigin`
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::set_registered_entity(T::MaxRegistrars::get(), T::EntityData::max_encoded_len() as u32))]
		pub fn set_registered_entity(
			origin: OriginFor<T>,
			registrar_index: RegistrarIndex,
			entity: T::AccountId,
			data: T::EntityData,
		) -> DispatchResult {
			let sender = T::RegistrarOrigin::ensure_origin(origin)?;

			Self::registrars()
				.get(registrar_index as usize)
				.filter(|acc| *acc == &sender)
				.ok_or(Error::<T, I>::InvalidRegistrar)?;

			let Registration { deposit, editable_by_registrar, .. } =
				Self::registries(&entity, registrar_index).ok_or(Error::<T, I>::InvalidEntity)?;
			ensure!(editable_by_registrar, Error::<T, I>::NotEditableByRegistrar);

			let required_deposit = Self::calculate_deposit(&data);
			ensure!(deposit >= required_deposit, Error::<T, I>::InsufficientDeposit);

			EntityRegistries::<T, I>::insert(
				&entity,
				registrar_index,
				Registration { deposit, data: Some(data), editable_by_registrar },
			);

			Self::deposit_event(Event::EntityRegistered { entity, registrar_index });

			Ok(())
		}
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
