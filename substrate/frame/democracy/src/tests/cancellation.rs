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

// File has been modified by Liberland in 2022. All modifications by Liberland are distributed under
// the MIT license.

// You should have received a copy of the MIT license along with this program. If not, see https://opensource.org/licenses/MIT

//! The tests for cancelation functionality.

use super::*;

#[test]
fn cancel_referendum_should_work() {
	new_test_ext().execute_with(|| {
		let r = Democracy::inject_referendum(
			2,
			set_balance_proposal(2),
			DispatchOrigin::Root,
			VoteThreshold::SuperMajorityApprove,
			0,
		);
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(1), r, aye(1)));
		assert_ok!(Democracy::cancel_referendum(RuntimeOrigin::root(), r.into()));
		assert!(Democracy::referendum_info(r).is_none());
		System::assert_last_event(RuntimeEvent::Democracy(crate::Event::Cancelled {
			ref_index: r,
		}));
		assert_eq!(Democracy::lowest_unbaked(), 0);

		next_block();

		next_block();

		assert_eq!(Democracy::lowest_unbaked(), 1);
		assert_eq!(Democracy::lowest_unbaked(), Democracy::referendum_count());
		assert_eq!(Balances::free_balance(42), 0);
	});
}

#[test]
fn cancel_approved_referendum_after_bake_should_fail_without_changes() {
	new_test_ext().execute_with(|| {
		let r = Democracy::inject_referendum(
			2,
			set_balance_proposal(2),
			DispatchOrigin::Root,
			VoteThreshold::SuperMajorityApprove,
			2,
		);
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(1), r, aye(1)));
		let owner = MetadataOwner::Referendum(r);
		let metadata_hash = note_preimage(1);
		MetadataOf::<Test>::insert(&owner, metadata_hash);

		next_block();

		let referendum = Democracy::referendum_info(r);
		assert_eq!(referendum, Some(ReferendumInfo::Finished { approved: true, end: 2 }));
		let metadata = MetadataOf::<Test>::get(&owner);
		let voting = VotingOf::<Test>::get(1).encode();
		let scheduled = pallet_scheduler::Agenda::<Test>::get(4);
		assert!(scheduled[0].is_some());
		let events = System::events();

		assert_noop!(
			Democracy::cancel_referendum(RuntimeOrigin::root(), r),
			Error::<Test>::ReferendumInvalid,
		);
		assert_eq!(Democracy::referendum_info(r), referendum);
		assert_eq!(MetadataOf::<Test>::get(&owner), metadata);
		assert_eq!(VotingOf::<Test>::get(1).encode(), voting);
		assert_eq!(pallet_scheduler::Agenda::<Test>::get(4), scheduled);
		assert_eq!(System::events(), events);

		fast_forward_to(4);
		assert_eq!(Balances::free_balance(42), 2);
	});
}

#[test]
fn cancel_rejected_referendum_after_bake_should_fail_without_changes() {
	new_test_ext().execute_with(|| {
		let r = Democracy::inject_referendum(
			2,
			set_balance_proposal(2),
			DispatchOrigin::Root,
			VoteThreshold::SuperMajorityApprove,
			2,
		);
		assert_ok!(Democracy::vote(RuntimeOrigin::signed(1), r, nay(1)));
		let owner = MetadataOwner::Referendum(r);
		let metadata_hash = note_preimage(1);
		MetadataOf::<Test>::insert(&owner, metadata_hash);

		next_block();

		let referendum = Democracy::referendum_info(r);
		assert_eq!(referendum, Some(ReferendumInfo::Finished { approved: false, end: 2 }));
		let metadata = MetadataOf::<Test>::get(&owner);
		let voting = VotingOf::<Test>::get(1).encode();
		let scheduled = pallet_scheduler::Agenda::<Test>::get(4);
		let events = System::events();

		assert_noop!(
			Democracy::cancel_referendum(RuntimeOrigin::root(), r),
			Error::<Test>::ReferendumInvalid,
		);
		assert_eq!(Democracy::referendum_info(r), referendum);
		assert_eq!(MetadataOf::<Test>::get(&owner), metadata);
		assert_eq!(VotingOf::<Test>::get(1).encode(), voting);
		assert_eq!(pallet_scheduler::Agenda::<Test>::get(4), scheduled);
		assert_eq!(System::events(), events);

		fast_forward_to(4);
		assert_eq!(Balances::free_balance(42), 0);
	});
}

#[test]
fn cancel_missing_referendum_should_fail_without_event() {
	new_test_ext().execute_with(|| {
		let events = System::events();
		assert_noop!(Democracy::cancel_referendum(RuntimeOrigin::signed(1), 0), BadOrigin);
		assert_noop!(
			Democracy::cancel_referendum(RuntimeOrigin::root(), 0),
			Error::<Test>::ReferendumInvalid,
		);
		assert_eq!(System::events(), events);
	});
}

#[test]
fn emergency_cancel_should_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(0);
		let r = Democracy::inject_referendum(
			2,
			set_balance_proposal(2),
			DispatchOrigin::Root,
			VoteThreshold::SuperMajorityApprove,
			2,
		);
		assert!(Democracy::referendum_status(r).is_ok());

		assert_noop!(Democracy::emergency_cancel(RuntimeOrigin::signed(3), r), BadOrigin);
		assert_ok!(Democracy::emergency_cancel(RuntimeOrigin::signed(4), r));
		assert!(Democracy::referendum_info(r).is_none());

		// some time later...

		let r = Democracy::inject_referendum(
			2,
			set_balance_proposal(2),
			DispatchOrigin::Root,
			VoteThreshold::SuperMajorityApprove,
			2,
		);
		assert!(Democracy::referendum_status(r).is_ok());
		assert_noop!(
			Democracy::emergency_cancel(RuntimeOrigin::signed(4), r),
			Error::<Test>::AlreadyCanceled,
		);
	});
}
