#![cfg(test)]
use crate::{
	mock::*,
	types::{LegislationTier::*, *},
	Error, Laws, VetosCount,
};
use frame_support::{assert_noop, assert_ok, error::BadOrigin, BoundedVec};
use pallet_democracy::Tally;
use sp_core::ConstU32;
const ZERO_ID: LegislationId = LegislationId { year: 0u32, index: 0u32 };

fn constitution_origin(ayes: u64, nays: u64, aye_voters: u64, nay_voters: u64) -> RuntimeOrigin {
	pallet_democracy::Origin::<Test>::Referendum(
		Tally { ayes, nays, aye_voters, nay_voters, turnout: ayes + nays },
		1000,
	)
	.try_into()
	.unwrap()
}

#[test]
fn add_law_requires_special_origin_for_treaty() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::add_law(
				RuntimeOrigin::signed(5),
				InternationalTreaty,
				ZERO_ID,
				Default::default()
			),
			BadOrigin
		);
		assert_noop!(
			LiberlandLegislation::add_law(
				RuntimeOrigin::root(),
				InternationalTreaty,
				ZERO_ID,
				Default::default()
			),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::add_law(
			RuntimeOrigin::signed(1),
			InternationalTreaty,
			ZERO_ID,
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
				Constitution,
				ZERO_ID,
				Default::default()
			),
			BadOrigin
		);
		assert_noop!(
			LiberlandLegislation::add_law(
				RuntimeOrigin::root(),
				Constitution,
				ZERO_ID,
				Default::default()
			),
			BadOrigin
		);
		assert_noop!(
			LiberlandLegislation::add_law(
				constitution_origin(74, 26, 3, 2), // not enough votes and not enough voters
				Constitution,
				ZERO_ID,
				Default::default()
			),
			BadOrigin
		);
		assert_noop!(
			LiberlandLegislation::add_law(
				constitution_origin(74, 26, 3, 1), // enough voters but not enough votes
				Constitution,
				ZERO_ID,
				Default::default()
			),
			BadOrigin
		);
		assert_noop!(
			LiberlandLegislation::add_law(
				constitution_origin(75, 25, 3, 2), // enough votes but not enough voters
				Constitution,
				ZERO_ID,
				Default::default()
			),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::add_law(
			constitution_origin(75, 25, 3, 1),
			Constitution,
			ZERO_ID,
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
				Decision,
				ZERO_ID,
				Default::default()
			),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::add_law(
			RuntimeOrigin::root(),
			Decision,
			ZERO_ID,
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
				InvalidTier,
				ZERO_ID,
				Default::default()
			),
			Error::<Test>::InvalidTier
		);
		assert_ok!(LiberlandLegislation::add_law(
			RuntimeOrigin::root(),
			Decision,
			ZERO_ID,
			Default::default()
		));
		assert_ok!(LiberlandLegislation::add_law(
			constitution_origin(100, 0, 1, 0),
			Constitution,
			(0u32, 1u32).into(),
			Default::default()
		));
	});
}

#[test]
fn cannot_overwrite_law() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_law(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			Default::default()
		));
		assert_noop!(
			LiberlandLegislation::add_law(RuntimeOrigin::root(), Law, ZERO_ID, Default::default()),
			Error::<Test>::LawAlreadyExists
		);
	});
}

#[test]
fn add_law_deposits_event() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_law(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			Default::default()
		));
		System::assert_last_event(super::Event::LawAdded { tier: Law, id: ZERO_ID }.into());
	});
}

#[test]
fn add_law_stores_correct_data() {
	new_test_ext().execute_with(|| {
		let content: BoundedVec<u8, ConstU32<65536>> = vec![1, 2, 3].try_into().unwrap();
		assert_ok!(LiberlandLegislation::add_law(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			content.clone()
		));
		assert_eq!(content, Laws::<Test>::get(Law, ZERO_ID));
	});
}

#[test]
fn repeal_law_requires_root() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::repeal_law(RuntimeOrigin::signed(5), Law, ZERO_ID),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::repeal_law(RuntimeOrigin::root(), Law, ZERO_ID));
	});
}

#[test]
fn data_disappears_after_repeal() {
	new_test_ext().execute_with(|| {
		let content: BoundedVec<u8, ConstU32<65536>> = vec![1, 2, 3].try_into().unwrap();
		let empty: BoundedVec<u8, ConstU32<65536>> = Default::default();
		assert_ok!(LiberlandLegislation::add_law(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			content.clone()
		));
		assert_ok!(LiberlandLegislation::repeal_law(RuntimeOrigin::root(), Law, ZERO_ID));
		assert_eq!(empty, Laws::<Test>::get(Law, ZERO_ID));
	});
}

#[test]
fn allows_repeal_of_unexisting_law() {
	new_test_ext().execute_with(|| {
		let empty: BoundedVec<u8, ConstU32<65536>> = Default::default();
		assert_ok!(LiberlandLegislation::repeal_law(RuntimeOrigin::root(), Law, ZERO_ID));
		assert_eq!(empty, Laws::<Test>::get(Law, ZERO_ID));
		System::assert_last_event(super::Event::LawRepealed { tier: Law, id: ZERO_ID }.into());
	});
}

#[test]
fn repeal_deposits_event() {
	new_test_ext().execute_with(|| {
		let content: BoundedVec<u8, ConstU32<65536>> = vec![1, 2, 3].try_into().unwrap();
		assert_ok!(LiberlandLegislation::add_law(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			content.clone()
		));
		assert_ok!(LiberlandLegislation::repeal_law(RuntimeOrigin::root(), Law, ZERO_ID));
		System::assert_last_event(super::Event::LawRepealed { tier: Law, id: ZERO_ID }.into());
	});
}

#[test]
fn cant_headcount_veto_low_tiers() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::submit_veto(RuntimeOrigin::signed(0), Constitution, ZERO_ID),
			Error::<Test>::InvalidTier
		);
	});
}

#[test]
fn cant_headcount_veto_as_noncitizen() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::submit_veto(RuntimeOrigin::signed(10), Decision, ZERO_ID),
			Error::<Test>::NonCitizen
		);
	});
}

#[test]
fn correctly_tracks_veto_count() {
	new_test_ext().execute_with(|| {
		assert_eq!(VetosCount::<Test>::get(Decision, ZERO_ID), 0);
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(1), Decision, ZERO_ID));
		assert_eq!(VetosCount::<Test>::get(Decision, ZERO_ID), 1);
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(1), Decision, ZERO_ID));
		assert_eq!(VetosCount::<Test>::get(Decision, ZERO_ID), 1);
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(2), Decision, ZERO_ID));
		assert_eq!(VetosCount::<Test>::get(Decision, ZERO_ID), 2);
		assert_ok!(LiberlandLegislation::revert_veto(RuntimeOrigin::signed(2), Decision, ZERO_ID));
		assert_eq!(VetosCount::<Test>::get(Decision, ZERO_ID), 1);
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(2), Decision, ZERO_ID));
		assert_eq!(VetosCount::<Test>::get(Decision, ZERO_ID), 2);
		assert_ok!(LiberlandLegislation::revert_veto(RuntimeOrigin::signed(1), Decision, ZERO_ID));
		assert_eq!(VetosCount::<Test>::get(Decision, ZERO_ID), 1);
		assert_ok!(LiberlandLegislation::revert_veto(RuntimeOrigin::signed(2), Decision, ZERO_ID));
		assert_eq!(VetosCount::<Test>::get(Decision, ZERO_ID), 0);
		assert_ok!(LiberlandLegislation::revert_veto(RuntimeOrigin::signed(2), Decision, ZERO_ID));
		assert_eq!(VetosCount::<Test>::get(Decision, ZERO_ID), 0);
	});
}

#[test]
fn can_headcount_veto_as_citizen() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(1), Decision, ZERO_ID));
		System::assert_last_event(
			super::Event::VetoSubmitted { tier: Decision, id: ZERO_ID, account: 1 }.into(),
		);
	});
}

#[test]
fn can_revert_veto() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(1), Decision, ZERO_ID));
		assert_ok!(LiberlandLegislation::revert_veto(RuntimeOrigin::signed(1), Decision, ZERO_ID));
		System::assert_last_event(
			super::Event::VetoReverted { tier: Decision, id: ZERO_ID, account: 1 }.into(),
		);
	});
}

#[test]
fn invalid_vetos_are_ignored() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(1), Decision, ZERO_ID));
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(2), Decision, ZERO_ID));
		assert_ok!(Identity::clear_identity(RuntimeOrigin::signed(2)));
		assert_noop!(
			LiberlandLegislation::trigger_headcount_veto(
				RuntimeOrigin::signed(0),
				Decision,
				ZERO_ID
			),
			Error::<Test>::InsufficientVetoCount
		);
	});
}

#[test]
fn reverted_vetos_are_ignored() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(1), Decision, ZERO_ID));
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(2), Decision, ZERO_ID));
		assert_ok!(LiberlandLegislation::revert_veto(RuntimeOrigin::signed(2), Decision, ZERO_ID));
		assert_noop!(
			LiberlandLegislation::trigger_headcount_veto(
				RuntimeOrigin::signed(0),
				Decision,
				ZERO_ID
			),
			Error::<Test>::InsufficientVetoCount
		);
	});
}

#[test]
fn can_trigger_with_enough_vetos() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(1), Decision, ZERO_ID));
		assert_noop!(
			LiberlandLegislation::trigger_headcount_veto(
				RuntimeOrigin::signed(0),
				Decision,
				ZERO_ID
			),
			Error::<Test>::InsufficientVetoCount
		);
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(2), Decision, ZERO_ID));
		assert_ok!(LiberlandLegislation::trigger_headcount_veto(
			RuntimeOrigin::signed(0),
			Decision,
			ZERO_ID
		));
		System::assert_last_event(
			super::Event::LawRepealedByHeadcountVeto { tier: Decision, id: ZERO_ID }.into(),
		);
	});
}

#[test]
fn headcount_veto_actually_removes_law() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_law(
			RuntimeOrigin::root(),
			Decision,
			ZERO_ID,
			vec![1, 2, 3, 4].try_into().unwrap()
		));
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(1), Decision, ZERO_ID));
		assert_ok!(LiberlandLegislation::submit_veto(RuntimeOrigin::signed(2), Decision, ZERO_ID));
		assert_ok!(LiberlandLegislation::trigger_headcount_veto(
			RuntimeOrigin::signed(0),
			Decision,
			ZERO_ID
		));

		let empty: BoundedVec<u8, ConstU32<65536>> = vec![].try_into().unwrap();
		assert_eq!(LiberlandLegislation::laws(Decision, ZERO_ID), empty);
	});
}
