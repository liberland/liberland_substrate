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
	}: _<T::RuntimeOrigin>(origin.into(), 0)
	verify {
		ensure!((JudgesSignatures::<T, I>::get(0).unwrap().len() as u32)  == T::MaxSignatures::get() , "Judges signatures len is not max");
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

		let mut accounts = vec![];

		let last_acc: T::AccountId = account("a", 1, 1);
		let mut parties = accounts.clone();
		parties.push(last_acc.clone());

		let parties: BoundedVec<T::AccountId, T::MaxSignatures> = parties.try_into().unwrap();
		let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value()/ 2u32.into());
		assert_ok!(ContractsRegistry::<T, I>::create_contract(origin.clone().into(), data.clone(), parties.clone()));

		for acc in accounts {
			let origin = RawOrigin::Signed(acc.clone());
			assert_ok!(ContractsRegistry::<T, I>::party_sign_contract(origin.clone().into(), 0));
		}

		let _ = T::Currency::make_free_balance_be(&last_acc, BalanceOf::<T, I>::max_value()/ 2u32.into());
		let origin = RawOrigin::Signed(last_acc.clone());
	}: _<T::RuntimeOrigin>(origin.into(), 0)
	verify {
		ensure!((PartiesSignatures::<T, I>::get(0).unwrap().len() as u32)  == T::MaxSignatures::get() , "Parties len is not max");
	}

	remove_judge {
		let origin =
			T::AddJudgeOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let user: T::AccountId = account("user", 0, SEED);
		assert_ok!(ContractsRegistry::<T, I>::add_judge(origin.clone(), user.clone()));
	}: _<T::RuntimeOrigin>(origin, user.clone())
	verify {
		ensure!(Judges::<T, I>::get(user), "Judge not removed");
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
