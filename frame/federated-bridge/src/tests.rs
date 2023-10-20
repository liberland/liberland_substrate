#![cfg(test)]

use crate::{
	mock::*, Admin, BridgeState, Error, EthAddress, Event, Fee, IncomingReceipt,
	IncomingReceiptStatus, IncomingReceipts, ReceiptId, Relays, State, StatusOf, SuperAdmin,
	VotesRequired, Voting, Watchers,
};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{
	traits::{AccountIdConversion, BadOrigin},
	TokenError,
};

fn eth_recipient(n: u8) -> EthAddress {
	let mut addr: EthAddress = Default::default();
	addr[0] = n;
	addr
}

fn gen_receipt(recipient: u64, amount: u64) -> (ReceiptId, IncomingReceipt<u64, u64>) {
	let mut id: ReceiptId = Default::default();
	id[0] = recipient as u8;

	(id, IncomingReceipt { eth_block_number: 10, substrate_recipient: recipient, amount })
}

fn bridge_wallet() -> u64 {
	BridgePalletId::get().into_account_truncating()
}

/* DEPOSITS */
#[test]
fn deposit_fails_on_stopped_bridge() {
	new_test_ext().execute_with(|| {
		assert_ok!(Bridge::set_state(RuntimeOrigin::signed(100), BridgeState::Stopped));
		assert_noop!(
			Bridge::deposit(RuntimeOrigin::signed(0), 1, eth_recipient(0)),
			Error::<Test>::BridgeStopped
		);
	});
}

#[test]
fn deposit_emits_receipt() {
	new_test_ext().execute_with(|| {
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(0), 2, eth_recipient(0)));
		System::assert_last_event(
			Event::<Test>::OutgoingReceipt { from: 0, amount: 2, eth_recipient: eth_recipient(0) }
				.into(),
		);
	});
}

#[test]
fn deposit_takes_token_from_caller() {
	new_test_ext().execute_with(|| {
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(0), 2, eth_recipient(0)));
		assert_eq!(Balances::free_balance(0), 98);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(0), 90, eth_recipient(0)));
		assert_eq!(Balances::free_balance(0), 8);
	});
}

#[test]
fn deposit_stores_token_in_bridge() {
	new_test_ext().execute_with(|| {
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(0), 2, eth_recipient(0)));
		assert_eq!(Balances::free_balance(bridge_wallet()), 2);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(0), 90, eth_recipient(0)));
		assert_eq!(Balances::free_balance(bridge_wallet()), 92);
	});
}

#[test]
fn deposit_fails_on_insufficient_funds() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Bridge::deposit(RuntimeOrigin::signed(0), 101, eth_recipient(0)),
			TokenError::FundsUnavailable,
		);
	});
}

#[test]
fn deposit_failes_on_nonsiged_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(Bridge::deposit(RuntimeOrigin::root(), 101, eth_recipient(0)), BadOrigin);
	});
}

#[test]
fn deposit_respects_max_total_locked() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Bridge::deposit(RuntimeOrigin::signed(200), 10001, eth_recipient(0)),
			Error::<Test>::TooMuchLocked
		);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(200), 9998, eth_recipient(0)));
		assert_noop!(
			Bridge::deposit(RuntimeOrigin::signed(200), 3, eth_recipient(0)),
			Error::<Test>::TooMuchLocked
		);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(200), 2, eth_recipient(0)));
	});
}

#[test]
fn deposit_respects_minimum_transfer() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Bridge::deposit(RuntimeOrigin::signed(200), 1, eth_recipient(0)),
			Error::<Test>::TooSmallAmount
		);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(200), 2, eth_recipient(0)));
	});
}

/* VOTING */

#[test]
fn vote_fails_on_non_relay() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1);
		assert_noop!(
			Bridge::vote_withdraw(RuntimeOrigin::root(), receipt_id, receipt.clone()),
			BadOrigin
		);
		assert_noop!(
			Bridge::vote_withdraw(RuntimeOrigin::signed(10), receipt_id, receipt.clone()),
			Error::<Test>::Unauthorized
		);
		assert_noop!(
			Bridge::vote_withdraw(RuntimeOrigin::signed(3), receipt_id, receipt),
			Error::<Test>::Unauthorized
		);
	});
}

#[test]
fn vote_fails_on_stopped_bridge() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1);
		assert_ok!(Bridge::set_state(RuntimeOrigin::signed(100), BridgeState::Stopped));
		assert_noop!(
			Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt),
			Error::<Test>::BridgeStopped
		);
	});
}

#[test]
fn vote_succeeds_even_after_reaching_required_votes() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1);
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(2), receipt_id, receipt));
	});
}

#[test]
fn vote_fails_on_processed_receipt() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(0), 2, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		System::set_block_number(11);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id));
		assert_noop!(
			Bridge::vote_withdraw(RuntimeOrigin::signed(2), receipt_id, receipt),
			Error::<Test>::AlreadyProcessed
		);
	});
}

#[test]
fn vote_stores_receipt_details() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1);
		assert_eq!(IncomingReceipts::<Test>::get(receipt_id), None);
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_eq!(IncomingReceipts::<Test>::get(receipt_id), Some(receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		assert_eq!(IncomingReceipts::<Test>::get(receipt_id), Some(receipt));
	});
}

#[test]
fn vote_stops_bridge_on_mismatched_receipts() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1);
		let (_, receipt2) = gen_receipt(1, 2);
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt2));
		System::assert_last_event(Event::<Test>::StateChanged(BridgeState::Stopped).into());
		assert_noop!(
			Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt),
			Error::<Test>::BridgeStopped
		);
		assert_eq!(State::<Test>::get(), BridgeState::Stopped);
	});
}

#[test]
fn voting_is_idempotent() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1);
		assert_eq!(Voting::<Test>::get(receipt_id), vec![]);
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_eq!(Voting::<Test>::get(receipt_id), vec![0]);
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_eq!(Voting::<Test>::get(receipt_id), vec![0]);
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		assert_eq!(Voting::<Test>::get(receipt_id), vec![0, 1]);
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_eq!(Voting::<Test>::get(receipt_id), vec![0, 1]);
	});
}

#[test]
fn voting_deposits_event() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1);
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		System::assert_last_event(
			Event::<Test>::Vote { relay: 0, receipt_id, block_number: 10 }.into(),
		);
	});
}

#[test]
fn voting_gracefully_handles_too_many_votes() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1);

		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(2), receipt_id, receipt.clone()));

		for i in 3..10 {
			assert_ok!(Bridge::add_relay(RuntimeOrigin::signed(101), i));
			assert_ok!(Bridge::vote_withdraw(
				RuntimeOrigin::signed(i),
				receipt_id,
				receipt.clone()
			));
		}
		assert_ok!(Bridge::remove_relay(RuntimeOrigin::signed(101), 0));
		assert_ok!(Bridge::add_relay(RuntimeOrigin::signed(101), 10));
		assert_noop!(
			Bridge::vote_withdraw(RuntimeOrigin::signed(10), receipt_id, receipt.clone()),
			Error::<Test>::TooManyVotes
		);
	});
}

#[test]
fn voting_sets_approved_status() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1);

		assert_eq!(StatusOf::<Test>::get(receipt_id), IncomingReceiptStatus::Voting);
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_eq!(StatusOf::<Test>::get(receipt_id), IncomingReceiptStatus::Voting);
		System::set_block_number(2);
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		assert_eq!(StatusOf::<Test>::get(receipt_id), IncomingReceiptStatus::Approved(2));
		System::set_block_number(3);
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(2), receipt_id, receipt.clone()));
		assert_eq!(StatusOf::<Test>::get(receipt_id), IncomingReceiptStatus::Approved(2));
	});
}

/* WITHDRAWALS */

#[test]
fn withdraw_fails_on_nonexistent_receipt() {
	new_test_ext().execute_with(|| {
		let (receipt_id, _) = gen_receipt(0, 1);
		System::set_block_number(11);
		assert_noop!(
			Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id),
			Error::<Test>::UnknownReceiptId
		);
	});
}

#[test]
fn withdraw_fails_on_not_enough_votes() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1);
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt));
		System::set_block_number(11);
		assert_noop!(
			Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id),
			Error::<Test>::NotApproved
		);
	});
}

#[test]
fn withdraw_fails_on_stopped_bridge() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1);
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt));
		assert_ok!(Bridge::set_state(RuntimeOrigin::signed(100), BridgeState::Stopped));
		System::set_block_number(11);
		assert_noop!(
			Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id),
			Error::<Test>::BridgeStopped
		);
	});
}

#[test]
fn withdraw_deposits_event() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(0), 10, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		System::set_block_number(11);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id));
		System::assert_last_event(Event::<Test>::Processed(receipt_id).into());
	});
}

#[test]
fn withdraw_takes_tokens_from_bridge() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(0), 10, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		System::set_block_number(11);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id));
		assert_eq!(Balances::free_balance(bridge_wallet()), 9);
	});
}

#[test]
fn withdraw_sends_tokens_to_recipient() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(50, 1);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(0), 10, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		System::set_block_number(11);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id));
		assert_eq!(Balances::free_balance(50), 1);
	});
}

#[test]
fn withdraw_fails_on_broke_caller() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(50, 1);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(0), 10, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		System::set_block_number(11);
		assert_noop!(
			Bridge::withdraw(RuntimeOrigin::signed(50), receipt_id),
			TokenError::FundsUnavailable,
		);
	});
}

#[test]
fn withdraw_takes_fee_from_caller() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(50, 1);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(0), 10, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		System::set_block_number(11);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(4), receipt_id));
		assert_eq!(Balances::free_balance(4), 96);
	});
}

#[test]
fn withdraw_distributes_rewards_to_relays_that_voted() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(50, 1);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(4), 10, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		System::set_block_number(11);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(4), receipt_id));
		assert_eq!(Balances::free_balance(0), 102);
		assert_eq!(Balances::free_balance(1), 102);
		assert_eq!(Balances::free_balance(3), 100); // didnt vote
	});
}

#[test]
fn withdraw_updates_receipt_status() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(50, 1);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(4), 10, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		assert_eq!(StatusOf::<Test>::get(receipt_id), IncomingReceiptStatus::Approved(1));
		System::set_block_number(11);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(4), receipt_id));
		assert_eq!(StatusOf::<Test>::get(receipt_id), IncomingReceiptStatus::Processed(11));
	});
}

#[test]
fn withdraw_works_only_once() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(50, 1);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(4), 10, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		System::set_block_number(11);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(4), receipt_id));
		assert_noop!(
			Bridge::withdraw(RuntimeOrigin::signed(4), receipt_id),
			Error::<Test>::AlreadyProcessed
		);
	});
}

#[test]
fn withdrawal_delay_is_enforced_correctly() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(50, 1);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(4), 10, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		System::set_block_number(15);
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		System::set_block_number(24);
		assert_noop!(
			Bridge::withdraw(RuntimeOrigin::signed(4), receipt_id),
			Error::<Test>::TooSoon
		);
		System::set_block_number(25);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(4), receipt_id));
	});
}

#[test]
fn withdrawal_delay_is_enforced_correctly_with_votes_after_approval() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(50, 1);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(4), 10, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		System::set_block_number(15);
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		System::set_block_number(20);
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(2), receipt_id, receipt.clone()));
		System::set_block_number(24);
		assert_noop!(
			Bridge::withdraw(RuntimeOrigin::signed(4), receipt_id),
			Error::<Test>::TooSoon
		);
		System::set_block_number(25);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(4), receipt_id));
	});
}

/* Watcher can stop */

#[test]
fn emergency_stop_checks_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Bridge::emergency_stop(RuntimeOrigin::signed(10)),
			Error::<Test>::Unauthorized
		);
		assert_noop!(Bridge::emergency_stop(RuntimeOrigin::root()), BadOrigin);
		assert_ok!(Bridge::emergency_stop(RuntimeOrigin::signed(0)));
	});
}

#[test]
fn emergency_stop_deposits_both_events() {
	new_test_ext().execute_with(|| {
		assert_ok!(Bridge::emergency_stop(RuntimeOrigin::signed(0)));
		System::assert_has_event(Event::<Test>::StateChanged(BridgeState::Stopped).into());
		System::assert_has_event(Event::<Test>::EmergencyStop.into());
	});
}

#[test]
fn emergency_stop_works() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 10);
		assert_ok!(Bridge::emergency_stop(RuntimeOrigin::signed(0)));
		assert_noop!(
			Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt),
			Error::<Test>::BridgeStopped
		);
		assert_eq!(State::<Test>::get(), BridgeState::Stopped);
	});
}

/* Admin stuff */

#[test]
fn set_fee_checks_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(Bridge::set_fee(RuntimeOrigin::signed(0), 10), Error::<Test>::Unauthorized);
		assert_ok!(Bridge::set_fee(RuntimeOrigin::signed(100), 10));
		assert_ok!(Bridge::set_fee(RuntimeOrigin::signed(101), 10));
	});
}

#[test]
fn set_fee_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Fee::<Test>::get(), 4);
		assert_ok!(Bridge::set_fee(RuntimeOrigin::signed(100), 100));
		assert_eq!(Fee::<Test>::get(), 100);
	});
}

#[test]
fn set_fee_respects_min_max() {
	new_test_ext().execute_with(|| {
		assert_noop!(Bridge::set_fee(RuntimeOrigin::signed(100), 9), Error::<Test>::InvalidValue);
		assert_noop!(Bridge::set_fee(RuntimeOrigin::signed(100), 101), Error::<Test>::InvalidValue);
	});
}

#[test]
fn remove_relay_checks_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Bridge::remove_relay(RuntimeOrigin::signed(0), 10),
			Error::<Test>::Unauthorized
		);
		assert_ok!(Bridge::remove_relay(RuntimeOrigin::signed(100), 1));
		assert_ok!(Bridge::remove_relay(RuntimeOrigin::signed(101), 2));
	});
}

#[test]
fn remove_relay_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Relays::<Test>::get(), vec![0, 1, 2]);
		assert_ok!(Bridge::remove_relay(RuntimeOrigin::signed(101), 1));
		assert_eq!(Relays::<Test>::get(), vec![0, 2]);
		assert_ok!(Bridge::remove_relay(RuntimeOrigin::signed(101), 0));
		assert_eq!(Relays::<Test>::get(), vec![2]);
		assert_ok!(Bridge::remove_relay(RuntimeOrigin::signed(101), 2));
		assert_eq!(Relays::<Test>::get(), vec![]);
	});
}

#[test]
fn remove_relay_fails_on_nonexistent() {
	new_test_ext().execute_with(|| {
		assert_ok!(Bridge::remove_relay(RuntimeOrigin::signed(101), 1));
		assert_noop!(
			Bridge::remove_relay(RuntimeOrigin::signed(101), 1),
			Error::<Test>::InvalidRelay
		);
		assert_noop!(
			Bridge::remove_relay(RuntimeOrigin::signed(101), 3),
			Error::<Test>::InvalidRelay
		);
	});
}

#[test]
fn add_watcher_checks_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(Bridge::add_watcher(RuntimeOrigin::signed(0), 5), Error::<Test>::Unauthorized);
		assert_ok!(Bridge::add_watcher(RuntimeOrigin::signed(100), 6));
		assert_ok!(Bridge::add_watcher(RuntimeOrigin::signed(101), 7));
	});
}

#[test]
fn add_watcher_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Watchers::<Test>::get(), vec![0, 3]);
		assert_ok!(Bridge::add_watcher(RuntimeOrigin::signed(101), 1));
		assert_eq!(Watchers::<Test>::get(), vec![0, 3, 1]);
		assert_ok!(Bridge::add_watcher(RuntimeOrigin::signed(101), 2));
		assert_eq!(Watchers::<Test>::get(), vec![0, 3, 1, 2]);
	});
}

#[test]
fn add_watcher_fails_on_existing() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Bridge::add_watcher(RuntimeOrigin::signed(101), 0),
			Error::<Test>::AlreadyExists
		);
		assert_noop!(
			Bridge::add_watcher(RuntimeOrigin::signed(101), 3),
			Error::<Test>::AlreadyExists
		);
	});
}

#[test]
fn add_watcher_respects_max_relays() {
	new_test_ext().execute_with(|| {
		assert_ok!(Bridge::add_watcher(RuntimeOrigin::signed(101), 1));
		assert_ok!(Bridge::add_watcher(RuntimeOrigin::signed(101), 2));
		for i in 4..10 {
			assert_ok!(Bridge::add_watcher(RuntimeOrigin::signed(101), i));
		}
		assert_noop!(
			Bridge::add_watcher(RuntimeOrigin::signed(101), 50),
			Error::<Test>::TooManyWatchers
		);
	});
}

#[test]
fn set_state_checks_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Bridge::set_state(RuntimeOrigin::signed(0), BridgeState::Stopped),
			Error::<Test>::Unauthorized
		);
		assert_ok!(Bridge::set_state(RuntimeOrigin::signed(100), BridgeState::Stopped));
		assert_ok!(Bridge::set_state(RuntimeOrigin::signed(101), BridgeState::Stopped));
	});
}

#[test]
fn set_state_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Bridge::set_state(RuntimeOrigin::signed(100), BridgeState::Stopped));
		assert_eq!(State::<Test>::get(), BridgeState::Stopped);
		assert_ok!(Bridge::set_state(RuntimeOrigin::signed(100), BridgeState::Active));
		assert_eq!(State::<Test>::get(), BridgeState::Active);
		assert_ok!(Bridge::set_state(RuntimeOrigin::signed(100), BridgeState::Stopped));
		assert_eq!(State::<Test>::get(), BridgeState::Stopped);
	});
}

#[test]
fn set_admin_checks_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(Bridge::set_admin(RuntimeOrigin::signed(0), 0), Error::<Test>::Unauthorized);
		assert_ok!(Bridge::set_admin(RuntimeOrigin::signed(100), 0));
		assert_ok!(Bridge::set_admin(RuntimeOrigin::signed(101), 1));
		assert_ok!(Bridge::set_admin(RuntimeOrigin::root(), 2));
	});
}

#[test]
fn set_admin_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Bridge::set_admin(RuntimeOrigin::root(), 0));
		assert_eq!(Admin::<Test>::get(), Some(0));
		assert_ok!(Bridge::set_admin(RuntimeOrigin::root(), 1));
		assert_eq!(Admin::<Test>::get(), Some(1));
	});
}

/* Superadmin stuff */

#[test]
fn set_votes_required_checks_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Bridge::set_votes_required(RuntimeOrigin::signed(0), 10),
			Error::<Test>::Unauthorized
		);
		assert_noop!(
			Bridge::set_votes_required(RuntimeOrigin::signed(100), 10),
			Error::<Test>::Unauthorized
		);
		assert_ok!(Bridge::set_votes_required(RuntimeOrigin::signed(101), 10));
	});
}

#[test]
fn set_votes_required_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(VotesRequired::<Test>::get(), 2);
		assert_ok!(Bridge::set_votes_required(RuntimeOrigin::signed(101), 100));
		assert_eq!(VotesRequired::<Test>::get(), 100);
	});
}

#[test]
fn set_votes_required_respects_minimum() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Bridge::set_votes_required(RuntimeOrigin::signed(101), 1),
			Error::<Test>::InvalidValue
		);
	});
}

#[test]
fn add_relay_checks_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(Bridge::add_relay(RuntimeOrigin::signed(0), 5), Error::<Test>::Unauthorized);
		assert_noop!(Bridge::add_relay(RuntimeOrigin::signed(100), 5), Error::<Test>::Unauthorized);
		assert_ok!(Bridge::add_relay(RuntimeOrigin::signed(101), 5));
	});
}

#[test]
fn add_relay_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Relays::<Test>::get(), vec![0, 1, 2]);
		assert_ok!(Bridge::add_relay(RuntimeOrigin::signed(101), 5));
		assert_eq!(Relays::<Test>::get(), vec![0, 1, 2, 5]);
		assert_ok!(Bridge::add_relay(RuntimeOrigin::signed(101), 3));
		assert_eq!(Relays::<Test>::get(), vec![0, 1, 2, 5, 3]);
	});
}

#[test]
fn add_relay_fails_on_existing() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Bridge::add_relay(RuntimeOrigin::signed(101), 0),
			Error::<Test>::AlreadyExists
		);
		assert_noop!(
			Bridge::add_relay(RuntimeOrigin::signed(101), 2),
			Error::<Test>::AlreadyExists
		);
	});
}

#[test]
fn add_relay_respects_max_relays() {
	new_test_ext().execute_with(|| {
		for i in 3..10 {
			assert_ok!(Bridge::add_relay(RuntimeOrigin::signed(101), i));
		}
		assert_noop!(
			Bridge::add_relay(RuntimeOrigin::signed(101), 50),
			Error::<Test>::TooManyRelays
		);
	});
}

#[test]
fn remove_watcher_checks_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Bridge::remove_watcher(RuntimeOrigin::signed(0), 10),
			Error::<Test>::Unauthorized
		);
		assert_noop!(
			Bridge::remove_watcher(RuntimeOrigin::signed(100), 1),
			Error::<Test>::Unauthorized
		);
		assert_ok!(Bridge::remove_watcher(RuntimeOrigin::signed(101), 0));
	});
}

#[test]
fn remove_watcher_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Watchers::<Test>::get(), vec![0, 3]);
		assert_ok!(Bridge::remove_watcher(RuntimeOrigin::signed(101), 3));
		assert_eq!(Watchers::<Test>::get(), vec![0]);
		assert_ok!(Bridge::remove_watcher(RuntimeOrigin::signed(101), 0));
		assert_eq!(Watchers::<Test>::get(), vec![]);
	});
}

#[test]
fn remove_watcher_fails_on_nonexistent() {
	new_test_ext().execute_with(|| {
		assert_ok!(Bridge::remove_watcher(RuntimeOrigin::signed(101), 0));
		assert_noop!(
			Bridge::remove_watcher(RuntimeOrigin::signed(101), 0),
			Error::<Test>::InvalidWatcher
		);
		assert_noop!(
			Bridge::remove_watcher(RuntimeOrigin::signed(101), 1),
			Error::<Test>::InvalidWatcher
		);
	});
}

#[test]
fn set_super_admin_checks_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Bridge::set_super_admin(RuntimeOrigin::signed(0), 0),
			Error::<Test>::Unauthorized
		);
		assert_noop!(
			Bridge::set_super_admin(RuntimeOrigin::signed(100), 0),
			Error::<Test>::Unauthorized
		);
		assert_ok!(Bridge::set_super_admin(RuntimeOrigin::signed(101), 1));
		assert_ok!(Bridge::set_super_admin(RuntimeOrigin::root(), 2));
	});
}

#[test]
fn set_super_admin_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Bridge::set_super_admin(RuntimeOrigin::root(), 0));
		assert_eq!(SuperAdmin::<Test>::get(), Some(0));
		assert_ok!(Bridge::set_super_admin(RuntimeOrigin::root(), 1));
		assert_eq!(SuperAdmin::<Test>::get(), Some(1));
	});
}

/* RATE LIMIT */

#[test]
fn rate_limit_doesnt_prevent_single_transaction_right_at_limit() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1000);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(200), 1000, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		System::set_block_number(11);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id));
	});
}

#[test]
fn rate_limit_doesnt_multiple_transactions_right_at_limit() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 500);
		let (receipt_id2, receipt2) = gen_receipt(1, 500);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(200), 1000, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id2, receipt2.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id2, receipt2.clone()));

		System::set_block_number(11);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id));
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id2));
	});
}

#[test]
fn rate_limit_allows_using_up_decayed_amount_immediately() {
	new_test_ext().execute_with(|| {
		// our decay rate in mock is 10 tokens per block
		let (receipt_id, receipt) = gen_receipt(0, 1000);
		let (receipt_id2, receipt2) = gen_receipt(1, 10);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(200), 10000, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id2, receipt2.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id2, receipt2.clone()));

		System::set_block_number(11);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id));
		System::set_block_number(12);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id2));
	});
}

#[test]
fn rate_limit_goes_to_zero_after_window() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1000);
		let (receipt_id2, receipt2) = gen_receipt(1, 1000);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(200), 10000, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id2, receipt2.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id2, receipt2.clone()));

		System::set_block_number(11);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id));
		System::set_block_number(111);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id2));
	});
}

#[test]
fn rate_limit_prevents_single_big_withdrawals() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1001);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(200), 10000, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		System::set_block_number(11);
		assert_noop!(
			Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id),
			Error::<Test>::RateLimited
		);
	});
}

#[test]
fn rate_limit_prevents_multiple_withdrawals_over_limit_in_single_block() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 501);
		let (receipt_id2, receipt2) = gen_receipt(1, 500);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(200), 10000, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id2, receipt2.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id2, receipt2.clone()));

		System::set_block_number(11);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id));
		assert_noop!(
			Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id2),
			Error::<Test>::RateLimited
		);
	});
}

#[test]
fn rate_limit_respects_decay() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(0, 1000);
		let (receipt_id2, receipt2) = gen_receipt(1, 500);
		let (receipt_id3, receipt3) = gen_receipt(2, 501);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(200), 10000, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id2, receipt2.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id2, receipt2.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id3, receipt3.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id3, receipt3.clone()));

		System::set_block_number(100);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id));
		System::set_block_number(149);
		assert_noop!(
			Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id2),
			Error::<Test>::RateLimited
		);
		System::set_block_number(150);
		assert_noop!(
			Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id3),
			Error::<Test>::RateLimited
		);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id2));
	});
}

/* OTHER */
#[test]
fn max_total_locked_is_respected_after_withdrawals() {
	new_test_ext().execute_with(|| {
		let (receipt_id, receipt) = gen_receipt(200, 1000);
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(200), 1000, eth_recipient(0)));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(0), receipt_id, receipt.clone()));
		assert_ok!(Bridge::vote_withdraw(RuntimeOrigin::signed(1), receipt_id, receipt.clone()));

		System::set_block_number(200);
		assert_noop!(
			Bridge::deposit(RuntimeOrigin::signed(0), 9001, eth_recipient(0)),
			Error::<Test>::TooMuchLocked
		);
		assert_ok!(Bridge::withdraw(RuntimeOrigin::signed(0), receipt_id));
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(200), 9001, eth_recipient(0)));
		assert_ok!(Bridge::deposit(RuntimeOrigin::signed(200), 998, eth_recipient(0)));
		assert_noop!(
			Bridge::deposit(RuntimeOrigin::signed(0), 2, eth_recipient(0)),
			Error::<Test>::TooMuchLocked
		);
	});
}
