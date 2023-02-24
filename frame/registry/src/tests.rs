use crate::{mock::*, Error, Event, Registration, Request};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::{BadOrigin, Hash};

type DataOf<T> = <T as pallet_registry::Config>::EntityData;
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

		assert_ok!(Registry::set_entity(RuntimeOrigin::signed(0), data1.clone(), false));
		assert_eq!(
			Registry::requests(0),
			Some(Request { deposit: 9u64, data: data1.clone(), editable_by_registrar: false })
		);

		assert_ok!(Registry::set_entity(RuntimeOrigin::signed(1), data2.clone(), false));
		assert_eq!(
			Registry::requests(0),
			Some(Request { deposit: 9u64, data: data1.clone(), editable_by_registrar: false })
		);
		assert_eq!(
			Registry::requests(1),
			Some(Request { deposit: 9u64, data: data2.clone(), editable_by_registrar: false })
		);

		assert_ok!(Registry::set_entity(RuntimeOrigin::signed(0), data2.clone(), false));
		assert_eq!(
			Registry::requests(0),
			Some(Request { deposit: 9u64, data: data2.clone(), editable_by_registrar: false })
		);
		assert_eq!(
			Registry::requests(1),
			Some(Request { deposit: 9u64, data: data2.clone(), editable_by_registrar: false })
		);
	});
}

#[test]
fn register_entity_works() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let entity = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::set_entity(entity.clone(), data.clone(), false));
		assert_eq!(Registry::registries(1, 0), None);
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(registrar, 0, 1, HashingOf::<Test>::hash_of(&data)));
		assert_eq!(
			Registry::registries(1, 0),
			Some(Registration { deposit: 9u64, data: Some(data), editable_by_registrar: false })
		);
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

		assert_ok!(Registry::set_entity(entity.clone(), data.clone(), false));
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
		assert_ok!(Registry::set_entity(entity.clone(), data.clone(), false));

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

		assert_ok!(Registry::set_entity(entity.clone(), data.clone(), false));
		assert_eq!(Registry::registries(1, 0), None);
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(registrar, 0, 1, HashingOf::<Test>::hash_of(&data)));
		assert_eq!(
			Registry::registries(1, 0),
			Some(Registration {
				deposit: 9u64,
				data: Some(data.clone()),
				editable_by_registrar: false
			})
		);

		assert_ok!(Registry::set_entity(entity, data2.clone(), false));
		assert_eq!(
			Registry::requests(1),
			Some(Request { deposit: 9u64, data: data2, editable_by_registrar: false })
		);
		assert_eq!(
			Registry::registries(1, 0),
			Some(Registration { deposit: 9u64, data: Some(data), editable_by_registrar: false })
		);
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

		assert_ok!(Registry::set_entity(entity.clone(), data.clone(), false));
		assert_eq!(Registry::registries(2, 0), None);
		assert_eq!(Registry::registries(2, 1), None);

		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(registrar1, 0, 2, HashingOf::<Test>::hash_of(&data)));
		assert_eq!(
			Registry::registries(2, 0),
			Some(Registration {
				deposit: 9u64,
				data: Some(data.clone()),
				editable_by_registrar: false,
			})
		);
		assert_eq!(Registry::registries(2, 1), None);

		assert_ok!(Registry::set_entity(entity.clone(), data2.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 1));
		assert_ok!(Registry::register_entity(registrar2, 1, 2, HashingOf::<Test>::hash_of(&data2)));
		assert_eq!(
			Registry::registries(2, 0),
			Some(Registration {
				deposit: 9u64,
				data: Some(data.clone()),
				editable_by_registrar: false,
			})
		);
		assert_eq!(
			Registry::registries(2, 1),
			Some(Registration {
				deposit: 9u64,
				data: Some(data2.clone()),
				editable_by_registrar: false,
			})
		);
	});
}

#[test]
fn clear_entity_wipes_only_request() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let entity = RuntimeOrigin::signed(2);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::set_entity(entity.clone(), data.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(registrar, 0, 2, HashingOf::<Test>::hash_of(&data)));

		assert_eq!(
			Registry::requests(2),
			Some(Request { deposit: 9u64, data: data.clone(), editable_by_registrar: false })
		);
		assert_eq!(
			Registry::registries(2, 0),
			Some(Registration {
				deposit: 9u64,
				data: Some(data.clone()),
				editable_by_registrar: false,
			})
		);

		assert_ok!(Registry::clear_entity(entity));
		assert_eq!(Registry::requests(2), None);
		assert_eq!(
			Registry::registries(2, 0),
			Some(Registration {
				deposit: 9u64,
				data: Some(data.clone()),
				editable_by_registrar: false,
			})
		);
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

		assert_ok!(Registry::set_entity(entity.clone(), data.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::request_registration(entity.clone(), 1));
		assert_ok!(Registry::register_entity(registrar1, 0, 2, HashingOf::<Test>::hash_of(&data)));
		assert_ok!(Registry::register_entity(registrar2, 1, 2, HashingOf::<Test>::hash_of(&data)));

		assert_eq!(
			Registry::requests(2),
			Some(Request { deposit: 9u64, data: data.clone(), editable_by_registrar: false })
		);
		assert_eq!(
			Registry::registries(2, 0),
			Some(Registration {
				deposit: 9u64,
				data: Some(data.clone()),
				editable_by_registrar: false,
			})
		);
		assert_eq!(
			Registry::registries(2, 1),
			Some(Registration {
				deposit: 9u64,
				data: Some(data.clone()),
				editable_by_registrar: false,
			})
		);

		assert_ok!(Registry::unregister(entity.clone(), 0));
		assert_eq!(
			Registry::requests(2),
			Some(Request { deposit: 9u64, data: data.clone(), editable_by_registrar: false })
		);
		assert_eq!(Registry::registries(2, 0), None);
		assert_eq!(
			Registry::registries(2, 1),
			Some(Registration {
				deposit: 9u64,
				data: Some(data.clone()),
				editable_by_registrar: false,
			})
		);

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

		assert_ok!(Registry::set_entity(almost_broke.clone(), empty.clone(), false));
		assert_noop!(
			Registry::set_entity(almost_broke, non_empty.clone(), false),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
		assert_noop!(
			Registry::set_entity(broke.clone(), empty, false),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
		assert_noop!(
			Registry::set_entity(broke, non_empty, false),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
	})
}

#[test]
fn set_entity_reserves_correct_amounts() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let data3: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = RuntimeOrigin::signed(0);

		assert_ok!(Registry::set_entity(entity.clone(), data1.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 5u64);
		assert_ok!(Registry::set_entity(entity.clone(), data3.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 9u64);
		assert_ok!(Registry::set_entity(entity.clone(), data3.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 9u64);
	})
}

#[test]
fn set_entity_frees_reserves_if_possible() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let data3: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = RuntimeOrigin::signed(0);

		assert_ok!(Registry::set_entity(entity.clone(), data3.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 9u64);
		assert_ok!(Registry::set_entity(entity.clone(), data1.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 5u64);
		assert_eq!(Balances::free_balance(0), 95u64);
		assert_ok!(Registry::set_entity(entity.clone(), data3.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 9u64);
	})
}

#[test]
fn clear_entity_refunds_deposit() {
	new_test_ext().execute_with(|| {
		let data3: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = RuntimeOrigin::signed(0);

		assert_ok!(Registry::set_entity(entity.clone(), data3.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 9u64);
		assert_ok!(Registry::clear_entity(entity.clone()));
		assert_eq!(Balances::reserved_balance(0), 0u64);
		assert_eq!(Balances::free_balance(0), 100u64);
		assert_ok!(Registry::set_entity(entity.clone(), data3.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 9u64);
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

		assert_ok!(Registry::set_entity(entity.clone(), data.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::request_registration(entity.clone(), 1));
		assert_ok!(Registry::register_entity(registrar1, 0, 2, HashingOf::<Test>::hash_of(&data)));
		assert_ok!(Registry::register_entity(registrar2, 1, 2, HashingOf::<Test>::hash_of(&data)));
		assert_ok!(Registry::clear_entity(entity.clone()));

		assert_eq!(Balances::reserved_balance(2), 18u64);
		assert_ok!(Registry::unregister(entity.clone(), 0));
		assert_eq!(Balances::reserved_balance(2), 9u64);
		assert_eq!(Balances::free_balance(2), 91u64);
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

		assert_ok!(Registry::set_entity(entity.clone(), data1.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_eq!(Balances::reserved_balance(0), 5u64 + 5u64);

		assert_ok!(Registry::set_entity(entity.clone(), data3.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 9u64 + 5u64);

		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_eq!(Balances::reserved_balance(0), 9u64 + 9u64);
	})
}

#[test]
fn clear_entity_fails_on_nonexistent_entity() {
	new_test_ext().execute_with(|| {
		let entity = RuntimeOrigin::signed(0);
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		assert_noop!(Registry::clear_entity(entity.clone()), Error::<Test>::InvalidEntity);
		assert_ok!(Registry::set_entity(entity.clone(), data1.clone(), false));
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
		assert_noop!(
			Registry::request_registration(entity.clone(), 0),
			Error::<Test>::InvalidEntity
		);
	})
}

#[test]
fn register_entity_verifies_deposit_amount() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let data2: DataOf<Test> = vec![1, 2].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let entity = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::set_entity(entity.clone(), data1.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::set_entity(entity.clone(), data2.clone(), false));
		assert_noop!(
			Registry::register_entity(registrar.clone(), 0, 1, HashingOf::<Test>::hash_of(&data2)),
			Error::<Test>::InsufficientDeposit
		);
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			1,
			HashingOf::<Test>::hash_of(&data2)
		));
		assert_ok!(Registry::set_entity(entity.clone(), data1.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			1,
			HashingOf::<Test>::hash_of(&data1)
		));
	});
}

#[test]
fn refund_works_for_unregistered() {
	new_test_ext().execute_with(|| {
		let data2: DataOf<Test> = vec![1, 2].try_into().unwrap();
		let entity = RuntimeOrigin::signed(0);
		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 1));
		assert_ok!(Registry::set_entity(entity.clone(), data2.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_eq!(Balances::reserved_balance(0), 7u64 + 7u64);
		assert_ok!(Registry::refund(entity.clone(), 0));
		assert_eq!(Balances::reserved_balance(0), 7u64);
		assert_eq!(Balances::free_balance(0), 93u64);
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

		assert_ok!(Registry::set_entity(entity.clone(), data2.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			0,
			HashingOf::<Test>::hash_of(&data2)
		));
		assert_ok!(Registry::clear_entity(entity.clone()));
		assert_eq!(Balances::reserved_balance(0), 7u64);

		assert_ok!(Registry::set_entity(entity.clone(), data1.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			0,
			HashingOf::<Test>::hash_of(&data1)
		));
		assert_ok!(Registry::clear_entity(entity.clone()));
		assert_eq!(Balances::reserved_balance(0), 7u64);

		assert_ok!(Registry::refund(entity.clone(), 0));
		assert_eq!(Balances::reserved_balance(0), 5u64);
		assert_eq!(Balances::free_balance(0), 95u64);
	});
}

#[test]
fn add_registrar_deposits_event() {
	new_test_ext().execute_with(|| {
		let registrar_index = 0;
		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		System::assert_last_event(Event::<Test>::RegistrarAdded { registrar_index }.into());
	})
}

#[test]
fn set_entity_deposits_event() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = 1;
		assert_ok!(Registry::set_entity(RuntimeOrigin::signed(entity), data1, false));
		System::assert_last_event(Event::<Test>::EntitySet { entity }.into());
	})
}

#[test]
fn clear_entity_deposits_event() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = 1;
		assert_ok!(Registry::set_entity(RuntimeOrigin::signed(entity), data1, false));
		assert_ok!(Registry::clear_entity(RuntimeOrigin::signed(entity)));
		System::assert_last_event(Event::<Test>::EntityCleared { entity }.into());
	})
}

#[test]
fn unregister_deposits_event() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let entity = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::set_entity(entity.clone(), data.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(registrar, 0, 1, HashingOf::<Test>::hash_of(&data)));
		assert_ok!(Registry::unregister(entity.clone(), 0));
		System::assert_last_event(
			Event::<Test>::EntityUnregistered { entity: 1, registrar_index: 0 }.into(),
		);
	})
}

#[test]
fn request_registration_deposits_event() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::set_entity(entity.clone(), data.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		System::assert_last_event(
			Event::<Test>::RegistrationRequested { entity: 1, registrar_index: 0 }.into(),
		);
	})
}

#[test]
fn register_entity_deposits_event() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let entity = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::set_entity(entity.clone(), data.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(registrar, 0, 1, HashingOf::<Test>::hash_of(&data)));
		System::assert_last_event(
			Event::<Test>::EntityRegistered { entity: 1, registrar_index: 0 }.into(),
		);
	})
}

#[test]
fn refund_deposits_event() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::set_entity(entity.clone(), data.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::refund(entity.clone(), 0));
		System::assert_last_event(
			Event::<Test>::RefundProcessed { entity: 1, registrar_index: 0 }.into(),
		);
	})
}

#[test]
fn instances_dont_mix() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = RuntimeOrigin::signed(1);
		let registrar = RuntimeOrigin::signed(0);

		assert_ok!(SecondRegistry::add_registrar(RuntimeOrigin::root(), 0));
		assert_ok!(SecondRegistry::set_entity(entity.clone(), data.clone(), false));
		assert_ok!(SecondRegistry::request_registration(entity.clone(), 0));
		assert_ok!(SecondRegistry::register_entity(
			registrar,
			0,
			1,
			HashingOf::<Test>::hash_of(&data)
		));

		assert_eq!(Registry::requests(1), None);
		assert_eq!(Registry::registries(1, 0), None);
	})
}

#[test]
fn set_registered_entity_fails_on_non_editable() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = RuntimeOrigin::signed(1);
		let registrar = RuntimeOrigin::signed(0);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::set_entity(entity.clone(), data.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			1,
			HashingOf::<Test>::hash_of(&data)
		));
		assert_noop!(
			Registry::set_registered_entity(registrar, 0, 1, data),
			Error::<Test>::NotEditableByRegistrar
		);
	})
}

#[test]
fn set_registered_entity_fails_on_not_registered() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = RuntimeOrigin::signed(1);
		let registrar = RuntimeOrigin::signed(0);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::set_entity(entity.clone(), data.clone(), false));
		assert_noop!(
			Registry::set_registered_entity(registrar, 0, 1, data),
			Error::<Test>::InvalidEntity
		);
	})
}

#[test]
fn set_registered_entity_verifies_origin() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = RuntimeOrigin::signed(1);
		let registrar = RuntimeOrigin::signed(0);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_noop!(
			Registry::set_registered_entity(registrar, 1, 1, data.clone()),
			Error::<Test>::InvalidRegistrar
		);
		assert_noop!(
			Registry::set_registered_entity(entity, 0, 1, data),
			Error::<Test>::InvalidRegistrar
		);
	})
}

#[test]
fn set_registered_entity_fails_on_bigger_data() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let data2: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = RuntimeOrigin::signed(1);
		let registrar = RuntimeOrigin::signed(0);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::set_entity(entity.clone(), data1.clone(), true));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			1,
			HashingOf::<Test>::hash_of(&data1)
		));
		assert_noop!(
			Registry::set_registered_entity(registrar, 0, 1, data2),
			Error::<Test>::InsufficientDeposit
		);
	})
}

#[test]
fn set_registered_entity_works_with_same_size_and_smaller_data() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let data2: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let entity = RuntimeOrigin::signed(1);
		let registrar = RuntimeOrigin::signed(0);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::set_entity(entity.clone(), data2.clone(), true));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			1,
			HashingOf::<Test>::hash_of(&data2)
		));
		assert_ok!(Registry::set_registered_entity(registrar.clone(), 0, 1, data2));
		assert_ok!(Registry::set_registered_entity(registrar, 0, 1, data1));
	})
}

#[test]
fn collective_can_be_registrar() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1].try_into().unwrap();
		let entity = RuntimeOrigin::signed(1);
		let collective_account_id = CollectiveAccountId::get();
		let registrar_origin = pallet_collective::Origin::<Test>::Members(1, 1);

		assert_ok!(RegistryWithCollectives::add_registrar(
			RuntimeOrigin::root(),
			collective_account_id
		));
		assert_ok!(RegistryWithCollectives::set_entity(entity.clone(), data.clone(), true));
		assert_ok!(RegistryWithCollectives::request_registration(entity.clone(), 0));
		assert_ok!(RegistryWithCollectives::register_entity(
			registrar_origin.into(),
			0,
			1,
			HashingOf::<Test>::hash_of(&data)
		));
	})
}

#[test]
fn collective_can_be_entity() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1].try_into().unwrap();
		let collective_account_id = CollectiveAccountId::get();
		let entity_origin: RuntimeOrigin = pallet_collective::Origin::<Test>::Members(1, 1).into();

		assert_ok!(RegistryWithCollectives::set_entity(entity_origin.clone(), data.clone(), true));
		assert_eq!(
			RegistryWithCollectives::requests(collective_account_id),
			Some(Request { deposit: 5u64, data: data.clone(), editable_by_registrar: true })
		);
		assert_ok!(RegistryWithCollectives::request_registration(entity_origin.clone(), 0));
		assert_ok!(RegistryWithCollectives::refund(entity_origin.clone(), 0));
		assert_ok!(RegistryWithCollectives::clear_entity(entity_origin.clone()));
	})
}

#[test]
fn force_unregister_verifies_origin() {
	new_test_ext().execute_with(|| {
		let registrar = RuntimeOrigin::signed(0);
		let entity = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_noop!(Registry::force_unregister(entity, 0, 0), Error::<Test>::InvalidRegistrar);
		assert_noop!(Registry::force_unregister(registrar, 1, 0), Error::<Test>::InvalidRegistrar);
	})
}

#[test]
fn force_unregister_fails_on_unregistered_entity() {
	new_test_ext().execute_with(|| {
		let registrar = RuntimeOrigin::signed(0);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_noop!(Registry::force_unregister(registrar, 0, 0), Error::<Test>::InvalidEntity);
	})
}

#[test]
fn force_unregister_works() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let entity = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::set_entity(entity.clone(), data.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			1,
			HashingOf::<Test>::hash_of(&data)
		));
		assert!(matches!(Registry::registries(1, 0), Some(_)));
		assert_ok!(Registry::force_unregister(registrar, 0, 1));
		assert_eq!(Registry::registries(1, 0), None);
	})
}

#[test]
fn force_unregister_refunds_deposit() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let entity = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::set_entity(entity.clone(), data.clone(), false));
		assert_ok!(Registry::request_registration(entity.clone(), 0));
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			1,
			HashingOf::<Test>::hash_of(&data)
		));
		assert_ok!(Registry::clear_entity(entity.clone()));
		assert_eq!(Balances::reserved_balance(1), 5u64);
		assert_ok!(Registry::force_unregister(registrar, 0, 1));
		assert_eq!(Balances::reserved_balance(1), 0u64);
		assert_eq!(Balances::free_balance(1), 100u64);
	})
}
