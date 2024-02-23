/*
Copyright © 2023 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Office;
use frame_benchmarking::{account, benchmarks_instance_pallet, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use sp_std::prelude::*;

const SEED: u32 = 0;

benchmarks_instance_pallet! {
	set_admin {
		let old_admin: T::AccountId = account("admin", 0, SEED);
		Office::<T, I>::set_admin(RawOrigin::Root.into(), old_admin.clone()).unwrap();
		let origin = RawOrigin::Signed(old_admin).into();
		let new_admin: T::AccountId = account("admin", 1, SEED);
	}: _<T::RuntimeOrigin>(origin, new_admin.clone())
	verify {
		assert_eq!(Office::<T, I>::admin(), Some(new_admin));
	}

	set_clerk {
		let old_admin: T::AccountId = account("admin", 0, SEED);
		Office::<T, I>::set_admin(RawOrigin::Root.into(), old_admin.clone()).unwrap();
		let origin = RawOrigin::Signed(old_admin).into();
		let new_clerk: T::AccountId = account("clerk", 0, SEED);
	}: _<T::RuntimeOrigin>(origin, new_clerk.clone(), Default::default())
	verify {
		assert_eq!(Office::<T, I>::clerks(new_clerk), Some(Default::default()));
	}

	remove_clerk {
		let old_admin: T::AccountId = account("admin", 0, SEED);
		Office::<T, I>::set_admin(RawOrigin::Root.into(), old_admin.clone()).unwrap();
		let origin: T::RuntimeOrigin = RawOrigin::Signed(old_admin).into();
		let new_clerk: T::AccountId = account("clerk", 0, SEED);
		Office::<T, I>::set_clerk(origin.clone(), new_clerk.clone(), Default::default()).unwrap();
		assert_eq!(Office::<T, I>::clerks(new_clerk.clone()), Some(Default::default()));
	}: _<T::RuntimeOrigin>(origin, new_clerk.clone())
	verify {
		assert_eq!(Office::<T, I>::clerks(new_clerk), None);
	}

	execute {
		let old_admin: T::AccountId = account("admin", 0, SEED);
		Office::<T, I>::set_admin(RawOrigin::Root.into(), old_admin.clone()).unwrap();
		let origin: T::RuntimeOrigin = RawOrigin::Signed(old_admin).into();
		let new_clerk: T::AccountId = account("clerk", 0, SEED);
		Office::<T, I>::set_clerk(origin.clone(), new_clerk.clone(), Default::default()).unwrap();

		let origin: T::RuntimeOrigin = RawOrigin::Signed(new_clerk.clone()).into();
		let call: <T as frame_system::Config>::RuntimeCall = frame_system::Call::remark { remark: vec![] }.into();
	}: _<T::RuntimeOrigin>(origin, Box::new(call.into()))
}

impl_benchmark_test_suite!(Office, crate::mock::new_test_ext(), crate::mock::Test,);
