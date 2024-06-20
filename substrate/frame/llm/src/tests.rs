#![cfg(test)]

use crate::{
	mock::*, Config, Courts, Electionlock, ElectionlockDuration, Error, Event, LLMAccount,
	LLMPolitics, LastRelease, RemarkData, Withdrawlock, WithdrawlockDuration,
};
use codec::Compact;
use frame_support::{
	assert_noop, assert_ok,
	error::BadOrigin,
	traits::{
		tokens::{
			fungible::{Inspect, Mutate},
			Preservation,
		},
		OnInitialize,
	},
};
use liberland_traits::{CitizenshipChecker, LLM as LLMTrait};
use pallet_identity::{Data, IdentityInfo};
use sp_runtime::traits::{BlakeTwo256, Hash};

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
		let next_block =
			LastRelease::<Test>::get() + <Test as Config>::InflationEventInterval::get();
		let treasury_before = Assets::balance(id, treasury);
		let vault_before = Assets::balance(id, vault);

		System::set_block_number(next_block);
		LLM::on_initialize(next_block);

		let factor = <Test as Config>::InflationEventReleaseFactor::get();
		let expected_release = factor.mul_floor(vault_before);
		let treasury_change = Assets::balance(id, treasury) - treasury_before;
		let vault_change = vault_before - Assets::balance(id, vault);

		assert!(next_block == LastRelease::<Test>::get());
		assert_eq!(treasury_change, expected_release);
		assert_eq!(vault_change, expected_release);
		assert_eq!(Assets::total_supply(id), TOTALLLM::get());

		System::assert_has_event(Event::ReleasedLLM(treasury, expected_release).into());
	});
}

#[test]
fn releases_multiple_times_if_behind() {
	new_test_ext().execute_with(|| {
		let id = LLM::llm_id();
		let treasury = LLM::get_llm_treasury_account();
		let vault = LLM::get_llm_vault_account();
		let last_release = LastRelease::<Test>::get();
		let interval = <Test as Config>::InflationEventInterval::get();

		let mut future_block = last_release + interval * 5;

		for i in 1..=5 {
			let treasury_before = Assets::balance(id, treasury);
			let vault_before = Assets::balance(id, vault);

			System::set_block_number(future_block);
			LLM::on_initialize(future_block);

			let factor = <Test as Config>::InflationEventReleaseFactor::get();
			let expected_release = factor.mul_floor(vault_before);
			let treasury_change = Assets::balance(id, treasury) - treasury_before;
			let vault_change = vault_before - Assets::balance(id, vault);

			assert_eq!(LastRelease::<Test>::get(), last_release + interval * i);
			assert_eq!(treasury_change, expected_release);
			assert_eq!(vault_change, expected_release);
			assert_eq!(Assets::total_supply(id), TOTALLLM::get());
			System::assert_has_event(Event::ReleasedLLM(treasury, expected_release).into());

			future_block += 1;
		}

		let vault_before = Assets::balance(id, vault);
		System::set_block_number(future_block);
		LLM::on_initialize(future_block);
		let vault_change = vault_before - Assets::balance(id, vault);
		assert_eq!(vault_change, 0u64);
		assert_eq!(LastRelease::<Test>::get(), last_release + interval * 5);
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
fn send_llm_to_politipool_locks_llm() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(1);
		let id = LLM::llm_id();
		let politipool = LLM::get_llm_politipool_account();
		assert_ok!(LLM::send_llm_to_politipool(origin, 2, 10));
		System::assert_has_event(
			pallet_assets::Event::Transferred { asset_id: id, from: 1, to: 2, amount: 10 }.into(),
		);
		System::assert_has_event(
			pallet_assets::Event::Transferred { asset_id: id, from: 2, to: politipool, amount: 10 }
				.into(),
		);
		System::assert_last_event(Event::LLMPoliticsLocked(2, 10).into());
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
		assert_ok!(LLM::politics_lock(origin.clone(), 1000));
		assert_ok!(LLM::politics_unlock(origin.clone()));
		System::assert_last_event(Event::LLMPoliticsUnlocked(1, 8).into());
	});
}

#[test]
fn politics_unlock_releases_dot8742_percent() {
	new_test_ext().execute_with(|| {
		let id = LLM::llm_id();
		let politipool = LLM::get_llm_politipool_account();
		let origin = RuntimeOrigin::signed(2);

		assert_ok!(LLM::politics_lock(origin.clone(), 6000));
		assert_ok!(LLM::politics_unlock(origin.clone()));

		assert_eq!(Assets::balance(id, 2), 52);
		assert_eq!(LLMPolitics::<Test>::get(2), 6000 - 52);
		assert_eq!(Assets::balance(id, politipool), 6000 - 52);

		System::set_block_number(Withdrawlock::<Test>::get(2) + 1);
		assert_ok!(LLM::politics_unlock(origin.clone()));
		assert_eq!(Assets::balance(id, 2), 52 + 51);
		assert_eq!(LLMPolitics::<Test>::get(2), 6000 - 52 - 51);
		assert_eq!(Assets::balance(id, politipool), 6000 - 52 - 51);
	});
}

#[test]
fn only_approved_accounts_can_call_treasury_llm_transfer() {
	new_test_ext().execute_with(|| {
		let unapproved = RuntimeOrigin::signed(1);
		let approved = RuntimeOrigin::root();

		assert_noop!(LLM::treasury_llm_transfer(unapproved, 1, 1), BadOrigin);
		assert_ok!(LLM::treasury_llm_transfer(approved, 1, 1));
	});
}

#[test]
fn treasury_llm_transfer_calls_assets() {
	new_test_ext().execute_with(|| {
		let approved = RuntimeOrigin::root();
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
fn only_approved_accounts_can_call_treasury_llm_transfer_to_politipool() {
	new_test_ext().execute_with(|| {
		let unapproved = RuntimeOrigin::signed(1);
		let approved = RuntimeOrigin::root();

		assert_noop!(LLM::treasury_llm_transfer_to_politipool(unapproved, 1, 1), BadOrigin);
		assert_ok!(LLM::treasury_llm_transfer_to_politipool(approved, 1, 1));
	});
}

#[test]
fn treasury_llm_transfer_to_politipool_locks_funds() {
	new_test_ext().execute_with(|| {
		let approved = RuntimeOrigin::root();
		let id = LLM::llm_id();
		let treasury = LLM::get_llm_treasury_account();
		let politipool = LLM::get_llm_politipool_account();
		assert_ok!(LLM::treasury_llm_transfer_to_politipool(approved.clone(), 1, 10));
		System::assert_has_event(
			pallet_assets::Event::Transferred { asset_id: id, from: treasury, to: 1, amount: 10 }
				.into(),
		);
		System::assert_has_event(
			pallet_assets::Event::Transferred { asset_id: id, from: 1, to: politipool, amount: 10 }
				.into(),
		);
		System::assert_last_event(Event::LLMPoliticsLocked(1, 10).into());
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

		assert_ok!(LLM::politics_lock(origin2.clone(), 990));
		assert_eq!(LLM::get_politi_pooled_amount(), 1000);

		assert_ok!(LLM::politics_unlock(origin2.clone()));
		assert_eq!(LLM::get_politi_pooled_amount(), 992);

		assert_ok!(LLM::politics_unlock(origin.clone()));
		assert_eq!(LLM::get_politi_pooled_amount(), 992);
	});
}

#[test]
fn get_llm_politics_works() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(1);
		let origin2 = RuntimeOrigin::signed(2);

		assert_ok!(LLM::politics_lock(origin.clone(), 110));
		assert_eq!(LLM::get_llm_politics(&1), 110);

		assert_ok!(LLM::politics_lock(origin.clone(), 3));
		assert_eq!(LLM::get_llm_politics(&1), 113);

		assert_ok!(LLM::politics_lock(origin.clone(), 2));
		assert_eq!(LLM::get_llm_politics(&1), 115);

		assert_ok!(LLM::politics_lock(origin2.clone(), 1144));
		assert_eq!(LLM::get_llm_politics(&2), 1144);

		assert_ok!(LLM::politics_unlock(origin2.clone()));
		assert_eq!(LLM::get_llm_politics(&2), 1134);

		assert_ok!(LLM::politics_unlock(origin.clone()));
		assert_eq!(LLM::get_llm_politics(&1), 114);
	});
}

fn setup_identity(id: u64, citizen: bool, eligible_on: Option<Vec<u8>>, judgement: bool) {
	let data = Data::Raw(b"1".to_vec().try_into().unwrap());
	let mut additional = vec![];
	if let Some(n) = eligible_on {
		additional.push((
			Data::Raw(b"eligible_on".to_vec().try_into().unwrap()),
			Data::Raw(n.try_into().unwrap()),
		));
	};

	if citizen {
		additional.push((Data::Raw(b"citizen".to_vec().try_into().unwrap()), data.clone()));
	};

	let info = IdentityInfo {
		twitter: data.clone(),
		additional: additional.try_into().unwrap(),
		display: data.clone(),
		legal: data.clone(),
		web: data.clone(),
		riot: data.clone(),
		email: data.clone(),
		pgp_fingerprint: Some([0; 20]),
		image: data,
	};

	let o = RuntimeOrigin::signed(id);
	Identity::set_identity(o, Box::new(info.clone())).unwrap();
	if judgement {
		Identity::provide_judgement(
			RuntimeOrigin::signed(0),
			0,
			id,
			pallet_identity::Judgement::KnownGood,
			BlakeTwo256::hash_of(&info),
		)
		.unwrap();
	}
}

#[test]
fn ensure_politics_allowed_fails_for_noncitizen() {
	new_test_ext().execute_with(|| {
		// no judgement at all
		assert_noop!(LLM::ensure_politics_allowed(&10), Error::<Test>::NonCitizen);

		// judgment OK, eligible_on ok, but missing citizen field
		setup_identity(11, false, Some(vec![0u8]), true);
		assert_noop!(LLM::ensure_politics_allowed(&11), Error::<Test>::NonCitizen);

		// judgment OK, citizen ok, but missing eligible_on
		setup_identity(12, true, None, true);
		assert_noop!(LLM::ensure_politics_allowed(&12), Error::<Test>::NonCitizen);

		// judgment OK, citizen ok eligible_on set but in the future
		setup_identity(13, true, Some(vec![0x40, 0x42, 0x0F]), true);
		assert_noop!(LLM::ensure_politics_allowed(&13), Error::<Test>::NonCitizen);

		System::set_block_number(999_999); // still future
		assert_noop!(LLM::ensure_politics_allowed(&13), Error::<Test>::NonCitizen);

		assert_ok!(LLM::transfer_from_vault(13, 5000));
		assert_ok!(LLM::politics_lock(RuntimeOrigin::signed(13), 5000));
		System::set_block_number(1_000_000); // and its ok
		assert_ok!(LLM::ensure_politics_allowed(&13));
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

fn assert_approx_eq(amount1: u64, amount2: u64, epsilon: u64) {
	assert!(amount1 > amount2 - epsilon);
	assert!(amount1 < amount2 + epsilon);
}

#[test]
fn releases_correct_amounts() {
	new_test_ext().execute_with(|| {
		let id = LLM::llm_id();
		let treasury = LLM::get_llm_treasury_account();
		let vault = LLM::get_llm_vault_account();

		// undo fake sends
		Assets::transfer(RuntimeOrigin::signed(1), Compact(1), vault, 6000).unwrap();
		Assets::transfer(RuntimeOrigin::signed(2), Compact(1), vault, 6000).unwrap();

		assert_eq!(Assets::balance(id, treasury), 7_000_000);
		assert_eq!(Assets::balance(id, vault), 63_000_000);

		let expected = [13_300_000, 18_970_000, 24_073_000, 28_665_700];

		// we can be off by 1 grain per month due to rounding issues
		let epsilon = 12;
		let mut cumulative_epsilon = 0;

		for expected_treasury_balance in expected {
			for _ in 1..=12 {
				// trigger 12 release events, simulating 1 year
				let next_block =
					LastRelease::<Test>::get() + <Test as Config>::InflationEventInterval::get();
				System::set_block_number(next_block);
				LLM::on_initialize(next_block);
			}

			cumulative_epsilon += epsilon;

			assert_approx_eq(
				Assets::balance(id, treasury),
				expected_treasury_balance,
				cumulative_epsilon,
			);
			assert_approx_eq(
				Assets::balance(id, vault),
				70_000_000 - expected_treasury_balance,
				cumulative_epsilon,
			);
			assert_approx_eq(Assets::total_supply(id), TOTALLLM::get(), cumulative_epsilon);
		}
	});
}

#[test]
fn correctly_tracks_number_of_citizens() {
	new_test_ext().execute_with(|| {
		let root = RuntimeOrigin::root();
		assert_eq!(LLM::citizens_count(), 6);

		// set identity resets judgement - strips citizenship even if valid
		setup_identity(1, true, Some(vec![0]), false);
		assert_eq!(LLM::citizens_count(), 5);

		// judgement restores citizenship
		let info = Identity::identity(1).unwrap().info;
		Identity::provide_judgement(
			RuntimeOrigin::signed(0),
			0,
			1,
			pallet_identity::Judgement::KnownGood,
			BlakeTwo256::hash_of(&info),
		)
		.unwrap();
		assert_eq!(LLM::citizens_count(), 6);

		// clear identity strips citizenship
		Identity::clear_identity(RuntimeOrigin::signed(1)).unwrap();
		assert_eq!(LLM::citizens_count(), 5);

		// set non-citizen identity doesnt affect count
		setup_identity(99, false, None, false);
		assert_eq!(LLM::citizens_count(), 5);

		// clear identity doesnt affect count if done on non-citizen
		Identity::clear_identity(RuntimeOrigin::signed(99)).unwrap();
		assert_eq!(LLM::citizens_count(), 5);

		// kill identity strips citizenship
		Identity::kill_identity(root, 2).unwrap();
		assert_eq!(LLM::citizens_count(), 4);
	})
}

#[test]
fn only_approved_accounts_can_call_treasury_lld_transfer() {
	new_test_ext().execute_with(|| {
		let unapproved = RuntimeOrigin::signed(1);
		let approved = RuntimeOrigin::root();

		assert_noop!(LLM::treasury_lld_transfer(unapproved, 1, 1), BadOrigin);
		assert_ok!(LLM::treasury_lld_transfer(approved, 1, 1));
	});
}

#[test]
fn treasury_lld_transfer_calls_balances() {
	new_test_ext().execute_with(|| {
		let approved = RuntimeOrigin::root();
		let treasury = LLM::get_llm_treasury_account();
		assert_ok!(LLM::treasury_lld_transfer(approved.clone(), 1, 10));
		System::assert_has_event(
			pallet_balances::Event::Transfer { from: treasury, to: 1, amount: 10 }.into(),
		);
	});
}

#[test]
fn transfer_trait_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(LLM::transfer(1, 2, 6001), AssetsError::<Test>::BalanceLow);
		assert_ok!(LLM::transfer(1, 2, 6000));
		assert_eq!(LLM::balance(1), 0);
		assert_eq!(LLM::balance(2), 12000);
	});
}

#[test]
fn fungible_traits_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(
			<LLM as Inspect<<Test as frame_system::Config>::AccountId>>::balance(&99),
			<u8 as Into<<Test as pallet_assets::Config>::Balance>>::into(0u8)
		);
		<LLM as Mutate<<Test as frame_system::Config>::AccountId>>::set_balance(&99, 100u8.into());
		assert_eq!(
			<LLM as Inspect<<Test as frame_system::Config>::AccountId>>::balance(&99),
			<u8 as Into<<Test as pallet_assets::Config>::Balance>>::into(100u8)
		);
		assert_ok!(<LLM as Mutate<<Test as frame_system::Config>::AccountId>>::transfer(
			&99,
			&999,
			100u8.into(),
			Preservation::Expendable
		));
		assert_eq!(
			<LLM as Inspect<<Test as frame_system::Config>::AccountId>>::balance(&99),
			<u8 as Into<<Test as pallet_assets::Config>::Balance>>::into(0u8)
		);
		assert_eq!(
			<LLM as Inspect<<Test as frame_system::Config>::AccountId>>::balance(&999),
			<u8 as Into<<Test as pallet_assets::Config>::Balance>>::into(100u8)
		);
	});
}

#[test]
fn remark_deposits_event() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(1);
		let data: RemarkData = vec![1, 2, 3].try_into().unwrap();
		assert_ok!(LLM::remark(origin.clone(), data.clone()));
		System::assert_last_event(Event::Remarked(data).into());
	});
}

#[test]
fn set_courts_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(LLM::set_courts(RuntimeOrigin::root(), vec![105, 110].try_into().unwrap()));
		assert_eq!(Courts::<Test>::get(), vec![105, 110]);
	});
}

#[test]
fn only_approved_accounts_can_call_force_transfer() {
	new_test_ext().execute_with(|| {
		let unapproved = RuntimeOrigin::signed(2);
		let approved = RuntimeOrigin::signed(1);

		let from = LLMAccount::Locked(1);
		let to = LLMAccount::Locked(2);
		let amount = 1;

		assert_noop!(
			LLM::force_transfer(unapproved, from.clone(), to.clone(), amount),
			Error::<Test>::NotCourt
		);

		assert_ok!(LLM::politics_lock(RuntimeOrigin::signed(1), 1));
		assert_ok!(LLM::force_transfer(approved, from, to, amount));
	});
}

#[test]
fn cant_force_transfer_more_than_balance() {
	new_test_ext().execute_with(|| {
		let court = RuntimeOrigin::signed(1);
		let from = LLMAccount::Locked(1);
		let to = LLMAccount::Locked(2);
		let amount = 1;

		assert_noop!(LLM::force_transfer(court, from, to, amount), Error::<Test>::LowBalance);
	});
}

#[test]
fn cant_force_transfer_from_liquid() {
	new_test_ext().execute_with(|| {
		let court = RuntimeOrigin::signed(1);
		let from = LLMAccount::Liquid(1);
		let to = LLMAccount::Locked(2);
		let amount = 1;

		assert_noop!(LLM::force_transfer(court, from, to, amount), Error::<Test>::InvalidAccount);
	});
}

#[test]
fn force_updates_balances_correctly_to_liquid() {
	new_test_ext().execute_with(|| {
		let court = RuntimeOrigin::signed(1);
		let id = LLM::llm_id();
		let politipool = LLM::get_llm_politipool_account();
		let from = LLMAccount::Locked(1);
		let to = LLMAccount::Liquid(2);
		let amount = 2;

		assert_ok!(LLM::politics_lock(RuntimeOrigin::signed(1), 3));
		assert_ok!(LLM::politics_lock(RuntimeOrigin::signed(2), 2));
		let politipool_before = Assets::balance(id, politipool);
		let from_before = Assets::balance(id, 1);
		let to_before = Assets::balance(id, 2);

		assert_ok!(LLM::force_transfer(court, from, to, amount));

		assert_eq!(Assets::balance(id, politipool), politipool_before - amount);
		assert_eq!(Assets::balance(id, 1), from_before);
		assert_eq!(Assets::balance(id, 2), to_before + amount);

		assert_eq!(LLMPolitics::<Test>::get(1), 3 - amount);
		assert_eq!(LLMPolitics::<Test>::get(2), 2);
	});
}

#[test]
fn force_updates_balances_correctly_to_locked() {
	new_test_ext().execute_with(|| {
		let court = RuntimeOrigin::signed(1);
		let id = LLM::llm_id();
		let politipool = LLM::get_llm_politipool_account();
		let from = LLMAccount::Locked(1);
		let to = LLMAccount::Locked(2);
		let amount = 2;

		assert_ok!(LLM::politics_lock(RuntimeOrigin::signed(1), 3));
		assert_ok!(LLM::politics_lock(RuntimeOrigin::signed(2), 2));
		let politipool_before = Assets::balance(id, politipool);
		let from_before = Assets::balance(id, 1);
		let to_before = Assets::balance(id, 2);

		assert_ok!(LLM::force_transfer(court, from, to, amount));

		assert_eq!(Assets::balance(id, politipool), politipool_before);
		assert_eq!(Assets::balance(id, 1), from_before);
		assert_eq!(Assets::balance(id, 2), to_before);

		assert_eq!(LLMPolitics::<Test>::get(1), 3 - amount);
		assert_eq!(LLMPolitics::<Test>::get(2), 2 + amount);
	});
}
