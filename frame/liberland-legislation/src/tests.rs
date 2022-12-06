use crate::{mock::*, Error, Laws};
use frame_support::{assert_noop, assert_ok, error::BadOrigin, BoundedVec};
use sp_core::ConstU32;

#[test]
fn add_law_requires_root() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::add_law(RuntimeOrigin::signed(5), 0, 0, Default::default()),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::add_law(RuntimeOrigin::root(), 0, 0, Default::default()));
	});
}

#[test]
fn cannot_overwrite_law() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_law(RuntimeOrigin::root(), 0, 0, Default::default()));
		assert_noop!(
			LiberlandLegislation::add_law(RuntimeOrigin::root(), 0, 0, Default::default()),
			Error::<Test>::LawAlreadyExists
		);
	});
}

#[test]
fn add_law_deposits_event() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_law(RuntimeOrigin::root(), 0, 0, Default::default()));
		System::assert_last_event(super::Event::LawAdded { index1: 0, index2: 0 }.into());
	});
}

#[test]
fn add_law_stores_correct_data() {
	new_test_ext().execute_with(|| {
		let content: BoundedVec<u8, ConstU32<65536>> = vec![1, 2, 3].try_into().unwrap();
		assert_ok!(LiberlandLegislation::add_law(RuntimeOrigin::root(), 0, 0, content.clone()));
		assert_eq!(content, Laws::<Test>::get(0, 0));
	});
}

#[test]
fn repeal_law_requires_root() {
	new_test_ext().execute_with(|| {
		assert_noop!(LiberlandLegislation::repeal_law(RuntimeOrigin::signed(5), 0, 0), BadOrigin);
		assert_ok!(LiberlandLegislation::repeal_law(RuntimeOrigin::root(), 0, 0));
	});
}

#[test]
fn data_disappears_after_repeal() {
	new_test_ext().execute_with(|| {
		let content: BoundedVec<u8, ConstU32<65536>> = vec![1, 2, 3].try_into().unwrap();
		let empty: BoundedVec<u8, ConstU32<65536>> = Default::default();
		assert_ok!(LiberlandLegislation::add_law(RuntimeOrigin::root(), 0, 0, content.clone()));
		assert_ok!(LiberlandLegislation::repeal_law(RuntimeOrigin::root(), 0, 0));
		assert_eq!(empty, Laws::<Test>::get(0, 0));
	});
}

#[test]
fn allows_repeal_of_unexisting_law() {
	new_test_ext().execute_with(|| {
		let empty: BoundedVec<u8, ConstU32<65536>> = Default::default();
		assert_ok!(LiberlandLegislation::repeal_law(RuntimeOrigin::root(), 0, 0));
		assert_eq!(empty, Laws::<Test>::get(0, 0));
		System::assert_last_event(super::Event::LawRepealed { index1: 0, index2: 0 }.into());
	});
}

#[test]
fn repeal_deposits_event() {
	new_test_ext().execute_with(|| {
		let content: BoundedVec<u8, ConstU32<65536>> = vec![1, 2, 3].try_into().unwrap();
		assert_ok!(LiberlandLegislation::add_law(RuntimeOrigin::root(), 0, 0, content.clone()));
		assert_ok!(LiberlandLegislation::repeal_law(RuntimeOrigin::root(), 0, 0));
		System::assert_last_event(super::Event::LawRepealed { index1: 0, index2: 0 }.into());
	});
}
