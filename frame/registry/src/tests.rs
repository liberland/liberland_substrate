#![cfg(test)]

use crate::{mock::*, Error, Event, Registration, Request, Requests};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::{BadOrigin, Hash};

type DataOf<T> = <T as pallet_registry::Config>::EntityData;
type HashingOf<T> = <T as frame_system::Config>::Hashing;

/* add_registry */
#[test]
fn add_registry_requires_correct_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(Registry::add_registry(RuntimeOrigin::signed(0), 0), BadOrigin);
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
	});
}

#[test]
fn add_registry_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Registry::registrars().len(), 0);
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_eq!(Registry::registrars().len(), 1);
		assert_eq!(Registry::registrars().get(0), Some(&0u64));

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 10));
		assert_eq!(Registry::registrars().len(), 2);
		assert_eq!(Registry::registrars().get(0), Some(&0u64));
		assert_eq!(Registry::registrars().get(1), Some(&10u64));
	});
}

#[test]
fn add_registry_respects_max_registrars() {
	new_test_ext().execute_with(|| {
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 1));
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 2));
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 3));
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 4));
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 5));
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 6));
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 7));
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 8));
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 9));
		assert_noop!(
			Registry::add_registry(RuntimeOrigin::root(), 10),
			Error::<Test>::TooManyRegistrars
		);
	});
}

#[test]
fn add_registry_deposits_event() {
	new_test_ext().execute_with(|| {
		let registry_index = 0;
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		System::assert_last_event(Event::<Test>::RegistryAdded { registry_index }.into());
	})
}

/* request_entity */
#[test]
fn request_entity_gives_sequential_ids() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let o = RuntimeOrigin::signed(0);

		for i in 0u32..5u32 {
			assert_ok!(Registry::request_entity(o.clone(), 0, data1.clone(), false));
			System::assert_last_event(
				Event::<Test>::RegistrationRequested { entity_id: i, registry_index: 0 }.into(),
			);
		}
		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(1), 0, data1.clone(), false));
		System::assert_last_event(
			Event::<Test>::RegistrationRequested { entity_id: 5, registry_index: 0 }.into(),
		);
	});
}

#[test]
fn request_entity_tracks_entity_owner() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(0), 0, data1.clone(), false));
		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(1), 0, data1.clone(), false));
		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(2), 0, data1.clone(), false));
		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(1), 0, data1.clone(), false));
		assert_eq!(Registry::entity_owner(0), Some(0));
		assert_eq!(Registry::entity_owner(1), Some(1));
		assert_eq!(Registry::entity_owner(2), Some(2));
		assert_eq!(Registry::entity_owner(3), Some(1));
	});
}

#[test]
fn request_entity_tracks_owners_entities() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(0), 0, data1.clone(), false));
		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(1), 0, data1.clone(), false));
		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(2), 0, data1.clone(), false));
		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(1), 0, data1.clone(), false));
		assert_eq!(Registry::owner_entities(0, 0), true);
		assert_eq!(Registry::owner_entities(1, 1), true);
		assert_eq!(Registry::owner_entities(2, 2), true);
		assert_eq!(Registry::owner_entities(1, 3), true);

		assert_eq!(Registry::owner_entities(0, 1), false);
	});
}

#[test]
fn request_entity_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Registry::requests(0, 0), None);

		let data1: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let data2: DataOf<Test> = vec![3, 2, 1].try_into().unwrap();

		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(0), 0, data1.clone(), false));
		assert_eq!(
			Registry::requests(0, 0),
			Some(Request { deposit: 9u64, data: data1.clone(), editable_by_registrar: false })
		);

		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(0), 0, data2.clone(), false));
		assert_eq!(
			Registry::requests(0, 0),
			Some(Request { deposit: 9u64, data: data1.clone(), editable_by_registrar: false })
		);
		assert_eq!(
			Registry::requests(0, 1),
			Some(Request { deposit: 9u64, data: data2.clone(), editable_by_registrar: false })
		);

		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(0), 0, data2.clone(), false));
		assert_eq!(
			Registry::requests(0, 0),
			Some(Request { deposit: 9u64, data: data1.clone(), editable_by_registrar: false })
		);
		assert_eq!(
			Registry::requests(0, 1),
			Some(Request { deposit: 9u64, data: data2.clone(), editable_by_registrar: false })
		);
		assert_eq!(
			Registry::requests(0, 2),
			Some(Request { deposit: 9u64, data: data2.clone(), editable_by_registrar: false })
		);
	});
}

#[test]
fn request_entity_fails_on_broke() {
	new_test_ext().execute_with(|| {
		let broke = RuntimeOrigin::signed(4);
		let empty: DataOf<Test> = vec![].try_into().unwrap();
		let non_empty: DataOf<Test> = vec![1].try_into().unwrap();

		assert_noop!(
			Registry::request_entity(broke.clone(), 0, empty, false),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
		assert_noop!(
			Registry::request_entity(broke, 0, non_empty, false),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
	})
}

#[test]
fn request_entity_reserves_correct_amounts() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let data3: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let owner = RuntimeOrigin::signed(0);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 1));

		assert_ok!(Registry::request_entity(owner.clone(), 0, data1.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 5u64);

		assert_ok!(Registry::request_entity(owner.clone(), 0, data3.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 5u64 + 9u64);

		assert_ok!(Registry::request_entity(owner.clone(), 0, data1.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 5u64 + 9u64 + 5u64);
	})
}

#[test]
fn request_entity_deposits_event() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let owner = 1;
		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(owner), 0, data1, false));
		System::assert_last_event(
			Event::<Test>::RegistrationRequested { entity_id: 0, registry_index: 0 }.into(),
		);
	})
}

/* request_registration */
#[test]
fn request_registration_frees_reserves_if_possible() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let data3: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let owner = RuntimeOrigin::signed(0);

		assert_ok!(Registry::request_entity(owner.clone(), 0, data3.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 9u64);
		assert_ok!(Registry::request_registration(owner.clone(), 0, 0, data1.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 5u64);
		assert_eq!(Balances::free_balance(0), 95u64);
		assert_ok!(Registry::request_registration(owner.clone(), 0, 0, data3.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 9u64);
	})
}

#[test]
fn request_registration_failes_for_nonexistent_entities() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let owner = RuntimeOrigin::signed(0);

		assert_noop!(
			Registry::request_registration(owner.clone(), 0, 0, data1.clone(), false),
			Error::<Test>::InvalidEntity
		);
	})
}

#[test]
fn request_registration_works_for_different_registries() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let data3: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let owner = RuntimeOrigin::signed(0);

		assert_ok!(Registry::request_entity(owner.clone(), 0, data3.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 9u64);
		assert_ok!(Registry::request_registration(owner.clone(), 1, 0, data1.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 9u64 + 5u64);
		assert_ok!(Registry::request_registration(owner.clone(), 1, 0, data3.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 9u64 + 9u64);
	})
}

#[test]
fn request_registration_deposits_event() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let owner = 1;
		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(owner), 0, data1.clone(), false));
		// make sure we change last event
		assert_ok!(Balances::transfer(RuntimeOrigin::signed(0), 1, 1));
		assert_ok!(Registry::request_registration(
			RuntimeOrigin::signed(owner),
			0,
			0,
			data1,
			false
		));
		System::assert_last_event(
			Event::<Test>::RegistrationRequested { entity_id: 0, registry_index: 0 }.into(),
		);
	})
}

#[test]
fn request_registration_verifies_origin() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(0), 0, data1.clone(), false));
		assert_noop!(
			Registry::request_registration(RuntimeOrigin::signed(1), 0, 0, data1.clone(), false),
			Error::<Test>::InvalidEntity
		);
		assert_ok!(Registry::request_registration(RuntimeOrigin::signed(0), 0, 0, data1, false));
	})
}

/* cancel_request */
#[test]
fn cancel_request_wipes_only_request() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let owner = RuntimeOrigin::signed(2);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::request_entity(owner.clone(), 0, data.clone(), false));
		assert_ok!(Registry::register_entity(registrar, 0, 0, HashingOf::<Test>::hash_of(&data)));
		assert_ok!(Registry::request_registration(owner.clone(), 0, 0, data.clone(), false));

		assert_eq!(
			Registry::requests(0, 0),
			Some(Request { deposit: 9u64, data: data.clone(), editable_by_registrar: false })
		);
		assert_eq!(
			Registry::registries(0, 0),
			Some(Registration { deposit: 9u64, data: data.clone(), editable_by_registrar: false })
		);

		assert_ok!(Registry::cancel_request(owner, 0, 0));
		assert_eq!(Registry::requests(0, 0), None);
		assert_eq!(
			Registry::registries(0, 0),
			Some(Registration { deposit: 9u64, data: data.clone(), editable_by_registrar: false })
		);
	});
}

#[test]
fn cancel_request_refunds_deposit() {
	new_test_ext().execute_with(|| {
		let data3: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let owner = RuntimeOrigin::signed(0);

		assert_ok!(Registry::request_entity(owner.clone(), 0, data3.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 9u64);
		assert_ok!(Registry::cancel_request(owner.clone(), 0, 0));
		assert_eq!(Balances::reserved_balance(0), 0u64);
		assert_eq!(Balances::free_balance(0), 100u64);
		assert_ok!(Registry::request_entity(owner.clone(), 0, data3.clone(), false));
		assert_eq!(Balances::reserved_balance(0), 9u64);
	})
}

#[test]
fn cancel_request_fails_on_nonexistent_entity() {
	new_test_ext().execute_with(|| {
		let owner = RuntimeOrigin::signed(0);
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		assert_noop!(Registry::cancel_request(owner.clone(), 0, 0), Error::<Test>::InvalidEntity);
		assert_ok!(Registry::request_entity(owner.clone(), 0, data1.clone(), false));
		assert_ok!(Registry::cancel_request(owner.clone(), 0, 0));
		assert_noop!(Registry::cancel_request(owner.clone(), 0, 0), Error::<Test>::InvalidEntity);
	})
}

#[test]
fn cancel_request_deposits_event() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let owner = 1;
		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(owner), 0, data1, false));
		assert_ok!(Registry::cancel_request(RuntimeOrigin::signed(owner), 0, 0));
		System::assert_last_event(
			Event::<Test>::RegistrationRequestCanceled { entity_id: 0, registry_index: 0 }.into(),
		);
	})
}

#[test]
fn cancel_request_verifies_origin() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		assert_ok!(Registry::request_entity(RuntimeOrigin::signed(0), 0, data1.clone(), false));
		assert_noop!(
			Registry::cancel_request(RuntimeOrigin::signed(1), 0, 0),
			Error::<Test>::InvalidEntity
		);
		assert_ok!(Registry::cancel_request(RuntimeOrigin::signed(0), 0, 0));
	})
}

/* unregister */
#[test]
fn unregister_verifies_origin() {
	new_test_ext().execute_with(|| {
		let registrar = RuntimeOrigin::signed(0);
		let owner = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_noop!(Registry::unregister(owner, 0, 0), Error::<Test>::InvalidRegistry);
		assert_noop!(Registry::unregister(registrar, 1, 0), Error::<Test>::InvalidRegistry);
	})
}

#[test]
fn unregister_fails_on_unregistered_entity() {
	new_test_ext().execute_with(|| {
		let registrar = RuntimeOrigin::signed(0);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_noop!(Registry::unregister(registrar, 0, 0), Error::<Test>::InvalidEntity);
	})
}

#[test]
fn unregister_works() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let owner = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::request_entity(owner.clone(), 0, data.clone(), false));
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			0,
			HashingOf::<Test>::hash_of(&data)
		));
		assert!(matches!(Registry::registries(0, 0), Some(_)));
		assert_ok!(Registry::unregister(registrar, 0, 0));
		assert_eq!(Registry::registries(0, 0), None);
	})
}

#[test]
fn unregister_refunds_deposit() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let owner = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::request_entity(owner.clone(), 0, data.clone(), false));
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			0,
			HashingOf::<Test>::hash_of(&data)
		));
		assert_eq!(Balances::reserved_balance(1), 5u64);
		assert_ok!(Registry::unregister(registrar, 0, 0));
		assert_eq!(Balances::reserved_balance(1), 0u64);
		assert_eq!(Balances::free_balance(1), 100u64);
	})
}

#[test]
fn unregister_deposits_event() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let owner = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::request_entity(owner.clone(), 0, data.clone(), false));
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			0,
			HashingOf::<Test>::hash_of(&data)
		));
		assert_ok!(Registry::unregister(registrar, 0, 0));
		System::assert_last_event(
			Event::<Test>::EntityUnregistered { entity_id: 0, registry_index: 0 }.into(),
		);
	})
}

/* register_entity */
#[test]
fn register_entity_works() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let owner = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_eq!(Registry::registries(0, 0), None);
		assert_ok!(Registry::request_entity(owner.clone(), 0, data.clone(), false));
		assert_ok!(Registry::register_entity(registrar, 0, 0, HashingOf::<Test>::hash_of(&data)));
		assert_eq!(Registry::requests(0, 0), None,);
		assert_eq!(
			Registry::registries(0, 0),
			Some(Registration { deposit: 9u64, data, editable_by_registrar: false })
		);
	});
}

#[test]
fn register_entity_verifies_hash() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let data2: DataOf<Test> = vec![3, 2, 1].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let owner = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::request_entity(owner.clone(), 0, data.clone(), false));
		assert_noop!(
			Registry::register_entity(registrar.clone(), 0, 0, HashingOf::<Test>::hash_of(&data2)),
			Error::<Test>::MismatchedData
		);
		assert_ok!(Registry::register_entity(registrar, 0, 0, HashingOf::<Test>::hash_of(&data)));
	});
}

#[test]
fn register_entity_verifies_origin() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar1 = RuntimeOrigin::signed(0);
		let registrar2 = RuntimeOrigin::signed(1);
		let owner = RuntimeOrigin::signed(2);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 1));
		assert_ok!(Registry::request_entity(owner.clone(), 0, data.clone(), false));
		assert_ok!(Registry::request_entity(owner.clone(), 1, data.clone(), false));

		assert_noop!(
			Registry::register_entity(registrar1, 1, 0, HashingOf::<Test>::hash_of(&data)),
			Error::<Test>::InvalidRegistry
		);

		assert_noop!(
			Registry::register_entity(owner, 1, 0, HashingOf::<Test>::hash_of(&data)),
			Error::<Test>::InvalidRegistry
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
			Error::<Test>::InvalidRegistry
		);
	});
}

#[test]
fn register_entity_verifies_deposit_amount() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));

		Requests::<Test>::insert(
			0,
			1,
			Request {
				deposit: 4u64, // should be 5 to match data
				data: data1.clone(),
				editable_by_registrar: false,
			},
		);
		assert_noop!(
			Registry::register_entity(registrar.clone(), 0, 1, HashingOf::<Test>::hash_of(&data1)),
			Error::<Test>::InsufficientDeposit
		);
	});
}

#[test]
fn register_entity_deposits_event() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let owner = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::request_entity(owner.clone(), 0, data.clone(), false));
		assert_ok!(Registry::register_entity(registrar, 0, 0, HashingOf::<Test>::hash_of(&data)));
		System::assert_last_event(
			Event::<Test>::EntityRegistered { entity_id: 0, registry_index: 0 }.into(),
		);
	})
}

/* set_registered_entity */
#[test]
fn set_registered_entity_fails_on_non_editable() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let owner = RuntimeOrigin::signed(1);
		let registrar = RuntimeOrigin::signed(0);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::request_entity(owner.clone(), 0, data.clone(), false));
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			0,
			HashingOf::<Test>::hash_of(&data)
		));
		assert_noop!(
			Registry::set_registered_entity(registrar, 0, 0, data),
			Error::<Test>::NotEditableByRegistrar
		);
	})
}

#[test]
fn set_registered_entity_fails_on_not_registered() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let owner = RuntimeOrigin::signed(1);
		let registrar = RuntimeOrigin::signed(0);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::request_entity(owner.clone(), 0, data.clone(), false));
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
		let owner = RuntimeOrigin::signed(1);
		let registrar = RuntimeOrigin::signed(0);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_noop!(
			Registry::set_registered_entity(registrar, 1, 0, data.clone()),
			Error::<Test>::InvalidRegistry
		);
		assert_noop!(
			Registry::set_registered_entity(owner, 0, 0, data),
			Error::<Test>::InvalidRegistry
		);
	})
}

#[test]
fn set_registered_entity_fails_on_bigger_data() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let data2: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let owner = RuntimeOrigin::signed(1);
		let registrar = RuntimeOrigin::signed(0);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::request_entity(owner.clone(), 0, data1.clone(), true));
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			0,
			HashingOf::<Test>::hash_of(&data1)
		));
		assert_noop!(
			Registry::set_registered_entity(registrar, 0, 0, data2),
			Error::<Test>::InsufficientDeposit
		);
	})
}

#[test]
fn set_registered_entity_works_with_same_size_and_smaller_data() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let data2: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let owner = RuntimeOrigin::signed(1);
		let registrar = RuntimeOrigin::signed(0);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::request_entity(owner.clone(), 0, data2.clone(), true));
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			0,
			HashingOf::<Test>::hash_of(&data2)
		));
		assert_ok!(Registry::set_registered_entity(registrar.clone(), 0, 0, data2));
		assert_ok!(Registry::set_registered_entity(registrar, 0, 0, data1));
	})
}

#[test]
fn set_registered_entity_deposits_event() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let data2: DataOf<Test> = vec![2, 1, 3].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let owner = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::request_entity(owner.clone(), 0, data.clone(), true));
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			0,
			HashingOf::<Test>::hash_of(&data)
		));
		assert_ok!(Registry::set_registered_entity(registrar, 0, 0, data2));
		System::assert_last_event(
			Event::<Test>::EntityRegistered { entity_id: 0, registry_index: 0 }.into(),
		);
	})
}

/* other */
#[test]
fn new_requests_dont_overwrite_registry() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let data2: DataOf<Test> = vec![3, 2, 1].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let owner = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::request_entity(owner.clone(), 0, data.clone(), false));
		assert_eq!(Registry::registries(0, 0), None);
		assert_ok!(Registry::register_entity(registrar, 0, 0, HashingOf::<Test>::hash_of(&data)));
		assert_eq!(
			Registry::registries(0, 0),
			Some(Registration { deposit: 9u64, data: data.clone(), editable_by_registrar: false })
		);

		assert_ok!(Registry::request_registration(owner, 0, 0, data2.clone(), false));
		assert_eq!(
			Registry::requests(0, 0),
			Some(Request { deposit: 9u64, data: data2, editable_by_registrar: false })
		);
		assert_eq!(
			Registry::registries(0, 0),
			Some(Registration { deposit: 9u64, data, editable_by_registrar: false })
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
		let owner = RuntimeOrigin::signed(2);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 1));

		assert_eq!(Registry::registries(0, 2), None);
		assert_eq!(Registry::registries(1, 2), None);

		assert_ok!(Registry::request_entity(owner.clone(), 0, data.clone(), false));
		assert_ok!(Registry::register_entity(registrar1, 0, 0, HashingOf::<Test>::hash_of(&data)));
		assert_eq!(
			Registry::registries(0, 0),
			Some(Registration { deposit: 9u64, data: data.clone(), editable_by_registrar: false })
		);
		assert_eq!(Registry::registries(1, 0), None);

		assert_ok!(Registry::request_registration(owner.clone(), 1, 0, data2.clone(), false));
		assert_ok!(Registry::register_entity(registrar2, 1, 0, HashingOf::<Test>::hash_of(&data2)));
		assert_eq!(
			Registry::registries(0, 0),
			Some(Registration { deposit: 9u64, data: data.clone(), editable_by_registrar: false })
		);
		assert_eq!(
			Registry::registries(1, 0),
			Some(Registration { deposit: 9u64, data: data2.clone(), editable_by_registrar: false })
		);
	});
}

#[test]
fn instances_dont_mix() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let owner = RuntimeOrigin::signed(1);
		let registrar = RuntimeOrigin::signed(0);

		assert_ok!(SecondRegistry::add_registry(RuntimeOrigin::root(), 0));
		assert_ok!(SecondRegistry::request_entity(owner.clone(), 0, data.clone(), false));
		assert_ok!(SecondRegistry::register_entity(
			registrar,
			0,
			0,
			HashingOf::<Test>::hash_of(&data)
		));

		assert_eq!(Registry::requests(0, 0), None);
		assert_eq!(Registry::registries(0, 0), None);
	})
}

#[test]
fn collective_can_be_registrar() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1].try_into().unwrap();
		let owner = RuntimeOrigin::signed(1);
		let collective_account_id = CollectiveAccountId::get();
		let registrar_origin: RuntimeOrigin =
			pallet_collective::Origin::<Test>::Members(1, 1).into();

		assert_ok!(RegistryWithCollectives::add_registry(
			RuntimeOrigin::root(),
			collective_account_id
		));
		assert_ok!(RegistryWithCollectives::request_entity(owner.clone(), 0, data.clone(), true));
		assert_ok!(RegistryWithCollectives::register_entity(
			registrar_origin.clone(),
			0,
			0,
			HashingOf::<Test>::hash_of(&data)
		));
		assert_ok!(RegistryWithCollectives::unregister(registrar_origin.clone(), 0, 0));
	})
}

#[test]
fn collective_can_have_entity() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1].try_into().unwrap();
		let collective_account_id = CollectiveAccountId::get();
		let owner_origin: RuntimeOrigin = pallet_collective::Origin::<Test>::Members(1, 1).into();

		assert_ok!(RegistryWithCollectives::request_entity(
			owner_origin.clone(),
			0,
			data.clone(),
			true
		));
		assert_eq!(
			RegistryWithCollectives::requests(0, 0),
			Some(Request { deposit: 5u64, data: data.clone(), editable_by_registrar: true })
		);
		assert_eq!(RegistryWithCollectives::entity_owner(0), Some(collective_account_id),);
		assert_ok!(RegistryWithCollectives::cancel_request(owner_origin.clone(), 0, 0));
	})
}

#[test]
fn whole_process_deposits_test() {
	new_test_ext().execute_with(|| {
		let data1: DataOf<Test> = vec![1].try_into().unwrap();
		let data3: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let owner = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));

		// initial, small data
		assert_ok!(Registry::request_entity(owner.clone(), 0, data1.clone(), false));
		assert_eq!(Balances::reserved_balance(1), 5u64);
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			0,
			HashingOf::<Test>::hash_of(&data1)
		));
		assert_eq!(Balances::reserved_balance(1), 5u64);

		// increase data, should increase reserves
		assert_ok!(Registry::request_registration(owner.clone(), 0, 0, data3.clone(), false));
		// 5 for registered data + 9 for registration
		assert_eq!(Balances::reserved_balance(1), 14u64);
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			0,
			HashingOf::<Test>::hash_of(&data3)
		));
		// 5 for old registration was refunded
		assert_eq!(Balances::reserved_balance(1), 9u64);

		// decrease data, should decrease reserves
		assert_ok!(Registry::request_registration(owner.clone(), 0, 0, data1.clone(), false));
		assert_eq!(Balances::reserved_balance(1), 14u64);
		assert_ok!(Registry::register_entity(
			registrar.clone(),
			0,
			0,
			HashingOf::<Test>::hash_of(&data1)
		));
		assert_eq!(Balances::reserved_balance(1), 5u64);

		// unregister, refund
		assert_ok!(Registry::unregister(registrar.clone(), 0, 0));
		assert_eq!(Balances::reserved_balance(1), 0u64);
	})
}

#[test]
fn genesis_build_works() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![0, 1, 2].try_into().unwrap();
		assert_eq!(GenesisTestRegistry::registrars(), vec![0, 1]);
		assert_eq!(GenesisTestRegistry::entity_owner(0), Some(999));
		assert_eq!(GenesisTestRegistry::owner_entities(999, 0), true);
		assert_eq!(
			GenesisTestRegistry::registries(0, 0),
			Some(Registration { deposit: 9u64, data, editable_by_registrar: false })
		);
		assert_eq!(Balances::reserved_balance(999), 9u64);
	})
}

/* see https://github.com/liberland/liberland_substrate/issues/250
#[test]
fn unregister_wipes_only_single_registry_data() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar1 = RuntimeOrigin::signed(0);
		let registrar2 = RuntimeOrigin::signed(1);
		let owner = RuntimeOrigin::signed(2);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 1));

		assert_ok!(Registry::request_entity(owner.clone(), data.clone(), false));
		assert_ok!(Registry::request_entity(owner.clone(), 0));
		assert_ok!(Registry::request_entity(owner.clone(), 1));
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

		assert_ok!(Registry::unregister(owner.clone(), 0));
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
fn unregister_refunds_deposits() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar1 = RuntimeOrigin::signed(0);
		let registrar2 = RuntimeOrigin::signed(1);
		let owner = RuntimeOrigin::signed(2);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 1));

		assert_ok!(Registry::request_entity(owner.clone(), data.clone(), false));
		assert_ok!(Registry::request_entity(owner.clone(), 0));
		assert_ok!(Registry::request_entity(owner.clone(), 1));
		assert_ok!(Registry::register_entity(registrar1, 0, 2, HashingOf::<Test>::hash_of(&data)));
		assert_ok!(Registry::register_entity(registrar2, 1, 2, HashingOf::<Test>::hash_of(&data)));
		assert_ok!(Registry::cancel_request(owner.clone()));

		assert_eq!(Balances::reserved_balance(2), 18u64);
		assert_ok!(Registry::unregister(owner.clone(), 0));
		assert_eq!(Balances::reserved_balance(2), 9u64);
		assert_eq!(Balances::free_balance(2), 91u64);
		assert_ok!(Registry::unregister(entity, 1));
		assert_eq!(Balances::reserved_balance(2), 0u64);
		assert_eq!(Balances::free_balance(2), 100u64);
	})
}

#[test]
fn unregister_deposits_event() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1, 2, 3].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let owner = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registry(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::request_entity(owner.clone(), data.clone(), false));
		assert_ok!(Registry::request_entity(owner.clone(), 0));
		assert_ok!(Registry::register_entity(registrar, 0, 1, HashingOf::<Test>::hash_of(&data)));
		assert_ok!(Registry::unregister(owner.clone(), 0));
		System::assert_last_event(
			Event::<Test>::EntityUnregistered { entity: 1, registry_index: 0 }.into(),
		);
	})
}

#[test]
fn unregister_fails_on_not_registered_entity() {
	new_test_ext().execute_with(|| {
		let owner = RuntimeOrigin::signed(0);
		assert_noop!(Registry::unregister(owner.clone(), 0), Error::<Test>::InvalidEntity);
	})
}
*/
