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

use frame_support::BoundedVec;
pub type RegistrarIndex = u32;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::{DispatchResult, *},
		traits::{Currency, NamedReservableCurrency},
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use scale_info::prelude::vec;
	use sp_runtime::{traits::Hash, Saturating};

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	type ReserveIdentifierOf<T> = <<T as Config>::Currency as NamedReservableCurrency<
		<T as frame_system::Config>::AccountId,
	>>::ReserveIdentifier;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: NamedReservableCurrency<Self::AccountId>;

		#[pallet::constant]
		type MaxRegistrars: Get<u32>;

		#[pallet::constant]
		type BaseDeposit: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type ByteDeposit: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type ReserveIdentifier: Get<&'static ReserveIdentifierOf<Self>>;

		type RegistrarOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		type EntityData: Parameter + Member + MaxEncodedLen;
	}

	#[pallet::error]
	pub enum Error<T> {
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
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		RegistrarAdded { idx: RegistrarIndex },
		RequestedUpdate { what: T::AccountId },
		RegistryUpdated { what: T::AccountId, registrar: RegistrarIndex },
		EntityCleared { what: T::AccountId },
	}

	#[pallet::storage]
	#[pallet::getter(fn requests)]
	pub(super) type EntityRequests<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		(BalanceOf<T>, T::EntityData),
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn registries)]
	pub(super) type EntityRegistries<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		RegistrarIndex,
		(BalanceOf<T>, Option<T::EntityData>),
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn registrars)]
	pub(super) type Registrars<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxRegistrars>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub registrars: BoundedVec<T::AccountId, T::MaxRegistrars>,
		pub _phantom: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { registrars: Default::default(), _phantom: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			Registrars::<T>::put(&self.registrars);
		}
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn add_registrar(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			T::RegistrarOrigin::ensure_origin(origin)?;

			let idx = Registrars::<T>::try_mutate(
				|registrars| -> Result<RegistrarIndex, DispatchError> {
					registrars.try_push(account).map_err(|_| Error::<T>::TooManyRegistrars)?;
					Ok((registrars.len() - 1) as RegistrarIndex)
				},
			)?;

			Self::deposit_event(Event::RegistrarAdded { idx });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn set_entity(origin: OriginFor<T>, data: T::EntityData) -> DispatchResult {
			let entity = ensure_signed(origin)?;
			let required_deposit = Self::calculate_deposit(&data);
			let old_deposit =
				Self::requests(&entity).map(|(deposit, _)| deposit).unwrap_or_default();

			if required_deposit >= old_deposit {
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

			EntityRequests::<T>::insert(&entity, (required_deposit, data));

			Self::deposit_event(Event::RequestedUpdate { what: entity });

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn clear_entity(origin: OriginFor<T>) -> DispatchResult {
			let entity = ensure_signed(origin)?;

			if let Some((deposit, _)) = Self::requests(&entity) {
				// refund deposit
				T::Currency::unreserve_named(T::ReserveIdentifier::get(), &entity, deposit);
			} else {
				return Err(Error::<T>::InvalidEntity.into());
			}
			EntityRequests::<T>::remove(&entity);

			Self::deposit_event(Event::EntityCleared { what: entity });

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn unregister(origin: OriginFor<T>, registrar: RegistrarIndex) -> DispatchResult {
			let entity = ensure_signed(origin)?;
			if let Some((deposit, _)) = Self::registries(&entity, registrar) {
				// refund deposit
				T::Currency::unreserve_named(T::ReserveIdentifier::get(), &entity, deposit);
			} else {
				return Err(Error::<T>::InvalidEntity.into());
			}
			EntityRegistries::<T>::remove(&entity, registrar);

			Self::deposit_event(Event::EntityCleared { what: entity });

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn request_registration(
			origin: OriginFor<T>,
			registrar: RegistrarIndex,
		) -> DispatchResult {
			let entity = ensure_signed(origin)?;

			let data =
				Self::requests(&entity).map(|(_, data)| data).ok_or(Error::<T>::InvalidEntity)?;

			let (old_deposit, old_data) = Self::registries(&entity, registrar).unwrap_or_default();
			let new_deposit = Self::calculate_deposit(&data);
			let required_deposit = old_deposit.max(new_deposit);

			if required_deposit > old_deposit {
				// reserve more
				T::Currency::reserve_named(
					T::ReserveIdentifier::get(),
					&entity,
					required_deposit - old_deposit,
				)?;
			}

			EntityRegistries::<T>::insert(&entity, registrar, (required_deposit, old_data));

			Self::deposit_event(Event::EntityCleared { what: entity });

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(10_000)]
		pub fn register_entity(
			origin: OriginFor<T>,
			registrar: RegistrarIndex,
			entity: T::AccountId,
			data: T::Hash,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::registrars()
				.get(registrar as usize)
				.filter(|acc| *acc == &sender)
				.ok_or(Error::<T>::InvalidRegistrar)?;

			let request_data =
				Self::requests(&entity).map(|(_, data)| data).ok_or(Error::<T>::InvalidEntity)?;

			ensure!(T::Hashing::hash_of(&request_data) == data, Error::<T>::MismatchedData);

			let deposit = Self::registries(&entity, &registrar)
				.map(|(deposit, _)| deposit)
				.unwrap_or_default();

			let required_deposit = Self::calculate_deposit(&request_data);
			ensure!(deposit >= required_deposit, Error::<T>::InsufficientDeposit);

			EntityRegistries::<T>::insert(&entity, registrar, (deposit, Some(request_data)));

			Self::deposit_event(Event::RegistryUpdated { what: entity, registrar });

			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(10_000)]
		pub fn refund(origin: OriginFor<T>, registrar: RegistrarIndex) -> DispatchResult {
			let entity = ensure_signed(origin)?;
			let (deposit, data) =
				Self::registries(&entity, &registrar).ok_or(Error::<T>::InvalidEntity)?;
			let required_deposit =
				data.as_ref().map(|data| Self::calculate_deposit(data)).unwrap_or_default();
			let refund = deposit.saturating_sub(required_deposit); // shouldn't saturate, but lets be defensive

			T::Currency::unreserve_named(T::ReserveIdentifier::get(), &entity, refund);
			EntityRegistries::<T>::insert(&entity, registrar, (required_deposit, data));

			Self::deposit_event(Event::RegistryUpdated { what: entity, registrar });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn calculate_deposit(data: &T::EntityData) -> BalanceOf<T> {
			let data_len = data.encoded_size() as u32;
			let required_deposit = T::BaseDeposit::get()
				.saturating_add(T::ByteDeposit::get().saturating_mul(data_len.into()));
			required_deposit
		}
	}
}
