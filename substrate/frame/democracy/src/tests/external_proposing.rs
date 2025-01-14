// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
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

//! The tests for functionality concerning the "external" origin.

use super::*;

#[test]
fn veto_external_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(0);
		assert_ok!(Democracy::external_propose(RuntimeOrigin::signed(2), set_balance_proposal(2),));
		assert!(<NextExternal<Test>>::exists());

		let h = set_balance_proposal(2).hash();
		assert_ok!(Democracy::veto_external(RuntimeOrigin::signed(3), h));
		// cancelled.
		assert!(!<NextExternal<Test>>::exists());
		// fails - same proposal can't be resubmitted.
		assert_noop!(
			Democracy::external_propose(RuntimeOrigin::signed(2), set_balance_proposal(2),),
			Error::<Test>::ProposalBlacklisted
		);

		fast_forward_to(1);
		// fails as we're still in cooloff period.
		assert_noop!(
			Democracy::external_propose(RuntimeOrigin::signed(2), set_balance_proposal(2),),
			Error::<Test>::ProposalBlacklisted
		);

		fast_forward_to(2);
		// works; as we're out of the cooloff period.
		assert_ok!(Democracy::external_propose(RuntimeOrigin::signed(2), set_balance_proposal(2),));
		assert!(<NextExternal<Test>>::exists());

		// 3 can't veto the same thing twice.
		assert_noop!(
			Democracy::veto_external(RuntimeOrigin::signed(3), h),
			Error::<Test>::AlreadyVetoed
		);

		// 4 vetoes.
		assert_ok!(Democracy::veto_external(RuntimeOrigin::signed(4), h));
		// cancelled again.
		assert!(!<NextExternal<Test>>::exists());

		fast_forward_to(3);
		// same proposal fails as we're still in cooloff
		assert_noop!(
			Democracy::external_propose(RuntimeOrigin::signed(2), set_balance_proposal(2)),
			Error::<Test>::ProposalBlacklisted
		);
		// different proposal works fine.
		assert_ok!(Democracy::external_propose(RuntimeOrigin::signed(2), set_balance_proposal(3),));
	});
}

#[test]
fn external_blacklisting_should_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(0);

		assert_ok!(Democracy::external_propose(RuntimeOrigin::signed(2), set_balance_proposal(2),));

		let hash = set_balance_proposal(2).hash();
		assert_ok!(Democracy::blacklist(RuntimeOrigin::root(), hash, None));

		fast_forward_to(2);
		assert_noop!(Democracy::referendum_status(0), Error::<Test>::ReferendumInvalid);

		assert_noop!(
			Democracy::external_propose(RuntimeOrigin::signed(2), set_balance_proposal(2)),
			Error::<Test>::ProposalBlacklisted,
		);
	});
}

#[test]
fn external_referendum_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(0);
		assert_noop!(
			Democracy::external_propose(RuntimeOrigin::signed(1), set_balance_proposal(2),),
			BadOrigin,
		);
		assert_ok!(Democracy::external_propose(RuntimeOrigin::signed(2), set_balance_proposal(2),));
		assert_noop!(
			Democracy::external_propose(RuntimeOrigin::signed(2), set_balance_proposal(1),),
			Error::<Test>::DuplicateProposal
		);
		fast_forward_to(2);
		assert_eq!(
			Democracy::referendum_status(0),
			Ok(ReferendumStatus {
				end: 4,
				proposal: set_balance_proposal(2),
				dispatch_origin: DispatchOrigin::Root, 
				threshold: VoteThreshold::SuperMajorityApprove,
				delay: 2,
				tally: Tally { ayes: 0, nays: 0, aye_voters: 00000, nay_voters: 00000, turnout: 0 },
			})
		);
	});
}

#[test]
fn external_majority_referendum_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(0);
		assert_noop!(
			Democracy::external_propose_majority(RuntimeOrigin::signed(1), set_balance_proposal(2)),
			BadOrigin,
		);
		assert_ok!(Democracy::external_propose_majority(
			RuntimeOrigin::signed(3),
			set_balance_proposal(2)
		));
		fast_forward_to(2);
		assert_eq!(
			Democracy::referendum_status(0),
			Ok(ReferendumStatus {
				end: 4,
				proposal: set_balance_proposal(2),
				dispatch_origin: DispatchOrigin::Root, 
				threshold: VoteThreshold::SimpleMajority,
				delay: 2,
				tally: Tally { ayes: 0, nays: 0, aye_voters: 00000, nay_voters: 00000, turnout: 0 },
			})
		);
	});
}

#[test]
fn external_default_referendum_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(0);
		assert_noop!(
			Democracy::external_propose_default(RuntimeOrigin::signed(3), set_balance_proposal(2)),
			BadOrigin,
		);
		assert_ok!(Democracy::external_propose_default(
			RuntimeOrigin::signed(1),
			set_balance_proposal(2)
		));
		fast_forward_to(2);
		assert_eq!(
			Democracy::referendum_status(0),
			Ok(ReferendumStatus {
				end: 4,
				proposal: set_balance_proposal(2),
				dispatch_origin: DispatchOrigin::Root, 
				threshold: VoteThreshold::SuperMajorityAgainst,
				delay: 2,
				tally: Tally { ayes: 0, nays: 0, aye_voters: 00000, nay_voters: 00000, turnout: 0 },
			})
		);
	});
}

#[test]
fn external_and_public_interleaving_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(0);
		assert_ok!(Democracy::external_propose(RuntimeOrigin::signed(2), set_balance_proposal(1),));
		assert_ok!(propose_set_balance(6, 2, 2));

		fast_forward_to(2);

		// both waiting: both start.
		assert_eq!(
			Democracy::referendum_status(0),
			Ok(ReferendumStatus {
				end: 4,
				proposal: set_balance_proposal(2),
				dispatch_origin: DispatchOrigin::Root, 
				threshold: VoteThreshold::SuperMajorityApprove,
				delay: 2,
				tally: Tally { ayes: 0, nays: 0, aye_voters: 00000, nay_voters: 00000, turnout: 0 },
			})
		);
		assert_eq!(
			Democracy::referendum_status(1),
			Ok(ReferendumStatus {
				end: 4,
				proposal: set_balance_proposal(1),
				dispatch_origin: DispatchOrigin::Root, 
				threshold: VoteThreshold::SuperMajorityApprove,
				delay: 2,
				tally: Tally { ayes: 0, nays: 0, aye_voters: 00000, nay_voters: 00000, turnout: 0 },
			})
		);
	});
}
