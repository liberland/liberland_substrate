use crate::{
	mock::*,
	Error, Data,
//	EntityRegistry, EntityRequests, Registrars
};
use frame_support::{assert_noop, assert_ok,};
use sp_runtime::traits::BadOrigin;
use sp_runtime::traits::Hash;

type DataOf<T> = Data<<T as pallet_registry::Config>::MaxDataLength>;
type HashingOf<T> = <T as frame_system::Config>::Hashing;

#[test]
fn add_registrar_requires_correct_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Registry::add_registrar(RuntimeOrigin::signed(0), 0),
			BadOrigin
		);
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
		assert_noop!(Registry::add_registrar(RuntimeOrigin::root(), 0), Error::<Test>::TooManyRegistrars);
	});
}

#[test]
fn set_entity_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Registry::requests(0), None);

		let data1: DataOf<Test> = vec![1,2,3].try_into().unwrap();
		let data2: DataOf<Test> = vec![3,2,1].try_into().unwrap();

		assert_ok!(Registry::set_entity(RuntimeOrigin::signed(0), data1.clone()));
		assert_eq!(Registry::requests(0), Some(data1.clone()));

		assert_ok!(Registry::set_entity(RuntimeOrigin::signed(1), data2.clone()));
		assert_eq!(Registry::requests(0), Some(data1.clone()));
		assert_eq!(Registry::requests(1), Some(data2.clone()));

		assert_ok!(Registry::set_entity(RuntimeOrigin::signed(0), data2.clone()));
		assert_eq!(Registry::requests(0), Some(data2.clone()));
		assert_eq!(Registry::requests(1), Some(data2.clone()));
	});
}

#[test]
fn register_entity_works() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1,2,3].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let entity = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::set_entity(entity, data.clone()));
		assert_eq!(Registry::registries(1, 0), None);
		assert_ok!(Registry::register_entity(registrar, 0, 1, HashingOf::<Test>::hash_of(&data)));
		assert_eq!(Registry::registries(1, 0), Some(data));
	});
}

#[test]
fn register_entity_verifies_hash() {
    new_test_ext().execute_with(|| {
        let data: DataOf<Test> = vec![1,2,3].try_into().unwrap();
        let data2: DataOf<Test> = vec![3,2,1].try_into().unwrap();
        let registrar = RuntimeOrigin::signed(0);
        let entity = RuntimeOrigin::signed(1);

        assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));

        assert_ok!(Registry::set_entity(entity, data.clone()));
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
        let data: DataOf<Test> = vec![1,2,3].try_into().unwrap();
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
			Registry::register_entity(RuntimeOrigin::root(), 1, 2, HashingOf::<Test>::hash_of(&data)),
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
		let data: DataOf<Test> = vec![1,2,3].try_into().unwrap();
		let data2: DataOf<Test> = vec![3,2,1].try_into().unwrap();
		let registrar = RuntimeOrigin::signed(0);
		let entity = RuntimeOrigin::signed(1);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));

		assert_ok!(Registry::set_entity(entity.clone(), data.clone()));
		assert_eq!(Registry::registries(1, 0), None);
		assert_ok!(Registry::register_entity(registrar, 0, 1, HashingOf::<Test>::hash_of(&data)));
		assert_eq!(Registry::registries(1, 0), Some(data.clone()));

		assert_ok!(Registry::set_entity(entity, data2.clone()));
		assert_eq!(Registry::requests(1), Some(data2));
		assert_eq!(Registry::registries(1, 0), Some(data));
	});
}

#[test]
fn multiple_registries_dont_collide() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1,2,3].try_into().unwrap();
		let data2: DataOf<Test> = vec![3,2,1].try_into().unwrap();
		let registrar1 = RuntimeOrigin::signed(0);
		let registrar2 = RuntimeOrigin::signed(1);
		let entity = RuntimeOrigin::signed(2);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 1));

		assert_ok!(Registry::set_entity(entity.clone(), data.clone()));
		assert_eq!(Registry::registries(2, 0), None);
		assert_eq!(Registry::registries(2, 1), None);

		assert_ok!(Registry::register_entity(registrar1, 0, 2, HashingOf::<Test>::hash_of(&data)));
		assert_eq!(Registry::registries(2, 0), Some(data.clone()));
		assert_eq!(Registry::registries(2, 1), None);

		assert_ok!(Registry::set_entity(entity.clone(), data2.clone()));
		assert_ok!(Registry::register_entity(registrar2, 1, 2, HashingOf::<Test>::hash_of(&data2)));
		assert_eq!(Registry::registries(2, 0), Some(data));
		assert_eq!(Registry::registries(2, 1), Some(data2));
	});
}

#[test]
fn clear_entity_wipes_everything() {
	new_test_ext().execute_with(|| {
		let data: DataOf<Test> = vec![1,2,3].try_into().unwrap();
		let registrar1 = RuntimeOrigin::signed(0);
		let registrar2 = RuntimeOrigin::signed(1);
		let entity = RuntimeOrigin::signed(2);

		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 0));
		assert_ok!(Registry::add_registrar(RuntimeOrigin::root(), 1));

		assert_ok!(Registry::set_entity(entity.clone(), data.clone()));
		assert_ok!(Registry::register_entity(registrar1, 0, 2, HashingOf::<Test>::hash_of(&data)));
		assert_ok!(Registry::register_entity(registrar2, 1, 2, HashingOf::<Test>::hash_of(&data)));

		assert_eq!(Registry::requests(2), Some(data.clone()));
		assert_eq!(Registry::registries(2, 0), Some(data.clone()));
		assert_eq!(Registry::registries(2, 1), Some(data.clone()));

		assert_ok!(Registry::clear_entity(entity));
		assert_eq!(Registry::requests(2), None);
		assert_eq!(Registry::registries(2, 0), None);
		assert_eq!(Registry::registries(2, 1), None);
	});
}