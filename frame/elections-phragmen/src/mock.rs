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

use super::*;
use crate as pallet_elections_phragmen;
use crate as elections_phragmen;
use frame_support::{
	dispatch::DispatchResultWithPostInfo,
	parameter_types,
	traits::{ConstU32, ConstU64},
};
use frame_system::ensure_signed;
use pallet_identity::{IdentityInfo,Data};
use sp_runtime::{
	testing::{H256, Header},
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

use frame_support::{construct_runtime, ord_parameter_types, traits::EnsureOneOf};
use frame_system::{EnsureRoot, EnsureSignedBy};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>},
		Elections: pallet_elections_phragmen::{Pallet, Call, Event<T>, Config<T>},
		LLM: pallet_llm::{Pallet, Call, Storage, Event<T>},
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<2>;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

impl pallet_assets::Config for Test {
	type Event = Event;
	type Balance = u64;
	type AssetId = u32;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<u64>;
	type AssetDeposit = ConstU64<1>;
	type AssetAccountDeposit = ConstU64<10>;
	type MetadataDepositBase = ConstU64<1>;
	type MetadataDepositPerByte = ConstU64<1>;
	type ApprovalDeposit = ConstU64<1>;
	type StringLimit = ConstU32<50>;
	type Freezer = ();
	type WeightInfo = ();
	type Extra = ();
}

parameter_types! {
	pub const MaxAdditionalFields: u32 = 2;
	pub const MaxRegistrars: u32 = 20;
}

ord_parameter_types! {
	pub const One: u64 = 1;
	pub const Two: u64 = 2;
}
type EnsureOneOrRoot = EnsureOneOf<EnsureRoot<u64>, EnsureSignedBy<One, u64>>;
type EnsureTwoOrRoot = EnsureOneOf<EnsureRoot<u64>, EnsureSignedBy<Two, u64>>;
impl pallet_identity::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type Slashed = ();
	type BasicDeposit = ConstU64<0>;
	type FieldDeposit = ConstU64<0>;
	type SubAccountDeposit = ConstU64<0>;
	type MaxSubAccounts = ConstU32<2>;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type RegistrarOrigin = EnsureOneOrRoot;
	type ForceOrigin = EnsureTwoOrRoot;
	type WeightInfo = ();
}

parameter_types! {
	pub const TOTALLLM: u64 = 70000000u64;
	pub const PREMINTLLM: u64 = 7000000u64;
	pub const ASSETID: u32 = 0u32;
}

impl pallet_llm::Config for Test {
	type Event = Event;
	type TotalSupply = TOTALLLM;
	type PreMintedAmount = PREMINTLLM;
	type AssetId = u32;
}
pub struct TestChangeMembers;
impl ChangeMembers<u64> for TestChangeMembers {
	fn change_members_sorted(incoming: &[u64], outgoing: &[u64], new: &[u64]) {
		// new, incoming, outgoing must be sorted.
		let mut new_sorted = new.to_vec();
		new_sorted.sort();
		assert_eq!(new, &new_sorted[..]);

		let mut incoming_sorted = incoming.to_vec();
		incoming_sorted.sort();
		assert_eq!(incoming, &incoming_sorted[..]);

		let mut outgoing_sorted = outgoing.to_vec();
		outgoing_sorted.sort();
		assert_eq!(outgoing, &outgoing_sorted[..]);

		// incoming and outgoing must be disjoint
		for x in incoming.iter() {
			assert!(outgoing.binary_search(x).is_err());
		}

		let mut old_plus_incoming = MEMBERS.with(|m| m.borrow().to_vec());
		old_plus_incoming.extend_from_slice(incoming);
		old_plus_incoming.sort();

		let mut new_plus_outgoing = new.to_vec();
		new_plus_outgoing.extend_from_slice(outgoing);
		new_plus_outgoing.sort();

		assert_eq!(old_plus_incoming, new_plus_outgoing, "change members call is incorrect!");

		MEMBERS.with(|m| *m.borrow_mut() = new.to_vec());
		PRIME.with(|p| *p.borrow_mut() = None);
	}

	fn set_prime(who: Option<u64>) {
		PRIME.with(|p| *p.borrow_mut() = who);
	}

	fn get_prime() -> Option<u64> {
		PRIME.with(|p| *p.borrow())
	}
}

parameter_types! {
	pub const ElectionsPhragmenPalletId: LockIdentifier = *b"phrelect";
	pub static VotingBondBase: u64 = 2;
	pub static VotingBondFactor: u64 = 0;
	pub static CandidacyBond: u64 = 3;
	pub static DesiredMembers: u32 = 2;
	pub static DesiredRunnersUp: u32 = 0;
	pub static TermDuration: u64 = 5;
	pub static Members: Vec<u64> = vec![];
	pub static Prime: Option<u64> = None;
}

impl pallet_elections_phragmen::Config for Test {
	type PalletId = ElectionsPhragmenPalletId;
	type Event = Event;
	type Currency = Balances;
	type CurrencyToVote = frame_support::traits::SaturatingCurrencyToVote;
	type ChangeMembers = TestChangeMembers;
	type InitializeMembers = ();
	type CandidacyBond = CandidacyBond;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type TermDuration = TermDuration;
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type LoserCandidate = ();
	type KickedMember = ();
	type WeightInfo = ();
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Citizenship = LLM;
	type LLM = LLM;
}

pub struct ExtBuilder {
	balance_factor: u64,
	genesis_members: Vec<(u64, u64)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { balance_factor: 1, genesis_members: vec![] }
	}
}

impl ExtBuilder {
	pub fn voter_bond(self, bond: u64) -> Self {
		VOTING_BOND_BASE.with(|v| *v.borrow_mut() = bond);
		self
	}
	pub fn voter_bond_factor(self, bond: u64) -> Self {
		VOTING_BOND_FACTOR.with(|v| *v.borrow_mut() = bond);
		self
	}
	pub fn desired_runners_up(self, count: u32) -> Self {
		DESIRED_RUNNERS_UP.with(|v| *v.borrow_mut() = count);
		self
	}
	pub fn term_duration(self, duration: u64) -> Self {
		TERM_DURATION.with(|v| *v.borrow_mut() = duration);
		self
	}
	pub fn genesis_members(mut self, members: Vec<(u64, u64)>) -> Self {
		MEMBERS
			.with(|m| *m.borrow_mut() = members.iter().map(|(m, _)| m.clone()).collect::<Vec<_>>());
		self.genesis_members = members;
		self
	}
	pub fn desired_members(self, count: u32) -> Self {
		DESIRED_MEMBERS.with(|m| *m.borrow_mut() = count);
		self
	}
	pub fn balance_factor(mut self, factor: u64) -> Self {
		self.balance_factor = factor;
		self
	}
	pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
		MEMBERS.with(|m| {
			*m.borrow_mut() =
				self.genesis_members.iter().map(|(m, _)| m.clone()).collect::<Vec<_>>()
		});
		let balances = vec![
			(1, 10 * self.balance_factor),
			(2, 20 * self.balance_factor),
			(3, 30 * self.balance_factor),
			(4, 40 * self.balance_factor),
			(5, 50 * self.balance_factor),
			(6, 60 * self.balance_factor),
			(7, 1),
		];
		let mut ext: sp_io::TestExternalities = GenesisConfig {
			balances: pallet_balances::GenesisConfig::<Test> { balances: balances.clone() },
			elections: elections_phragmen::GenesisConfig::<Test> { members: self.genesis_members },
		}
		.build_storage()
		.unwrap()
		.into();
		ext.execute_with(|| {
			setup_citizenships(balances);
		});
		ext.execute_with(pre_conditions);
		ext.execute_with(test);
		ext.execute_with(post_conditions)
	}
}

pub fn candidate_ids() -> Vec<u64> {
	Elections::candidates().into_iter().map(|(c, _)| c).collect::<Vec<_>>()
}

pub fn candidate_deposit(who: &u64) -> u64 {
	Elections::candidates()
		.into_iter()
		.find_map(|(c, d)| if c == *who { Some(d) } else { None })
		.unwrap_or_default()
}

pub fn voter_deposit(who: &u64) -> u64 {
	Elections::voting(who).deposit
}

pub fn runners_up_ids() -> Vec<u64> {
	Elections::runners_up().into_iter().map(|r| r.who).collect::<Vec<_>>()
}

pub fn members_ids() -> Vec<u64> {
	Elections::members_ids()
}

pub fn members_and_stake() -> Vec<(u64, u64)> {
	Elections::members().into_iter().map(|m| (m.who, m.stake)).collect::<Vec<_>>()
}

pub fn runners_up_and_stake() -> Vec<(u64, u64)> {
	Elections::runners_up()
		.into_iter()
		.map(|r| (r.who, r.stake))
		.collect::<Vec<_>>()
}

pub fn all_voters() -> Vec<u64> {
	Voting::<Test>::iter().map(|(v, _)| v).collect::<Vec<u64>>()
}

pub fn balances(who: &u64) -> (u64, u64) {
	(Balances::free_balance(who), Balances::reserved_balance(who))
}

pub fn has_lock(who: &u64) -> u64 {
	Balances::locks(who)
		.get(0)
		.cloned()
		.map(|lock| {
			assert_eq!(lock.id, ElectionsPhragmenPalletId::get());
			lock.amount
		})
		.unwrap_or_default()
}

pub fn intersects<T: PartialEq>(a: &[T], b: &[T]) -> bool {
	a.iter().any(|e| b.contains(e))
}

pub fn ensure_members_sorted() {
	let mut members = Elections::members().clone();
	members.sort_by_key(|m| m.who);
	assert_eq!(Elections::members(), members);
}

pub fn ensure_candidates_sorted() {
	let mut candidates = Elections::candidates().clone();
	candidates.sort_by_key(|(c, _)| *c);
	assert_eq!(Elections::candidates(), candidates);
}

pub fn locked_stake_of(who: &u64) -> u64 {
	Voting::<Test>::get(who).stake
}

pub fn ensure_members_has_approval_stake() {
	// we filter members that have no approval state. This means that even we have more seats
	// than candidates, we will never ever chose a member with no votes.
	assert!(Elections::members()
		.iter()
		.chain(Elections::runners_up().iter())
		.all(|s| s.stake != u64::zero()));
}

pub fn ensure_member_candidates_runners_up_disjoint() {
	// members, candidates and runners-up must always be disjoint sets.
	assert!(!intersects(&members_ids(), &candidate_ids()));
	assert!(!intersects(&members_ids(), &runners_up_ids()));
	assert!(!intersects(&candidate_ids(), &runners_up_ids()));
}

pub fn setup_citizenships(account_balances: Vec<(u64, u64)>) {
	let data = Data::Raw(b"1".to_vec().try_into().unwrap());
	let info = IdentityInfo {
		citizen: data.clone(),
		additional: vec![].try_into().unwrap(),
		display: data.clone(),
		legal: data.clone(),
		web: data.clone(),
		riot: data.clone(),
		email: data.clone(),
		pgp_fingerprint: Some([0; 20]),
		image: data,
	};

	Identity::add_registrar(Origin::root(), 0).unwrap();
	for (id, balance) in account_balances {
		let o = Origin::signed(id);
		LLM::fake_send(o.clone(), id, balance).unwrap();
		LLM::politics_lock(o.clone(), balance).unwrap();
		Identity::set_identity(o, Box::new(info.clone())).unwrap();
		Identity::provide_judgement(
			Origin::signed(0),
			0,
			id,
			pallet_identity::Judgement::KnownGood,
		)
		.unwrap();
	}
}

pub fn pre_conditions() {
	System::set_block_number(1);
	ensure_members_sorted();
	ensure_candidates_sorted();
	ensure_member_candidates_runners_up_disjoint();
}

pub fn post_conditions() {
	ensure_members_sorted();
	ensure_candidates_sorted();
	ensure_member_candidates_runners_up_disjoint();
	ensure_members_has_approval_stake();
}

pub fn submit_candidacy(origin: Origin) -> DispatchResultWithPostInfo {
	Elections::submit_candidacy(origin, Elections::candidates().len() as u32)
}

pub fn vote(origin: Origin, votes: Vec<u64>, stake: u64) -> DispatchResultWithPostInfo {
	// historical note: helper function was created in a period of time in which the API of vote
	// call was changing. Currently it is a wrapper for the original call and does not do much.
	// Nonetheless, totally harmless.
	ensure_signed(origin.clone()).expect("vote origin must be signed");
	Elections::vote(origin, votes, stake)
}

pub fn votes_of(who: &u64) -> Vec<u64> {
	Voting::<Test>::get(who).votes
}
