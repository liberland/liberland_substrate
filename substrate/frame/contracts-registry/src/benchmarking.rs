#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::v1::{account, BenchmarkError};
use frame_benchmarking::{benchmarks_instance_pallet, impl_benchmark_test_suite};
use frame_support::{
	assert_ok, ensure,
	traits::{EnsureOrigin, Get},
};
use frame_system::RawOrigin;
use sp_runtime::BoundedVec;
use sp_std::vec;

use sp_runtime::traits::Bounded;

use crate::Pallet as ContractsRegistry;

const SEED: u32 = 0;

fn get_data<T: Config<I>, I: 'static>(b: u8, s: usize) -> BoundedVec<u8, T::MaxContractContentLen> {
	vec![b; s].try_into().unwrap()
}

benchmarks_instance_pallet! {
	add_judge {
		let origin =
			T::AddJudgeOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let user: T::AccountId = account("user", 0, SEED);
		assert_ok!(ContractsRegistry::<T, I>::add_judge(origin.clone(), user.clone()));
	}: _<T::RuntimeOrigin>(origin, user.clone())
	verify {
		ensure!(Judges::<T, I>::get(user), "Judges len is not max");
	}

	judge_sign_contract {
		let acc: T::AccountId = account("a", 1, SEED);
		let origin = RawOrigin::Signed(acc.clone());
		let data = get_data::<T, I>(1, 1 as usize);
		let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());

		let parties: BoundedVec<T::AccountId, T::MaxSignatures> = vec![].try_into().unwrap();
		assert_ok!(ContractsRegistry::<T, I>::create_contract(origin.clone().into(), data.clone(), parties.clone()));

		let acc: T::AccountId = account("a", 1, 1);
		let origin = RawOrigin::Signed(acc.clone());
		assert_ok!(ContractsRegistry::<T, I>::add_judge(RawOrigin::Root.into(), acc));
		for i in 1..T::MaxSignatures::get() {
			let j: T::AccountId = account("judge", i, 1);
			let jo = RawOrigin::Signed(j.clone()).into();
			assert_ok!(ContractsRegistry::<T, I>::add_judge(RawOrigin::Root.into(), j));
			assert_ok!(ContractsRegistry::<T, I>::judge_sign_contract(jo, 0));
		}

	}: _<T::RuntimeOrigin>(origin.into(), 0)
	verify {
		ensure!((JudgesSignatures::<T, I>::get(0).unwrap().len() as u32)  == T::MaxSignatures::get() , "Judges signatures length mismatch");
	}

	create_contract {
		let s in 0 .. T::MaxContractContentLen::get() - 1;

		let acc: T::AccountId = account("a", 1, SEED);
		let origin = RawOrigin::Signed(acc.clone());

		let data = get_data::<T, I>(1, s as usize);
		let parties: BoundedVec<T::AccountId, T::MaxSignatures> = vec![acc.clone(); T::MaxSignatures::get() as usize].try_into().unwrap();
		let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value()/ 2u32.into());
	}: _<T::RuntimeOrigin>(origin.into(), data, parties)
	verify {
		Contracts::<T, I>::get(0).unwrap()
	}

	party_sign_contract {
		let acc: T::AccountId = account("a", 1, SEED);
		let origin = RawOrigin::Signed(acc.clone());

		let data = get_data::<T, I>(1, 3 as usize);

		let mut parties = vec![];
		for i in 0..(T::MaxSignatures::get()-1) {
			let signatory = account("sig", i, 1);
			parties.push(signatory);
		}

		let last_acc: T::AccountId = account("a", 1, 1);
		parties.push(last_acc.clone());

		let parties: BoundedVec<T::AccountId, T::MaxSignatures> = parties.try_into().unwrap();
		let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value()/ 2u32.into());
		assert_ok!(ContractsRegistry::<T, I>::create_contract(origin.clone().into(), data.clone(), parties.clone()));

		for i in 0..(T::MaxSignatures::get()-1) {
			let signatory_origin = RawOrigin::Signed(parties[i as usize].clone()).into();
			assert_ok!(ContractsRegistry::<T, I>::party_sign_contract(signatory_origin, 0));
		}

		let _ = T::Currency::make_free_balance_be(&last_acc, BalanceOf::<T, I>::max_value()/ 2u32.into());
		let origin = RawOrigin::Signed(last_acc.clone());
	}: _<T::RuntimeOrigin>(origin.into(), 0)
	verify {
		ensure!((PartiesSignatures::<T, I>::get(0).unwrap().len() as u32)  == T::MaxSignatures::get(), "Parties signatures length mismatch");
	}

	remove_judge {
		let origin =
			T::AddJudgeOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let user: T::AccountId = account("user", 0, SEED);
		assert_ok!(ContractsRegistry::<T, I>::add_judge(origin.clone(), user.clone()));
	}: _<T::RuntimeOrigin>(origin, user.clone())
	verify {
		ensure!(!Judges::<T, I>::get(user), "Judge not removed");
	}

	remove_contract {
		let acc: T::AccountId = account("a", 1, SEED);
		let origin = RawOrigin::Signed(acc.clone());

		let data = get_data::<T, I>(1, 2 as usize);
		let parties: BoundedVec<T::AccountId, T::MaxSignatures> = vec![acc.clone(); T::MaxSignatures::get() as usize].try_into().unwrap();
		let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value()/ 2u32.into());
		assert_ok!(ContractsRegistry::<T, I>::create_contract(origin.clone().into(), data, parties));
		assert!(Contracts::<T, I>::get(0).is_some());
	}: _<T::RuntimeOrigin>(origin.into(), 0)
	verify {
		assert!(Contracts::<T, I>::get(0).is_none());
	}
}

impl_benchmark_test_suite!(ContractsRegistry, crate::mock::new_test_ext(), crate::mock::Test,);
