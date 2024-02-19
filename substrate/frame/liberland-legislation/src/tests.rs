#![cfg(test)]
use crate::{
	mock::*,
	types::{LegislationTier::*, *},
	Error, Legislation, LegislationVersion, VetosCount,
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

fn sample_legislation() -> BoundedVec<LegislationContent, ConstU32<1024>> {
	let section: LegislationContent = vec![1, 2, 3].try_into().unwrap();

	return vec![section.clone(), section.clone(), section.clone()].try_into().unwrap();
}

#[test]
fn cant_add_empty_legislation() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::add_legislation(
				RuntimeOrigin::signed(5),
				InternationalTreaty,
				ZERO_ID,
				vec![].try_into().unwrap(),
			),
			Error::<Test>::EmptyLegislation,
		);
	});
}

#[test]
fn add_legislation_requires_special_origin_for_treaty() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::add_legislation(
				RuntimeOrigin::signed(5),
				InternationalTreaty,
				ZERO_ID,
				sample_legislation(),
			),
			BadOrigin
		);
		assert_noop!(
			LiberlandLegislation::add_legislation(
				RuntimeOrigin::root(),
				InternationalTreaty,
				ZERO_ID,
				sample_legislation(),
			),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::signed(1),
			InternationalTreaty,
			ZERO_ID,
			sample_legislation(),
		));
	});
}

#[test]
fn add_legislation_requires_referendum_origin_for_constitution() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::add_legislation(
				RuntimeOrigin::signed(5),
				Constitution,
				ZERO_ID,
				sample_legislation(),
			),
			BadOrigin
		);
		assert_noop!(
			LiberlandLegislation::add_legislation(
				RuntimeOrigin::root(),
				Constitution,
				ZERO_ID,
				sample_legislation(),
			),
			BadOrigin
		);
		assert_noop!(
			LiberlandLegislation::add_legislation(
				constitution_origin(74, 26, 3, 2), // not enough votes and not enough voters
				Constitution,
				ZERO_ID,
				sample_legislation(),
			),
			BadOrigin
		);
		assert_noop!(
			LiberlandLegislation::add_legislation(
				constitution_origin(74, 26, 3, 1), // enough voters but not enough votes
				Constitution,
				ZERO_ID,
				sample_legislation(),
			),
			BadOrigin
		);
		assert_noop!(
			LiberlandLegislation::add_legislation(
				constitution_origin(75, 25, 3, 2), // enough votes but not enough voters
				Constitution,
				ZERO_ID,
				sample_legislation(),
			),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::add_legislation(
			constitution_origin(75, 25, 3, 1),
			Constitution,
			ZERO_ID,
			sample_legislation(),
		));
	});
}

#[test]
fn add_legislation_requires_root() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::add_legislation(
				RuntimeOrigin::signed(5),
				Decision,
				ZERO_ID,
				sample_legislation(),
			),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Decision,
			ZERO_ID,
			sample_legislation(),
		));
	});
}

#[test]
fn add_legislation_tier_must_be_valid() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::add_legislation(
				RuntimeOrigin::root(),
				InvalidTier,
				ZERO_ID,
				sample_legislation(),
			),
			Error::<Test>::InvalidTier
		);
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Decision,
			ZERO_ID,
			sample_legislation(),
		));
		assert_ok!(LiberlandLegislation::add_legislation(
			constitution_origin(100, 0, 1, 0),
			Constitution,
			(0u32, 1u32).into(),
			sample_legislation(),
		));
	});
}

#[test]
fn cannot_overwrite_legislation() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		assert_noop!(
			LiberlandLegislation::add_legislation(
				RuntimeOrigin::root(),
				Law,
				ZERO_ID,
				sample_legislation()
			),
			Error::<Test>::LegislationAlreadyExists
		);
	});
}

#[test]
fn add_legislation_deposits_event() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		System::assert_last_event(super::Event::LegislationAdded { tier: Law, id: ZERO_ID }.into());
	});
}

#[test]
fn add_legislation_stores_correct_data() {
	new_test_ext().execute_with(|| {
		let content: LegislationContent = vec![1, 2, 3].try_into().unwrap();
		let content2: LegislationContent = vec![5, 5, 5].try_into().unwrap();
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			vec![content.clone(), content2.clone()].try_into().unwrap(),
		));
		assert_eq!(Some(Some(content)), Legislation::<Test>::get((Law, ZERO_ID, 0)));
		assert_eq!(Some(Some(content2)), Legislation::<Test>::get((Law, ZERO_ID, 1)));
		assert_eq!(None, Legislation::<Test>::get((Law, ZERO_ID, 2)));
	});
}

#[test]
fn repeal_constitution_legislation_requires_special_origin() {
	new_test_ext().execute_with(|| {
		let id = LegislationId { year: 2023, index: 1 };
		assert_ok!(LiberlandLegislation::add_legislation(
			constitution_origin(75, 25, 3, 1),
			Constitution,
			id,
			sample_legislation(),
		));
		assert_noop!(
			LiberlandLegislation::repeal_legislation(RuntimeOrigin::signed(5), Constitution, id, 1),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::repeal_legislation(
			constitution_origin(75, 25, 3, 1),
			Constitution,
			id,
			1
		));
	});
}

#[test]
fn cant_repeal_constitution_zero() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			constitution_origin(75, 25, 3, 1),
			Constitution,
			ZERO_ID,
			sample_legislation(),
		));
		assert_noop!(
			LiberlandLegislation::repeal_legislation(
				constitution_origin(75, 25, 3, 1),
				Constitution,
				ZERO_ID,
				1
			),
			Error::<Test>::ProtectedLegislation,
		);
	});
}

#[test]
fn repeal_treaty_legislation_requires_special_origin() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::signed(1),
			InternationalTreaty,
			ZERO_ID,
			sample_legislation(),
		));
		assert_noop!(
			LiberlandLegislation::repeal_legislation(
				RuntimeOrigin::signed(5),
				InternationalTreaty,
				ZERO_ID,
				1
			),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::repeal_legislation(
			RuntimeOrigin::signed(1),
			InternationalTreaty,
			ZERO_ID,
			1
		));
	});
}

#[test]
fn repeal_legislation_requires_root() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		assert_noop!(
			LiberlandLegislation::repeal_legislation(RuntimeOrigin::signed(5), Law, ZERO_ID, 1),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::repeal_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			1
		));
	});
}

#[test]
fn repeal_legislation_verifies_witness_data() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		assert_noop!(
			LiberlandLegislation::repeal_legislation(RuntimeOrigin::root(), Law, ZERO_ID, 0),
			Error::<Test>::InvalidWitness
		);
		assert_noop!(
			LiberlandLegislation::repeal_legislation(RuntimeOrigin::root(), Law, ZERO_ID, 2),
			Error::<Test>::InvalidWitness
		);
		assert_ok!(LiberlandLegislation::repeal_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			1
		));
	});
}

#[test]
fn data_disappears_after_repeal() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		assert_ok!(LiberlandLegislation::repeal_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			1
		));
		assert_eq!(Legislation::<Test>::get((Law, ZERO_ID, 0)), Some(None));
		assert_eq!(Legislation::<Test>::get((Law, ZERO_ID, 1)), Some(None));
	});
}

#[test]
fn disallows_repeal_of_unexisting_law() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::repeal_legislation(RuntimeOrigin::root(), Law, ZERO_ID, 0),
			Error::<Test>::InvalidLegislation,
		);
	});
}

#[test]
fn repeal_deposits_event() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		assert_ok!(LiberlandLegislation::repeal_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			1
		));
		System::assert_last_event(
			super::Event::LegislationRepealed { tier: Law, id: ZERO_ID, section: None }.into(),
		);
	});
}

#[test]
fn repeal_bumps_version() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		assert_ok!(LiberlandLegislation::repeal_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			1
		));
		assert_eq!(LegislationVersion::<Test>::get((Law, ZERO_ID, Some(0))), 2);
		assert_eq!(LegislationVersion::<Test>::get((Law, ZERO_ID, Some(1))), 2);
	});
}

#[test]
fn repeal_constitution_legislation_section_requires_special_origin() {
	new_test_ext().execute_with(|| {
		let id = LegislationId { year: 2023, index: 1 };
		assert_ok!(LiberlandLegislation::add_legislation(
			constitution_origin(75, 25, 3, 1),
			Constitution,
			id,
			sample_legislation(),
		));
		assert_noop!(
			LiberlandLegislation::repeal_legislation_section(
				RuntimeOrigin::signed(5),
				Constitution,
				id,
				0,
				1
			),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::repeal_legislation_section(
			constitution_origin(75, 25, 3, 1),
			Constitution,
			id,
			0,
			1
		));
	});
}

#[test]
fn cant_repeal_constitution_zero_sections() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			constitution_origin(75, 25, 3, 1),
			Constitution,
			ZERO_ID,
			sample_legislation(),
		));
		assert_noop!(
			LiberlandLegislation::repeal_legislation_section(
				constitution_origin(75, 25, 3, 1),
				Constitution,
				ZERO_ID,
				0,
				1
			),
			Error::<Test>::ProtectedLegislation,
		);
	});
}

#[test]
fn repeal_treaty_legislation_section_requires_special_origin() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::signed(1),
			InternationalTreaty,
			ZERO_ID,
			sample_legislation(),
		));
		assert_noop!(
			LiberlandLegislation::repeal_legislation_section(
				RuntimeOrigin::signed(5),
				InternationalTreaty,
				ZERO_ID,
				0,
				1
			),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::repeal_legislation_section(
			RuntimeOrigin::signed(1),
			InternationalTreaty,
			ZERO_ID,
			0,
			1
		));
	});
}

#[test]
fn repeal_legislation_section_requires_root() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		assert_noop!(
			LiberlandLegislation::repeal_legislation_section(
				RuntimeOrigin::signed(5),
				Law,
				ZERO_ID,
				0,
				1
			),
			BadOrigin
		);
		assert_ok!(LiberlandLegislation::repeal_legislation_section(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			0,
			1
		));
	});
}

#[test]
fn repeal_legislation_section_verifies_witness_data() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		assert_noop!(
			LiberlandLegislation::repeal_legislation_section(
				RuntimeOrigin::root(),
				Law,
				ZERO_ID,
				0,
				0
			),
			Error::<Test>::InvalidWitness
		);
		assert_noop!(
			LiberlandLegislation::repeal_legislation_section(
				RuntimeOrigin::root(),
				Law,
				ZERO_ID,
				0,
				2
			),
			Error::<Test>::InvalidWitness
		);
		assert_ok!(LiberlandLegislation::repeal_legislation_section(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			0,
			1
		));
	});
}

#[test]
fn data_disappears_after_repeal_section() {
	new_test_ext().execute_with(|| {
		let sections = sample_legislation();
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sections.clone(),
		));
		assert_ok!(LiberlandLegislation::repeal_legislation_section(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			1,
			1
		));
		assert_eq!(Legislation::<Test>::get((Law, ZERO_ID, 0)), Some(Some(sections[0].clone())));
		assert_eq!(Legislation::<Test>::get((Law, ZERO_ID, 1)), Some(None));
		assert_eq!(Legislation::<Test>::get((Law, ZERO_ID, 2)), Some(Some(sections[2].clone())));
	});
}

#[test]
fn disallows_repeal_section_of_unexisting_law() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::repeal_legislation_section(
				RuntimeOrigin::root(),
				Law,
				ZERO_ID,
				0,
				0
			),
			Error::<Test>::InvalidLegislation,
		);
	});
}

#[test]
fn repeal_section_deposits_event() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		assert_ok!(LiberlandLegislation::repeal_legislation_section(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			0,
			1
		));
		System::assert_last_event(
			super::Event::LegislationRepealed { tier: Law, id: ZERO_ID, section: Some(0) }.into(),
		);
	});
}

#[test]
fn repeal_section_bumps_version() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		assert_ok!(LiberlandLegislation::repeal_legislation_section(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			1,
			1
		));
		assert_eq!(LegislationVersion::<Test>::get((Law, ZERO_ID, Some(1))), 2);
	});
}

#[test]
fn cant_headcount_veto_low_tiers() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::submit_veto(
				RuntimeOrigin::signed(0),
				Constitution,
				ZERO_ID,
				None
			),
			Error::<Test>::InvalidTier
		);
	});
}

#[test]
fn cant_headcount_veto_as_noncitizen() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::submit_veto(RuntimeOrigin::signed(10), Decision, ZERO_ID, None),
			Error::<Test>::NonCitizen
		);
	});
}

#[test]
fn correctly_tracks_veto_count() {
	new_test_ext().execute_with(|| {
		assert_eq!(VetosCount::<Test>::get((Decision, ZERO_ID, None::<LegislationSection>)), 0);
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(1),
			Decision,
			ZERO_ID,
			None::<LegislationSection>
		));
		assert_eq!(VetosCount::<Test>::get((Decision, ZERO_ID, None::<LegislationSection>)), 1);
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(1),
			Decision,
			ZERO_ID,
			None::<LegislationSection>
		));
		assert_eq!(VetosCount::<Test>::get((Decision, ZERO_ID, None::<LegislationSection>)), 1);
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(2),
			Decision,
			ZERO_ID,
			None::<LegislationSection>
		));
		assert_eq!(VetosCount::<Test>::get((Decision, ZERO_ID, None::<LegislationSection>)), 2);
		assert_ok!(LiberlandLegislation::revert_veto(
			RuntimeOrigin::signed(2),
			Decision,
			ZERO_ID,
			None::<LegislationSection>
		));
		assert_eq!(VetosCount::<Test>::get((Decision, ZERO_ID, None::<LegislationSection>)), 1);
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(2),
			Decision,
			ZERO_ID,
			None::<LegislationSection>
		));
		assert_eq!(VetosCount::<Test>::get((Decision, ZERO_ID, None::<LegislationSection>)), 2);
		assert_ok!(LiberlandLegislation::revert_veto(
			RuntimeOrigin::signed(1),
			Decision,
			ZERO_ID,
			None::<LegislationSection>
		));
		assert_eq!(VetosCount::<Test>::get((Decision, ZERO_ID, None::<LegislationSection>)), 1);
		assert_ok!(LiberlandLegislation::revert_veto(
			RuntimeOrigin::signed(2),
			Decision,
			ZERO_ID,
			None::<LegislationSection>
		));
		assert_eq!(VetosCount::<Test>::get((Decision, ZERO_ID, None::<LegislationSection>)), 0);
		assert_ok!(LiberlandLegislation::revert_veto(
			RuntimeOrigin::signed(2),
			Decision,
			ZERO_ID,
			None::<LegislationSection>
		));
		assert_eq!(VetosCount::<Test>::get((Decision, ZERO_ID, None::<LegislationSection>)), 0);
	});
}

#[test]
fn can_headcount_veto_as_citizen() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(1),
			Decision,
			ZERO_ID,
			None
		));
		System::assert_last_event(
			super::Event::VetoSubmitted { tier: Decision, id: ZERO_ID, section: None, account: 1 }
				.into(),
		);
	});
}

#[test]
fn can_revert_veto() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(1),
			Decision,
			ZERO_ID,
			None
		));
		assert_ok!(LiberlandLegislation::revert_veto(
			RuntimeOrigin::signed(1),
			Decision,
			ZERO_ID,
			None
		));
		System::assert_last_event(
			super::Event::VetoReverted { tier: Decision, id: ZERO_ID, section: None, account: 1 }
				.into(),
		);
	});
}

#[test]
fn invalid_vetos_are_ignored() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(1),
			Decision,
			ZERO_ID,
			None
		));
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(2),
			Decision,
			ZERO_ID,
			None
		));
		assert_ok!(Identity::clear_identity(RuntimeOrigin::signed(2)));
		assert_noop!(
			LiberlandLegislation::trigger_headcount_veto(
				RuntimeOrigin::signed(0),
				Decision,
				ZERO_ID,
			),
			Error::<Test>::InsufficientVetoCount
		);
	});
}

#[test]
fn reverted_vetos_are_ignored() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(1),
			Decision,
			ZERO_ID,
			None
		));
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(2),
			Decision,
			ZERO_ID,
			None
		));
		assert_ok!(LiberlandLegislation::revert_veto(
			RuntimeOrigin::signed(2),
			Decision,
			ZERO_ID,
			None
		));
		assert_noop!(
			LiberlandLegislation::trigger_headcount_veto(
				RuntimeOrigin::signed(0),
				Decision,
				ZERO_ID,
			),
			Error::<Test>::InsufficientVetoCount
		);
	});
}

#[test]
fn can_trigger_with_enough_vetos() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(1),
			Decision,
			ZERO_ID,
			None
		));
		assert_noop!(
			LiberlandLegislation::trigger_headcount_veto(
				RuntimeOrigin::signed(0),
				Decision,
				ZERO_ID,
			),
			Error::<Test>::InsufficientVetoCount
		);
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(2),
			Decision,
			ZERO_ID,
			None
		));
		assert_ok!(LiberlandLegislation::trigger_headcount_veto(
			RuntimeOrigin::signed(0),
			Decision,
			ZERO_ID,
		));
		System::assert_last_event(
			super::Event::LegislationRepealedByHeadcountVeto {
				tier: Decision,
				id: ZERO_ID,
				section: None,
			}
			.into(),
		);
	});
}

#[test]
fn headcount_veto_actually_removes_law() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Decision,
			ZERO_ID,
			sample_legislation(),
		));
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(1),
			Decision,
			ZERO_ID,
			None
		));
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(2),
			Decision,
			ZERO_ID,
			None
		));
		assert_ok!(LiberlandLegislation::trigger_headcount_veto(
			RuntimeOrigin::signed(0),
			Decision,
			ZERO_ID,
		));

		assert_eq!(LiberlandLegislation::legislation((Decision, ZERO_ID, 0)), Some(None));
	});
}

#[test]
fn can_trigger_section_veto_with_enough_vetos() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(1),
			Decision,
			ZERO_ID,
			Some(0)
		));
		assert_noop!(
			LiberlandLegislation::trigger_section_headcount_veto(
				RuntimeOrigin::signed(0),
				Decision,
				ZERO_ID,
				0,
			),
			Error::<Test>::InsufficientVetoCount
		);
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(2),
			Decision,
			ZERO_ID,
			Some(0)
		));
		assert_ok!(LiberlandLegislation::trigger_section_headcount_veto(
			RuntimeOrigin::signed(0),
			Decision,
			ZERO_ID,
			0,
		));
		System::assert_last_event(
			super::Event::LegislationRepealedByHeadcountVeto {
				tier: Decision,
				id: ZERO_ID,
				section: Some(0),
			}
			.into(),
		);
	});
}

#[test]
fn section_headcount_veto_actually_removes_law() {
	new_test_ext().execute_with(|| {
		let sections = sample_legislation();
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Decision,
			ZERO_ID,
			sections.clone(),
		));
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(1),
			Decision,
			ZERO_ID,
			Some(1)
		));
		assert_ok!(LiberlandLegislation::submit_veto(
			RuntimeOrigin::signed(2),
			Decision,
			ZERO_ID,
			Some(1)
		));
		assert_ok!(LiberlandLegislation::trigger_section_headcount_veto(
			RuntimeOrigin::signed(0),
			Decision,
			ZERO_ID,
			1,
		));

		assert_eq!(
			Legislation::<Test>::get((Decision, ZERO_ID, 0)),
			Some(Some(sections[0].clone()))
		);
		assert_eq!(Legislation::<Test>::get((Decision, ZERO_ID, 1)), Some(None));
		assert_eq!(
			Legislation::<Test>::get((Decision, ZERO_ID, 2)),
			Some(Some(sections[2].clone()))
		);
	});
}

#[test]
fn amend_legislation_requires_special_origin_for_treaty() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::signed(1),
			InternationalTreaty,
			ZERO_ID,
			sample_legislation(),
		));
		assert_noop!(
			LiberlandLegislation::amend_legislation(
				RuntimeOrigin::root(),
				InternationalTreaty,
				ZERO_ID,
				0,
				Default::default(),
				1
			),
			BadOrigin
		);
		assert_noop!(
			LiberlandLegislation::amend_legislation(
				RuntimeOrigin::signed(2),
				InternationalTreaty,
				ZERO_ID,
				0,
				Default::default(),
				1
			),
			BadOrigin
		);
	});
}

#[test]
fn amend_legislation_requires_referendum_origin_for_constitution() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			constitution_origin(75, 25, 3, 1),
			Constitution,
			ZERO_ID,
			sample_legislation(),
		));
		assert_noop!(
			LiberlandLegislation::amend_legislation(
				constitution_origin(75, 25, 3, 2),
				Constitution,
				ZERO_ID,
				0,
				Default::default(),
				1,
			),
			BadOrigin
		);
	});
}

#[test]
fn amend_legislation_requires_root() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Decision,
			ZERO_ID,
			sample_legislation(),
		));
		assert_noop!(
			LiberlandLegislation::amend_legislation(
				RuntimeOrigin::signed(1),
				Decision,
				ZERO_ID,
				0,
				Default::default(),
				1,
			),
			BadOrigin
		);
	});
}

#[test]
fn amend_legislation_needs_existing_legislation() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiberlandLegislation::amend_legislation(
				RuntimeOrigin::root(),
				Law,
				ZERO_ID,
				10,
				Default::default(),
				0,
			),
			Error::<Test>::InvalidLegislation
		);
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		assert_ok!(LiberlandLegislation::amend_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			10,
			Default::default(),
			0,
		));
	});
}

#[test]
fn amend_can_overwrite_legislation() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		assert_ok!(LiberlandLegislation::amend_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			0,
			Default::default(),
			1
		),);
		assert_eq!(Legislation::<Test>::get((Law, ZERO_ID, 0)).unwrap().unwrap().len(), 0);
	});
}

#[test]
fn amend_legislation_deposits_event() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		assert_ok!(LiberlandLegislation::amend_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			0,
			Default::default(),
			1
		),);
		System::assert_last_event(
			super::Event::LegislationAmended { tier: Law, id: ZERO_ID, section: 0 }.into(),
		);
	});
}

#[test]
fn amend_legislation_stores_correct_data() {
	new_test_ext().execute_with(|| {
		let content: LegislationContent = vec![5, 5, 5].try_into().unwrap();
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		assert_ok!(LiberlandLegislation::amend_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			1,
			content.clone(),
			1,
		));
		assert_eq!(Some(Some(content)), Legislation::<Test>::get((Law, ZERO_ID, 1)));
	});
}

#[test]
fn cant_amend_constitution_zero() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			constitution_origin(75, 25, 3, 1),
			Constitution,
			ZERO_ID,
			sample_legislation(),
		));
		assert_noop!(
			LiberlandLegislation::amend_legislation(
				constitution_origin(75, 25, 3, 1),
				Constitution,
				ZERO_ID,
				0,
				Default::default(),
				1
			),
			Error::<Test>::ProtectedLegislation,
		);
	});
}

#[test]
fn amend_legislation_verifies_witness_data() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		assert_noop!(
			LiberlandLegislation::amend_legislation(
				RuntimeOrigin::root(),
				Law,
				ZERO_ID,
				0,
				Default::default(),
				0,
			),
			Error::<Test>::InvalidWitness
		);
		assert_noop!(
			LiberlandLegislation::amend_legislation(
				RuntimeOrigin::root(),
				Law,
				ZERO_ID,
				0,
				Default::default(),
				2,
			),
			Error::<Test>::InvalidWitness
		);
		assert_ok!(LiberlandLegislation::amend_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			0,
			Default::default(),
			1,
		));
	});
}

#[test]
fn amend_bumps_version() {
	new_test_ext().execute_with(|| {
		assert_ok!(LiberlandLegislation::add_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			sample_legislation(),
		));
		assert_ok!(LiberlandLegislation::amend_legislation(
			RuntimeOrigin::root(),
			Law,
			ZERO_ID,
			1,
			Default::default(),
			1
		));
		assert_eq!(LegislationVersion::<Test>::get((Law, ZERO_ID, Some(0))), 1);
		assert_eq!(LegislationVersion::<Test>::get((Law, ZERO_ID, Some(1))), 2);
	});
}
