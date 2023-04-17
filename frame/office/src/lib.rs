//! # Liberland Office Pallet
//!
//! ## Overview
//!
//! Office pallet is a general-purpose pallet for executing external calls using
//! a single AccountId by a centrally managed set of Accounts.
//!
//! ## Terminology
//!
//! * PalletId - Id that will be used to derive AccountId for dispatching calls
//! * Admin - AccountId that can add/update/remove clerks
//! * Clerk - AccountId that's authorized to execute calls using this pallet
//!
//! ## Pallet Config
//!
//! * `PalletId` - PalletId that's used to derive AccountId for dispatching external calls
//! * `AdminOrigin` - origin for checking admin - must return AccountId on success
//! * `ForceOrigin` - origin that can add/remove clerks and update admin without the admin AccountId
//!   check
//! * `CallFilter` - InstanceFilter for filtering calls by clerk - see mock.rs for example
//! * `WeightInfo` - see [Substrate docs](https://docs.substrate.io/reference/how-to-guides/weights/use-custom-weights/)
//!
//! ## Genesis Config
//!
//! * `admin`: Initial admin
//! * `clerks`: Initial clerks
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `set_admin`: Change admin
//! * `set_clerk`: Add or update clerk
//! * `remove_clerk`: Remove clerk
//! * `execute`: Execute an external call using this pallet as origin
//!
//! License: MIT/
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
use weights::WeightInfo;

use codec::MaxEncodedLen;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::GetDispatchInfo,
		pallet_prelude::{DispatchResult, *},
		traits::{InstanceFilter, OriginTrait},
		PalletId,
	};
	use frame_system::{pallet_prelude::*, RawOrigin};
	use scale_info::prelude::vec;
	use sp_runtime::traits::{AccountIdConversion, Dispatchable};
	use sp_std::prelude::*;

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

		/// The overarching call type
		type RuntimeCall: Parameter
			+ GetDispatchInfo
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
			+ IsType<<Self as frame_system::Config>::RuntimeCall>;

		/// PalletId that will be used to derive AccountId for calls originating
		/// from this pallet
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Origin that can add/remove clerks and change admin - must match AccountId
		type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

		/// Origin that can add/remove clerks and change admin - success is enough
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// InstanceFilter for filtering calls by clerk - see mock.rs for example
		type CallFilter: Parameter
			+ Member
			+ InstanceFilter<<Self as Config<I>>::RuntimeCall>
			+ MaybeSerializeDeserialize
			+ Default
			+ MaxEncodedLen;

		/// WeightInfo
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Caller is not authorized to perform given action
		NoPermission,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// New admin set
		AdminChanged { new_admin: T::AccountId },
		/// Call executed by clerk
		CallExecuted { result: DispatchResult },
		/// Clerk added or updated
		ClerkSet { account: T::AccountId, call_filter: T::CallFilter },
		/// Clerk removed
		ClerkRemoved { account: T::AccountId },
	}

	#[pallet::storage]
	#[pallet::getter(fn admin)]
	/// Current admin
	pub(super) type Admin<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn clerks)]
	/// Clerks and their CallFilters
	pub(super) type Clerks<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::CallFilter, OptionQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		/// Initial admin
		pub admin: Option<T::AccountId>,
		/// Initial clerks
		pub clerks: Vec<(T::AccountId, T::CallFilter)>,
	}

	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
		fn build(&self) {
			Admin::<T, I>::set(self.admin.clone());
			for (account, call_filter) in &self.clerks {
				Clerks::<T, I>::insert(account, call_filter);
			}
		}
	}

	#[cfg(feature = "std")]
	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			Self { admin: None, clerks: vec![] }
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::set_admin())]
		/// Change admin
		///
		/// * `account` - AccountId of the new admin
		///
		/// Emits `AdminChanged`.
		///
		/// Must be called by current admin or ForceOrigin.
		pub fn set_admin(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			if let Err(origin) = T::ForceOrigin::try_origin(origin) {
				let caller = T::AdminOrigin::ensure_origin(origin)?;
				ensure!(caller == Self::admin().unwrap(), Error::<T, I>::NoPermission);
			}

			Admin::<T, I>::put(&account);
			Self::deposit_event(Event::<T, I>::AdminChanged { new_admin: account });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::set_clerk())]
		/// Add or update clerk
		///
		/// * `account` - AccountId of the clerk
		/// * `call_filter` - CallFilter for this clerk
		///
		/// Emits `ClerkSet`.
		///
		/// Must be called by admin or ForceOrigin.
		pub fn set_clerk(
			origin: OriginFor<T>,
			account: T::AccountId,
			call_filter: T::CallFilter,
		) -> DispatchResult {
			if let Err(origin) = T::ForceOrigin::try_origin(origin) {
				let caller = T::AdminOrigin::ensure_origin(origin)?;
				ensure!(caller == Self::admin().unwrap(), Error::<T, I>::NoPermission);
			}

			Clerks::<T, I>::insert(&account, &call_filter);
			Self::deposit_event(Event::<T, I>::ClerkSet { account, call_filter });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::remove_clerk())]
		/// Remove clerk
		///
		/// * `account` - AccountId of the clerk
		///
		/// Emits `ClerkRemoved`.
		///
		/// Must be called by admin or ForceOrigin.
		pub fn remove_clerk(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			if let Err(origin) = T::ForceOrigin::try_origin(origin) {
				let caller = T::AdminOrigin::ensure_origin(origin)?;
				ensure!(caller == Self::admin().unwrap(), Error::<T, I>::NoPermission);
			}

			if let Some(_) = Self::clerks(&account) {
				Clerks::<T, I>::remove(&account);
				Self::deposit_event(Event::<T, I>::ClerkRemoved { account });
			}
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight({
			let info = call.get_dispatch_info();
			(T::WeightInfo::execute().saturating_add(info.weight), info.class)
		})]
		/// Execute an external call with Origin Signed by account derived from
		/// PalletId.
		///
		/// Subject to clerks CallFilter.
		///
		/// * `call` - call to execute
		///
		/// Emits `CallExecuted`.
		///
		/// Must be called by a clerk.
		pub fn execute(
			origin: OriginFor<T>,
			call: Box<<T as Config<I>>::RuntimeCall>,
		) -> DispatchResult {
			let clerk = ensure_signed(origin)?;
			let call_filter = Self::clerks(&clerk).ok_or(Error::<T, I>::NoPermission)?;
			Self::do_execute(*call, call_filter);
			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		fn do_execute(call: <T as Config<I>>::RuntimeCall, call_filter: T::CallFilter) {
			let call_account_id = T::PalletId::get().into_account_truncating();
			let mut origin: T::RuntimeOrigin = RawOrigin::Signed(call_account_id).into();
			origin.add_filter(move |call| {
				call_filter.filter(<T as Config<I>>::RuntimeCall::from_ref(call))
			});
			let res = call.dispatch(origin);
			Self::deposit_event(Event::CallExecuted {
				result: res.map(|_| ()).map_err(|e| e.error),
			});
		}
	}
}
