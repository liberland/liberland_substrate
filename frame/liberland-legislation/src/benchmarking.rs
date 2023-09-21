/*
Copyright © 2023 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::{types::*, Pallet as Legislation};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use liberland_traits::LLInitializer;
use sp_runtime::{traits::ConstU32, BoundedVec};
use sp_std::prelude::*;
use LegislationTier::*;

const SEED: u32 = 0;
const ZERO_ID: LegislationId = LegislationId { year: 0u32, index: 0u32 };

fn add_vetos<T: Config>(c: u32) {
	for i in 0..c {
		let acc: T::AccountId = account("a", i, SEED);
		T::LLInitializer::make_test_citizen(&acc);
		let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
		Legislation::<T>::submit_veto(origin.clone(), Law, ZERO_ID).unwrap();
	}
}

benchmarks! {
	add_law {
		let s in 1 .. 65536;
		let data: BoundedVec<u8, ConstU32<65536>> = [1u8].repeat(s as usize).try_into().unwrap();
		let origin: T::RuntimeOrigin = RawOrigin::Root.into();
	}: _<T::RuntimeOrigin>(origin, Law, ZERO_ID, data)
	verify {
		assert_eq!(Legislation::<T>::laws(Law, ZERO_ID).len(), s as usize);
	}

	repeal_law {
		let data: BoundedVec<u8, ConstU32<65536>> = [1u8].repeat(10 as usize).try_into().unwrap();
		let origin: T::RuntimeOrigin = RawOrigin::Root.into();
		Legislation::<T>::add_law(origin.clone(), Law, ZERO_ID, data).unwrap();
	}: _<T::RuntimeOrigin>(origin, Law, ZERO_ID)
	verify {
		assert_eq!(Legislation::<T>::laws(Law, ZERO_ID).len(), 0);
	}

	submit_veto {
		let acc: T::AccountId = account("a", 0, SEED);
		T::LLInitializer::make_test_citizen(&acc);
		let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	}: _<T::RuntimeOrigin>(origin, Law, ZERO_ID)
	verify {
		assert_eq!(Legislation::<T>::vetos((Law, ZERO_ID, acc)), Some(true));
	}

	revert_veto {
		let acc: T::AccountId = account("a", 0, SEED);
		T::LLInitializer::make_test_citizen(&acc);
		let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
		Legislation::<T>::submit_veto(origin.clone(), Law, ZERO_ID).unwrap();
		assert_eq!(Legislation::<T>::vetos((Law, ZERO_ID, acc.clone())), Some(true));
	}: _<T::RuntimeOrigin>(origin, Law, ZERO_ID)
	verify {
		assert_eq!(Legislation::<T>::vetos((Law, ZERO_ID, acc)), None);
	}

	trigger_headcount_veto {
		let c in 16 .. 1000 => add_vetos::<T>(c);
		let acc: T::AccountId = account("a", 0, SEED);
		let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	}: _<T::RuntimeOrigin>(origin, Law, ZERO_ID)
	verify {
		assert_eq!(Legislation::<T>::vetos((Law, ZERO_ID, acc)), None);
	}
}

impl_benchmark_test_suite!(Legislation, crate::mock::new_test_ext(), crate::mock::Test,);
