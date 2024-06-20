/*
Copyright © 2023 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::{LLMPolitics, Pallet as LLM};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use sp_core::Get;
use sp_runtime::{BoundedVec, Saturating};
use sp_std::prelude::*;

const SEED: u32 = 0;

benchmarks! {
	politics_lock {
		let user: T::AccountId = account("user", 0, SEED);
		let amount: T::Balance = 100u8.into();
		LLM::<T>::transfer_from_treasury(user.clone(), amount.clone()).unwrap();
		let origin = RawOrigin::Signed(user.clone());
		assert_eq!(LLMPolitics::<T>::get(&user), 0u8.into());
	}: _(origin, amount.clone())
	verify {
		assert_eq!(LLMPolitics::<T>::get(&user), amount);
	}

	politics_unlock {
		let user: T::AccountId = account("user", 0, SEED);
		let amount: T::Balance = 10000u32.into();
		LLM::<T>::transfer_from_treasury(user.clone(), amount.clone()).unwrap();
		let origin = RawOrigin::Signed(user.clone());
		LLM::<T>::politics_lock(origin.clone().into(), amount.clone()).unwrap();
		assert_eq!(LLMPolitics::<T>::get(&user), amount.clone());
	}: _(origin)
	verify {
		assert!(LLMPolitics::<T>::get(&user) < amount);
	}

	treasury_llm_transfer {
		let user: T::AccountId = account("user", 0, SEED);
		let amount: T::Balance = 100u32.into();
		assert_eq!(LLM::<T>::balance(user.clone()), 0u8.into());
	}: _(RawOrigin::Root, user.clone(), amount.clone())
	verify {
		assert_eq!(LLM::<T>::balance(user), amount);
	}

	treasury_llm_transfer_to_politipool {
		let user: T::AccountId = account("user", 0, SEED);
		let amount: T::Balance = 100u32.into();
		assert_eq!(LLMPolitics::<T>::get(&user), 0u8.into());
	}: _(RawOrigin::Root, user.clone(), amount.clone())
	verify {
		assert_eq!(LLMPolitics::<T>::get(&user), amount);
	}

	send_llm_to_politipool {
		let user: T::AccountId = account("user", 0, SEED);
		let amount: T::Balance = 100u8.into();
		LLM::<T>::transfer_from_treasury(user.clone(), amount.clone()).unwrap();
		let origin = RawOrigin::Signed(user.clone());

		let user2: T::AccountId = account("user", 1, SEED);
		assert_eq!(LLMPolitics::<T>::get(&user2), 0u8.into());
	}: _(origin, user2.clone(), amount.clone())
	verify {
		assert_eq!(LLMPolitics::<T>::get(&user2), amount);
	}

	send_llm {
		let user: T::AccountId = account("user", 0, SEED);
		let amount: T::Balance = 100u8.into();
		LLM::<T>::transfer_from_treasury(user.clone(), amount.clone()).unwrap();
		let origin = RawOrigin::Signed(user.clone());

		let user2: T::AccountId = account("user", 1, SEED);
		assert_eq!(LLM::<T>::balance(user2.clone()), 0u8.into());
		assert_eq!(LLM::<T>::balance(user.clone()), amount.clone());
	}: _(origin, user2.clone(), amount.clone())
	verify {
		assert_eq!(LLM::<T>::balance(user), 0u8.into());
		assert_eq!(LLM::<T>::balance(user2), amount);
	}

	treasury_lld_transfer {
		let user: T::AccountId = account("user", 0, SEED);
		let amount = <<T as Config>::Currency as Currency<T::AccountId>>::minimum_balance();
		<<T as Config>::Currency as Currency<T::AccountId>>::make_free_balance_be(
			&LLM::<T>::get_llm_treasury_account(),
			amount.saturating_mul(2u8.into()),
		);
		assert_eq!(<<T as Config>::Currency as Currency<T::AccountId>>::total_balance(&user), 0u8.into());
	}: _(RawOrigin::Root, user.clone(), amount.clone())
	verify {
		assert_eq!(<<T as Config>::Currency as Currency<T::AccountId>>::total_balance(&user), amount);
	}

	remark {
		let l in 0 .. 64;
		let data: RemarkData = [1u8].repeat(l as usize).try_into().unwrap();
	}: _(RawOrigin::Root, data.clone())
	verify {
		let e: <T as Config>::RuntimeEvent = Event::Remarked(data).into();
		frame_system::Pallet::<T>::assert_last_event(e.into());
	}

	force_transfer {
		let user: T::AccountId = account("user", 0, SEED);
		let user2: T::AccountId = account("user", 1, SEED);
		let amount: T::Balance = 10000u32.into();
		LLM::<T>::transfer_from_treasury(user.clone(), amount.clone()).unwrap();
		LLM::<T>::set_courts(RawOrigin::Root.into(), vec![user.clone()].try_into().unwrap()).unwrap();
		let origin = RawOrigin::Signed(user.clone());
		LLM::<T>::politics_lock(origin.clone().into(), amount.clone()).unwrap();
		assert_eq!(LLMPolitics::<T>::get(&user), amount.clone());
	}: _(origin, LLMAccount::Locked(user.clone()), LLMAccount::Liquid(user2.clone()), amount.clone())
	verify {
		assert_eq!(LLMPolitics::<T>::get(&user), 0u8.into());
	}

	set_courts {
		let l in 1 .. T::MaxCourts::get();
		let mut courts: Vec<T::AccountId> = vec![];
		for i in 1..=l {
			courts.push(account("court", i, SEED));
		}
		let courts: BoundedVec<T::AccountId, T::MaxCourts> = courts.try_into().unwrap();
	}: _(RawOrigin::Root, courts.clone())
	verify {
		assert_eq!(Courts::<T>::get(), courts);
	}
}

impl_benchmark_test_suite!(LLM, crate::mock::new_test_ext(), crate::mock::Test,);
