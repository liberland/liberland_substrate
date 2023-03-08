/*
Copyright © 2023 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as LLM;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::pallet_prelude::*;
use frame_system::RawOrigin;
use pallet_assets::Pallet as AssetsPallet;
use sp_std::prelude::*;

const SEED: u32 = 0;

benchmarks! {
	politics_lock {
		let account: T::AccountId = account("acc", 0, SEED);
		let origin = RawOrigin::Signed(account.clone()).into();
		let amount: T::Balance = 1u8.into();
		LLM::<T>::transfer_from_vault(account.clone(), amount.clone()).unwrap();
		let expected = LLM::<T>::llm_politics(&account) + amount.clone();
	}: _<T::RuntimeOrigin>(origin, amount)
	verify {
		assert_eq!(LLM::<T>::llm_politics(account), expected);
	}

	politics_unlock {
		let account: T::AccountId = account("acc", 0, SEED);
		let origin: T::RuntimeOrigin = RawOrigin::Signed(account.clone()).into();
		let amount: T::Balance = 10000u32.into();

		LLM::<T>::transfer_from_vault(account.clone(), amount.clone()).unwrap();
		LLM::<T>::politics_lock(origin.clone(), amount.clone()).unwrap();

		let asset_id = <T as Config>::AssetId::get();
		assert_eq!(AssetsPallet::<T>::balance(asset_id, &account), 0u8.into());
	}: _<T::RuntimeOrigin>(origin)
	verify {
		assert!(AssetsPallet::<T>::balance(asset_id, &account) > 0u8.into());
	}

	treasury_llm_transfer {
		let senate = account("senate", 0, SEED);
		LLM::<T>::set_senate(RawOrigin::Root.into(), Some(senate)).unwrap();
		let origin = RawOrigin::Signed(LLM::<T>::senate().unwrap()).into();
		let account = account("acc", 0, SEED);
		let asset_id = <T as Config>::AssetId::get();
		assert_eq!(AssetsPallet::<T>::balance(asset_id, &account), 0u8.into());
	}: _<T::RuntimeOrigin>(origin, account.clone(), 1u8.into())
	verify {
		assert_eq!(AssetsPallet::<T>::balance(asset_id, &account), 1u8.into());
	}

	treasury_llm_transfer_to_politipool {
		let senate = account("senate", 0, SEED);
		LLM::<T>::set_senate(RawOrigin::Root.into(), Some(senate)).unwrap();
		let origin = RawOrigin::Signed(LLM::<T>::senate().unwrap()).into();
		let account: T::AccountId = account("acc", 0, SEED);
		assert_eq!(LLM::<T>::llm_politics(&account), 0u8.into());
	}: _<T::RuntimeOrigin>(origin, account.clone(), 1u8.into())
	verify {
		assert_eq!(LLM::<T>::llm_politics(&account), 1u8.into());
	}

	send_llm_to_politipool {
		let origin = RawOrigin::Signed(LLM::<T>::get_llm_vault_account()).into();
		let account: T::AccountId = account("acc", 0, SEED);
		assert_eq!(LLM::<T>::llm_politics(&account), 0u8.into());
	}: _<T::RuntimeOrigin>(origin, account.clone(), 1u8.into())
	verify {
		assert_eq!(LLM::<T>::llm_politics(&account), 1u8.into());
	}

	send_llm {
		let origin = RawOrigin::Signed(LLM::<T>::get_llm_vault_account()).into();
		let account = account("acc", 0, SEED);
		let asset_id = <T as Config>::AssetId::get();
		assert_eq!(AssetsPallet::<T>::balance(asset_id, &account), 0u8.into());
	}: _<T::RuntimeOrigin>(origin, account.clone(), 1u8.into())
	verify {
		assert_eq!(AssetsPallet::<T>::balance(asset_id, &account), 1u8.into());
	}

	set_senate {
		let origin = RawOrigin::Root.into();
		let account: T::AccountId = account("acc", 0, SEED);
	}: _<T::RuntimeOrigin>(origin, Some(account.clone()))
	verify {
		assert_eq!(LLM::<T>::senate(), Some(account));
	}
}

impl_benchmark_test_suite!(LLM, crate::mock::new_test_ext(), crate::mock::Test,);
