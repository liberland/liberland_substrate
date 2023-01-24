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

//! The tests for functionality concerning delegation.

use super::*;

#[test]
fn single_proposal_should_work_with_delegation() {
	new_test_ext().execute_with(|| {
		System::set_block_number(0);

		assert_ok!(propose_set_balance(1, 2, 1));

		fast_forward_to(2);

		// Delegate first vote.
		assert_ok!(Democracy::delegate(RuntimeOrigin::signed(2), 1, Conviction::None, 20));
		let r = 0;
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(1), r, aye(1)));
		assert_eq!(tally(r), Tally { ayes: 30, nays: 0, turnout: 30, aye_voters: 20000, nay_voters: 0 });

		// Delegate a second vote.
		assert_ok!(Democracy::delegate(RuntimeOrigin::signed(3), 1, Conviction::None, 30));
		assert_eq!(tally(r), Tally { ayes: 60, nays: 0, turnout: 60, aye_voters: 30000, nay_voters: 0 });

		// Reduce first vote.
		assert_ok!(Democracy::delegate(RuntimeOrigin::signed(2), 1, Conviction::None, 10));
		assert_eq!(tally(r), Tally { ayes: 50, nays: 0, turnout: 50, aye_voters: 30000, nay_voters: 0 });

		// Second vote delegates to first; we don't do tiered delegation, so it doesn't get used.
		assert_ok!(Democracy::delegate(RuntimeOrigin::signed(3), 2, Conviction::None, 30));
		assert_eq!(tally(r), Tally { ayes: 20, nays: 0, turnout: 20, aye_voters: 20000, nay_voters: 0 });

		// Main voter cancels their vote
		assert_ok!(Democracy::remove_vote(RuntimeOrigin::signed(1), r));
		assert_eq!(tally(r), Tally { ayes: 0, nays: 0, aye_voters: 00000, nay_voters: 00000, turnout: 0 });

		// First delegator delegates half funds with conviction; nothing changes yet.
		assert_ok!(Democracy::delegate(RuntimeOrigin::signed(2), 1, Conviction::Locked1x, 10));
		assert_eq!(tally(r), Tally { ayes: 0, nays: 0, aye_voters: 00000, nay_voters: 00000, turnout: 0 });

		// Main voter reinstates their vote
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(1), r, aye(1)));
		assert_eq!(tally(r), Tally { ayes: 20, nays: 0, turnout: 20, aye_voters: 20000, nay_voters: 0 });
	});
}

#[test]
fn self_delegation_not_allowed() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Democracy::delegate(RuntimeOrigin::signed(1), 1, Conviction::None, 10),
			Error::<Test>::Nonsense,
		);
	});
}

#[test]
fn cyclic_delegation_should_unwind() {
	new_test_ext().execute_with(|| {
		System::set_block_number(0);

		assert_ok!(propose_set_balance(1, 2, 1));

		fast_forward_to(2);

		// Check behavior with cycle.
		assert_ok!(Democracy::delegate(RuntimeOrigin::signed(2), 1, Conviction::None, 20));
		assert_ok!(Democracy::delegate(RuntimeOrigin::signed(3), 2, Conviction::None, 30));
		assert_ok!(Democracy::delegate(RuntimeOrigin::signed(1), 3, Conviction::None, 10));
		let r = 0;
		assert_ok!(Democracy::undelegate(RuntimeOrigin::signed(3)));
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(3), r, aye(3)));
		assert_ok!(Democracy::undelegate(RuntimeOrigin::signed(1)));
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(1), r, nay(1)));

		// Delegated vote is counted.
		assert_eq!(tally(r), Tally { ayes: 30, nays: 30, turnout: 60, aye_voters: 10000, nay_voters: 20000  });
	});
}

#[test]
fn single_proposal_should_work_with_vote_and_delegation() {
	// If transactor already voted, delegated vote is overwritten.
	new_test_ext().execute_with(|| {
		System::set_block_number(0);

		assert_ok!(propose_set_balance(1, 2, 1));

		fast_forward_to(2);

		let r = 0;
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(1), r, aye(1)));
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(2), r, nay(2)));
		assert_eq!(tally(r), Tally { ayes: 10, nays: 20, turnout: 30, aye_voters: 10000, nay_voters: 10000  });

		// Delegate vote.
		assert_ok!(Democracy::remove_vote(RuntimeOrigin::signed(2), r));
		assert_ok!(Democracy::delegate(RuntimeOrigin::signed(2), 1, Conviction::None, 20));
		// Delegated vote replaces the explicit vote.
		assert_eq!(tally(r), Tally { ayes: 30, nays: 0, turnout: 30 , aye_voters: 20000, nay_voters: 0 });
	});
}

#[test]
fn single_proposal_should_work_with_undelegation() {
	new_test_ext().execute_with(|| {
		System::set_block_number(0);

		assert_ok!(propose_set_balance(1, 2, 1));

		// Delegate and undelegate vote.
		assert_ok!(Democracy::delegate(RuntimeOrigin::signed(2), 1, Conviction::None, 20));
		assert_ok!(Democracy::undelegate(RuntimeOrigin::signed(2)));

		fast_forward_to(2);
		let r = 0;
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(1), r, aye(1)));

		// Delegated vote is not counted.
		assert_eq!(tally(r), Tally { ayes: 10, nays: 0, turnout: 10, aye_voters: 10000, nay_voters: 0  });
	});
}

#[test]
fn single_proposal_should_work_with_delegation_and_vote() {
	// If transactor voted, delegated vote is overwritten.
	new_test_ext().execute_with(|| {
		let r = begin_referendum();
		// Delegate, undelegate and vote.
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(1), r, aye(1)));
		assert_ok!(Democracy::delegate(RuntimeOrigin::signed(2), 1, Conviction::None, 20));
		assert_eq!(tally(r), Tally { ayes: 30, nays: 0, turnout: 30, aye_voters: 20000, nay_voters: 0  });
		assert_ok!(Democracy::undelegate(RuntimeOrigin::signed(2)));
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(2), r, aye(2)));
		// Delegated vote is not counted.
		assert_eq!(tally(r), Tally { ayes: 30, nays: 0, turnout: 30, aye_voters: 20000, nay_voters: 0  });
	});
}

#[test]
fn conviction_should_be_honored_in_delegation() {
	// If transactor voted, delegated vote is overwritten.
	new_test_ext().execute_with(|| {
		let r = begin_referendum();
		// Delegate and vote.
		assert_ok!(Democracy::delegate(RuntimeOrigin::signed(2), 1, Conviction::Locked6x, 20));
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(1), r, aye(1)));
		// Delegated vote is huge.
		assert_eq!(tally(r), Tally { ayes: 30, nays: 0, turnout: 30, aye_voters: 20000, nay_voters: 0  });
	});
}

#[test]
fn split_vote_delegation_should_be_ignored() {
	// If transactor voted, delegated vote is overwritten.
	new_test_ext().execute_with(|| {
		let r = begin_referendum();
		assert_ok!(Democracy::delegate(RuntimeOrigin::signed(2), 1, Conviction::Locked6x, 20));
		assert_ok!(Democracy::vote(
			RuntimeOrigin::signed(1),
			r,
			AccountVote::Split { aye: 10, nay: 0 }
		));
		// Delegated vote is huge.
		assert_eq!(tally(r), Tally { ayes: 10, nays: 0, turnout: 10, aye_voters: 10000, nay_voters: 0  });
	});
}

#[test]
fn redelegation_keeps_lock() {
	// If transactor voted, delegated vote is overwritten.
	new_test_ext().execute_with(|| {
		let r = begin_referendum();
		// Delegate and vote.
		assert_ok!(Democracy::delegate(RuntimeOrigin::signed(2), 1, Conviction::Locked6x, 20));
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(1), r, aye(1)));
		// Delegated vote is huge.
		assert_eq!(tally(r), Tally { ayes: 30, nays: 0, turnout: 30, aye_voters: 20000, nay_voters: 0  });

		let mut prior_lock = vote::PriorLock::default();

		// Locked balance of delegator exists
		assert_eq!(VotingOf::<Test>::get(2).locked_balance(), 20);
		assert_eq!(VotingOf::<Test>::get(2).prior(), &prior_lock);

		// Delegate someone else at a lower conviction and amount
		assert_ok!(Democracy::delegate(RuntimeOrigin::signed(2), 3, Conviction::None, 10));

		// 6x prior should appear w/ locked balance.
		prior_lock.accumulate(98, 20);
		assert_eq!(VotingOf::<Test>::get(2).prior(), &prior_lock);
		assert_eq!(VotingOf::<Test>::get(2).locked_balance(), 20);
		// Unlock shouldn't work
		assert_ok!(Democracy::unlock(RuntimeOrigin::signed(2), 2));
		assert_eq!(VotingOf::<Test>::get(2).prior(), &prior_lock);
		assert_eq!(VotingOf::<Test>::get(2).locked_balance(), 20);

		fast_forward_to(100);

		// Now unlock can remove the prior lock and reduce the locked amount.
		assert_eq!(VotingOf::<Test>::get(2).prior(), &prior_lock);
		assert_ok!(Democracy::unlock(RuntimeOrigin::signed(2), 2));
		assert_eq!(VotingOf::<Test>::get(2).prior(), &vote::PriorLock::default());
		assert_eq!(VotingOf::<Test>::get(2).locked_balance(), 10);
	});
}
