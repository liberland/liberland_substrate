use crate::{
	mock::*,
	Data,
	//	EntityRegistry, EntityRequests, Registrars
	Error,
};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::{BadOrigin, Hash};

type DataOf<T> = Data<<T as pallet_registry::Config>::MaxDataLength>;
type HashingOf<T> = <T as frame_system::Config>::Hashing;

#[test]
fn add_registrar_requires_correct_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(Registry::add_registrar(RuntimeOrigin::signed(0), 0), BadOrigin);
		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
	});
}

#[test]
fn add_registrar_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Registry::registrars().len(), 0);
		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_eq!(Registry::registrars().len(), 1);
		assert_eq!(Registry::registrars().get(0), Some(&0u64));

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 10));
		assert_eq!(Registry::registrars().len(), 2);
		assert_eq!(Registry::registrars().get(0), Some(&0u64));
		assert_eq!(Registry::registrars().get(1), Some(&10u64));
	});
}

#[test]
fn add_registrar_respects_max_registrars() {
	new_test_ext().execute_with(|| {
		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_noop!(
			Registry::add_registrar(RuntimeOrigin::root(), 0),
			Error::<Test>::TooManyRegistrars
		);
	});
}

#[test]
fn set_entity_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Registry::requests(0), None);

		let data1: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let data2: DataOf<Test> = vec![3, 2, 1].try_into().unwrap();

		assert_ok!(Registry::set_entity(RuntimeOrigin::signed(0), data1.clone()));
		assert_eq!(Registry::requests(0), Some((7u64, data1.clone())));

		assert_ok!(Registry::set_entity(RuntimeOrigin::signed(1), data2.clone()));
		assert_eq!(Registry::requests(0), Some((7u64, data1.clone())));
		assert_eq!(Registry::requests(1), Some((7u64, data2.clone())));

		assert_ok!(Registry::set_entity(RuntimeOrigin::signed(0), data2.clone()));
		assert_eq!(Registry::requests(0), Some((7u64, data2.clone())));
		assert_eq!(Registry::requests(1), Some((7u64, data2.clone())));
	});
}

#[test]
fn register_entity_works() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let entity = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::set_entity(entity.clone(), data.clone()));
		assert_eq!(Registry::registries(1, 0), None);
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(registrar, 0, 1, HashingOf::<Test>::hash_of(&data)));
		assert_eq!(Registry::registries(1, 0), Some((7u64, Some(data))));
	});
}

#[test]
fn register_entity_verifies_hash() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let data2: DataOf<Test> = vec![3, 2, 1].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let entity = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::set_entity(entity.clone(), data.clone()));
		assert_ok!(Registry::request_registration(entity, 0));
		assert_noop!(
			Registry::register_entity(registrar.clone(), 0, 1, HashingOf::<Test>::hash_of(&data2)),
			Error::<Test>::MismatchedData
		);
		assert_ok!(Registry::register_entity(registrar, 0, 1, HashingOf::<Test>::hash_of(&data)));
	});
}

#[test]
fn register_entity_verifies_origin() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar1 = RuntimeOrigin::signed(0);
		let registrar2 = RuntimeOrigin::signed(1);
		let entity = RuntimeOrigin::signed(2);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 1));
		assert_ok!(Registry::set_entity(entity.clone(), data.clone()));

		assert_noop!(
			Registry::register_entity(registrar1, 1, 2, HashingOf::<Test>::hash_of(&data)),
			Error::<Test>::InvalidRegistrar
		);

		assert_noop!(
			Registry::register_entity(entity, 1, 2, HashingOf::<Test>::hash_of(&data)),
			Error::<Test>::InvalidRegistrar
		);

		assert_noop!(
			Registry::register_entity(
				RuntimeOrigin::root(),
				1,
				2,
				HashingOf::<Test>::hash_of(&data)
			),
			BadOrigin
		);

		assert_noop!(
			Registry::register_entity(registrar2, 0, 2, HashingOf::<Test>::hash_of(&data)),
			Error::<Test>::InvalidRegistrar
		);
	});
}

#[test]
fn new_requests_dont_overwrite_registry() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let data2: DataOf<Test> = vec![3, 2, 1].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let entity = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::set_entity(entity.clone(), data.clone()));
		assert_eq!(Registry::registries(1, 0), None);
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(registrar, 0, 1, HashingOf::<Test>::hash_of(&data)));
		assert_eq!(Registry::registries(1, 0), Some((7u64, Some(data.clone()))));

		assert_ok!(Registry::set_entity(entity, data2.clone()));
		assert_eq!(Registry::requests(1), Some((7u64, data2)));
		assert_eq!(Registry::registries(1, 0), Some((7u64, Some(data))));
	});
}

#[test]
fn multiple_registries_dont_collide() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let data2: DataOf<Test> = vec![3, 2, 1].try_into().unwrap();
		let registrar1 = RuntimeOrigin::signed(0);
		let registrar2 = RuntimeOrigin::signed(1);
		let entity = RuntimeOrigin::signed(2);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 1));

		assert_ok!(Registry::set_entity(entity.clone(), data.clone()));
		assert_eq!(Registry::registries(2, 0), None);
		assert_eq!(Registry::registries(2, 1), None);

		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(registrar1, 0, 2, HashingOf::<Test>::hash_of(&data)));
		assert_eq!(Registry::registries(2, 0), Some((7u64, Some(data.clone()))));
		assert_eq!(Registry::registries(2, 1), None);

		assert_ok!(Registry::set_entity(entity.clone(), data2.clone()));
		assert_ok!(Registry::request_registration(entity.clone(), 1));
		assert_ok!(Registry::register_entity(registrar2, 1, 2, HashingOf::<Test>::hash_of(&data2)));
		assert_eq!(Registry::registries(2, 0), Some((7u64, Some(data))));
		assert_eq!(Registry::registries(2, 1), Some((7u64, Some(data2))));
	});
}

#[test]
fn clear_entity_wipes_only_request() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let entity = RuntimeOrigin::signed(2);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::set_entity(entity.clone(), data.clone()));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(registrar, 0, 2, HashingOf::<Test>::hash_of(&data)));

		assert_eq!(Registry::requests(2), Some((7u64, data.clone())));
		assert_eq!(Registry::registries(2, 0), Some((7u64, Some(data.clone()))));

		assert_ok!(Registry::clear_entity(entity));
		assert_eq!(Registry::requests(2), None);
		assert_eq!(Registry::registries(2, 0), Some((7u64, Some(data.clone()))));
	});
}


#[test]
fn unregister_wipes_only_single_registry_data() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar1 = RuntimeOrigin::signed(0);
		let registrar2 = RuntimeOrigin::signed(1);
		let entity = RuntimeOrigin::signed(2);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 1));

		assert_ok!(Registry::set_entity(entity.clone(), data.clone()));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::request_registration(entity.clone(), 1));
		assert_ok!(Registry::register_entity(registrar1, 0, 2, HashingOf::<Test>::hash_of(&data)));
		assert_ok!(Registry::register_entity(registrar2, 1, 2, HashingOf::<Test>::hash_of(&data)));

		assert_eq!(Registry::requests(2), Some((7u64, data.clone())));
		assert_eq!(Registry::registries(2, 0), Some((7u64, Some(data.clone()))));
		assert_eq!(Registry::registries(2, 1), Some((7u64, Some(data.clone()))));

		assert_ok!(Registry::unregister(entity.clone(), 0));
		assert_eq!(Registry::requests(2), Some((7u64, data.clone())));
		assert_eq!(Registry::registries(2, 0), None);
		assert_eq!(Registry::registries(2, 1), Some((7u64, Some(data.clone()))));

		assert_ok!(Registry::unregister(entity, 1));
		assert_eq!(Registry::registries(2, 1), None);
	});
}

#[test]
fn set_entity_fails_on_broke() {
	new_test_ext().execute_with(|| {
		let almost_broke = RuntimeOrigin::signed(3);
		let broke = RuntimeOrigin::signed(4);
		let empty: DataOf<Test> = vec![].try_into().unwrap();
		let non_empty: DataOf<Test> = vec![1].try_into().unwrap();

		assert_ok!(Registry::set_entity(almost_broke.clone(), empty.clone()));
		assert_noop!(Registry::set_entity(almost_broke, non_empty.clone()), pallet_balances::Error::<Test>::InsufficientBalance);
		assert_noop!(Registry::set_entity(broke.clone(), empty), pallet_balances::Error::<Test>::InsufficientBalance);
		assert_noop!(Registry::set_entity(broke, non_empty), pallet_balances::Error::<Test>::InsufficientBalance);
	})
}

#[test]
fn set_entity_reserves_correct_amounts() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let data3: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = RuntimeOrigin::signed(0);

		assert_ok!(Registry::set_entity(entity.clone(), data1.clone()));
		assert_eq!(Balances::reserved_balance(0), 3u64);
		assert_ok!(Registry::set_entity(entity.clone(), data3.clone()));
		assert_eq!(Balances::reserved_balance(0), 7u64);
		assert_ok!(Registry::set_entity(entity.clone(), data3.clone()));
		assert_eq!(Balances::reserved_balance(0), 7u64);
	})
}

#[test]
fn set_entity_frees_reserves_if_possible() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let data3: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = RuntimeOrigin::signed(0);

		assert_ok!(Registry::set_entity(entity.clone(), data3.clone()));
		assert_eq!(Balances::reserved_balance(0), 7u64);
		assert_ok!(Registry::set_entity(entity.clone(), data1.clone()));
		assert_eq!(Balances::reserved_balance(0), 3u64);
		assert_eq!(Balances::free_balance(0), 97u64);
		assert_ok!(Registry::set_entity(entity.clone(), data3.clone()));
		assert_eq!(Balances::reserved_balance(0), 7u64);
	})
}

#[test]
fn clear_entity_refunds_deposit() {
	new_test_ext().execute_with(|| {
		let data3: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = RuntimeOrigin::signed(0);

		assert_ok!(Registry::set_entity(entity.clone(), data3.clone()));
		assert_eq!(Balances::reserved_balance(0), 7u64);
		assert_ok!(Registry::clear_entity(entity.clone()));
		assert_eq!(Balances::reserved_balance(0), 0u64);
		assert_eq!(Balances::free_balance(0), 100u64);
		assert_ok!(Registry::set_entity(entity.clone(), data3.clone()));
		assert_eq!(Balances::reserved_balance(0), 7u64);
	})
}

#[test]
fn unregister_refunds_deposits() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar1 = RuntimeOrigin::signed(0);
		let registrar2 = RuntimeOrigin::signed(1);
		let entity = RuntimeOrigin::signed(2);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 1));

		assert_ok!(Registry::set_entity(entity.clone(), data.clone()));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::request_registration(entity.clone(), 1));
		assert_ok!(Registry::register_entity(registrar1, 0, 2, HashingOf::<Test>::hash_of(&data)));
		assert_ok!(Registry::register_entity(registrar2, 1, 2, HashingOf::<Test>::hash_of(&data)));
		assert_ok!(Registry::clear_entity(entity.clone()));

		assert_eq!(Balances::reserved_balance(2), 14u64);
		assert_ok!(Registry::unregister(entity.clone(), 0));
		assert_eq!(Balances::reserved_balance(2), 7u64);
		assert_eq!(Balances::free_balance(2), 93u64);
		assert_ok!(Registry::unregister(entity, 1));
		assert_eq!(Balances::reserved_balance(2), 0u64);
		assert_eq!(Balances::free_balance(2), 100u64);
	})
}

#[test]
fn request_registration_reserves_correct_amounts() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let data3: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = RuntimeOrigin::signed(0);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 1));

		assert_ok!(Registry::set_entity(entity.clone(), data1.clone()));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_eq!(Balances::reserved_balance(0), 3u64 + 3u64);

		assert_ok!(Registry::set_entity(entity.clone(), data3.clone()));
		assert_eq!(Balances::reserved_balance(0), 7u64 + 3u64);

		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_eq!(Balances::reserved_balance(0), 7u64 + 7u64);
	})
}

#[test]
fn clear_entity_fails_on_nonexistent_entity() {
	new_test_ext().execute_with(|| {
		let entity = RuntimeOrigin::signed(0);
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		assert_noop!(Registry::clear_entity(entity.clone()), Error::<Test>::InvalidEntity);
		assert_ok!(Registry::set_entity(entity.clone(), data1.clone()));
		assert_ok!(Registry::clear_entity(entity.clone()));
		assert_noop!(Registry::clear_entity(entity.clone()), Error::<Test>::InvalidEntity);
	})
}

#[test]
fn unregister_fails_on_not_registered_entity() {
	new_test_ext().execute_with(|| {
		let entity = RuntimeOrigin::signed(0);
		assert_noop!(Registry::unregister(entity.clone(), 0), Error::<Test>::InvalidEntity);
	})
}

#[test]
fn request_registration_fails_on_invalid_entity() {
	new_test_ext().execute_with(|| {
		let entity = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_noop!(Registry::request_registration(entity.clone(), 0), Error::<Test>::InvalidEntity);
	})
}

#[test]
fn register_entity_verifies_deposit_amount() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1,].try_into().unwrap();
		let data2: DataOf<Test> = vec![1, 2].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let entity = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::set_entity(entity.clone(), data1.clone()));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::set_entity(entity.clone(), data2.clone()));
		assert_noop!(Registry::register_entity(registrar.clone(), 0, 1, HashingOf::<Test>::hash_of(&data2)), Error::<Test>::InsufficientDeposit);
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(registrar.clone(), 0, 1, HashingOf::<Test>::hash_of(&data2)));
		assert_ok!(Registry::set_entity(entity.clone(), data1.clone()));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(registrar.clone(), 0, 1, HashingOf::<Test>::hash_of(&data1)));
	});
}

#[test]
fn refund_works_for_unregistered() {
	new_test_ext().execute_with(|| {
		let data2: DataOf<Test> = vec![1, 2].try_into().unwrap();
		let entity = RuntimeOrigin::signed(0);
		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 1));
		assert_ok!(Registry::set_entity(entity.clone(), data2.clone()));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_eq!(Balances::reserved_balance(0), 5u64 + 5u64);
		assert_ok!(Registry::refund(entity.clone(), 0));
		assert_eq!(Balances::reserved_balance(0), 5u64);
		assert_eq!(Balances::free_balance(0), 95u64);
	});
}

#[test]
fn refund_works_for_reduced_size() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let data2: DataOf<Test> = vec![1, 2].try_into().unwrap();
		let entity = RuntimeOrigin::signed(0);
		let registrar = RuntimeOrigin::signed(1);
		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 1));

		assert_ok!(Registry::set_entity(entity.clone(), data2.clone()));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(registrar.clone(), 0, 0, HashingOf::<Test>::hash_of(&data2)));
		assert_ok!(Registry::clear_entity(entity.clone()));
		assert_eq!(Balances::reserved_balance(0), 5u64);

		assert_ok!(Registry::set_entity(entity.clone(), data1.clone()));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(registrar.clone(), 0, 0, HashingOf::<Test>::hash_of(&data1)));
		assert_ok!(Registry::clear_entity(entity.clone()));
		assert_eq!(Balances::reserved_balance(0), 5u64);

		assert_ok!(Registry::refund(entity.clone(), 0));
		assert_eq!(Balances::reserved_balance(0), 3u64);
		assert_eq!(Balances::free_balance(0), 97u64);
	});
}