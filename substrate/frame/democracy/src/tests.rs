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

//! The crate's tests.

use super::*;
use crate as pallet_democracy;
use frame_support::{
	assert_noop, assert_ok, ord_parameter_types, parameter_types,
	traits::{
		AsEnsureOriginWithArg, ConstU32, ConstU64, Contains, EitherOfDiverse, EqualPrivilegeOnly,
		OnInitialize, SortedMembers, Everything,
	},
	weights::Weight,
};
use frame_system::{EnsureRoot, EnsureSigned, EnsureSignedBy};
use sp_core::H256;
use sp_runtime::{
	testing::TestSignature,
	traits::{BadOrigin, BlakeTwo256, Hash, IdentityLookup},
	BuildStorage,
	Perbill,
	Permill,
};

mod cancellation;
mod decoders;
mod delegation;
mod external_proposing;
mod fast_tracking;
mod lock_voting;
mod metadata;
mod public_proposals;
mod scheduling;
mod voting;

const AYE: Vote = Vote { aye: true, conviction: Conviction::None };
const NAY: Vote = Vote { aye: false, conviction: Conviction::None };
const BIG_AYE: Vote = Vote { aye: true, conviction: Conviction::Locked1x };
const BIG_NAY: Vote = Vote { aye: false, conviction: Conviction::Locked1x };

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Nfts: pallet_nfts,
		Preimage: pallet_preimage,
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		Democracy: pallet_democracy::{Pallet, Call, Origin<T>, Storage, Config<T>, Event<T>},
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>},
		LLM: pallet_llm::{Pallet, Call, Storage, Config<T>, Event<T>},
		LiberlandInitializer: pallet_liberland_initializer,
	}
);
impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u64;
	type AssetId = u32;
	type AssetIdParameter = codec::Compact<u32>;
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
	type CallbackHandle = ();
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>;
	type RemoveItemsLimit = ConstU32<1000>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

// Test that a fitlered call can be dispatched.
pub struct BaseFilter;
impl Contains<RuntimeCall> for BaseFilter {
	fn contains(call: &RuntimeCall) -> bool {
		!matches!(call, &RuntimeCall::Balances(pallet_balances::Call::force_set_balance { .. }))
	}
}

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(
			Weight::from_parts(frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
		);
}
impl frame_system::Config for Test {
	type BaseCallFilter = BaseFilter;
	type BlockWeights = BlockWeights;
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}
parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * BlockWeights::get().max_block;
}

impl pallet_preimage::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<u64>;
	type BaseDeposit = ConstU64<0>;
	type ByteDeposit = ConstU64<0>;
}

impl pallet_scheduler::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<u64>;
	type MaxScheduledPerBlock = ConstU32<100>;
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = ();
}

impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type MaxLocks = ConstU32<10>;
	type Balance = u64;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type MaxHolds = ();
}

use pallet_nfts::PalletFeatures;
parameter_types! {
	pub storage Features: PalletFeatures = PalletFeatures::all_enabled();
	pub const MaxAttributesPerCall: u32 = 10;
}
impl pallet_nfts::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = u32;
	type ItemId = u32;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<u64>>;
	type ForceOrigin = frame_system::EnsureRoot<u64>;
	type Locker = ();
	type CollectionDeposit = ConstU64<2>;
	type ItemDeposit = ConstU64<1>;
	type MetadataDepositBase = ConstU64<1>;
	type AttributeDepositBase = ConstU64<1>;
	type DepositPerByte = ConstU64<1>;
	type StringLimit = ConstU32<50>;
	type KeyLimit = ConstU32<50>;
	type ValueLimit = ConstU32<50>;
	type ApprovalsLimit = ConstU32<10>;
	type ItemAttributesApprovalsLimit = ConstU32<2>;
	type MaxTips = ConstU32<10>;
	type MaxDeadlineDuration = ConstU64<10000>;
	type Features = Features;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
	type Citizenship = ();
	type MetadataValidator = ();
	type MaxAttributesPerCall = MaxAttributesPerCall;
	type OffchainSignature = TestSignature;
	type OffchainPublic = <TestSignature as sp_runtime::traits::Verify>::Signer;
}
parameter_types! {
	pub static PreimageByteDeposit: u64 = 0;
	pub static InstantAllowed: bool = false;
}
ord_parameter_types! {
	pub const One: u64 = 1;
	pub const Two: u64 = 2;
	pub const Three: u64 = 3;
	pub const Four: u64 = 4;
	pub const Five: u64 = 5;
	pub const Six: u64 = 6;
}
pub struct OneToFive;
impl SortedMembers<u64> for OneToFive {
	fn sorted_members() -> Vec<u64> {
		vec![1, 2, 3, 4, 5]
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn add(_m: &u64) {}
}

parameter_types! {
	pub const TOTALLLM: u64 = 70000000u64;
	pub const PRERELEASELLM: u64 = 7000000u64;
	pub const CitizenshipMinimum: u64 = 5000u64;
	pub const UnlockFactor: Permill = Permill::from_percent(10);
	pub const AssetId: u32 = 1;
	pub const AssetName: &'static str = "LiberTest Merit";
	pub const AssetSymbol: &'static str = "LTM";
	pub const InflationEventInterval: u64 = 1000;
	pub const InflationEventReleaseFactor: Perbill = Perbill::from_parts(8741611);
}

impl pallet_liberland_initializer::Config for Test {}

impl pallet_llm::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type TotalSupply = TOTALLLM;
	type PreReleasedAmount = PRERELEASELLM;
	type CitizenshipMinimumPooledLLM = CitizenshipMinimum;
	type UnlockFactor = UnlockFactor;
	type AssetId = AssetId;
	type AssetName = AssetName;
	type AssetSymbol = AssetSymbol;
	type InflationEventInterval = InflationEventInterval;
	type InflationEventReleaseFactor = InflationEventReleaseFactor;
	type OnLLMPoliticsUnlock = ();
	type SenateOrigin = EnsureRoot<u64>;
	type WeightInfo = ();
	type MaxCourts = ConstU32<1>;
}

parameter_types! {
	pub const MaxAdditionalFields: u32 = 2;
	pub const MaxRegistrars: u32 = 20;
}

type EnsureOneOrRoot = EitherOfDiverse<EnsureRoot<u64>, EnsureSignedBy<One, u64>>;
type EnsureTwoOrRoot = EitherOfDiverse<EnsureRoot<u64>, EnsureSignedBy<Two, u64>>;
impl pallet_identity::Config for Test {
	type RuntimeEvent = RuntimeEvent;
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
	type Citizenship = LLM;
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = pallet_balances::Pallet<Self>;
	type EnactmentPeriod = ConstU64<2>;
	type LaunchPeriod = ConstU64<2>;
	type VotingPeriod = ConstU64<2>;
	type VoteLockingPeriod = ConstU64<3>;
	type FastTrackVotingPeriod = ConstU64<2>;
	type MinimumDeposit = ConstU64<1>;
	type MaxDeposits = ConstU32<1000>;
	type MaxBlacklisted = ConstU32<5>;
	type SubmitOrigin = EnsureSigned<Self::AccountId>;
	type ExternalOrigin = EnsureSignedBy<Two, u64>;
	type ExternalMajorityOrigin = EnsureSignedBy<Three, u64>;
	type ExternalDefaultOrigin = EnsureSignedBy<One, u64>;
	type FastTrackOrigin = EnsureSignedBy<Five, u64>;
	type CancellationOrigin = EnsureSignedBy<Four, u64>;
	type BlacklistOrigin = EnsureRoot<u64>;
	type CancelProposalOrigin = EnsureRoot<u64>;
	type VetoOrigin = EnsureSignedBy<OneToFive, u64>;
	type CooloffPeriod = ConstU64<2>;
	type InstantOrigin = EnsureSignedBy<Six, u64>;
	type InstantAllowed = InstantAllowed;
	type Scheduler = Scheduler;
	type MaxVotes = ConstU32<100>;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
	type MaxProposals = ConstU32<100>;
	type Preimages = Preimage;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Citizenship = LLM;
	type LLM = LLM;
	type LLInitializer = LiberlandInitializer;
	type DelegateeFilter = Everything;
	type ProposalFee = ();
	type ProposalFeeAmount = ConstU64<10>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let balances = vec![(1, 100), (2, 200), (3, 300), (4, 400), (5, 500), (6, 600)];
	let mut llm_balances: Vec<(u64, u64, u64)> = balances.iter().map(|(id, _)| (*id, 6000, 5000)).collect();
	llm_balances.push((7, 1000, 1000));

	pallet_balances::GenesisConfig::<Test> { balances: balances.clone() }
		.assimilate_storage(&mut t)
		.unwrap();
	pallet_democracy::GenesisConfig::<Test>::default()
		.assimilate_storage(&mut t)
		.unwrap();
	pallet_llm::GenesisConfig::<Test>::default().assimilate_storage(&mut t).unwrap();
	pallet_liberland_initializer::GenesisConfig::<Test> {
		citizenship_registrar: Some(0),
		initial_citizens: llm_balances,
		..Default::default()
	}
	.assimilate_storage(&mut t)
	.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

#[test]
fn params_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(Democracy::referendum_count(), 0);
		assert_eq!(Balances::free_balance(42), 0);
		assert_eq!(Balances::total_issuance(), 2100);
	});
}

fn set_balance_proposal(value: u64) -> BoundedCallOf<Test> {
	let inner = pallet_balances::Call::force_set_balance { who: 42, new_free: value };
	let outer = RuntimeCall::Balances(inner);
	Preimage::bound(outer).unwrap()
}

#[test]
fn set_balance_proposal_is_correctly_filtered_out() {
	for i in 0..10 {
		let call = Preimage::realize(&set_balance_proposal(i)).unwrap().0;
		assert!(!<Test as frame_system::Config>::BaseCallFilter::contains(&call));
	}
}

fn propose_set_balance(who: u64, value: u64, delay: u64) -> DispatchResult {
	Democracy::propose(RuntimeOrigin::signed(who), set_balance_proposal(value), delay)
}

fn next_block() {
	System::set_block_number(System::block_number() + 1);
	Scheduler::on_initialize(System::block_number());
	Democracy::begin_block(System::block_number());
}

fn fast_forward_to(n: u64) {
	while System::block_number() < n {
		next_block();
	}
}

fn begin_referendum() -> ReferendumIndex {
	System::set_block_number(0);
	Balances::make_free_balance_be(&1, 100);
	assert_ok!(propose_set_balance(1, 2, 1));
	fast_forward_to(2);
	0
}

fn aye(who: u64) -> AccountVote<u64> {
	AccountVote::Standard { vote: AYE, balance: Balances::free_balance(&who) }
}

fn nay(who: u64) -> AccountVote<u64> {
	AccountVote::Standard { vote: NAY, balance: Balances::free_balance(&who) }
}

fn big_aye(who: u64) -> AccountVote<u64> {
	AccountVote::Standard { vote: BIG_AYE, balance: Balances::free_balance(&who) }
}

fn big_nay(who: u64) -> AccountVote<u64> {
	AccountVote::Standard { vote: BIG_NAY, balance: Balances::free_balance(&who) }
}

fn vote(aye: bool, balance: u64) -> AccountVote<u64> {
	AccountVote::Standard { vote: Vote { aye, conviction: Conviction::None }, balance }
}

fn vote_aye(balance: u64) -> AccountVote<u64> {
	vote(true, balance)
}

fn vote_nay(balance: u64) -> AccountVote<u64> {
	vote(false, balance)
}

fn tally(r: ReferendumIndex) -> Tally<u64> {
	Democracy::referendum_status(r).unwrap().tally
}

/// note a new preimage without registering.
fn note_preimage(who: u64) -> PreimageHash {
	use std::sync::atomic::{AtomicU8, Ordering};
	// note a new preimage on every function invoke.
	static COUNTER: AtomicU8 = AtomicU8::new(0);
	let data = vec![COUNTER.fetch_add(1, Ordering::Relaxed)];
	assert_ok!(Preimage::note_preimage(RuntimeOrigin::signed(who), data.clone()));
	let hash = BlakeTwo256::hash(&data);
	assert!(!Preimage::is_requested(&hash));
	hash
}
