//! # Liberland Custom Account Pallet
//!
//! ## Overview
//!
//! Custom Account pallet is a general-purpose pallet for executing arbitrary calls
//! using preconfigured AccountId as Origin. It allows filtering who can execute
//! calls (via configurable EnsureOrigin) and filtering what calls can be made.
//! ## Pallet Config
//!
//! * `PalletId` - PalletId that's used to derive AccountId for dispatching calls
//! * `ExecuteOrigin` - origin that can execute calls
//! * `CallFilter` - Contains for filtering calls by clerk - see mock.rs for example
//! * `WeightInfo` - see [Substrate docs](https://docs.substrate.io/reference/how-to-guides/weights/use-custom-weights/)
//!
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `execute`: Execute an external call
//!
//! License: MIT.
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
use frame_support::traits::Currency;
use frame_support::traits::Imbalance;

type NegativeImbalanceOf<T, I> = <<T as Config<I>>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

pub type BalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::GetDispatchInfo,
		pallet_prelude::{DispatchResult, *},
		traits::{Contains, OnUnbalanced, OriginTrait, SortedMembers},
		PalletId,
	};
	use frame_system::{pallet_prelude::*, RawOrigin};
	use scale_info::prelude::vec;
	use sp_runtime::traits::{AccountIdConversion, Dispatchable};
	use sp_std::prelude::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
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

		/// Origin that can call execute
		type ExecuteOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Contains for filtering calls - see mock.rs for example
		type CallFilter: Parameter
			+ Member
			+ Contains<<Self as Config<I>>::RuntimeCall>
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen;

		/// WeightInfo
		type WeightInfo: WeightInfo;

		type Currency: Currency<Self::AccountId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// Call executed
		CallExecuted {
			result: DispatchResult,
		},
		Deposit {
			value: BalanceOf<T, I>,
		},
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(0)]
		#[pallet::weight({
			let info = call.get_dispatch_info();
			(T::WeightInfo::execute().saturating_add(info.weight), info.class)
		})]
		/// Execute an external call with Origin Signed by account derived from
		/// PalletId.
		///
		/// Subject to CallFilter.
		///
		/// * `call` - call to execute
		///
		/// Emits `CallExecuted`.
		///
		/// Must be called by a ExecuteOrigin.
		pub fn execute(
			origin: OriginFor<T>,
			call: Box<<T as Config<I>>::RuntimeCall>,
		) -> DispatchResult {
			T::ExecuteOrigin::ensure_origin(origin)?;
			Self::do_execute(*call)
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		fn do_execute(call: <T as Config<I>>::RuntimeCall) -> DispatchResult {
			let call_account_id = T::PalletId::get().into_account_truncating();
			let mut origin: T::RuntimeOrigin = RawOrigin::Signed(call_account_id).into();
			origin.add_filter(move |call| {
				T::CallFilter::contains(<T as Config<I>>::RuntimeCall::from_ref(call))
			});
			let res = call.dispatch(origin).map(|_| ()).map_err(|e| e.error);
			Self::deposit_event(Event::CallExecuted { result: res });
			res
		}
	}

	impl<T: Config<I>, I: 'static> OnUnbalanced<NegativeImbalanceOf<T, I>> for Pallet<T, I> {
		fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<T, I>) {
			let numeric_amount = amount.peek();
			let call_account_id = T::PalletId::get().into_account_truncating();
			// Must resolve into existing but better to be safe.
			let _ = T::Currency::resolve_creating(&call_account_id, amount);
			Self::deposit_event(Event::Deposit { value: numeric_amount });
		}
	}

	impl<T: Config<I>, I: 'static> SortedMembers<T::AccountId> for Pallet<T, I> {
		fn sorted_members() -> Vec<T::AccountId> {
			vec![T::PalletId::get().into_account_truncating()]
		}
		#[cfg(feature = "runtime-benchmarks")]
		fn add(_m: &T::AccountId) {}
	}
}
