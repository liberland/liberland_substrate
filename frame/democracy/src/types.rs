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

//! Miscellaneous additional datatypes.

use crate::{AccountVote, Conviction, Vote, VoteThreshold};
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Saturating, Zero},
	RuntimeDebug,
};

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum DispatchOrigin {
	Root, // Dispatches as pallet_system::RawOrigin::Root
	Rich, // Dispatches as crate::RawOrigin::Referendum(tally, electorate)
}


/// Info regarding an ongoing referendum.
#[derive(Encode, MaxEncodedLen, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Tally<Balance> {
	/// The number of aye votes, expressed in terms of post-conviction lock-vote.
	pub ayes: Balance,
	/// The number of nay votes, expressed in terms of post-conviction lock-vote.
	pub nays: Balance,
	/// The number of aye voters
	pub aye_voters: u64,
	/// The number of nay voters
	pub nay_voters: u64,
	/// The amount of funds currently expressing its opinion. Pre-conviction.
	pub turnout: Balance,
}

/// Amount of votes and capital placed in delegation for an account.
#[derive(
	Encode, MaxEncodedLen, Decode, Default, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo,
)]
pub struct Delegations<Balance> {
	/// The number of votes (this is post-conviction).
	pub votes: Balance,
	pub voters: u64,
	/// The amount of raw capital, used for the turnout.
	pub capital: Balance,
}

impl<Balance: Saturating> Saturating for Delegations<Balance> {
	fn saturating_add(self, o: Self) -> Self {
		Self {
			votes: self.votes.saturating_add(o.votes),
			voters: self.voters.saturating_add(o.voters),
			capital: self.capital.saturating_add(o.capital),
		}
	}

	fn saturating_sub(self, o: Self) -> Self {
		Self {
			votes: self.votes.saturating_sub(o.votes),
			voters: self.voters.saturating_sub(o.voters),
			capital: self.capital.saturating_sub(o.capital),
		}
	}

	fn saturating_mul(self, _o: Self) -> Self {
		unimplemented!(); // this doesn't make sense with voters
	}

	fn saturating_pow(self, _exp: usize) -> Self {
		unimplemented!(); // this doesn't make sense with voters
	}
}

impl<
		Balance: From<u8>
			+ Zero
			+ Copy
			+ CheckedAdd
			+ CheckedSub
			+ CheckedMul
			+ CheckedDiv
			+ Bounded
			+ Saturating,
	> Tally<Balance>
{
	/// Create a new tally.
	pub fn new(vote: Vote, balance: Balance) -> Self {
		let Delegations { votes, voters, capital } = vote.conviction.votes(balance);
		Self {
			ayes: if vote.aye { votes } else { Zero::zero() },
			nays: if vote.aye { Zero::zero() } else { votes },
			aye_voters: if vote.aye { voters } else { 0 },
			nay_voters: if vote.aye { 0 } else { voters },
			turnout: capital,
		}
	}

	/// Add an account's vote into the tally.
	pub fn add(&mut self, vote: AccountVote<Balance>) -> Option<()> {
		match vote {
			AccountVote::Standard { vote, balance } => {
				let Delegations { votes, capital, voters } = vote.conviction.votes(balance);
				self.turnout = self.turnout.checked_add(&capital)?;
				match vote.aye {
					true => {
						self.ayes = self.ayes.checked_add(&votes)?;
						self.aye_voters = self.aye_voters.checked_add(voters)?;
					},
					false => {
						self.nays = self.nays.checked_add(&votes)?;
						self.nay_voters = self.nay_voters.checked_add(voters)?;
					}
				}
			},
			AccountVote::Split { aye, nay } => {
				let aye = Conviction::None.votes(aye);
				let nay = Conviction::None.votes(nay);
				self.turnout = self.turnout.checked_add(&aye.capital)?.checked_add(&nay.capital)?;
				self.ayes = self.ayes.checked_add(&aye.votes)?;
				self.nays = self.nays.checked_add(&nay.votes)?;
				// FIXME voters?
			},
		}
		Some(())
	}

	/// Remove an account's vote from the tally.
	pub fn remove(&mut self, vote: AccountVote<Balance>) -> Option<()> {
		match vote {
			AccountVote::Standard { vote, balance } => {
				let Delegations { votes, voters, capital } = vote.conviction.votes(balance);
				self.turnout = self.turnout.checked_sub(&capital)?;
				match vote.aye {
					true => {
						self.ayes = self.ayes.checked_sub(&votes)?;
						self.aye_voters = self.aye_voters.checked_sub(voters)?;
					},
					false => {
						self.nays = self.nays.checked_sub(&votes)?;
						self.nay_voters = self.nay_voters.checked_sub(voters)?;
					}
				}
			},
			AccountVote::Split { aye, nay } => {
				let aye = Conviction::None.votes(aye);
				let nay = Conviction::None.votes(nay);
				self.turnout = self.turnout.checked_sub(&aye.capital)?.checked_sub(&nay.capital)?;
				self.ayes = self.ayes.checked_sub(&aye.votes)?;
				self.nays = self.nays.checked_sub(&nay.votes)?;
				// FIXME voters
			},
		}
		Some(())
	}

	/// Increment some amount of votes.
	pub fn increase(&mut self, approve: bool, delegations: Delegations<Balance>) -> Option<()> {
		self.turnout = self.turnout.saturating_add(delegations.capital);
		match approve {
			true => {
				self.ayes = self.ayes.saturating_add(delegations.votes);
				self.aye_voters = self.aye_voters.saturating_add(delegations.voters);
			},
			false => {
				self.nays = self.nays.saturating_add(delegations.votes);
				self.nay_voters = self.nay_voters.saturating_add(delegations.voters);
			},
		}
		Some(())
	}

	/// Decrement some amount of votes.
	pub fn reduce(&mut self, approve: bool, delegations: Delegations<Balance>) -> Option<()> {
		self.turnout = self.turnout.saturating_sub(delegations.capital);
		match approve {
			true => {
				self.ayes = self.ayes.saturating_sub(delegations.votes);
				self.aye_voters = self.aye_voters.saturating_sub(delegations.voters);
			},
			false => {
				self.nays = self.nays.saturating_sub(delegations.votes);
				self.nay_voters = self.nay_voters.saturating_sub(delegations.voters);
			},
		}
		Some(())
	}
}

/// Info regarding an ongoing referendum.
#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct ReferendumStatus<BlockNumber, Proposal, Balance> {
	/// When voting on this referendum will end.
	pub end: BlockNumber,
	/// The proposal being voted on.
	pub proposal: Proposal,
	/// The origin to be used for dispatch
	pub dispatch_origin: DispatchOrigin,
	/// The thresholding mechanism to determine whether it passed.
	pub threshold: VoteThreshold,
	/// The delay (in blocks) to wait after a successful referendum before deploying.
	pub delay: BlockNumber,
	/// The current tally of votes in this referendum.
	pub tally: Tally<Balance>,
}

/// Info regarding a referendum, present or past.
#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum ReferendumInfo<BlockNumber, Proposal, Balance> {
	/// Referendum is happening, the arg is the block number at which it will end.
	Ongoing(ReferendumStatus<BlockNumber, Proposal, Balance>),
	/// Referendum finished at `end`, and has been `approved` or rejected.
	Finished { approved: bool, end: BlockNumber },
}

impl<BlockNumber, Proposal, Balance: Default> ReferendumInfo<BlockNumber, Proposal, Balance> {
	/// Create a new instance.
	pub fn new(
		end: BlockNumber,
		proposal: Proposal,
		dispatch_origin: DispatchOrigin,
		threshold: VoteThreshold,
		delay: BlockNumber,
	) -> Self {
		let s = ReferendumStatus { end, proposal, dispatch_origin, threshold, delay, tally: Tally::default() };
		ReferendumInfo::Ongoing(s)
	}
}

/// Whether an `unvote` operation is able to make actions that are not strictly always in the
/// interest of an account.
pub enum UnvoteScope {
	/// Permitted to do everything.
	Any,
	/// Permitted to do only the changes that do not need the owner's permission.
	OnlyExpired,
}
