use crate::{
	mock::*,
	traits::{CitizenshipChecker, LLM as LLMTrait},
	Electionlock, ElectionlockDuration, Error, Event, LLMPolitics, NextRelease, Withdrawlock,
	WithdrawlockDuration,
};
use frame_support::{assert_noop, assert_ok, traits::OnInitialize};
use hex_literal::hex;

type AssetsError<T> = pallet_assets::Error<T>;

#[test]
fn creates_llm_on_first_block() {
	new_test_ext().execute_with(|| {
		let id = LLM::llm_id();
		let treasury = LLM::get_llm_treasury_account();
		let vault = LLM::get_llm_vault_account();
		let initialized_to_citizens = Assets::balance(id, 1) + Assets::balance(id, 2);

		assert_eq!(Assets::maybe_total_supply(id), Some(TOTALLLM::get()));
		assert_eq!(Assets::maybe_balance(id, treasury), Some(PRERELEASELLM::get()));
		assert_eq!(
			Assets::maybe_balance(id, vault),
			Some(TOTALLLM::get() - PRERELEASELLM::get() - initialized_to_citizens)
		);
	});
}

#[test]
fn releases_on_future_block() {
	new_test_ext().execute_with(|| {
		let id = LLM::llm_id();
		let treasury = LLM::get_llm_treasury_account();
		let vault = LLM::get_llm_vault_account();
		let next_block = NextRelease::<Test>::get();
		let treasury_before = Assets::balance(id, treasury);
		let vault_before = Assets::balance(id, vault);

		System::set_block_number(next_block);
		LLM::on_initialize(next_block);

		let expected_release = vault_before / 10;
		let treasury_change = Assets::balance(id, treasury) - treasury_before;
		let vault_change = vault_before - Assets::balance(id, vault);

		assert!(next_block != NextRelease::<Test>::get());
		assert_eq!(treasury_change, expected_release);
		assert_eq!(vault_change, expected_release);
		assert_eq!(Assets::total_supply(id), TOTALLLM::get());

		System::assert_has_event(Event::ReleasedLLM(treasury, expected_release).into());
	});
}

#[test]
fn send_llm_calls_assets() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(1);
		assert_ok!(LLM::send_llm(origin.clone(), 2, 9));
		System::assert_has_event(
			pallet_assets::Event::Transferred { asset_id: 1, from: 1, to: 2, amount: 9 }.into(),
		);
	});
}

#[test]
fn cant_politics_lock_more_than_balance() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(1);
		assert_noop!(LLM::politics_lock(origin.clone(), 6001), AssetsError::<Test>::BalanceLow);
		assert_ok!(LLM::politics_lock(origin.clone(), 5999));
		assert_noop!(LLM::politics_lock(origin.clone(), 2), AssetsError::<Test>::BalanceLow);
		assert_ok!(LLM::politics_lock(origin.clone(), 1));
	});
}

#[test]
fn politics_lock_deposits_event() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(1);
		assert_ok!(LLM::politics_lock(origin.clone(), 9));
		System::assert_last_event(Event::LLMPoliticsLocked(1, 9).into());
	});
}

#[test]
fn politics_lock_properly_updates_balances() {
	new_test_ext().execute_with(|| {
		let id = LLM::llm_id();
		let politipool = LLM::get_llm_politipool_account();
		let origin = RuntimeOrigin::signed(1);
		let origin2 = RuntimeOrigin::signed(2);

		assert_ok!(LLM::politics_lock(origin.clone(), 4));
		assert_eq!(LLMPolitics::<Test>::get(1), 4);
		assert_eq!(Assets::balance(id, 1), 5996);
		assert_eq!(Assets::balance(id, politipool), 4);

		assert_ok!(LLM::politics_lock(origin.clone(), 4));
		assert_eq!(LLMPolitics::<Test>::get(1), 8);
		assert_eq!(Assets::balance(id, 1), 5992);
		assert_eq!(Assets::balance(id, politipool), 8);

		assert_ok!(LLM::politics_lock(origin.clone(), 2));
		assert_eq!(LLMPolitics::<Test>::get(1), 10);
		assert_eq!(Assets::balance(id, 1), 5990);
		assert_eq!(Assets::balance(id, politipool), 10);

		assert_ok!(LLM::politics_lock(origin2.clone(), 20));
		assert_eq!(LLMPolitics::<Test>::get(2), 20);
		assert_eq!(Assets::balance(id, 2), 5980);
		assert_eq!(Assets::balance(id, politipool), 30);
	});
}

#[test]
fn cant_politics_unlock_if_never_locked() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(1);
		assert_noop!(LLM::politics_unlock(origin.clone()), Error::<Test>::InvalidAccount);
	});
}

#[test]
fn cant_politics_unlock_if_withdraw_locked() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(1);
		assert_ok!(LLM::politics_lock(origin.clone(), 10));
		assert_ok!(LLM::politics_unlock(origin.clone()));
		assert_noop!(LLM::politics_unlock(origin.clone()), Error::<Test>::Gottawait);
		System::set_block_number(Withdrawlock::<Test>::get(1) + 1);
		assert_ok!(LLM::politics_unlock(origin.clone()));
		assert_noop!(LLM::politics_unlock(origin.clone()), Error::<Test>::Gottawait);
	});
}

#[test]
fn politics_unlock_deposits_event() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(1);
		assert_ok!(LLM::politics_lock(origin.clone(), 10));
		assert_ok!(LLM::politics_unlock(origin.clone()));
		System::assert_last_event(Event::LLMPoliticsUnlocked(1, 1).into());
	});
}

#[test]
fn politics_unlock_releases_10_percent() {
	new_test_ext().execute_with(|| {
		let id = LLM::llm_id();
		let politipool = LLM::get_llm_politipool_account();
		let origin = RuntimeOrigin::signed(2);

		assert_ok!(LLM::politics_lock(origin.clone(), 5000));
		assert_ok!(LLM::politics_unlock(origin.clone()));

		assert_eq!(Assets::balance(id, 2), 1000 + 500);
		assert_eq!(LLMPolitics::<Test>::get(2), 5000 - 500);
		assert_eq!(Assets::balance(id, politipool), 5000 - 500);

		System::set_block_number(Withdrawlock::<Test>::get(2) + 1);
		assert_ok!(LLM::politics_unlock(origin.clone()));
		assert_eq!(Assets::balance(id, 2), 1500 + 450);
		assert_eq!(LLMPolitics::<Test>::get(2), 4500 - 450);
		assert_eq!(Assets::balance(id, politipool), 4500 - 450);
	});
}

#[test]
fn only_approved_accounts_can_call_treasury_llm_transfer() {
	new_test_ext().execute_with(|| {
		let unapproved = RuntimeOrigin::signed(1);
		let approved = RuntimeOrigin::signed(LLM::account_id32_to_accountid(
			hex!["91c7c2ea588cc63a45a540d4f2dbbae7967d415d0daec3d6a5a0641e969c635c"].into(),
		));

		assert_noop!(LLM::treasury_llm_transfer(unapproved, 1, 1), Error::<Test>::InvalidAccount);
		assert_ok!(LLM::treasury_llm_transfer(approved, 1, 1));
	});
}

#[test]
fn treasury_llm_transfer_calls_assets() {
	new_test_ext().execute_with(|| {
		let approved = RuntimeOrigin::signed(LLM::account_id32_to_accountid(
			hex!["91c7c2ea588cc63a45a540d4f2dbbae7967d415d0daec3d6a5a0641e969c635c"].into(),
		));
		let id = LLM::llm_id();
		let treasury = LLM::get_llm_treasury_account();
		assert_ok!(LLM::treasury_llm_transfer(approved.clone(), 1, 10));
		System::assert_has_event(
			pallet_assets::Event::Transferred { asset_id: id, from: treasury, to: 1, amount: 10 }
				.into(),
		);
	});
}

#[test]
fn sets_locks_durations_on_genesis() {
	new_test_ext().execute_with(|| {
		assert_eq!(WithdrawlockDuration::<Test>::get(), 180);
		assert_eq!(ElectionlockDuration::<Test>::get(), 190);
	});
}

#[test]
fn check_pooled_llm_works() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(1);
		assert_eq!(LLM::check_pooled_llm(&1), false);
		assert_ok!(LLM::politics_lock(origin.clone(), 4999));
		assert_eq!(LLM::check_pooled_llm(&1), false);
		assert_ok!(LLM::politics_lock(origin.clone(), 1));
		assert_eq!(LLM::check_pooled_llm(&1), true);
		assert_ok!(LLM::politics_unlock(origin.clone()));
		assert_eq!(LLM::check_pooled_llm(&1), false);
	});
}

#[test]
fn is_election_unlocked_works() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(1);
		assert_eq!(LLM::is_election_unlocked(&1), true);
		assert_ok!(LLM::politics_lock(origin.clone(), 10));
		assert_eq!(LLM::is_election_unlocked(&1), true);
		assert_ok!(LLM::politics_unlock(origin.clone()));
		assert_eq!(LLM::is_election_unlocked(&1), false);
		System::set_block_number(Electionlock::<Test>::get(1));
		assert_eq!(LLM::is_election_unlocked(&1), false);
		System::set_block_number(Electionlock::<Test>::get(1) + 1);
		assert_eq!(LLM::is_election_unlocked(&1), true);
		assert_ok!(LLM::politics_unlock(origin.clone()));
		assert_eq!(LLM::is_election_unlocked(&1), false);
	});
}

#[test]
fn get_politi_pooled_amount_works() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(1);
		let origin2 = RuntimeOrigin::signed(2);

		assert_ok!(LLM::politics_lock(origin.clone(), 4));
		assert_eq!(LLM::get_politi_pooled_amount(), 4);

		assert_ok!(LLM::politics_lock(origin.clone(), 4));
		assert_eq!(LLM::get_politi_pooled_amount(), 8);

		assert_ok!(LLM::politics_lock(origin.clone(), 2));
		assert_eq!(LLM::get_politi_pooled_amount(), 10);

		assert_ok!(LLM::politics_lock(origin2.clone(), 20));
		assert_eq!(LLM::get_politi_pooled_amount(), 30);

		assert_ok!(LLM::politics_unlock(origin2.clone()));
		assert_eq!(LLM::get_politi_pooled_amount(), 28);

		assert_ok!(LLM::politics_unlock(origin.clone()));
		assert_eq!(LLM::get_politi_pooled_amount(), 27);
	});
}

#[test]
fn get_llm_politics_works() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(1);
		let origin2 = RuntimeOrigin::signed(2);

		assert_ok!(LLM::politics_lock(origin.clone(), 4));
		assert_eq!(LLM::get_llm_politics(&1), 4);

		assert_ok!(LLM::politics_lock(origin.clone(), 4));
		assert_eq!(LLM::get_llm_politics(&1), 8);

		assert_ok!(LLM::politics_lock(origin.clone(), 2));
		assert_eq!(LLM::get_llm_politics(&1), 10);

		assert_ok!(LLM::politics_lock(origin2.clone(), 20));
		assert_eq!(LLM::get_llm_politics(&2), 20);

		assert_ok!(LLM::politics_unlock(origin2.clone()));
		assert_eq!(LLM::get_llm_politics(&2), 18);

		assert_ok!(LLM::politics_unlock(origin.clone()));
		assert_eq!(LLM::get_llm_politics(&1), 9);
	});
}

#[test]
fn ensure_politics_allowed_fails_for_noncitizen() {
	new_test_ext().execute_with(|| {
		assert_noop!(LLM::ensure_politics_allowed(&10), Error::<Test>::NonCitizen);
	});
}

#[test]
fn ensure_politics_allowed_fails_for_locked_election_rights() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(1);
		assert_ok!(LLM::politics_lock(origin.clone(), 6000));
		assert_ok!(LLM::politics_unlock(origin.clone()));
		assert_noop!(LLM::ensure_politics_allowed(&1), Error::<Test>::Locked);
		System::set_block_number(Electionlock::<Test>::get(1));
		assert_noop!(LLM::ensure_politics_allowed(&1), Error::<Test>::Locked);
		System::set_block_number(Electionlock::<Test>::get(1) + 1);
		assert_ok!(LLM::ensure_politics_allowed(&1));
	});
}

#[test]
fn ensure_politics_allowed_fails_for_no_pooled_llm() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(1);
		assert_ok!(LLM::politics_lock(origin.clone(), 4999));
		assert_noop!(LLM::ensure_politics_allowed(&1), Error::<Test>::NoPolLLM);
	});
}

#[test]
fn ensure_politics_allowed_succeeds_for_valid_citizen() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(1);
		assert_ok!(LLM::politics_lock(origin.clone(), 5000));
		assert_ok!(LLM::ensure_politics_allowed(&1));
	});
}

#[test]
fn releases_correct_amounts() {
	new_test_ext().execute_with(|| {
		let id = LLM::llm_id();
		let treasury = LLM::get_llm_treasury_account();
		let vault = LLM::get_llm_vault_account();

		// undo fake sends
		Assets::transfer(RuntimeOrigin::signed(1), 1, vault, 6000).unwrap();
		Assets::transfer(RuntimeOrigin::signed(2), 1, vault, 6000).unwrap();

		assert_eq!(Assets::balance(id, treasury), 7_000_000);
		assert_eq!(Assets::balance(id, vault), 63_000_000);

		let expected = [13_300_000, 18_970_000, 24_073_000, 28_665_700];

		for expected_treasury_balance in expected {
			let next_block = NextRelease::<Test>::get();
			System::set_block_number(next_block);
			LLM::on_initialize(next_block);

			assert_eq!(Assets::balance(id, treasury), expected_treasury_balance);
			assert_eq!(Assets::balance(id, vault), 70_000_000 - expected_treasury_balance);
			assert_eq!(Assets::total_supply(id), TOTALLLM::get());
		}
	});
}
