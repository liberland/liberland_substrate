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
pub type Data<MaxLen> = BoundedVec<u8, MaxLen>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		traits::{Currency, ReservableCurrency},	
		pallet_prelude::{DispatchResult, *},
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use scale_info::prelude::vec;
	use sp_runtime::traits::Hash;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config
	{
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: ReservableCurrency<Self::AccountId>;
		
		#[pallet::constant]
		type MaxDataLength: Get<u32>;

		#[pallet::constant]
		type MaxRegistrars: Get<u32>;

		#[pallet::constant]
		type BaseDeposit: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type ByteDeposit: Get<BalanceOf<Self>>;

		type RegistrarOrigin: EnsureOrigin<Self::RuntimeOrigin>;
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
		Data<T::MaxDataLength>,
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
		Data<T::MaxDataLength>,
		OptionQuery,
	>;


	#[pallet::storage]
	#[pallet::getter(fn registrars)]
	pub(super) type Registrars<T: Config> = StorageValue<
		_,
		BoundedVec<T::AccountId, T::MaxRegistrars>,
		ValueQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub registrars: BoundedVec<T::AccountId, T::MaxRegistrars>,
		pub _phantom: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				registrars: Default::default(),
				_phantom: Default::default(),
			}
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
					registrars
						.try_push(account)
						.map_err(|_| Error::<T>::TooManyRegistrars)?;
					Ok((registrars.len() - 1) as RegistrarIndex)
				},
			)?;

			Self::deposit_event(Event::RegistrarAdded { idx });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn set_entity(origin: OriginFor<T>, data: Data<T::MaxDataLength>) -> DispatchResult {
			let entity = ensure_signed(origin)?;

			// FIXME handle deposits

			EntityRequests::<T>::insert(&entity, data);

			Self::deposit_event(Event::RequestedUpdate { what: entity });

			Ok(())
		}


		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn clear_entity(origin: OriginFor<T>) -> DispatchResult {
			let entity = ensure_signed(origin)?;

			// FIXME refund deposits

			EntityRequests::<T>::remove(&entity);

			// safe to ignore the result, as we will never have more than u32::MAX entries
			let _ = EntityRegistries::<T>::clear_prefix(&entity, u32::MAX, None);
			

			Self::deposit_event(Event::EntityCleared { what: entity });

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn register_entity(
			origin: OriginFor<T>,
			registrar: RegistrarIndex,
			entity: T::AccountId,
			data: T::Hash,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// FIXME handle deposits

			Registrars::<T>::get()
				.get(registrar as usize)
				.filter(|acc| *acc == &sender)
				.ok_or(Error::<T>::InvalidRegistrar)?;

			let request_data = EntityRequests::<T>::get(&entity).ok_or(Error::<T>::InvalidEntity)?;

			ensure!(T::Hashing::hash_of(&request_data) == data, Error::<T>::MismatchedData);

			EntityRegistries::<T>::insert(&entity, registrar, request_data);

			Self::deposit_event(Event::RegistryUpdated { what: entity, registrar });

			Ok(())
		}
	}
}