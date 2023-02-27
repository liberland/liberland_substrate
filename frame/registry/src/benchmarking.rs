#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Registry;
use frame_benchmarking::{account, benchmarks_instance_pallet, impl_benchmark_test_suite};
use frame_support::pallet_prelude::{ConstU32, DispatchResult};
use frame_system::RawOrigin;
use sp_core::Get;
use sp_runtime::traits::{Bounded, Hash, TrailingZeroInput};

const SEED: u32 = 0;

fn add_registrars<T: Config<I>, I: 'static>(r: u32) -> DispatchResult {
	for i in 1..=r {
		let acc = account("registrar", i, SEED);
		Registry::<T, I>::add_registrar(RawOrigin::Root.into(), acc)?;
	}
	Ok(())
}

fn get_data<T: Config<I>, I: 'static>(b: u8, s: usize) -> T::EntityData {
	let raw_data: BoundedVec<u8, ConstU32<1024>> = [b].repeat((s - 2) as usize).try_into().unwrap();
	let raw_data = raw_data.encode();
	Decode::decode(&mut TrailingZeroInput::new(raw_data.as_ref())).unwrap()
}

benchmarks_instance_pallet! {
  add_registrar {
	let r in 1 .. T::MaxRegistrars::get() - 1 => add_registrars::<T, I>(r)?;
	let acc = account("registrar", r, SEED);
  }: _<T::RuntimeOrigin>(RawOrigin::Root.into(), acc)
  verify {
	assert_eq!(Registry::<T, I>::registrars().len(),  (r + 1) as usize);
  }

  set_entity {
	let s in 2..T::EntityData::max_encoded_len() as u32;
	let acc: T::AccountId = account("entity", 0, SEED);
	let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());
	let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	let data = get_data::<T, I>(1, s as usize);
  }: _<T::RuntimeOrigin>(origin, data.clone(), false)
  verify {
	assert!(matches!(Registry::<T, I>::requests(acc), Some(Request { data: ndata, .. }) if ndata == data));
  }

  clear_entity {
	let acc: T::AccountId = account("entity", 0, SEED);
	let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());
	let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	let data = get_data::<T, I>(1, 100 as usize);
	Registry::<T, I>::set_entity(origin.clone(), data.clone(), false).unwrap();
	assert!(matches!(Registry::<T, I>::requests(acc.clone()), Some(Request { data: ndata, .. }) if ndata == data));
  }: _<T::RuntimeOrigin>(origin)
  verify {
	assert_eq!(Registry::<T, I>::requests(acc), None);
  }

  unregister {
	let acc: T::AccountId = account("entity", 0, SEED);
	let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());
	let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	let data = get_data::<T, I>(1, 100 as usize);
	Registry::<T, I>::set_entity(origin.clone(), data.clone(), false).unwrap();
	Registry::<T, I>::add_registrar(RawOrigin::Root.into(), acc.clone()).unwrap();
	Registry::<T, I>::request_registration(origin.clone(), 0).unwrap();
	Registry::<T, I>::register_entity(origin.clone(), 0, acc.clone(), T::Hashing::hash_of(&data)).unwrap();
	assert!(
		matches!(
			Registry::<T, I>::registries(acc.clone(), 0),
			Some(Registration { data: ndata, .. }) if ndata == Some(data)
		)
	);
  }: _<T::RuntimeOrigin>(origin, 0)
  verify {
	assert_eq!(Registry::<T, I>::registries(acc, 0), None);
  }

  force_unregister {
	let r in 1 .. T::MaxRegistrars::get() => add_registrars::<T, I>(r)?;

	let registrar: T::RuntimeOrigin = RawOrigin::Signed(account("registrar", r, SEED)).into();
	let reg_idx = r - 1;
	let acc: T::AccountId = account("entity", 0, SEED);
	let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());
	let data = get_data::<T, I>(1, 100 as usize);

	Registry::<T, I>::set_entity(origin.clone(), data.clone(), false).unwrap();
	Registry::<T, I>::request_registration(origin.clone(), reg_idx).unwrap();
	Registry::<T, I>::register_entity(registrar.clone(), reg_idx, acc.clone(), T::Hashing::hash_of(&data)).unwrap();
	assert!(
		matches!(
			Registry::<T, I>::registries(acc.clone(), reg_idx),
			Some(Registration { data: ndata, .. }) if ndata == Some(data)
		)
	);
  }: _<T::RuntimeOrigin>(registrar, reg_idx, acc.clone())
  verify {
	assert_eq!(Registry::<T, I>::registries(acc, reg_idx), None);
  }

  request_registration {
	let s in 2..T::EntityData::max_encoded_len() as u32;

	add_registrars::<T, I>(1)?;
	let registrar: T::RuntimeOrigin = RawOrigin::Signed(account("registrar", 1, SEED)).into();
	let reg_idx = 0;
	let acc: T::AccountId = account("entity", 0, SEED);
	let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());
	let data = get_data::<T, I>(1, s as usize);

	Registry::<T, I>::set_entity(origin.clone(), data.clone(), false).unwrap();
  }: _<T::RuntimeOrigin>(origin, reg_idx)
  verify {
	assert!(matches!(Registry::<T, I>::registries(acc, reg_idx), Some(_)));
  }

  register_entity {
	let r in 1 .. T::MaxRegistrars::get() => add_registrars::<T, I>(r)?;
	let s in 2..T::EntityData::max_encoded_len() as u32;

	let registrar: T::RuntimeOrigin = RawOrigin::Signed(account("registrar", r, SEED)).into();
	let reg_idx = r - 1;
	let acc: T::AccountId = account("entity", 0, SEED);
	let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());
	let data = get_data::<T, I>(1, s as usize);

	Registry::<T, I>::set_entity(origin.clone(), data.clone(), false).unwrap();
	Registry::<T, I>::request_registration(origin.clone(), reg_idx).unwrap();
	assert!(
		matches!(
			Registry::<T, I>::registries(acc.clone(), reg_idx),
			Some(_)
		)
	);
	let hash = T::Hashing::hash_of(&data);
  }: _<T::RuntimeOrigin>(registrar.clone(), reg_idx, acc.clone(), hash)
  verify {
	assert!(
		matches!(
			Registry::<T, I>::registries(acc.clone(), reg_idx),
			Some(Registration { data: ndata, .. }) if ndata == Some(data)
		)
	);
  }

  refund {
	let s in 2..T::EntityData::max_encoded_len() as u32;

	add_registrars::<T, I>(1)?;
	let registrar: T::RuntimeOrigin = RawOrigin::Signed(account("registrar", 1, SEED)).into();
	let reg_idx = 0;
	let acc: T::AccountId = account("entity", 0, SEED);
	let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());
	let data = get_data::<T, I>(1, s as usize);

	Registry::<T, I>::set_entity(origin.clone(), data.clone(), false).unwrap();
	Registry::<T, I>::request_registration(origin.clone(), reg_idx).unwrap();
	assert!(
		matches!(
			Registry::<T, I>::registries(acc.clone(), reg_idx),
			Some(_)
		)
	);
  }: _<T::RuntimeOrigin>(origin.clone(), reg_idx)
  verify {
	assert!(matches!(
		Registry::<T, I>::registries(acc.clone(), reg_idx),
		Some(Registration { deposit, .. }) if deposit == 0u8.into()
	));
  }

  set_registered_entity {
	let r in 1 .. T::MaxRegistrars::get() => add_registrars::<T, I>(r)?;
	let s in 2..T::EntityData::max_encoded_len() as u32;

	let registrar: T::RuntimeOrigin = RawOrigin::Signed(account("registrar", r, SEED)).into();
	let reg_idx = r - 1;
	let acc: T::AccountId = account("entity", 0, SEED);
	let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());
	let data = get_data::<T, I>(1, s as usize);
	let reg_data = get_data::<T, I>(2, s as usize);

	Registry::<T, I>::set_entity(origin.clone(), data.clone(), true).unwrap();
	Registry::<T, I>::request_registration(origin.clone(), reg_idx).unwrap();
	Registry::<T, I>::register_entity(registrar.clone(), reg_idx, acc.clone(), T::Hashing::hash_of(&data)).unwrap();
	assert!(matches!(
		Registry::<T, I>::registries(acc.clone(), reg_idx),
		Some(Registration { data: ndata, .. }) if ndata == Some(data)
	));
  }: _<T::RuntimeOrigin>(registrar.clone(), reg_idx, acc.clone(), reg_data.clone())
  verify {
	assert!(matches!(
		Registry::<T, I>::registries(acc.clone(), reg_idx),
		Some(Registration { data: ndata, .. }) if ndata == Some(reg_data)
	));
  }
}

impl_benchmark_test_suite!(Registry, crate::mock::new_test_ext(), crate::mock::Test,);
