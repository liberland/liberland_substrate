#![cfg(test)]

use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use sp_std::prelude::*;

use crate::{mock::*, Error};
use crate::{Contracts, Event, Judges};

#[test]
fn anyone_can_create_contract() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(2);

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			origin,
			vec![].try_into().unwrap(),
			Some(vec![].try_into().unwrap())
		));
		assert_eq!(Balances::free_balance(2), 97u64);
	});
}

#[test]
fn root_can_add_judge() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::root();
		let acc = 1;

		assert_ok!(ContractsRegistry::add_judge(origin, acc));
	});
}

#[test]
fn signer_can_not_add_judge() {
	new_test_ext().execute_with(|| {
		let acc = 1;
		let origin = RawOrigin::Signed(acc);

		assert_noop!(
			ContractsRegistry::add_judge(origin.into(), acc),
			frame_support::error::BadOrigin
		);
	});
}

#[test]
fn judge_can_not_sign_not_existing_contract() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::root();
		let acc = 1;

		assert_ok!(ContractsRegistry::add_judge(origin, acc));

		let origin = RawOrigin::Signed(acc);
		assert_noop!(
			ContractsRegistry::judge_sign_contract(origin.into(), 0),
			Error::<Test>::ContractNotFound
		);
	});
}

#[test]
fn judge_can_sign_existing_contract() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::root();
		let acc = 1;

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			RuntimeOrigin::signed(2),
			vec![3, 4, 5].try_into().unwrap(),
			Some(vec![].try_into().unwrap())
		));
		assert_eq!(Balances::free_balance(2), 91u64);

		assert_ok!(ContractsRegistry::add_judge(origin, acc));

		let origin = RawOrigin::Signed(acc);
		assert_ok!(ContractsRegistry::judge_sign_contract(origin.into(), 0));
	});
}

#[test]
fn only_judge_can_use_judge_sign_contract() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(2);

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			origin.clone().into(),
			vec![3, 4, 5].try_into().unwrap(),
			Some(vec![].try_into().unwrap())
		));
		assert_eq!(Balances::free_balance(2), 91u64);

		assert_noop!(
			ContractsRegistry::judge_sign_contract(origin.into(), 0),
			Error::<Test>::NotJudge
		);
	});
}

#[test]
fn judge_already_signed_contract() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::root();

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			RuntimeOrigin::signed(2),
			vec![3, 4, 5].try_into().unwrap(),
			Some(vec![].try_into().unwrap())
		));
		assert_eq!(Balances::free_balance(2), 91u64);

		let acc = 1;
		assert_ok!(ContractsRegistry::add_judge(origin, acc));

		let origin = RawOrigin::Signed(acc);
		assert_ok!(ContractsRegistry::judge_sign_contract(origin.clone().into(), 0));

		assert_noop!(
			ContractsRegistry::judge_sign_contract(origin.into(), 0),
			Error::<Test>::AlreadySigned
		);
	});
}

#[test]
fn party_can_sign_contract() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(2);
		let acc = 1;

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			origin,
			vec![3, 4, 5].try_into().unwrap(),
			Some(vec![acc].try_into().unwrap())
		));
		assert_eq!(Balances::free_balance(2), 91u64);

		let origin = RawOrigin::Signed(acc);
		assert_ok!(ContractsRegistry::party_sign_contract(origin.into(), 0));
	});
}

#[test]
fn party_can_not_sign_not_existing_contract() {
	new_test_ext().execute_with(|| {
		let acc = 1;
		let origin = RawOrigin::Signed(acc);
		assert_noop!(
			ContractsRegistry::party_sign_contract(origin.into(), 0),
			Error::<Test>::ContractNotFound
		);
	});
}

#[test]
fn party_can_not_sign_not_a_party() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(2);
		let acc = 1;

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			origin,
			vec![3, 4, 5].try_into().unwrap(),
			Some(vec![].try_into().unwrap())
		));
		assert_eq!(Balances::free_balance(2), 91u64);

		let origin = RawOrigin::Signed(acc);
		assert_noop!(
			ContractsRegistry::party_sign_contract(origin.into(), 0),
			Error::<Test>::NotParty
		);
	});
}

#[test]
fn party_already_signed_contract() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(2);
		let acc = 1;

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			origin,
			vec![3, 4, 5].try_into().unwrap(),
			Some(vec![acc].try_into().unwrap())
		));
		assert_eq!(Balances::free_balance(2), 91u64);

		let origin = RawOrigin::Signed(acc);
		assert_ok!(ContractsRegistry::party_sign_contract(origin.clone().into(), 0));
		assert_noop!(
			ContractsRegistry::party_sign_contract(origin.into(), 0),
			Error::<Test>::AlreadySigned
		);
	});
}

#[test]
fn root_can_remove_judge() {
	new_test_ext().execute_with(|| {
		let judge: u64 = 0;
		let origin = RuntimeOrigin::root();
		assert_ok!(ContractsRegistry::add_judge(origin.clone(), judge.clone()));
		assert_noop!(
			ContractsRegistry::remove_judge(RuntimeOrigin::signed(2), judge),
			frame_support::error::BadOrigin
		);

		assert!(Judges::<Test>::get(judge));
	});
}

#[test]
fn random_can_not_remove_judge() {
	new_test_ext().execute_with(|| {
		let judge: u64 = 0;
		let origin = RuntimeOrigin::root();
		assert_ok!(ContractsRegistry::add_judge(origin.clone(), judge.clone()));
		assert_ok!(ContractsRegistry::remove_judge(origin, judge));

		assert!(!Judges::<Test>::get(judge));
	});
}

#[test]
fn add_judge_deposits_event() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::root();
		let judge = 1;

		assert_ok!(ContractsRegistry::add_judge(origin, judge));
		System::assert_last_event(Event::<Test>::AddedJudge { judge }.into());
	});
}

#[test]
fn judge_sign_contract_deposits_event() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::root();
		let signer = 1;

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			RuntimeOrigin::signed(2),
			vec![3, 4, 5].try_into().unwrap(),
			Some(vec![].try_into().unwrap())
		));
		assert_eq!(Balances::free_balance(2), 91u64);

		assert_ok!(ContractsRegistry::add_judge(origin, signer));

		let origin = RawOrigin::Signed(signer);
		assert_ok!(ContractsRegistry::judge_sign_contract(origin.into(), 0));
		System::assert_last_event(Event::<Test>::JudgeSigned { contract_id: 0, signer }.into());
	});
}

#[test]
fn create_contract_deposits_event() {
	new_test_ext().execute_with(|| {
		let signer = 2;
		let origin = RuntimeOrigin::signed(signer);
		Balances::make_free_balance_be(&signer, 100);

		assert_eq!(Balances::free_balance(signer), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			origin,
			vec![3, 4, 5].try_into().unwrap(),
			Some(vec![].try_into().unwrap())
		));
		System::assert_last_event(
			Event::<Test>::ContractCreated { contract_id: 0, creator: signer }.into(),
		);
	});
}

#[test]
fn party_sign_contract_deposits_event() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(2);
		let signer = 1;

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			origin,
			vec![3, 4, 5].try_into().unwrap(),
			Some(vec![signer].try_into().unwrap())
		));
		assert_eq!(Balances::free_balance(2), 91u64);

		let origin = RawOrigin::Signed(signer);
		assert_ok!(ContractsRegistry::party_sign_contract(origin.into(), 0));
		System::assert_last_event(Event::<Test>::PartySigned { contract_id: 0, signer }.into());
	});
}

#[test]
fn remove_judge_deposits_event() {
	new_test_ext().execute_with(|| {
		let judge = 0;
		let origin = RuntimeOrigin::root();

		assert_ok!(ContractsRegistry::add_judge(origin.clone(), judge));
		assert_ok!(ContractsRegistry::remove_judge(origin.into(), judge));
		System::assert_last_event(Event::<Test>::RemovedJudge { judge }.into());
	});
}

#[test]
fn creator_can_remove_contract() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(2);

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			origin.clone(),
			vec![].try_into().unwrap(),
			Some(vec![].try_into().unwrap())
		));
		assert_eq!(Balances::free_balance(2), 97u64);
		assert!(Contracts::<Test>::get(0).is_some());
		assert_ok!(ContractsRegistry::remove_contract(origin.into(), 0));
		assert!(Contracts::<Test>::get(0).is_none());
		assert_eq!(Balances::free_balance(2), 100u64);
	});
}

#[test]
fn anyone_can_remove_contract() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(2);

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			origin.clone(),
			vec![].try_into().unwrap(),
			Some(vec![].try_into().unwrap())
		));
		assert_eq!(Balances::free_balance(2), 97u64);
		assert!(Contracts::<Test>::get(0).is_some());
		assert_ok!(ContractsRegistry::remove_contract(RuntimeOrigin::signed(3), 0));
		assert!(Contracts::<Test>::get(0).is_none());
		assert_eq!(Balances::free_balance(2), 100u64);
	});
}

#[test]
fn can_not_remove_contract_signed_by_parties() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(2);

		let parties = 0;
		let parties_origin = RuntimeOrigin::signed(parties);

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			origin.clone(),
			vec![].try_into().unwrap(),
			Some(vec![parties].try_into().unwrap())
		));
		assert_eq!(Balances::free_balance(2), 97u64);
		assert!(Contracts::<Test>::get(0).is_some());
		assert_ok!(ContractsRegistry::party_sign_contract(parties_origin.into(), 0));
		assert_noop!(
			ContractsRegistry::remove_contract(origin.into(), 0),
			Error::<Test>::ContractInUse
		);
		assert!(Contracts::<Test>::get(0).is_some());
		assert_eq!(Balances::free_balance(2), 97u64);
	});
}

#[test]
fn can_not_remove_contract_signed_by_judge() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(2);

		let judge = 0;
		let judge_origin = RuntimeOrigin::signed(judge);

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			origin.clone(),
			vec![].try_into().unwrap(),
			Some(vec![].try_into().unwrap())
		));
		assert_eq!(Balances::free_balance(2), 97u64);
		assert!(Contracts::<Test>::get(0).is_some());
		assert_ok!(ContractsRegistry::add_judge(RuntimeOrigin::root(), judge));
		assert_ok!(ContractsRegistry::judge_sign_contract(judge_origin.into(), 0));
		assert_noop!(
			ContractsRegistry::remove_contract(origin.into(), 0),
			Error::<Test>::ContractInUse
		);
		assert!(Contracts::<Test>::get(0).is_some());
		assert_eq!(Balances::free_balance(2), 97u64);
	});
}

#[test]
fn remove_contract_deposits_event() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(2);

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			origin.clone(),
			vec![].try_into().unwrap(),
			Some(vec![].try_into().unwrap())
		));
		assert_eq!(Balances::free_balance(2), 97u64);
		assert!(Contracts::<Test>::get(0).is_some());
		assert_ok!(ContractsRegistry::remove_contract(origin.into(), 0));
		assert!(Contracts::<Test>::get(0).is_none());
		assert_eq!(Balances::free_balance(2), 100u64);
		System::assert_last_event(Event::<Test>::ContractRemoved { contract_id: 0 }.into());
	});
}

#[test]
fn create_contract_successfully_reserve_deposit() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(2);

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			origin.clone(),
			vec![1, 2, 3].try_into().unwrap(),
			Some(vec![].try_into().unwrap())
		));
		assert_eq!(Balances::free_balance(2), 91u64);
	});
}

#[test]
fn remove_contract_successfully_unreserve_deposit() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(2);

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			origin.clone(),
			vec![1, 2, 3].try_into().unwrap(),
			Some(vec![].try_into().unwrap())
		));
		assert_eq!(Balances::free_balance(2), 91u64);
		assert_ok!(ContractsRegistry::remove_contract(origin.clone(), 0));
		assert_eq!(Balances::free_balance(2), 100u64);
	});
}

#[test]
fn anyone_can_sign_contract_without_party() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(2);

		assert_eq!(Balances::free_balance(2), 100u64);
		assert_ok!(ContractsRegistry::create_contract(
			origin,
			vec![3, 4, 5].try_into().unwrap(),
			None
		));
		assert_eq!(Balances::free_balance(2), 91u64);

		assert_ok!(ContractsRegistry::party_sign_contract(RawOrigin::Signed(0).into(), 0));
		assert_ok!(ContractsRegistry::party_sign_contract(RawOrigin::Signed(1).into(), 0));
	});
}
