/*
Copyright © 2022 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	traits::{Currency, NamedReservableCurrency},
	BoundedVec,
};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

pub type RegistrarIndex = u32;

#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Request<Balance, EntityData> {
	pub deposit: Balance,
	pub data: EntityData,
	pub editable_by_registrar: bool,
}

#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Registration<Balance, EntityData>
where
	Balance: Default,
{
	pub deposit: Balance,
	pub data: Option<EntityData>,
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

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::{DispatchResult, *};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use scale_info::prelude::vec;
	use sp_runtime::{traits::Hash, Saturating};

	type BalanceOf<T, I> =
		<<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
		type Currency: NamedReservableCurrency<Self::AccountId>;

		#[pallet::constant]
		type MaxRegistrars: Get<u32>;

		#[pallet::constant]
		type BaseDeposit: Get<BalanceOf<Self, I>>;

		#[pallet::constant]
		type ByteDeposit: Get<BalanceOf<Self, I>>;

		#[pallet::constant]
		type ReserveIdentifier: Get<&'static ReserveIdentifierOf<Self, I>>;

		type RegistrarOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		type EntityData: Parameter + Member + MaxEncodedLen;
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
		RegistrarAdded { registrar_index: RegistrarIndex },
		EntitySet { entity: T::AccountId },
		EntityCleared { entity: T::AccountId },
		EntityUnregistered { entity: T::AccountId, registrar_index: RegistrarIndex },
		RegistrationRequested { entity: T::AccountId, registrar_index: RegistrarIndex },
		EntityRegistered { entity: T::AccountId, registrar_index: RegistrarIndex },
		RefundProcessed { entity: T::AccountId, registrar_index: RegistrarIndex },
	}

	#[pallet::storage]
	#[pallet::getter(fn requests)]
	pub(super) type EntityRequests<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::AccountId, RequestOf<T, I>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn registries)]
	pub(super) type EntityRegistries<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		RegistrarIndex,
		RegistrationOf<T, I>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn registrars)]
	pub(super) type Registrars<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxRegistrars>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
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
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn add_registrar(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			T::RegistrarOrigin::ensure_origin(origin)?;

			let registrar_index = Registrars::<T, I>::try_mutate(
				|registrars| -> Result<RegistrarIndex, DispatchError> {
					registrars.try_push(account).map_err(|_| Error::<T, I>::TooManyRegistrars)?;
					Ok((registrars.len() - 1) as RegistrarIndex)
				},
			)?;

			Self::deposit_event(Event::RegistrarAdded { registrar_index });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn set_entity(
			origin: OriginFor<T>,
			data: T::EntityData,
			editable_by_registrar: bool,
		) -> DispatchResult {
			let entity = ensure_signed(origin)?;
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

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn clear_entity(origin: OriginFor<T>) -> DispatchResult {
			let entity = ensure_signed(origin)?;

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

		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn unregister(origin: OriginFor<T>, registrar_index: RegistrarIndex) -> DispatchResult {
			let entity = ensure_signed(origin)?;
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

		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn request_registration(
			origin: OriginFor<T>,
			registrar_index: RegistrarIndex,
		) -> DispatchResult {
			let entity = ensure_signed(origin)?;

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

		#[pallet::call_index(5)]
		#[pallet::weight(10_000)]
		pub fn register_entity(
			origin: OriginFor<T>,
			registrar_index: RegistrarIndex,
			entity: T::AccountId,
			data: T::Hash,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

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

		#[pallet::call_index(6)]
		#[pallet::weight(10_000)]
		pub fn refund(origin: OriginFor<T>, registrar_index: RegistrarIndex) -> DispatchResult {
			let entity = ensure_signed(origin)?;
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

		#[pallet::call_index(7)]
		#[pallet::weight(10_000)]
		pub fn set_registered_entity(
			origin: OriginFor<T>,
			registrar_index: RegistrarIndex,
			entity: T::AccountId,
			data: T::EntityData,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::registrars()
				.get(registrar_index as usize)
				.filter(|acc| *acc == &sender)
				.ok_or(Error::<T, I>::InvalidRegistrar)?;

			let Registration { deposit, editable_by_registrar, .. } = Self::registries(&entity, registrar_index).ok_or(Error::<T, I>::InvalidEntity)?;
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
