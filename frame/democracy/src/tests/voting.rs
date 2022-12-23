// This file is part of Substrate.

// Copyright (C) 2017-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// File has been modified by Liberland in 2022. All modifications by Liberland are distributed under the MIT license.

// You should have received a copy of the MIT license along with this program. If not, see https://opensource.org/licenses/MIT

//! The tests for normal voting functionality.

use super::*;

#[test]
fn overvoting_should_fail() {
	new_test_ext().execute_with(|| {
		let r = begin_referendum();
		assert_noop!(
			Democracy::vote(RuntimeOrigin::signed(1), r, vote_aye(5001)),
			Error::<Test>::InsufficientLLM
		);
	});
}

#[test]
fn split_voting_should_work() {
	new_test_ext().execute_with(|| {
		let r = begin_referendum();
		let v = AccountVote::Split { aye: 4000, nay: 1001 };
		assert_noop!(
			Democracy::vote(RuntimeOrigin::signed(5), r, v),
			Error::<Test>::InsufficientLLM
		);
		let v = AccountVote::Split { aye: 4000, nay: 1000 };
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(5), r, v));

		assert_eq!(tally(r), Tally { ayes: 400, nays: 100, turnout: 5000 });
	});
}

#[test]
fn split_vote_cancellation_should_work() {
	new_test_ext().execute_with(|| {
		let r = begin_referendum();
		let v = AccountVote::Split { aye: 30, nay: 20 };
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(5), r, v));
		assert_ok!(Democracy::remove_vote(RuntimeOrigin::signed(5), r));
		assert_eq!(tally(r), Tally { ayes: 0, nays: 0, turnout: 0 });
		assert_ok!(Democracy::unlock(RuntimeOrigin::signed(5), 5));
		assert_eq!(Balances::locks(5), vec![]);
	});
}

#[test]
fn single_proposal_should_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(0);
		assert_ok!(propose_set_balance(1, 2, 1));
		let r = 0;
		assert!(Democracy::referendum_info(r).is_none());

		// start of 2 => next referendum scheduled.
		fast_forward_to(2);
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(1), r, aye(1)));

		assert_eq!(Democracy::referendum_count(), 1);
		assert_eq!(
			Democracy::referendum_status(0),
			Ok(ReferendumStatus {
				end: 4,
				proposal: set_balance_proposal(2),
				threshold: VoteThreshold::SuperMajorityApprove,
				delay: 2,
				tally: Tally { ayes: 1, nays: 0, turnout: 10 },
			})
		);

		fast_forward_to(3);

		// referendum still running
		assert_ok!(Democracy::referendum_status(0));

		// referendum runs during 2 and 3, ends @ start of 4.
		fast_forward_to(4);

		assert_noop!(Democracy::referendum_status(0), Error::<Test>::ReferendumInvalid);
		assert!(pallet_scheduler::Agenda::<Test>::get(6)[0].is_some());

		// referendum passes and wait another two blocks for enactment.
		fast_forward_to(6);

		assert_eq!(Balances::free_balance(42), 2);
	});
}

#[test]
fn controversial_voting_should_work() {
	new_test_ext().execute_with(|| {
		let r = Democracy::inject_referendum(
			2,
			set_balance_proposal(2),
			VoteThreshold::SuperMajorityApprove,
			0,
		);

		assert_ok!(Democracy::vote(RuntimeOrigin::signed(1), r, vote_aye(5000)));
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(2), r, vote_nay(5000)));
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(3), r, vote_nay(5000)));
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(4), r, vote_aye(5000)));
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(5), r, vote_nay(4000)));
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(6), r, vote_aye(5000)));

		assert_eq!(tally(r), Tally { ayes: 1500, nays: 1400, turnout: 29000 });

		next_block();
		next_block();

		assert_eq!(Balances::free_balance(42), 2);
	});
}

#[test]
fn controversial_low_turnout_voting_should_work() {
	new_test_ext().execute_with(|| {
		let r = Democracy::inject_referendum(
			2,
			set_balance_proposal(2),
			VoteThreshold::SuperMajorityApprove,
			0,
		);
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(5), r, big_nay(5)));
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(6), r, big_aye(6)));

		assert_eq!(tally(r), Tally { ayes: 60, nays: 50, turnout: 110 });

		next_block();
		next_block();

		assert_eq!(Balances::free_balance(42), 0);
	});
}

#[test]
fn passing_low_turnout_voting_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(42), 0);
		assert_eq!(Balances::total_issuance(), 210);

		let r = Democracy::inject_referendum(
			2,
			set_balance_proposal(2),
			VoteThreshold::SuperMajorityApprove,
			0,
		);
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(4), r, vote_aye(5000)));
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(5), r, vote_nay(5000)));
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(6), r, vote_aye(5000)));
		assert_eq!(tally(r), Tally { ayes: 1000, nays: 500, turnout: 15000 });

		next_block();
		next_block();
		assert_eq!(Balances::free_balance(42), 2);
	});
}
