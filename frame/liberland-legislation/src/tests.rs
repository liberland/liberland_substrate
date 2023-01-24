use crate::{mock::*, Error, Laws, VetosCount, LegislationTier::*};
use frame_support::{assert_noop, assert_ok, error::BadOrigin, BoundedVec};
use pallet_democracy::Tally;
use sp_core::ConstU32;

fn constitution_origin(ayes: u64, nays: u64, aye_voters: u64, nay_voters: u64) -> RuntimeOrigin {
	pallet_democracy::Origin::<Test>::Referendum(Tally { ayes, nays, aye_voters, nay_voters, turnout: ayes + nays }, 1000)
		.try_into()
		.unwrap()
}

#[test]
fn add_law_requires_special_origin_for_treaty() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::add_law(
				RuntimeOrigin::signed(5),
				InternationalTreaty as u32,
				0,
				Default::default()
			),
			BadOrigin
		);
		assert_noop!(
			LiberlandLegislation::add_law(
				RuntimeOrigin::root(),
				InternationalTreaty as u32,
				0,
				Default::default()
			),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::add_law(
			RuntimeOrigin::signed(1),
			InternationalTreaty as u32,
			0,
			Default::default()
		));
	});
}

#[test]
fn add_law_requires_referendum_origin_for_constitution() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::add_law(
				RuntimeOrigin::signed(5),
				Constitution as u32,
				0,
				Default::default()
			),
			BadOrigin
		);
		assert_noop!(
			LiberlandLegislation::add_law(
				RuntimeOrigin::root(),
				Constitution as u32,
				0,
				Default::default()
			),
			BadOrigin
		);
		assert_noop!(
			LiberlandLegislation::add_law(
				constitution_origin(74, 26, 3, 2), // not enough votes and not enough voters
				Constitution as u32,
				0,
				Default::default()
			),
			BadOrigin
		);
		assert_noop!(
			LiberlandLegislation::add_law(
				constitution_origin(74, 26, 3, 1), // enough voters but not enough votes
				Constitution as u32,
				0,
				Default::default()
			),
			BadOrigin
		);
		assert_noop!(
			LiberlandLegislation::add_law(
				constitution_origin(75, 25, 3, 2), // enough votes but not enough voters
				Constitution as u32,
				0,
				Default::default()
			),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::add_law(
			constitution_origin(75, 25, 3, 1),
			Constitution as u32,
			0,
			Default::default()
		));
	});
}

#[test]
fn add_law_requires_root() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::add_law(
				RuntimeOrigin::signed(5),
				Decision as u32,
				0,
				Default::default()
			),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::add_law(
			RuntimeOrigin::root(),
			Decision as u32,
			0,
			Default::default()
		));
	});
}

#[test]
fn add_law_tier_must_be_valid() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::add_law(
				RuntimeOrigin::root(),
				InvalidTier as u32,
				0,
				Default::default()
			),
			Error::<Test>::InvalidTier
		);
		assert_ok!(LiberlandLegislation::add_law(
			RuntimeOrigin::root(),
			Decision as u32,
			0,
			Default::default()
		));
		assert_ok!(LiberlandLegislation::add_law(
			constitution_origin(100, 0, 1, 0),
			Constitution as u32,
			1,
			Default::default()
		));
	});
}

#[test]
fn cannot_overwrite_law() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_law(RuntimeOrigin::root(), 2, 0, Default::default()));
		assert_noop!(
			LiberlandLegislation::add_law(RuntimeOrigin::root(), 2, 0, Default::default()),
			Error::<Test>::LawAlreadyExists
		);
	});
}

#[test]
fn add_law_deposits_event() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_law(RuntimeOrigin::root(), 2, 0, Default::default()));
		System::assert_last_event(super::Event::LawAdded { tier: 2, index: 0 }.into());
	});
}

#[test]
fn add_law_stores_correct_data() {
	new_test_ext().execute_with(|| {
		let content: BoundedVec<u8, ConstU32<65536>> = vec![1, 2, 3].try_into().unwrap();
		assert_ok!(LiberlandLegislation::add_law(RuntimeOrigin::root(), 2, 0, content.clone()));
		assert_eq!(content, Laws::<Test>::get(2, 0));
	});
}

#[test]
fn repeal_law_requires_root() {
	new_test_ext().execute_with(|| {
		assert_noop!(LiberlandLegislation::repeal_law(RuntimeOrigin::signed(5), 2, 0), BadOrigin);
		assert_ok!(LiberlandLegislation::repeal_law(RuntimeOrigin::root(), 2, 0));
	});
}

#[test]
fn data_disappears_after_repeal() {
	new_test_ext().execute_with(|| {
		let content: BoundedVec<u8, ConstU32<65536>> = vec![1, 2, 3].try_into().unwrap();
		let empty: BoundedVec<u8, ConstU32<65536>> = Default::default();
		assert_ok!(LiberlandLegislation::add_law(RuntimeOrigin::root(), 2, 0, content.clone()));
		assert_ok!(LiberlandLegislation::repeal_law(RuntimeOrigin::root(), 2, 0));
		assert_eq!(empty, Laws::<Test>::get(2, 0));
	});
}

#[test]
fn allows_repeal_of_unexisting_law() {
	new_test_ext().execute_with(|| {
		let empty: BoundedVec<u8, ConstU32<65536>> = Default::default();
		assert_ok!(LiberlandLegislation::repeal_law(RuntimeOrigin::root(), 2, 0));
		assert_eq!(empty, Laws::<Test>::get(2, 0));
		System::assert_last_event(super::Event::LawRepealed { tier: 2, index: 0 }.into());
	});
}

#[test]
fn repeal_deposits_event() {
	new_test_ext().execute_with(|| {
		let content: BoundedVec<u8, ConstU32<65536>> = vec![1, 2, 3].try_into().unwrap();
		assert_ok!(LiberlandLegislation::add_law(RuntimeOrigin::root(), 2, 0, content.clone()));
		assert_ok!(LiberlandLegislation::repeal_law(RuntimeOrigin::root(), 2, 0));
		System::assert_last_event(super::Event::LawRepealed { tier: 2, index: 0 }.into());
	});
}

#[test]
fn cant_headcount_veto_low_tiers() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::submit_veto(RuntimeOrigin::signed(0), Constitution as u32, 0),
			Error::<Test>::InvalidTier
		);
	});
}

#[test]
fn cant_headcount_veto_as_noncitizen() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::submit_veto(RuntimeOrigin::signed(10), Decision as u32, 0),
			Error::<Test>::NonCitizen
		);
	});
}

#[test]
fn correctly_tracks_veto_count() {
	new_test_ext().execute_with(|| {
		assert_eq!(VetosCount::<Test>::get(Decision as u32, 0), 0);
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(1), Decision as u32, 0));
		assert_eq!(VetosCount::<Test>::get(Decision as u32, 0), 1);
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(1), Decision as u32, 0));
		assert_eq!(VetosCount::<Test>::get(Decision as u32, 0), 1);
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(2), Decision as u32, 0));
		assert_eq!(VetosCount::<Test>::get(Decision as u32, 0), 2);
		assert_ok!(LiberlandLegislation::revert_veto(RuntimeOrigin::signed(2), Decision as u32, 0));
		assert_eq!(VetosCount::<Test>::get(Decision as u32, 0), 1);
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(2), Decision as u32, 0));
		assert_eq!(VetosCount::<Test>::get(Decision as u32, 0), 2);
		assert_ok!(LiberlandLegislation::revert_veto(RuntimeOrigin::signed(1), Decision as u32, 0));
		assert_eq!(VetosCount::<Test>::get(Decision as u32, 0), 1);
		assert_ok!(LiberlandLegislation::revert_veto(RuntimeOrigin::signed(2), Decision as u32, 0));
		assert_eq!(VetosCount::<Test>::get(Decision as u32, 0), 0);
		assert_ok!(LiberlandLegislation::revert_veto(RuntimeOrigin::signed(2), Decision as u32, 0));
		assert_eq!(VetosCount::<Test>::get(Decision as u32, 0), 0);
	});
}

#[test]
fn can_headcount_veto_as_citizen() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(1), Decision as u32, 0));
		System::assert_last_event(
			super::Event::VetoSubmitted { tier: Decision as u32, index: 0, account: 1 }.into(),
		);
	});
}

#[test]
fn can_revert_veto() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(1), Decision as u32, 0));
		assert_ok!(LiberlandLegislation::revert_veto(RuntimeOrigin::signed(1), Decision as u32, 0));
		System::assert_last_event(
			super::Event::VetoReverted { tier: Decision as u32, index: 0, account: 1 }.into(),
		);
	});
}

#[test]
fn invalid_vetos_are_ignored() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(1), Decision as u32, 0));
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(2), Decision as u32, 0));
		assert_ok!(Identity::clear_identity(RuntimeOrigin::signed(2)));
		assert_noop!(
			LiberlandLegislation::trigger_headcount_veto(
				RuntimeOrigin::signed(0),
				Decision as u32,
				0
			),
			Error::<Test>::InsufficientVetoCount
		);
	});
}

#[test]
fn reverted_vetos_are_ignored() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(1), Decision as u32, 0));
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(2), Decision as u32, 0));
		assert_ok!(LiberlandLegislation::revert_veto(RuntimeOrigin::signed(2), Decision as u32, 0));
		assert_noop!(
			LiberlandLegislation::trigger_headcount_veto(
				RuntimeOrigin::signed(0),
				Decision as u32,
				0
			),
			Error::<Test>::InsufficientVetoCount
		);
	});
}

#[test]
fn can_trigger_with_enough_vetos() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(1), Decision as u32, 0));
		assert_noop!(
			LiberlandLegislation::trigger_headcount_veto(
				RuntimeOrigin::signed(0),
				Decision as u32,
				0
			),
			Error::<Test>::InsufficientVetoCount
		);
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(2), Decision as u32, 0));
		assert_ok!(LiberlandLegislation::trigger_headcount_veto(
			RuntimeOrigin::signed(0),
			Decision as u32,
			0
		));
		System::assert_last_event(
			super::Event::LawRepealedByHeadcountVeto { tier: Decision as u32, index: 0 }.into(),
		);
	});
}

#[test]
fn headcount_veto_actually_removes_law() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_law(
			RuntimeOrigin::root(),
			Decision as u32,
			0,
			vec![1, 2, 3, 4].try_into().unwrap()
		));
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(1), Decision as u32, 0));
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(2), Decision as u32, 0));
		assert_ok!(LiberlandLegislation::trigger_headcount_veto(
			RuntimeOrigin::signed(0),
			Decision as u32,
			0
		));

		let empty: BoundedVec<u8, ConstU32<65536>> = vec![].try_into().unwrap();
		assert_eq!(LiberlandLegislation::laws(Decision as u32, 0), empty);
	});
}
