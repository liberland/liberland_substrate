/*
Copyright © 2023 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

#![cfg(feature = "runtime-benchmarks")]

use super::*;
#[allow(unused_imports)]
use crate::Pallet as CustomAccount;
use frame_benchmarking::{benchmarks_instance_pallet, impl_benchmark_test_suite};
use frame_support::traits::EnsureOrigin;
use sp_std::prelude::*;

benchmarks_instance_pallet! {
	execute {
		let origin: T::RuntimeOrigin = T::ExecuteOrigin::try_successful_origin().map_err(|_| "ExecuteOrigin without try_successful_origin!")?;
		let call: <T as frame_system::Config>::RuntimeCall = frame_system::Call::remark_with_event { remark: vec![] }.into();
	}: _<T::RuntimeOrigin>(origin, Box::new(call.into()))
}

impl_benchmark_test_suite!(CustomAccount, crate::mock::new_test_ext(), crate::mock::Test,);
