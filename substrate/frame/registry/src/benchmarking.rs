#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Registry;
use frame_benchmarking::{account, benchmarks_instance_pallet, impl_benchmark_test_suite};
use frame_support::pallet_prelude::DispatchResult;
use frame_system::RawOrigin;
use sp_core::Get;
use sp_runtime::traits::{Bounded, Hash, TrailingZeroInput};

use sp_std::vec::Vec;

const SEED: u32 = 0;

fn add_registries<T: Config<I>, I: 'static>(r: u32) -> DispatchResult {
	for i in 1..=r {
		let acc = account("registrar", i, SEED);
		Registry::<T, I>::add_registry(RawOrigin::Root.into(), acc)?;
	}
	Ok(())
}

fn get_data<T: Config<I>, I: 'static>(b: u8, s: usize) -> T::EntityData {
	let raw_data: Vec<u8> = [b].repeat((s - 2) as usize);
	let raw_data = raw_data.encode();
	Decode::decode(&mut TrailingZeroInput::new(raw_data.as_ref())).unwrap()
}

benchmarks_instance_pallet! {
  add_registry {
	let r in 1 .. T::MaxRegistrars::get() - 2 => add_registries::<T, I>(r)?;
	let acc = account("registrar", r, SEED);
	let registrars_before = Registry::<T, I>::registrars().len();
  }: _<T::RuntimeOrigin>(RawOrigin::Root.into(), acc)
  verify {
	assert_eq!(Registry::<T, I>::registrars().len(),  (registrars_before + 1) as usize);
  }

  request_entity {
	let s in 2 .. T::EntityData::max_encoded_len() as u32;
	let acc: T::AccountId = account("owner", 0, SEED);
	let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());
	let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	let data = get_data::<T, I>(1, s as usize);
  }: _<T::RuntimeOrigin>(origin, 0, data.clone(), false)
  verify {
	let entity_id: T::EntityId = 0u8.into();
	assert!(matches!(Registry::<T, I>::requests(0, entity_id), Some(Some(Request { data: ndata, .. })) if ndata == data));
  }

  request_registration {
	let s in 2 .. T::EntityData::max_encoded_len() as u32;
	let acc: T::AccountId = account("owner", 0, SEED);
	let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());
	let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	let old_data = get_data::<T, I>(1, s as usize);
	let data = get_data::<T, I>(2, s as usize);
	let entity_id: T::EntityId = 0u8.into();
	Registry::<T, I>::request_entity(origin.clone(), 0, data.clone(), false).unwrap();
  }: _<T::RuntimeOrigin>(origin, 0, entity_id.clone(), data.clone(), false)
  verify {
	assert!(matches!(Registry::<T, I>::requests(0, entity_id), Some(Some(Request { data: ndata, .. })) if ndata == data));
  }

  cancel_request {
	let acc: T::AccountId = account("owner", 0, SEED);
	let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());
	let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	let data = get_data::<T, I>(1, 100 as usize);
	Registry::<T, I>::request_entity(origin.clone(), 0, data.clone(), false).unwrap();
	let entity_id: T::EntityId = 0u8.into();
	assert!(matches!(Registry::<T, I>::requests(0, entity_id.clone()), Some(Some(Request { data: ndata, .. })) if ndata == data));
  }: _<T::RuntimeOrigin>(origin, 0, entity_id.clone())
  verify {
	assert_eq!(Registry::<T, I>::requests(0, entity_id), None);
  }

  /* see https://github.com/liberland/liberland_substrate/issues/250
  unregister {
	let acc: T::AccountId = account("entity", 0, SEED);
	let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());
	let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	let data = get_data::<T, I>(1, 100 as usize);
	Registry::<T, I>::request_entity(origin.clone(), data.clone(), false).unwrap();
	Registry::<T, I>::add_registry(RawOrigin::Root.into(), acc.clone()).unwrap();
	Registry::<T, I>::request_entity(origin.clone(), 0).unwrap();
	Registry::<T, I>::register_entity(origin.clone(), 0, acc.clone(), T::Hashing::hash_of(&data)).unwrap();
	assert!(
		matches!(
			Registry::<T, I>::registries(acc.clone(), 0),
			Some(Registration { data: ndata, .. }) if ndata == data
		)
	);
  }: _<T::RuntimeOrigin>(origin, 0)
  verify {
	assert_eq!(Registry::<T, I>::registries(acc, 0), None);
  }
  */

  unregister {
	let r in 1 .. T::MaxRegistrars::get() - 2 => add_registries::<T, I>(r)?;

	let registrar: T::RuntimeOrigin = RawOrigin::Signed(account("registrar", r, SEED)).into();
	let reg_idx: u32 = Registry::<T, I>::registrars().len() as u32 - 1;
	let acc: T::AccountId = account("owner", 0, SEED);
	let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());
	let data = get_data::<T, I>(1, 100 as usize);
	let entity_id: T::EntityId = 0u8.into();

	Registry::<T, I>::request_entity(origin.clone(), reg_idx, data.clone(), false).unwrap();
	Registry::<T, I>::register_entity(registrar.clone(), reg_idx, entity_id.clone(), T::Hashing::hash_of(&data)).unwrap();
	assert!(
		matches!(
			Registry::<T, I>::registries(reg_idx, entity_id.clone()),
			Some(Registration { data: ndata, .. }) if ndata == data
		)
	);
  }: _<T::RuntimeOrigin>(registrar, reg_idx, entity_id.clone(), false)
  verify {
	assert_eq!(Registry::<T, I>::registries(reg_idx, entity_id), None);
  }

  register_entity {
	let r in 1 .. T::MaxRegistrars::get() - 2 => add_registries::<T, I>(r)?;
	let s in 2 .. T::EntityData::max_encoded_len() as u32;

	let registrar: T::RuntimeOrigin = RawOrigin::Signed(account("registrar", r, SEED)).into();
	let reg_idx: u32 = Registry::<T, I>::registrars().len() as u32 - 1;
	let acc: T::AccountId = account("owner", 0, SEED);
	let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());
	let data = get_data::<T, I>(1, s as usize);
	let entity_id: T::EntityId = 0u8.into();

	Registry::<T, I>::request_entity(origin.clone(), reg_idx, data.clone(), false).unwrap();
	assert!(
		matches!(
			Registry::<T, I>::requests(reg_idx, entity_id.clone()),
			Some(_),
		)
	);
	assert!(
		matches!(
			Registry::<T, I>::registries(reg_idx, entity_id.clone()),
			None,
		)
	);
	let hash = T::Hashing::hash_of(&data);
  }: _<T::RuntimeOrigin>(registrar.clone(), reg_idx, entity_id.clone(), hash)
  verify {
	assert!(
		matches!(
			Registry::<T, I>::registries(reg_idx, entity_id),
			Some(Registration { data: ndata, .. }) if ndata == data
		)
	);
  }

  set_registered_entity {
	let r in 1 .. T::MaxRegistrars::get() - 2 => add_registries::<T, I>(r)?;
	let s in 2 .. T::EntityData::max_encoded_len() as u32;

	let registrar: T::RuntimeOrigin = RawOrigin::Signed(account("registrar", r, SEED)).into();
	let reg_idx: u32 = Registry::<T, I>::registrars().len() as u32 - 1;
	let acc: T::AccountId = account("owner", 0, SEED);
	let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());
	let data = get_data::<T, I>(1, s as usize);
	let reg_data = get_data::<T, I>(2, s as usize);
	let entity_id: T::EntityId = 0u8.into();

	Registry::<T, I>::request_entity(origin.clone(), reg_idx, data.clone(), true).unwrap();
	Registry::<T, I>::register_entity(registrar.clone(), reg_idx, entity_id.clone(), T::Hashing::hash_of(&data)).unwrap();
	assert!(matches!(
		Registry::<T, I>::registries(reg_idx, entity_id.clone()),
		Some(Registration { data: ndata, .. }) if ndata == data
	));
  }: _<T::RuntimeOrigin>(registrar.clone(), reg_idx, entity_id.clone(), reg_data.clone())
  verify {
	assert!(matches!(
		Registry::<T, I>::registries(reg_idx, entity_id),
		Some(Registration { data: ndata, .. }) if ndata == reg_data
	));
  }

  request_entity_unregister {
	let registrar: T::RuntimeOrigin = RawOrigin::Signed(account("registrar", 1, SEED)).into();
	add_registries::<T, I>(1)?;
	let reg_idx: u32 = Registry::<T, I>::registrars().len() as u32 - 1;
	let acc: T::AccountId = account("owner", 0, SEED);
	let origin: T::RuntimeOrigin = RawOrigin::Signed(acc.clone()).into();
	let _ = T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value() / 2u32.into());
	let entity_id: T::EntityId = 0u8.into();
	let data = get_data::<T, I>(1, 1000);

	Registry::<T, I>::request_entity(origin.clone(), reg_idx, data.clone(), true).unwrap();
	Registry::<T, I>::register_entity(registrar.clone(), reg_idx, entity_id.clone(), T::Hashing::hash_of(&data)).unwrap();
	assert!(matches!(
		Registry::<T, I>::registries(reg_idx, entity_id.clone()),
		Some(Registration { data: ndata, .. }) if ndata == data
	));
  }: _<T::RuntimeOrigin>(origin.clone(), reg_idx, entity_id.clone())
  verify {
	assert!(matches!(Registry::<T, I>::requests(reg_idx, entity_id), Some(None)));
  }
}

impl_benchmark_test_suite!(Registry, crate::mock::new_test_ext(), crate::mock::Test,);
