// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
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

use crate::{mock::*, Candidates, Config, Error, Renouncing, SeatHolder, Voter};
use frame_support::{assert_noop, assert_ok};

use frame_support::traits::OnInitialize;
use sp_runtime::{DispatchError, ModuleError};
use substrate_test_utils::assert_eq_uvec;
use pallet_llm::pallet::Error as LLMError;

#[test]
fn params_should_work() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(<Test as Config>::DesiredMembers::get(), 2);
		assert_eq!(<Test as Config>::DesiredRunnersUp::get(), 0);
		assert_eq!(<Test as Config>::VotingBondBase::get(), 2);
		assert_eq!(<Test as Config>::VotingBondFactor::get(), 0);
		assert_eq!(<Test as Config>::CandidacyBond::get(), 3);
		assert_eq!(<Test as Config>::TermDuration::get(), 5);
		assert_eq!(Elections::election_rounds(), 0);

		assert!(Elections::members().is_empty());
		assert!(Elections::runners_up().is_empty());

		assert!(candidate_ids().is_empty());
		assert_eq!(<Candidates<Test>>::decode_len(), None);
		assert!(Elections::is_candidate(&1).is_err());

		assert!(all_voters().is_empty());
		assert!(votes_of(&1).is_empty());
	});
}

#[test]
fn genesis_members_should_work() {
	ExtBuilder::default()
		.genesis_members(vec![(1, 10), (2, 20)])
		.build_and_execute(|| {
			System::set_block_number(1);
			assert_eq!(
				Elections::members(),
				vec![
					SeatHolder { who: 1, stake: 10, deposit: 0 },
					SeatHolder { who: 2, stake: 20, deposit: 0 }
				]
			);

			assert_eq!(Elections::voting(1), Voter { stake: 10u64, votes: vec![1], deposit: 0 });
			assert_eq!(Elections::voting(2), Voter { stake: 20u64, votes: vec![2], deposit: 0 });

			// they will persist since they have self vote.
			System::set_block_number(5);
			Elections::on_initialize(System::block_number());

			assert_eq!(members_ids(), vec![1, 2]);
		})
}

#[test]
fn genesis_voters_can_remove_lock() {
	ExtBuilder::default()
		.genesis_members(vec![(1, 10), (2, 20)])
		.build_and_execute(|| {
			System::set_block_number(1);

			assert_eq!(Elections::voting(1), Voter { stake: 10u64, votes: vec![1], deposit: 0 });
			assert_eq!(Elections::voting(2), Voter { stake: 20u64, votes: vec![2], deposit: 0 });

			assert_ok!(Elections::remove_voter(Origin::signed(1)));
			assert_ok!(Elections::remove_voter(Origin::signed(2)));

			assert_eq!(Elections::voting(1), Default::default());
			assert_eq!(Elections::voting(2), Default::default());
		})
}

#[test]
fn genesis_members_unsorted_should_work() {
	ExtBuilder::default()
		.genesis_members(vec![(2, 20), (1, 10)])
		.build_and_execute(|| {
			System::set_block_number(1);
			assert_eq!(
				Elections::members(),
				vec![
					SeatHolder { who: 1, stake: 10, deposit: 0 },
					SeatHolder { who: 2, stake: 20, deposit: 0 },
				]
			);

			assert_eq!(Elections::voting(1), Voter { stake: 10u64, votes: vec![1], deposit: 0 });
			assert_eq!(Elections::voting(2), Voter { stake: 20u64, votes: vec![2], deposit: 0 });

			// they will persist since they have self vote.
			System::set_block_number(5);
			Elections::on_initialize(System::block_number());

			assert_eq!(members_ids(), vec![1, 2]);
		})
}

#[test]
#[should_panic = "Genesis member does not have enough stake"]
fn genesis_members_cannot_over_stake_0() {
	// 10 cannot lock 20 as their stake and extra genesis will panic.
	ExtBuilder::default()
		.genesis_members(vec![(1, 20), (2, 20)])
		.build_and_execute(|| {});
}

#[test]
#[should_panic = "Duplicate member in elections-phragmen genesis: 2"]
fn genesis_members_cannot_be_duplicate() {
	ExtBuilder::default()
		.desired_members(3)
		.genesis_members(vec![(1, 10), (2, 10), (2, 10)])
		.build_and_execute(|| {});
}

#[test]
#[should_panic = "Cannot accept more than DesiredMembers genesis member"]
fn genesis_members_cannot_too_many() {
	ExtBuilder::default()
		.genesis_members(vec![(1, 10), (2, 10), (3, 30)])
		.desired_members(2)
		.build_and_execute(|| {});
}

#[test]
fn term_duration_zero_is_passive() {
	ExtBuilder::default().term_duration(0).build_and_execute(|| {
		assert_eq!(<Test as Config>::TermDuration::get(), 0);
		assert_eq!(<Test as Config>::DesiredMembers::get(), 2);
		assert_eq!(Elections::election_rounds(), 0);

		assert!(members_ids().is_empty());
		assert!(Elections::runners_up().is_empty());
		assert!(candidate_ids().is_empty());

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert!(members_ids().is_empty());
		assert!(Elections::runners_up().is_empty());
		assert!(candidate_ids().is_empty());
	});
}

#[test]
fn simple_candidate_submission_should_work() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(candidate_ids(), Vec::<u64>::new());
		assert!(Elections::is_candidate(&1).is_err());
		assert!(Elections::is_candidate(&2).is_err());

		assert_eq!(balances(&1), (10, 0));
		assert_ok!(submit_candidacy(Origin::signed(1)));
		assert_eq!(balances(&1), (7, 3));

		assert_eq!(candidate_ids(), vec![1]);

		assert!(Elections::is_candidate(&1).is_ok());
		assert!(Elections::is_candidate(&2).is_err());

		assert_eq!(balances(&2), (20, 0));
		assert_ok!(submit_candidacy(Origin::signed(2)));
		assert_eq!(balances(&2), (17, 3));

		assert_eq!(candidate_ids(), vec![1, 2]);

		assert!(Elections::is_candidate(&1).is_ok());
		assert!(Elections::is_candidate(&2).is_ok());

		assert_eq!(candidate_deposit(&1), 3);
		assert_eq!(candidate_deposit(&2), 3);
		assert_eq!(candidate_deposit(&3), 0);
	});
}

#[test]
fn updating_candidacy_bond_works() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(vote(Origin::signed(5), vec![5], 50));
		assert_eq!(Elections::candidates(), vec![(5, 3)]);

		// a runtime upgrade changes the bond.
		CANDIDACY_BOND.with(|v| *v.borrow_mut() = 4);

		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_eq!(Elections::candidates(), vec![(4, 4), (5, 3)]);

		// once elected, they each hold their candidacy bond, no more.
		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(balances(&4), (34, 6));
		assert_eq!(balances(&5), (45, 5));
		assert_eq!(
			Elections::members(),
			vec![
				SeatHolder { who: 4, stake: 34, deposit: 4 },
				SeatHolder { who: 5, stake: 45, deposit: 3 },
			]
		);
	})
}

#[test]
fn candidates_are_always_sorted() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(candidate_ids(), Vec::<u64>::new());

		assert_ok!(submit_candidacy(Origin::signed(3)));
		assert_eq!(candidate_ids(), vec![3]);
		assert_ok!(submit_candidacy(Origin::signed(1)));
		assert_eq!(candidate_ids(), vec![1, 3]);
		assert_ok!(submit_candidacy(Origin::signed(2)));
		assert_eq!(candidate_ids(), vec![1, 2, 3]);
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_eq!(candidate_ids(), vec![1, 2, 3, 4]);
	});
}

#[test]
fn dupe_candidate_submission_should_not_work() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(candidate_ids(), Vec::<u64>::new());
		assert_ok!(submit_candidacy(Origin::signed(1)));
		assert_eq!(candidate_ids(), vec![1]);
		assert_noop!(submit_candidacy(Origin::signed(1)), Error::<Test>::DuplicatedCandidate);
	});
}

#[test]
fn member_candidacy_submission_should_not_work() {
	// critically important to make sure that outgoing candidates and losers are not mixed up.
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(vote(Origin::signed(2), vec![5], 20));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![5]);
		assert!(Elections::runners_up().is_empty());
		assert!(candidate_ids().is_empty());

		assert_noop!(submit_candidacy(Origin::signed(5)), Error::<Test>::MemberSubmit);
	});
}

#[test]
fn runner_candidate_submission_should_not_work() {
	ExtBuilder::default().desired_runners_up(2).build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));

		assert_ok!(vote(Origin::signed(2), vec![5, 4], 20));
		assert_ok!(vote(Origin::signed(1), vec![3], 10));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![4, 5]);
		assert_eq!(runners_up_ids(), vec![3]);

		assert_noop!(submit_candidacy(Origin::signed(3)), Error::<Test>::RunnerUpSubmit);
	});
}

#[test]
fn simple_voting_should_work() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(candidate_ids(), Vec::<u64>::new());
		assert_eq!(balances(&2), (20, 0));

		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(vote(Origin::signed(2), vec![5], 20));

		assert_eq!(balances(&2), (18, 2));
		assert_eq!(has_lock(&2), 18);
	});
}

#[test]
fn can_vote_with_custom_stake() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(candidate_ids(), Vec::<u64>::new());
		assert_eq!(balances(&2), (20, 0));

		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(vote(Origin::signed(2), vec![5], 12));

		assert_eq!(balances(&2), (18, 2));
		assert_eq!(has_lock(&2), 12);
	});
}

#[test]
fn can_update_votes_and_stake() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(balances(&2), (20, 0));

		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(vote(Origin::signed(2), vec![5], 20));

		// User only locks up to their free balance.
		assert_eq!(balances(&2), (18, 2));
		assert_eq!(has_lock(&2), 18);
		assert_eq!(locked_stake_of(&2), 18);

		// can update; different stake; different lock and reserve.
		assert_ok!(vote(Origin::signed(2), vec![5, 4], 15));
		assert_eq!(balances(&2), (18, 2));
		assert_eq!(has_lock(&2), 15);
		assert_eq!(locked_stake_of(&2), 15);
	});
}

#[test]
fn updated_voting_bond_works() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));

		assert_eq!(balances(&2), (20, 0));
		assert_ok!(vote(Origin::signed(2), vec![5], 5));
		assert_eq!(balances(&2), (18, 2));
		assert_eq!(voter_deposit(&2), 2);

		// a runtime upgrade lowers the voting bond to 1. This guy still un-reserves 2 when
		// leaving.
		VOTING_BOND_BASE.with(|v| *v.borrow_mut() = 1);

		// proof that bond changed.
		assert_eq!(balances(&1), (10, 0));
		assert_ok!(vote(Origin::signed(1), vec![5], 5));
		assert_eq!(balances(&1), (9, 1));
		assert_eq!(voter_deposit(&1), 1);

		assert_ok!(Elections::remove_voter(Origin::signed(2)));
		assert_eq!(balances(&2), (20, 0));
	})
}

#[test]
fn voting_reserves_bond_per_vote() {
	ExtBuilder::default().voter_bond_factor(1).build_and_execute(|| {
		assert_eq!(balances(&2), (20, 0));

		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));

		// initial vote.
		assert_ok!(vote(Origin::signed(2), vec![5], 10));

		// 2 + 1
		assert_eq!(balances(&2), (17, 3));
		assert_eq!(Elections::voting(&2).deposit, 3);
		assert_eq!(has_lock(&2), 10);
		assert_eq!(locked_stake_of(&2), 10);

		// can update; different stake; different lock and reserve.
		assert_ok!(vote(Origin::signed(2), vec![5, 4], 15));
		// 2 + 2
		assert_eq!(balances(&2), (16, 4));
		assert_eq!(Elections::voting(&2).deposit, 4);
		assert_eq!(has_lock(&2), 15);
		assert_eq!(locked_stake_of(&2), 15);

		// stay at two votes with different stake.
		assert_ok!(vote(Origin::signed(2), vec![5, 3], 18));
		// 2 + 2
		assert_eq!(balances(&2), (16, 4));
		assert_eq!(Elections::voting(&2).deposit, 4);
		assert_eq!(has_lock(&2), 16);
		assert_eq!(locked_stake_of(&2), 16);

		// back to 1 vote.
		assert_ok!(vote(Origin::signed(2), vec![4], 12));
		// 2 + 1
		assert_eq!(balances(&2), (17, 3));
		assert_eq!(Elections::voting(&2).deposit, 3);
		assert_eq!(has_lock(&2), 12);
		assert_eq!(locked_stake_of(&2), 12);
	});
}

#[test]
fn cannot_vote_for_no_candidate() {
	ExtBuilder::default().build_and_execute(|| {
		assert_noop!(vote(Origin::signed(2), vec![], 20), Error::<Test>::NoVotes);
	});
}

#[test]
fn can_vote_for_old_members_even_when_no_new_candidates() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));

		assert_ok!(vote(Origin::signed(2), vec![4, 5], 20));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![4, 5]);
		assert!(candidate_ids().is_empty());

		assert_ok!(vote(Origin::signed(3), vec![4, 5], 10));
	});
}

#[test]
fn prime_works() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(3)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(5)));

		assert_ok!(vote(Origin::signed(1), vec![4, 3], 10));
		assert_ok!(vote(Origin::signed(2), vec![4], 20));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(5), vec![5], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![4, 5]);
		assert!(candidate_ids().is_empty());

		assert_ok!(vote(Origin::signed(3), vec![4, 5], 10));
		assert_eq!(PRIME.with(|p| *p.borrow()), Some(4));
	});
}

#[test]
fn prime_votes_for_exiting_members_are_removed() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(3)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(5)));

		assert_ok!(vote(Origin::signed(1), vec![4, 3], 10));
		assert_ok!(vote(Origin::signed(2), vec![4], 20));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(5), vec![5], 50));

		assert_ok!(Elections::renounce_candidacy(Origin::signed(4), Renouncing::Candidate(3)));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![3, 5]);
		assert!(candidate_ids().is_empty());

		assert_eq!(PRIME.with(|p| *p.borrow()), Some(5));
	});
}

#[test]
fn prime_is_kept_if_other_members_leave() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(5)));

		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(5), vec![5], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![4, 5]);
		assert_eq!(PRIME.with(|p| *p.borrow()), Some(5));
		assert_ok!(Elections::renounce_candidacy(Origin::signed(4), Renouncing::Member));

		assert_eq!(members_ids(), vec![5]);
		assert_eq!(PRIME.with(|p| *p.borrow()), Some(5));
	})
}

#[test]
fn prime_is_gone_if_renouncing() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(5)));

		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(5), vec![5], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![4, 5]);
		assert_eq!(PRIME.with(|p| *p.borrow()), Some(5));
		assert_ok!(Elections::renounce_candidacy(Origin::signed(5), Renouncing::Member));

		assert_eq!(members_ids(), vec![4]);
		assert_eq!(PRIME.with(|p| *p.borrow()), None);
	})
}

#[test]
fn cannot_vote_for_more_than_candidates_and_members_and_runners() {
	ExtBuilder::default()
		.desired_runners_up(1)
		.balance_factor(10)
		.build_and_execute(|| {
			// when we have only candidates
			assert_ok!(submit_candidacy(Origin::signed(5)));
			assert_ok!(submit_candidacy(Origin::signed(4)));
			assert_ok!(submit_candidacy(Origin::signed(3)));

			assert_noop!(
				// content of the vote is irrelevant.
				vote(Origin::signed(1), vec![9, 99, 999, 9999], 5),
				Error::<Test>::TooManyVotes,
			);

			assert_ok!(vote(Origin::signed(3), vec![3], 30));
			assert_ok!(vote(Origin::signed(4), vec![4], 40));
			assert_ok!(vote(Origin::signed(5), vec![5], 50));

			System::set_block_number(5);
			Elections::on_initialize(System::block_number());

			// now we have 2 members, 1 runner-up, and 1 new candidate
			assert_ok!(submit_candidacy(Origin::signed(2)));

			assert_ok!(vote(Origin::signed(1), vec![9, 99, 999, 9999], 5));
			assert_noop!(
				vote(Origin::signed(1), vec![9, 99, 999, 9_999, 99_999], 5),
				Error::<Test>::TooManyVotes,
			);
		});
}

#[test]
fn remove_voter_should_work() {
	ExtBuilder::default().voter_bond(8).build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));

		assert_ok!(vote(Origin::signed(2), vec![5], 20));
		assert_ok!(vote(Origin::signed(3), vec![5], 30));

		assert_eq_uvec!(all_voters(), vec![2, 3]);
		assert_eq!(balances(&2), (12, 8));
		assert_eq!(locked_stake_of(&2), 12);
		assert_eq!(balances(&3), (22, 8));
		assert_eq!(locked_stake_of(&3), 22);
		assert_eq!(votes_of(&2), vec![5]);
		assert_eq!(votes_of(&3), vec![5]);

		assert_ok!(Elections::remove_voter(Origin::signed(2)));

		assert_eq_uvec!(all_voters(), vec![3]);
		assert!(votes_of(&2).is_empty());
		assert_eq!(locked_stake_of(&2), 0);

		assert_eq!(balances(&2), (20, 0));
		assert_eq!(Balances::locks(&2).len(), 0);
	});
}

#[test]
fn non_voter_remove_should_not_work() {
	ExtBuilder::default().build_and_execute(|| {
		assert_noop!(Elections::remove_voter(Origin::signed(3)), Error::<Test>::MustBeVoter);
	});
}

#[test]
fn dupe_remove_should_fail() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(vote(Origin::signed(2), vec![5], 20));

		assert_ok!(Elections::remove_voter(Origin::signed(2)));
		assert!(all_voters().is_empty());

		assert_noop!(Elections::remove_voter(Origin::signed(2)), Error::<Test>::MustBeVoter);
	});
}

#[test]
fn removed_voter_should_not_be_counted() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));

		assert_ok!(vote(Origin::signed(5), vec![5], 50));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));

		assert_ok!(Elections::remove_voter(Origin::signed(4)));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![3, 5]);
	});
}

#[test]
fn simple_voting_rounds_should_work() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));

		assert_ok!(vote(Origin::signed(2), vec![5], 20));
		assert_ok!(vote(Origin::signed(4), vec![4], 15));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));

		assert_eq_uvec!(all_voters(), vec![2, 3, 4]);

		assert_eq!(votes_of(&2), vec![5]);
		assert_eq!(votes_of(&3), vec![3]);
		assert_eq!(votes_of(&4), vec![4]);

		assert_eq!(candidate_ids(), vec![3, 4, 5]);
		assert_eq!(<Candidates<Test>>::decode_len().unwrap(), 3);

		assert_eq!(Elections::election_rounds(), 0);

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(balances(&3), (25, 5));
		// votes for 5
		assert_eq!(balances(&2), (18, 2));
		assert_eq!(members_and_stake(), vec![(3, 25), (5, 18)]);
		assert!(Elections::runners_up().is_empty());

		assert_eq_uvec!(all_voters(), vec![2, 3, 4]);
		assert!(candidate_ids().is_empty());
		assert_eq!(<Candidates<Test>>::decode_len(), None);

		assert_eq!(Elections::election_rounds(), 1);
	});
}

#[test]
fn empty_term() {
	ExtBuilder::default().build_and_execute(|| {
		// no candidates, no nothing.
		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		System::assert_last_event(Event::Elections(super::Event::EmptyTerm));
	})
}

#[test]
fn all_outgoing() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));

		assert_ok!(vote(Origin::signed(5), vec![5], 50));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		System::assert_last_event(Event::Elections(super::Event::NewTerm {
			new_members: vec![(4, 35), (5, 45)],
		}));

		assert_eq!(members_and_stake(), vec![(4, 35), (5, 45)]);
		assert_eq!(runners_up_and_stake(), vec![]);

		assert_ok!(Elections::remove_voter(Origin::signed(5)));
		assert_ok!(Elections::remove_voter(Origin::signed(4)));

		System::set_block_number(10);
		Elections::on_initialize(System::block_number());

		System::assert_last_event(Event::Elections(super::Event::NewTerm { new_members: vec![] }));

		// outgoing have lost their bond.
		assert_eq!(balances(&4), (37, 0));
		assert_eq!(balances(&5), (47, 0));
	});
}

#[test]
fn defunct_voter_will_be_counted() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));

		// This guy's vote is pointless for this round.
		assert_ok!(vote(Origin::signed(3), vec![4], 30));
		assert_ok!(vote(Origin::signed(5), vec![5], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_and_stake(), vec![(5, 45)]);
		assert_eq!(Elections::election_rounds(), 1);

		// but now it has a valid target.
		assert_ok!(submit_candidacy(Origin::signed(4)));

		System::set_block_number(10);
		Elections::on_initialize(System::block_number());

		// candidate 4 is affected by an old vote.
		assert_eq!(members_and_stake(), vec![(4, 28), (5, 45)]);
		assert_eq!(Elections::election_rounds(), 2);
		assert_eq_uvec!(all_voters(), vec![3, 5]);
	});
}

#[test]
fn only_desired_seats_are_chosen() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));
		assert_ok!(submit_candidacy(Origin::signed(2)));

		assert_ok!(vote(Origin::signed(2), vec![2], 20));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(5), vec![5], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(Elections::election_rounds(), 1);
		assert_eq!(members_ids(), vec![4, 5]);
	});
}

#[test]
fn phragmen_should_not_self_vote() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert!(candidate_ids().is_empty());
		assert_eq!(Elections::election_rounds(), 1);
		assert!(members_ids().is_empty());

		System::assert_last_event(Event::Elections(super::Event::NewTerm { new_members: vec![] }));
	});
}

#[test]
fn runners_up_should_be_kept() {
	ExtBuilder::default().desired_runners_up(2).build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));
		assert_ok!(submit_candidacy(Origin::signed(2)));

		assert_ok!(vote(Origin::signed(2), vec![3], 20));
		assert_ok!(vote(Origin::signed(3), vec![2], 30));
		assert_ok!(vote(Origin::signed(4), vec![5], 40));
		assert_ok!(vote(Origin::signed(5), vec![4], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());
		// sorted based on account id.
		assert_eq!(members_ids(), vec![4, 5]);
		// sorted based on merit (least -> most)
		assert_eq!(runners_up_ids(), vec![3, 2]);

		// runner ups are still locked.
		assert_eq!(balances(&4), (35, 5));
		assert_eq!(balances(&5), (45, 5));
		assert_eq!(balances(&3), (25, 5));
	});
}

#[test]
fn runners_up_should_be_next_candidates() {
	ExtBuilder::default().desired_runners_up(2).build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));
		assert_ok!(submit_candidacy(Origin::signed(2)));

		assert_ok!(vote(Origin::signed(2), vec![2], 20));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(5), vec![5], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());
		assert_eq!(members_and_stake(), vec![(4, 35), (5, 45)]);
		assert_eq!(runners_up_and_stake(), vec![(2, 15), (3, 25)]);

		assert_ok!(vote(Origin::signed(5), vec![5], 10));

		System::set_block_number(10);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_and_stake(), vec![(3, 25), (4, 35)]);
		assert_eq!(runners_up_and_stake(), vec![(5, 10), (2, 15)]);
	});
}

#[test]
fn runners_up_lose_bond_once_outgoing() {
	ExtBuilder::default().desired_runners_up(1).build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(2)));

		assert_ok!(vote(Origin::signed(2), vec![2], 20));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(5), vec![5], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());
		assert_eq!(members_ids(), vec![4, 5]);
		assert_eq!(runners_up_ids(), vec![2]);
		assert_eq!(balances(&2), (15, 5));

		assert_ok!(submit_candidacy(Origin::signed(3)));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));

		System::set_block_number(10);
		Elections::on_initialize(System::block_number());

		assert_eq!(runners_up_ids(), vec![3]);
		assert_eq!(balances(&2), (15, 2));
	});
}

#[test]
fn members_lose_bond_once_outgoing() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(balances(&5), (50, 0));

		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_eq!(balances(&5), (47, 3));

		assert_ok!(vote(Origin::signed(5), vec![5], 50));
		assert_eq!(balances(&5), (45, 5));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());
		assert_eq!(members_ids(), vec![5]);

		assert_ok!(Elections::remove_voter(Origin::signed(5)));
		assert_eq!(balances(&5), (47, 3));

		System::set_block_number(10);
		Elections::on_initialize(System::block_number());
		assert!(members_ids().is_empty());

		assert_eq!(balances(&5), (47, 0));
	});
}

#[test]
fn candidates_lose_the_bond_when_outgoing() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(3)));

		assert_ok!(vote(Origin::signed(4), vec![5], 40));

		assert_eq!(balances(&5), (47, 3));
		assert_eq!(balances(&3), (27, 3));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![5]);

		// winner
		assert_eq!(balances(&5), (47, 3));
		// loser
		assert_eq!(balances(&3), (27, 0));
	});
}

#[test]
fn current_members_are_always_next_candidate() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));

		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(5), vec![5], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![4, 5]);
		assert_eq!(Elections::election_rounds(), 1);

		assert_ok!(submit_candidacy(Origin::signed(2)));
		assert_ok!(vote(Origin::signed(2), vec![2], 20));

		assert_ok!(submit_candidacy(Origin::signed(3)));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));

		assert_ok!(Elections::remove_voter(Origin::signed(4)));

		// 5 will persist as candidates despite not being in the list.
		assert_eq!(candidate_ids(), vec![2, 3]);

		System::set_block_number(10);
		Elections::on_initialize(System::block_number());

		// 4 removed; 5 and 3 are the new best.
		assert_eq!(members_ids(), vec![3, 5]);
	});
}

#[test]
fn election_state_is_uninterrupted() {
	// what I mean by uninterrupted:
	// given no input or stimulants the same members are re-elected.
	ExtBuilder::default().desired_runners_up(2).build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));
		assert_ok!(submit_candidacy(Origin::signed(2)));

		assert_ok!(vote(Origin::signed(5), vec![5], 50));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));
		assert_ok!(vote(Origin::signed(2), vec![2], 20));

		let check_at_block = |b: u32| {
			System::set_block_number(b.into());
			Elections::on_initialize(System::block_number());
			// we keep re-electing the same folks.
			assert_eq!(members_and_stake(), vec![(4, 35), (5, 45)]);
			assert_eq!(runners_up_and_stake(), vec![(2, 15), (3, 25)]);
			// no new candidates but old members and runners-up are always added.
			assert!(candidate_ids().is_empty());
			assert_eq!(Elections::election_rounds(), b / 5);
			assert_eq_uvec!(all_voters(), vec![2, 3, 4, 5]);
		};

		// this state will always persist when no further input is given.
		check_at_block(5);
		check_at_block(10);
		check_at_block(15);
		check_at_block(20);
	});
}

#[test]
fn remove_members_triggers_election() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));

		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(5), vec![5], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());
		assert_eq!(members_ids(), vec![4, 5]);
		assert_eq!(Elections::election_rounds(), 1);

		// a new candidate
		assert_ok!(submit_candidacy(Origin::signed(3)));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));

		assert_ok!(Elections::remove_member(Origin::root(), 4, false));

		assert_eq!(balances(&4), (35, 2)); // slashed
		assert_eq!(Elections::election_rounds(), 2); // new election round
		assert_eq!(members_ids(), vec![3, 5]); // new members
	});
}

#[test]
fn remove_member_should_indicate_replacement() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));

		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(5), vec![5], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());
		assert_eq!(members_ids(), vec![4, 5]);

		// no replacement yet.
		let unwrapped_error = Elections::remove_member(Origin::root(), 4, true).unwrap_err();
		assert!(matches!(
			unwrapped_error.error,
			DispatchError::Module(ModuleError { message: Some("InvalidReplacement"), .. })
		));
		assert!(unwrapped_error.post_info.actual_weight.is_some());
	});

	ExtBuilder::default().desired_runners_up(1).build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));

		assert_ok!(vote(Origin::signed(3), vec![3], 30));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(5), vec![5], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());
		assert_eq!(members_ids(), vec![4, 5]);
		assert_eq!(runners_up_ids(), vec![3]);

		// there is a replacement! and this one needs a weight refund.
		let unwrapped_error = Elections::remove_member(Origin::root(), 4, false).unwrap_err();
		assert!(matches!(
			unwrapped_error.error,
			DispatchError::Module(ModuleError { message: Some("InvalidReplacement"), .. })
		));
		assert!(unwrapped_error.post_info.actual_weight.is_some());
	});
}

#[test]
fn seats_should_be_released_when_no_vote() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));

		assert_ok!(vote(Origin::signed(2), vec![3], 20));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(5), vec![5], 50));

		assert_eq!(<Candidates<Test>>::decode_len().unwrap(), 3);

		assert_eq!(Elections::election_rounds(), 0);

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());
		assert_eq!(members_ids(), vec![3, 5]);
		assert_eq!(Elections::election_rounds(), 1);

		assert_ok!(Elections::remove_voter(Origin::signed(2)));
		assert_ok!(Elections::remove_voter(Origin::signed(3)));
		assert_ok!(Elections::remove_voter(Origin::signed(4)));
		assert_ok!(Elections::remove_voter(Origin::signed(5)));

		// meanwhile, no one cares to become a candidate again.
		System::set_block_number(10);
		Elections::on_initialize(System::block_number());
		assert!(members_ids().is_empty());
		assert_eq!(Elections::election_rounds(), 2);
	});
}

#[test]
fn incoming_outgoing_are_reported() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(5)));

		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(5), vec![5], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());
		assert_eq!(members_ids(), vec![4, 5]);

		assert_ok!(submit_candidacy(Origin::signed(1)));
		assert_ok!(submit_candidacy(Origin::signed(2)));
		assert_ok!(submit_candidacy(Origin::signed(3)));

		// 5 will change their vote and becomes an `outgoing`
		assert_ok!(vote(Origin::signed(5), vec![4], 8));
		// 4 will stay in the set
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		// 3 will become a winner
		assert_ok!(vote(Origin::signed(3), vec![3], 30));
		// these two are losers.
		assert_ok!(vote(Origin::signed(2), vec![2], 20));
		assert_ok!(vote(Origin::signed(1), vec![1], 10));

		System::set_block_number(10);
		Elections::on_initialize(System::block_number());

		// 3, 4 are new members, must still be bonded, nothing slashed.
		assert_eq!(members_and_stake(), vec![(3, 25), (4, 43)]);
		assert_eq!(balances(&3), (25, 5));
		assert_eq!(balances(&4), (35, 5));

		// 1 is a loser, slashed by 3.
		assert_eq!(balances(&1), (5, 2));

		// 5 is an outgoing loser. will also get slashed.
		assert_eq!(balances(&5), (45, 2));

		System::assert_has_event(Event::Elections(super::Event::NewTerm {
			new_members: vec![(4, 35), (5, 45)],
		}));
	})
}

#[test]
fn invalid_votes_are_moot() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));

		assert_ok!(vote(Origin::signed(3), vec![3], 30));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(5), vec![10], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq_uvec!(members_ids(), vec![3, 4]);
		assert_eq!(Elections::election_rounds(), 1);
	});
}

#[test]
fn members_are_sorted_based_on_id_runners_on_merit() {
	ExtBuilder::default().desired_runners_up(2).build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));
		assert_ok!(submit_candidacy(Origin::signed(2)));

		assert_ok!(vote(Origin::signed(2), vec![3], 20));
		assert_ok!(vote(Origin::signed(3), vec![2], 30));
		assert_ok!(vote(Origin::signed(4), vec![5], 40));
		assert_ok!(vote(Origin::signed(5), vec![4], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());
		// id: low -> high.
		assert_eq!(members_and_stake(), vec![(4, 45), (5, 35)]);
		// merit: low -> high.
		assert_eq!(runners_up_and_stake(), vec![(3, 15), (2, 25)]);
	});
}

#[test]
fn runner_up_replacement_maintains_members_order() {
	ExtBuilder::default().desired_runners_up(2).build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(2)));

		assert_ok!(vote(Origin::signed(2), vec![5], 20));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(5), vec![2], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![2, 4]);
		assert_ok!(Elections::remove_member(Origin::root(), 2, true));
		assert_eq!(members_ids(), vec![4, 5]);
	});
}

#[test]
fn can_renounce_candidacy_member_with_runners_bond_is_refunded() {
	ExtBuilder::default().desired_runners_up(2).build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));
		assert_ok!(submit_candidacy(Origin::signed(2)));

		assert_ok!(vote(Origin::signed(5), vec![5], 50));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));
		assert_ok!(vote(Origin::signed(2), vec![2], 20));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![4, 5]);
		assert_eq!(runners_up_ids(), vec![2, 3]);

		assert_ok!(Elections::renounce_candidacy(Origin::signed(4), Renouncing::Member));
		assert_eq!(balances(&4), (38, 2)); // 2 is voting bond.

		assert_eq!(members_ids(), vec![3, 5]);
		assert_eq!(runners_up_ids(), vec![2]);
	})
}

#[test]
fn can_renounce_candidacy_member_without_runners_bond_is_refunded() {
	ExtBuilder::default().desired_runners_up(2).build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));

		assert_ok!(vote(Origin::signed(5), vec![5], 50));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![4, 5]);
		assert!(runners_up_ids().is_empty());

		assert_ok!(Elections::renounce_candidacy(Origin::signed(4), Renouncing::Member));
		assert_eq!(balances(&4), (38, 2)); // 2 is voting bond.

		// no replacement
		assert_eq!(members_ids(), vec![5]);
		assert!(runners_up_ids().is_empty());
	})
}

#[test]
fn can_renounce_candidacy_runner_up() {
	ExtBuilder::default().desired_runners_up(2).build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));
		assert_ok!(submit_candidacy(Origin::signed(2)));

		assert_ok!(vote(Origin::signed(5), vec![4], 50));
		assert_ok!(vote(Origin::signed(4), vec![5], 40));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));
		assert_ok!(vote(Origin::signed(2), vec![2], 20));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![4, 5]);
		assert_eq!(runners_up_ids(), vec![2, 3]);

		assert_ok!(Elections::renounce_candidacy(Origin::signed(3), Renouncing::RunnerUp));
		assert_eq!(balances(&3), (28, 2)); // 2 is voting bond.

		assert_eq!(members_ids(), vec![4, 5]);
		assert_eq!(runners_up_ids(), vec![2]);
	})
}

#[test]
fn runner_up_replacement_works_when_out_of_order() {
	ExtBuilder::default().desired_runners_up(2).build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));
		assert_ok!(submit_candidacy(Origin::signed(2)));

		assert_ok!(vote(Origin::signed(2), vec![5], 20));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(5), vec![2], 50));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![2, 4]);
		assert_eq!(runners_up_ids(), vec![5, 3]);
		assert_ok!(Elections::renounce_candidacy(Origin::signed(3), Renouncing::RunnerUp));
		assert_eq!(members_ids(), vec![2, 4]);
		assert_eq!(runners_up_ids(), vec![5]);
	});
}

#[test]
fn can_renounce_candidacy_candidate() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_eq!(balances(&5), (47, 3));
		assert_eq!(candidate_ids(), vec![5]);

		assert_ok!(Elections::renounce_candidacy(Origin::signed(5), Renouncing::Candidate(1)));
		assert_eq!(balances(&5), (50, 0));
		assert!(candidate_ids().is_empty());
	})
}

#[test]
fn wrong_renounce_candidacy_should_fail() {
	ExtBuilder::default().build_and_execute(|| {
		assert_noop!(
			Elections::renounce_candidacy(Origin::signed(5), Renouncing::Candidate(0)),
			Error::<Test>::InvalidRenouncing,
		);
		assert_noop!(
			Elections::renounce_candidacy(Origin::signed(5), Renouncing::Member),
			Error::<Test>::InvalidRenouncing,
		);
		assert_noop!(
			Elections::renounce_candidacy(Origin::signed(5), Renouncing::RunnerUp),
			Error::<Test>::InvalidRenouncing,
		);
	})
}

#[test]
fn non_member_renounce_member_should_fail() {
	ExtBuilder::default().desired_runners_up(1).build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));

		assert_ok!(vote(Origin::signed(5), vec![5], 50));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![4, 5]);
		assert_eq!(runners_up_ids(), vec![3]);

		assert_noop!(
			Elections::renounce_candidacy(Origin::signed(3), Renouncing::Member),
			Error::<Test>::InvalidRenouncing,
		);
	})
}

#[test]
fn non_runner_up_renounce_runner_up_should_fail() {
	ExtBuilder::default().desired_runners_up(1).build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));

		assert_ok!(vote(Origin::signed(5), vec![5], 50));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![4, 5]);
		assert_eq!(runners_up_ids(), vec![3]);

		assert_noop!(
			Elections::renounce_candidacy(Origin::signed(4), Renouncing::RunnerUp),
			Error::<Test>::InvalidRenouncing,
		);
	})
}

#[test]
fn wrong_candidate_count_renounce_should_fail() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));

		assert_noop!(
			Elections::renounce_candidacy(Origin::signed(4), Renouncing::Candidate(2)),
			Error::<Test>::InvalidWitnessData,
		);

		assert_ok!(Elections::renounce_candidacy(Origin::signed(4), Renouncing::Candidate(3)));
	})
}

#[test]
fn renounce_candidacy_count_can_overestimate() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));
		// while we have only 3 candidates.
		assert_ok!(Elections::renounce_candidacy(Origin::signed(4), Renouncing::Candidate(4)));
	})
}

#[test]
fn unsorted_runners_up_are_detected() {
	ExtBuilder::default()
		.desired_runners_up(2)
		.desired_members(1)
		.build_and_execute(|| {
			assert_ok!(submit_candidacy(Origin::signed(5)));
			assert_ok!(submit_candidacy(Origin::signed(4)));
			assert_ok!(submit_candidacy(Origin::signed(3)));

			assert_ok!(vote(Origin::signed(5), vec![5], 50));
			assert_ok!(vote(Origin::signed(4), vec![4], 5));
			assert_ok!(vote(Origin::signed(3), vec![3], 15));

			System::set_block_number(5);
			Elections::on_initialize(System::block_number());

			assert_eq!(members_ids(), vec![5]);
			assert_eq!(runners_up_ids(), vec![4, 3]);

			assert_ok!(submit_candidacy(Origin::signed(2)));
			assert_ok!(vote(Origin::signed(2), vec![2], 10));

			System::set_block_number(10);
			Elections::on_initialize(System::block_number());

			assert_eq!(members_ids(), vec![5]);
			assert_eq!(runners_up_ids(), vec![2, 3]);

			// 4 is outgoing runner-up. Slash candidacy bond.
			assert_eq!(balances(&4), (35, 2));
			// 3 stays.
			assert_eq!(balances(&3), (25, 5));
		})
}

#[test]
fn member_to_runner_up_wont_slash() {
	ExtBuilder::default()
		.desired_runners_up(2)
		.desired_members(1)
		.build_and_execute(|| {
			assert_ok!(submit_candidacy(Origin::signed(4)));
			assert_ok!(submit_candidacy(Origin::signed(3)));
			assert_ok!(submit_candidacy(Origin::signed(2)));

			assert_ok!(vote(Origin::signed(4), vec![4], 40));
			assert_ok!(vote(Origin::signed(3), vec![3], 30));
			assert_ok!(vote(Origin::signed(2), vec![2], 20));

			System::set_block_number(5);
			Elections::on_initialize(System::block_number());

			assert_eq!(members_ids(), vec![4]);
			assert_eq!(runners_up_ids(), vec![2, 3]);

			assert_eq!(balances(&4), (35, 5));
			assert_eq!(balances(&3), (25, 5));
			assert_eq!(balances(&2), (15, 5));

			// this guy will shift everyone down.
			assert_ok!(submit_candidacy(Origin::signed(5)));
			assert_ok!(vote(Origin::signed(5), vec![5], 50));

			System::set_block_number(10);
			Elections::on_initialize(System::block_number());

			assert_eq!(members_ids(), vec![5]);
			assert_eq!(runners_up_ids(), vec![3, 4]);

			// 4 went from member to runner-up -- don't slash.
			assert_eq!(balances(&4), (35, 5));
			// 3 stayed runner-up -- don't slash.
			assert_eq!(balances(&3), (25, 5));
			// 2 was removed -- slash.
			assert_eq!(balances(&2), (15, 2));
		});
}

#[test]
fn runner_up_to_member_wont_slash() {
	ExtBuilder::default()
		.desired_runners_up(2)
		.desired_members(1)
		.build_and_execute(|| {
			assert_ok!(submit_candidacy(Origin::signed(4)));
			assert_ok!(submit_candidacy(Origin::signed(3)));
			assert_ok!(submit_candidacy(Origin::signed(2)));

			assert_ok!(vote(Origin::signed(4), vec![4], 40));
			assert_ok!(vote(Origin::signed(3), vec![3], 30));
			assert_ok!(vote(Origin::signed(2), vec![2], 20));

			System::set_block_number(5);
			Elections::on_initialize(System::block_number());

			assert_eq!(members_ids(), vec![4]);
			assert_eq!(runners_up_ids(), vec![2, 3]);

			assert_eq!(balances(&4), (35, 5));
			assert_eq!(balances(&3), (25, 5));
			assert_eq!(balances(&2), (15, 5));

			// swap some votes.
			assert_ok!(vote(Origin::signed(4), vec![2], 40));
			assert_ok!(vote(Origin::signed(2), vec![4], 20));

			System::set_block_number(10);
			Elections::on_initialize(System::block_number());

			assert_eq!(members_ids(), vec![2]);
			assert_eq!(runners_up_ids(), vec![4, 3]);

			// 2 went from runner to member, don't slash
			assert_eq!(balances(&2), (15, 5));
			// 4 went from member to runner, don't slash
			assert_eq!(balances(&4), (35, 5));
			// 3 stayed the same
			assert_eq!(balances(&3), (25, 5));
		});
}

#[test]
fn remove_and_replace_member_works() {
	let setup = || {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));

		assert_ok!(vote(Origin::signed(5), vec![5], 50));
		assert_ok!(vote(Origin::signed(4), vec![4], 40));
		assert_ok!(vote(Origin::signed(3), vec![3], 30));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![4, 5]);
		assert_eq!(runners_up_ids(), vec![3]);
	};

	// member removed, replacement found.
	ExtBuilder::default().desired_runners_up(1).build_and_execute(|| {
		setup();
		assert_eq!(Elections::remove_and_replace_member(&4, false), Ok(true));

		assert_eq!(members_ids(), vec![3, 5]);
		assert_eq!(runners_up_ids().len(), 0);
	});

	// member removed, no replacement found.
	ExtBuilder::default().desired_runners_up(1).build_and_execute(|| {
		setup();
		assert_ok!(Elections::renounce_candidacy(Origin::signed(3), Renouncing::RunnerUp));
		assert_eq!(Elections::remove_and_replace_member(&4, false), Ok(false));

		assert_eq!(members_ids(), vec![5]);
		assert_eq!(runners_up_ids().len(), 0);
	});

	// wrong member to remove.
	ExtBuilder::default().desired_runners_up(1).build_and_execute(|| {
		setup();
		assert!(matches!(Elections::remove_and_replace_member(&2, false), Err(_)));
	});
}

#[test]
fn no_desired_members() {
	// not interested in anything
	ExtBuilder::default()
		.desired_members(0)
		.desired_runners_up(0)
		.build_and_execute(|| {
			assert_eq!(Elections::candidates().len(), 0);

			assert_ok!(submit_candidacy(Origin::signed(4)));
			assert_ok!(submit_candidacy(Origin::signed(3)));
			assert_ok!(submit_candidacy(Origin::signed(2)));

			assert_eq!(Elections::candidates().len(), 3);

			assert_ok!(vote(Origin::signed(4), vec![4], 40));
			assert_ok!(vote(Origin::signed(3), vec![3], 30));
			assert_ok!(vote(Origin::signed(2), vec![2], 20));

			System::set_block_number(5);
			Elections::on_initialize(System::block_number());

			assert_eq!(members_ids().len(), 0);
			assert_eq!(runners_up_ids().len(), 0);
			assert_eq!(all_voters().len(), 3);
			assert_eq!(Elections::candidates().len(), 0);
		});

	// not interested in members
	ExtBuilder::default()
		.desired_members(0)
		.desired_runners_up(2)
		.build_and_execute(|| {
			assert_eq!(Elections::candidates().len(), 0);

			assert_ok!(submit_candidacy(Origin::signed(4)));
			assert_ok!(submit_candidacy(Origin::signed(3)));
			assert_ok!(submit_candidacy(Origin::signed(2)));

			assert_eq!(Elections::candidates().len(), 3);

			assert_ok!(vote(Origin::signed(4), vec![4], 40));
			assert_ok!(vote(Origin::signed(3), vec![3], 30));
			assert_ok!(vote(Origin::signed(2), vec![2], 20));

			System::set_block_number(5);
			Elections::on_initialize(System::block_number());

			assert_eq!(members_ids().len(), 0);
			assert_eq!(runners_up_ids(), vec![3, 4]);
			assert_eq!(all_voters().len(), 3);
			assert_eq!(Elections::candidates().len(), 0);
		});

	// not interested in runners-up
	ExtBuilder::default()
		.desired_members(2)
		.desired_runners_up(0)
		.build_and_execute(|| {
			assert_eq!(Elections::candidates().len(), 0);

			assert_ok!(submit_candidacy(Origin::signed(4)));
			assert_ok!(submit_candidacy(Origin::signed(3)));
			assert_ok!(submit_candidacy(Origin::signed(2)));

			assert_eq!(Elections::candidates().len(), 3);

			assert_ok!(vote(Origin::signed(4), vec![4], 40));
			assert_ok!(vote(Origin::signed(3), vec![3], 30));
			assert_ok!(vote(Origin::signed(2), vec![2], 20));

			System::set_block_number(5);
			Elections::on_initialize(System::block_number());

			assert_eq!(members_ids(), vec![3, 4]);
			assert_eq!(runners_up_ids().len(), 0);
			assert_eq!(all_voters().len(), 3);
			assert_eq!(Elections::candidates().len(), 0);
		});
}

#[test]
fn dupe_vote_is_moot() {
	ExtBuilder::default().desired_members(1).build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));
		assert_ok!(submit_candidacy(Origin::signed(2)));
		assert_ok!(submit_candidacy(Origin::signed(1)));

		// all these duplicate votes will not cause 2 to win.
		assert_ok!(vote(Origin::signed(1), vec![2, 2, 2, 2], 5));
		assert_ok!(vote(Origin::signed(2), vec![2, 2, 2, 2], 20));

		assert_ok!(vote(Origin::signed(3), vec![3], 30));

		System::set_block_number(5);
		Elections::on_initialize(System::block_number());

		assert_eq!(members_ids(), vec![3]);
	})
}

#[test]
fn remove_defunct_voter_works() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(submit_candidacy(Origin::signed(5)));
		assert_ok!(submit_candidacy(Origin::signed(4)));
		assert_ok!(submit_candidacy(Origin::signed(3)));

		// defunct
		assert_ok!(vote(Origin::signed(5), vec![5, 4], 5));
		// defunct
		assert_ok!(vote(Origin::signed(4), vec![4], 5));
		// ok
		assert_ok!(vote(Origin::signed(3), vec![3], 5));
		// ok
		assert_ok!(vote(Origin::signed(2), vec![3, 4], 5));

		assert_ok!(Elections::renounce_candidacy(Origin::signed(5), Renouncing::Candidate(3)));
		assert_ok!(Elections::renounce_candidacy(Origin::signed(4), Renouncing::Candidate(2)));
		assert_ok!(Elections::renounce_candidacy(Origin::signed(3), Renouncing::Candidate(1)));

		assert_ok!(Elections::clean_defunct_voters(Origin::root(), 4, 2));
	})
}

#[test]
fn poor_candidate_submission_should_not_work() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(candidate_ids(), Vec::<u64>::new());
		assert_noop!(
			submit_candidacy(Origin::signed(7)),
			Error::<Test>::InsufficientCandidateFunds,
		);
	});
}

#[test]
fn noncitizen_cannot_submit() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(candidate_ids(), Vec::<u64>::new());
		assert_noop!(
			submit_candidacy(Origin::signed(8)),
			LLMError::<Test>::NonCitizen,
		);
	});
}