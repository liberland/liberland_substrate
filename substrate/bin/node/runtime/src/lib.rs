// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR Assuming one believes your argument, a more charitable conclusion is that some people may be misguided, and a reexamination of their stance may be warranted..

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

// File has been modified by Liberland in 2022. All modifications by Liberland are distributed under the MIT license.

// You should have received a copy of the MIT license along with this program. If not, see https://opensource.org/licenses/MIT

//! The Substrate runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limits.
#![recursion_limit = "1024"]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_election_provider_support::{
	bounds::{ElectionBounds, ElectionBoundsBuilder},
	onchain, BalancingConfig, ElectionDataProvider, SequentialPhragmen, VoteWeight,
};
use frame_support::{
	construct_runtime,
	dispatch::DispatchClass,
	instances::{Instance2},
	ord_parameter_types,
	pallet_prelude::{InherentData, Get},
	inherent::CheckInherentsResult,
	parameter_types,
	traits::{
		Everything,
		fungible::{Balanced, Credit},
        tokens::nonfungibles_v2::Inspect,
		AsEnsureOriginWithArg, ConstBool, ConstU128, ConstU16, ConstU32, MapSuccess,
		Currency, EitherOf, EitherOfDiverse, Imbalance, InstanceFilter,
		KeyOwnerProofSystem, LockIdentifier, OnUnbalanced
	},
	weights::{
		constants::{
			BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
		},
		ConstantMultiplier, IdentityFee, Weight,
	},
	BoundedVec, PalletId,
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot, EnsureSigned, EnsureWithSuccess, EnsureSignedBy
};
pub use node_primitives::{AccountId, Signature};
use node_primitives::{Balance, BlockNumber, Hash, Moment, Nonce};
use pallet_asset_conversion::{NativeOrAssetId, NativeOrAssetIdConverter};
use pallet_election_provider_multi_phase::SolutionAccuracyOf;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_nfts::PalletFeatures;
use pallet_session::historical as pallet_session_historical;
pub use pallet_transaction_payment::{CurrencyAdapter, Multiplier, TargetedFeeAdjustment};
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo};
use sp_api::impl_runtime_apis;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str,
	curve::PiecewiseLinear,
	generic, impl_opaque_keys,
	traits::{
		self, BlakeTwo256, Block as BlockT, Bounded, NumberFor, OpaqueKeys,
		SaturatedConversion, StaticLookup, AccountIdConversion, AccountIdLookup,
	},
	transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, FixedPointNumber, Perbill, Percent, Permill, Perquintill,
	RuntimeDebug
};
use sp_std::prelude::*;
#[cfg(any(feature = "std", test))]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use static_assertions::const_assert;
use sp_runtime::traits::Keccak256;
use sp_runtime::transaction_validity::TransactionLongevity;
use bridge_types::LiberlandAssetId;

pub use bridge_types::{GenericNetworkId, SubNetworkId};

#[cfg(any(feature = "std", test))]
pub use frame_system::Call as SystemCall;
#[cfg(any(feature = "std", test))]
pub use pallet_balances::Call as BalancesCall;
#[cfg(any(feature = "std", test))]
pub use pallet_staking::StakerStatus;
#[cfg(any(feature = "std", test))]
pub use pallet_sudo::Call as SudoCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod impls;
mod migrations;
use impls::{
	Author, ToAccountId,
	IdentityCallFilter, RegistryCallFilter, NftsCallFilter, OnLLMPoliticsUnlock,
	ContainsMember, CouncilAccountCallFilter, EnsureCmp, ContractsCallFilter, SenateAccountCallFilter,
};

/// Constant values used within the runtime.
pub mod constants;
use constants::{currency::*, time::*, llm::*};
use sp_runtime::generic::Era;

/// Generated voter bag information.
mod voter_bags;

/// Runtime API definition for assets.
pub mod assets_api;

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Max size for serialized extrinsic params for this testing runtime.
/// This is a quite arbitrary but empirically battle tested value.
#[cfg(test)]
pub const CALL_PARAMS_MAX_SIZE: usize = 208;

/// Wasm binary unwrapped. If built with `SKIP_WASM_BUILD`, the function panics.
#[cfg(feature = "std")]
pub fn wasm_binary_unwrap() -> &'static [u8] {
	WASM_BINARY.expect(
		"Development wasm binary is not available. This means the client is built with \
		 `SKIP_WASM_BUILD` flag and it is only usable for production chains. Please rebuild with \
		 the flag disabled.",
	)
}

/// Runtime version.
#[cfg(not(feature = "testnet-runtime"))]
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("Liberland"),
	impl_name: create_runtime_str!("liberland-node"),
	authoring_version: 10,
	// Per convention: if the runtime behavior changes, increment spec_version
	// and set impl_version to 0. If only runtime
	// implementation changes and behavior does not, then leave spec_version as
	// is and increment impl_version.
	spec_version: 26,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

#[cfg(feature = "testnet-runtime")]
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("Liberland_testnet"),
	impl_name: create_runtime_str!("liberland-node"),
	authoring_version: 10,
	// Per convention: if the runtime behavior changes, increment spec_version
	// and set impl_version to 0. If only runtime
	// implementation changes and behavior does not, then leave spec_version as
	// is and increment impl_version.
	spec_version: 26,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// The BABE epoch configuration at genesis.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
	sp_consensus_babe::BabeEpochConfiguration {
		c: PRIMARY_PROBABILITY,
		allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
	};

/// Native version.
#[cfg(any(feature = "std", test))]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
		if let Some(fees) = fees_then_tips.next() {
			// for fees, 80% to treasury, 20% to author
			let mut split = fees.ration(80, 20);
			if let Some(tips) = fees_then_tips.next() {
				// for tips, if any, 0% to treasury, 100% to author (though this can be anything)
				tips.ration_merge_into(0, 100, &mut split);
			}
			CouncilAccount::on_unbalanced(split.0);
			Author::on_unbalanced(split.1);
		}
	}
}

/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 1 seconds of compute with a 6 second average block time, with maximum proof size.
const MAXIMUM_BLOCK_WEIGHT: Weight =
	Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND, u64::MAX);

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub MaxCollectivesProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

const_assert!(NORMAL_DISPATCH_RATIO.deconstruct() >= AVERAGE_ON_INITIALIZE_RATIO.deconstruct());

impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type DbWeight = RocksDbWeight;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = Nonce;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = AccountIdLookup<AccountId, ()>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

#[cfg(not(feature = "testnet-runtime"))]
parameter_types! {
	pub const LaunchPeriod: BlockNumber = 14 * DAYS;
	pub const VotingPeriod: BlockNumber = 14 * DAYS;
	pub const TermDuration: BlockNumber = 3 * 30 * DAYS;
	pub const EnactmentPeriod: BlockNumber = 14 * DAYS;
	pub const AssetName: &'static str = "Liberland Merit";
	pub const AssetSymbol: &'static str = "LLM";
	pub const SpendPeriod: BlockNumber = 7 * DAYS;
}

#[cfg(feature = "testnet-runtime")]
parameter_types! {
	pub const LaunchPeriod: BlockNumber = 7 * DAYS;
	pub const VotingPeriod: BlockNumber = 7 * DAYS;
	pub const TermDuration: BlockNumber = 10 * DAYS;
	pub const EnactmentPeriod: BlockNumber = 1 * DAYS;
	pub const AssetName: &'static str = "Liberland Kuna";
	pub const AssetSymbol: &'static str = "LKN";
	pub const SpendPeriod: BlockNumber = 60 * MINUTES;
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = ConstU32<100>;
	type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const AnnouncementDepositBase: Balance = deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
pub enum ProxyType {
	Any,
	NonTransfer,
	Governance,
	Staking,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => !matches!(
				c,
				RuntimeCall::Balances(..) |
					RuntimeCall::Assets(..) |
					RuntimeCall::Nfts(..)
			),
			ProxyType::Governance => matches!(
				c,
				RuntimeCall::Democracy(..) |
					RuntimeCall::Council(..) |
					RuntimeCall::TechnicalCommittee(..) |
					RuntimeCall::Elections(..) |
					RuntimeCall::Treasury(..)
			),
			ProxyType::Staking =>
				matches!(c, RuntimeCall::Staking(..)),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = ConstU32<32>;
	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
	type MaxPending = ConstU32<32>;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

type EnsureCouncilMajority = pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>;
type EnsureSenateMajority = pallet_collective::EnsureProportionMoreThan<AccountId, SenateCollective, 1, 2>;
type EnsureRootOrHalfCouncil = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EnsureCouncilMajority,
>;
type EnsureSenateOrCouncilMajority = EitherOfDiverse<
	EnsureSenateMajority,
	EnsureCouncilMajority,
>;
type EnsureRootOrHalfSenate = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EnsureSenateMajority,
>;

type EnsureRootOrHalfSenateOrCustomCouncil = EitherOfDiverse<
	EnsureRootOrHalfSenate,
	EnsureSignedBy<CouncilAccount, AccountId>
>;

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		RuntimeBlockWeights::get().max_block;
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRootOrHalfSenateOrCustomCouncil;
	#[cfg(feature = "runtime-benchmarks")]
	type MaxScheduledPerBlock = ConstU32<512>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MaxScheduledPerBlock = ConstU32<50>;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
	type OriginPrivilegeCmp = EnsureCmp<EnsureRootOrHalfSenate>;
	type Preimages = Preimage;
}

parameter_types! {
	pub const PreimageMaxSize: u32 = 4096 * 1024;
	pub const PreimageBaseDeposit: Balance = 1 * DOLLARS;
	// One cent: $10,000 / MB
	pub const PreimageByteDeposit: Balance = 1 * CENTS;
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type BaseDeposit = PreimageBaseDeposit;
	type ByteDeposit = PreimageByteDeposit;
}

parameter_types! {
	// NOTE: Currently it is not possible to change the epoch duration after the chain has started.
	//       Attempting to do so will brick block production.
	pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
	pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
	pub const ReportLongevity: u64 =
		BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
}

impl pallet_babe::Config for Runtime {
	type EpochDuration = EpochDuration;
	type ExpectedBlockTime = ExpectedBlockTime;
	type EpochChangeTrigger = pallet_babe::ExternalTrigger;
	type DisabledValidators = Session;
	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
	type MaxNominators = MaxNominatorRewardedPerValidator;
	type KeyOwnerProof =
		<Historical as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::Proof;
	type EquivocationReportSystem =
		pallet_babe::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 1 * DOLLARS;
	// For weight estimation, we assume that the most locks on an individual account will be 50.
	// This number may need to be adjusted in the future if this assumption no longer holds true.
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
	pub const MaxHolds: u32 = 2;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Runtime>;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxHolds = MaxHolds;
}

parameter_types! {
	// LLD transfer is ~156 bytes long (exact value depends on transferred amount)
	// We want 1 LLD transfer to cost 0.01 LLD (so that 1 LLD is OK for 100 simple txs)
	// 0.01 LLD / 156 ~= 0.000064 LLD per byte
	pub const TransactionByteFee: Balance = 6400 * MICROCENTS;
	pub const OperationalFeeMultiplier: u8 = 5;
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
	pub MaximumMultiplier: Multiplier = Bounded::max_value();
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = TargetedFeeAdjustment<
		Self,
		TargetBlockFullness,
		AdjustmentVariable,
		MinimumMultiplier,
		MaximumMultiplier,
	>;
}

impl pallet_asset_conversion_tx_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Fungibles = Assets;
	type OnChargeAssetTransaction =
		pallet_asset_conversion_tx_payment::AssetConversionAdapter<Balances, AssetConversion>;
}

parameter_types! {
	pub const MinimumPeriod: Moment = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	type Moment = Moment;
	type OnTimestampSet = Babe;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
	type EventHandler = (Staking, ImOnline);
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub grandpa: Grandpa,
		pub babe: Babe,
		pub im_online: ImOnline,
		pub authority_discovery: AuthorityDiscovery,
	}
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_staking::StashOf<Self>;
	type ShouldEndSession = Babe;
	type NextSessionRotation = Babe;
	type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

impl pallet_session::historical::Config for Runtime {
	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
	type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

pallet_staking_reward_curve::build! {
	const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
		// for millionth, 10_000 = 0.01 = 1%
		min_inflation:  0_005_000, // a.k.a. I_0; expressed in millionth
		max_inflation:  0_100_000, // a.k.a. inflation for ideal_stake; expressed in millionth
		ideal_stake:    0_750_000,
		falloff:        0_100_000, // a.k.a. decay rate, expressed in millionth
		max_piece_count:       40,
		test_precision: 0_005_000,
	);
}

parameter_types! {
	pub const SessionsPerEra: sp_staking::SessionIndex = 6;
	pub const BondingDuration: sp_staking::EraIndex = 28 * 24 / 6; // 28 days; Era is 6h long
	pub const SlashDeferDuration: sp_staking::EraIndex = BondingDuration::get() / 4;
	pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
	pub const MaxNominatorRewardedPerValidator: u32 = 256;
	pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
	pub OffchainRepeat: BlockNumber = 5;
	pub HistoryDepth: u32 = 84;
}

/// Upper limit on the number of NPOS nominations.
const MAX_QUOTA_NOMINATIONS: u32 = 16;

pub struct StakingBenchmarkingConfig;
impl pallet_staking::BenchmarkingConfig for StakingBenchmarkingConfig {
	type MaxNominators = ConstU32<1000>;
	type MaxValidators = ConstU32<1000>;
}

impl pallet_staking::Config for Runtime {
	type Currency = Balances;
	type CurrencyBalance = Balance;
	type UnixTime = Timestamp;
	type CurrencyToVote = sp_staking::currency_to_vote::U128CurrencyToVote;
	type RewardRemainder = CouncilAccount;
	type RuntimeEvent = RuntimeEvent;
	type Slash = CouncilAccount;
	type Reward = ();
	type SessionsPerEra = SessionsPerEra;
	type BondingDuration = BondingDuration;
	type SlashDeferDuration = SlashDeferDuration;
	/// A majority of the council can cancel the slash.
	type AdminOrigin = EnsureRootOrHalfCouncil;
	type SessionInterface = Self;
	type EraPayout = pallet_staking::ConvertCurve<RewardCurve>;
	type NextNewSession = Session;
	type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
	type OffendingValidatorsThreshold = OffendingValidatorsThreshold;
	type ElectionProvider = ElectionProviderMultiPhase;
	type GenesisElectionProvider = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type VoterList = VoterList;
	type NominationsQuota = pallet_staking::FixedNominationsQuota<MAX_QUOTA_NOMINATIONS>;
	// This a placeholder, to be introduced in the next PR as an instance of bags-list
	type TargetList = pallet_staking::UseValidatorsMap<Self>;
	type MaxUnlockingChunks = ConstU32<32>;
	type HistoryDepth = HistoryDepth;
	type EventListeners = ();
	type WeightInfo = pallet_staking::weights::SubstrateWeight<Runtime>;
	type BenchmarkingConfig = StakingBenchmarkingConfig;
	type Citizenship = LLM;
	#[cfg(any(test,feature = "runtime-benchmarks"))]
	type LLInitializer = LiberlandInitializer;
}

parameter_types! {
	// phase durations. 1/4 of the last session for each.
	pub const SignedPhase: u32 = EPOCH_DURATION_IN_BLOCKS / 4;
	pub const UnsignedPhase: u32 = EPOCH_DURATION_IN_BLOCKS / 4;

	// signed config
	pub const SignedRewardBase: Balance = 1 * DOLLARS;
	pub const SignedDepositBase: Balance = 1 * DOLLARS;
	pub const SignedDepositByte: Balance = 1 * CENTS;

	pub BetterUnsignedThreshold: Perbill = Perbill::from_rational(1u32, 10_000);

	// miner configs
	pub const MultiPhaseUnsignedPriority: TransactionPriority = StakingUnsignedPriority::get() - 1u64;
	pub MinerMaxWeight: Weight = RuntimeBlockWeights::get()
		.get(DispatchClass::Normal)
		.max_extrinsic.expect("Normal extrinsics have a weight limit configured; qed")
		.saturating_sub(BlockExecutionWeight::get());
	// Solution can occupy 90% of normal block size
	pub MinerMaxLength: u32 = Perbill::from_rational(9u32, 10) *
		*RuntimeBlockLength::get()
		.max
		.get(DispatchClass::Normal);
}

frame_election_provider_support::generate_solution_type!(
	#[compact]
	pub struct NposSolution16::<
		VoterIndex = u32,
		TargetIndex = u16,
		Accuracy = sp_runtime::PerU16,
		MaxVoters = MaxElectingVotersSolution,
	>(16)
);

parameter_types! {
	// Note: the EPM in this runtime runs the election on-chain. The election bounds must be
	// carefully set so that an election round fits in one block.
	pub ElectionBoundsMultiPhase: ElectionBounds = ElectionBoundsBuilder::default()
		.voters_count(10_000.into()).targets_count(1_500.into()).build();
	pub ElectionBoundsOnChain: ElectionBounds = ElectionBoundsBuilder::default()
		.voters_count(5_000.into()).targets_count(1_250.into()).build();

	pub MaxNominations: u32 = <NposSolution16 as frame_election_provider_support::NposSolution>::LIMIT as u32;
	pub MaxElectingVotersSolution: u32 = 40_000;
	// The maximum winners that can be elected by the Election pallet which is equivalent to the
	// maximum active validators the staking pallet can have.
	pub MaxActiveValidators: u32 = 1000;
}

/// The numbers configured here could always be more than the the maximum limits of staking pallet
/// to ensure election snapshot will not run out of memory. For now, we set them to smaller values
/// since the staking is bounded and the weight pipeline takes hours for this single pallet.
pub struct ElectionProviderBenchmarkConfig;
impl pallet_election_provider_multi_phase::BenchmarkingConfig for ElectionProviderBenchmarkConfig {
	const VOTERS: [u32; 2] = [1000, 2000];
	const TARGETS: [u32; 2] = [500, 1000];
	const ACTIVE_VOTERS: [u32; 2] = [500, 800];
	const DESIRED_TARGETS: [u32; 2] = [200, 400];
	const SNAPSHOT_MAXIMUM_VOTERS: u32 = 1000;
	const MINER_MAXIMUM_VOTERS: u32 = 1000;
	const MAXIMUM_TARGETS: u32 = 300;
}

/// Maximum number of iterations for balancing that will be executed in the embedded OCW
/// miner of election provider multi phase.
pub const MINER_MAX_ITERATIONS: u32 = 10;

/// A source of random balance for NposSolver, which is meant to be run by the OCW election miner.
pub struct OffchainRandomBalancing;
impl Get<Option<BalancingConfig>> for OffchainRandomBalancing {
	fn get() -> Option<BalancingConfig> {
		use sp_runtime::traits::TrailingZeroInput;
		let iterations = match MINER_MAX_ITERATIONS {
			0 => 0,
			max => {
				let seed = sp_io::offchain::random_seed();
				let random = <u32>::decode(&mut TrailingZeroInput::new(&seed))
					.expect("input is padded with zeroes; qed") %
					max.saturating_add(1);
				random as usize
			},
		};

		let config = BalancingConfig { iterations, tolerance: 0 };
		Some(config)
	}
}

pub struct OnChainSeqPhragmen;
impl onchain::Config for OnChainSeqPhragmen {
	type System = Runtime;
	type Solver = SequentialPhragmen<
		AccountId,
		pallet_election_provider_multi_phase::SolutionAccuracyOf<Runtime>,
	>;
	type DataProvider = <Runtime as pallet_election_provider_multi_phase::Config>::DataProvider;
	type WeightInfo = frame_election_provider_support::weights::SubstrateWeight<Runtime>;
	type MaxWinners = <Runtime as pallet_election_provider_multi_phase::Config>::MaxWinners;
	type Bounds = ElectionBoundsOnChain;
}

impl pallet_election_provider_multi_phase::MinerConfig for Runtime {
	type AccountId = AccountId;
	type MaxLength = MinerMaxLength;
	type MaxWeight = MinerMaxWeight;
	type Solution = NposSolution16;
	type MaxVotesPerVoter =
	<<Self as pallet_election_provider_multi_phase::Config>::DataProvider as ElectionDataProvider>::MaxVotesPerVoter;
	type MaxWinners = MaxActiveValidators;

	// The unsigned submissions have to respect the weight of the submit_unsigned call, thus their
	// weight estimate function is wired to this call's weight.
	fn solution_weight(v: u32, t: u32, a: u32, d: u32) -> Weight {
		<
			<Self as pallet_election_provider_multi_phase::Config>::WeightInfo
			as
			pallet_election_provider_multi_phase::WeightInfo
		>::submit_unsigned(v, t, a, d)
	}
}

impl pallet_election_provider_multi_phase::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EstimateCallFee = TransactionPayment;
	type SignedPhase = SignedPhase;
	type UnsignedPhase = UnsignedPhase;
	type BetterUnsignedThreshold = BetterUnsignedThreshold;
	type BetterSignedThreshold = ();
	type OffchainRepeat = OffchainRepeat;
	type MinerTxPriority = MultiPhaseUnsignedPriority;
	type MinerConfig = Self;
	type SignedMaxSubmissions = ConstU32<10>;
	type SignedRewardBase = SignedRewardBase;
	type SignedDepositBase = SignedDepositBase;
	type SignedDepositByte = SignedDepositByte;
	type SignedMaxRefunds = ConstU32<3>;
	type SignedDepositWeight = ();
	type SignedMaxWeight = MinerMaxWeight;
	type SlashHandler = CouncilAccount; // burn slashes
	type RewardHandler = (); // nothing to do upon rewards
	type DataProvider = Staking;
	type Fallback = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type GovernanceFallback = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type Solver = SequentialPhragmen<AccountId, SolutionAccuracyOf<Self>, OffchainRandomBalancing>;
	type WeightInfo = pallet_election_provider_multi_phase::weights::SubstrateWeight<Self>;
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type MaxWinners = MaxActiveValidators;
	type ElectionBounds = ElectionBoundsMultiPhase;
	type BenchmarkingConfig = ElectionProviderBenchmarkConfig;
}

parameter_types! {
	pub const BagThresholds: &'static [u64] = &voter_bags::THRESHOLDS;
}

type VoterBagsListInstance = pallet_bags_list::Instance1;
impl pallet_bags_list::Config<VoterBagsListInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	/// The voter bags-list is loosely kept up to date, and the real source of truth for the score
	/// of each node is the staking pallet.
	type ScoreProvider = Staking;
	type BagThresholds = BagThresholds;
	type Score = VoteWeight;
	type WeightInfo = pallet_bags_list::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const FastTrackVotingPeriod: BlockNumber = 3 * DAYS;
	pub const MinimumDeposit: Balance = 10 * GRAINS_IN_LLM;
	pub const CooloffPeriod: BlockNumber = 7 * DAYS;
	pub const MaxProposals: u32 = 100;
	pub const ProposalFeeAmount: Balance = 100 * DOLLARS;
}

impl pallet_democracy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type VoteLockingPeriod = EnactmentPeriod; // Same as EnactmentPeriod
	type MinimumDeposit = MinimumDeposit;
	type ExternalOrigin = EnsureCouncilMajority;
	type ExternalMajorityOrigin = EnsureCouncilMajority;

	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;

	/// A unanimous council can have the next scheduled referendum be a straight default-carries

	/// (NTB) vote.
	type ExternalDefaultOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
	type SubmitOrigin = EnsureSigned<AccountId>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin =
		EitherOfDiverse<
			pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 2, 3>,
			EnsureCouncilMajority,
		>;
	type InstantOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>;
	type InstantAllowed = frame_support::traits::ConstBool<true>;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	type CancellationOrigin = EnsureSenateOrCouncilMajority;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		EitherOf<
			EnsureSenateMajority,
			pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>,
		>
	>;
	type BlacklistOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		EnsureSenateOrCouncilMajority
	>;

	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cool-off period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type CooloffPeriod = CooloffPeriod;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = ConstU32<100>;
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
	type MaxProposals = MaxProposals;
	type Citizenship = LLM;
	type LLM = LLM;
	type LLInitializer = LiberlandInitializer;
	type Preimages = Preimage;
	type MaxDeposits = ConstU32<100>;
	type MaxBlacklisted = ConstU32<100>;
	type DelegateeFilter = ContainsMember<Runtime, CouncilCollective>;

	type ProposalFee = CouncilAccount;
	type ProposalFeeAmount = ProposalFeeAmount;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 7 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
	type MaxProposalWeight = MaxCollectivesProposalWeight;
}

parameter_types! {
	pub const CandidacyBond: Balance = 10 * GRAINS_IN_LLM;
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	pub const VotingBondBase: Balance = deposit(1, 64);
	// additional data per vote is 32 bytes (account id).
	pub const VotingBondFactor: Balance = deposit(0, 32);
	pub const DesiredMembers: u32 = 7;
	pub const DesiredRunnersUp: u32 = 7;
	pub const MaxVotesPerVoter: u32 = 16;
	pub const MaxVoters: u32 = 10 * 1000;
	pub const MaxCandidates: u32 = 1000;
	pub const ElectionsPhragmenPalletId: LockIdentifier = *b"phrelect";
}

// Make sure that there are no more than `MaxMembers` members elected via elections-phragmen.
const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());

impl pallet_elections_phragmen::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = ElectionsPhragmenPalletId;
	type Currency = Balances;
	type ChangeMembers = Council;
	// NOTE: this implies that council's genesis members cannot be set directly and must come from
	// this module.
	type InitializeMembers = Council;
	type CurrencyToVote = sp_staking::currency_to_vote::U128CurrencyToVote;
	type CandidacyBond = CandidacyBond;

	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type LoserCandidate = ();
	type KickedMember = ();
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type TermDuration = TermDuration;
	type MaxVoters = MaxVoters;
	type MaxVotesPerVoter = MaxVotesPerVoter;
	type MaxCandidates = MaxCandidates;
	type WeightInfo = pallet_elections_phragmen::weights::SubstrateWeight<Runtime>;
	type Citizenship = LLM;
	type LLM = LLM;
	type LLInitializer = LiberlandInitializer;
}

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 7 * DAYS;
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 100;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
	type MaxProposalWeight = MaxCollectivesProposalWeight;
}

parameter_types! {
	pub const SenateMotionDuration: BlockNumber = 7 * DAYS;
	pub const SenateMaxProposals: u32 = 100;
	pub const SenateMaxMembers: u32 = 100;
}
type SenateCollective = pallet_collective::Instance3;
impl pallet_collective::Config<SenateCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = SenateMotionDuration;
	type MaxProposals = SenateMaxProposals;
	type MaxMembers = SenateMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
	type MaxProposalWeight = MaxCollectivesProposalWeight;
}

impl pallet_membership::Config<pallet_membership::Instance1> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = EnsureRootOrHalfCouncil;
	type RemoveOrigin = EnsureRootOrHalfCouncil;
	type SwapOrigin = EnsureRootOrHalfCouncil;
	type ResetOrigin = EnsureRootOrHalfCouncil;
	type PrimeOrigin = EnsureRootOrHalfCouncil;
	type MembershipInitialized = TechnicalCommittee;
	type MembershipChanged = TechnicalCommittee;
	type MaxMembers = TechnicalMaxMembers;
	type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 1 * DOLLARS;
	pub const Burn: Permill = Permill::from_percent(0);
	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1 * DOLLARS;
	pub const DataDepositPerByte: Balance = 1 * CENTS;
	pub const BountyDepositBase: Balance = 1 * DOLLARS;
	pub const BountyDepositPayoutDelay: BlockNumber = 1 * DAYS;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const BountyUpdatePeriod: BlockNumber = 60 * DAYS;
	pub const MaximumReasonLength: u32 = 300;
	pub const MaxApprovals: u32 = 100;
	pub const MaxBalance: Balance = Balance::max_value();
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRootOrHalfCouncil;
	type RejectOrigin = EnsureRootOrHalfCouncil;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type ProposalBondMaximum = ();
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = ();
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	type MaxApprovals = MaxApprovals;
	type SpendOrigin = EnsureWithSuccess<
		EnsureRootOrHalfCouncil,
		AccountId,
		MaxBalance
	>;
}

parameter_types! {
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 5 * DOLLARS;
	pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
	pub const CuratorDepositMin: Balance = 1 * DOLLARS;
	pub const CuratorDepositMax: Balance = 100 * DOLLARS;
}

parameter_types! {
	pub const ChildBountyValueMinimum: Balance = 1 * DOLLARS;
}


parameter_types! {
	pub const DepositPerItem: Balance = deposit(1, 0);
	pub const DepositPerByte: Balance = deposit(0, 1);
	pub const DefaultDepositLimit: Balance = deposit(1024, 1024 * 1024);
	pub Schedule: pallet_contracts::Schedule<Runtime> = Default::default();
	pub CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(30);
}

impl pallet_contracts::Config for Runtime {
	type Time = Timestamp;
	type Randomness = pallet_babe::RandomnessFromTwoEpochsAgo<Self>;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type CallFilter = ContractsCallFilter;
	type DepositPerItem = DepositPerItem;
	type DepositPerByte = DepositPerByte;
	type DefaultDepositLimit = DefaultDepositLimit;
	type CallStack = [pallet_contracts::Frame<Self>; 5];
	type WeightPrice = pallet_transaction_payment::Pallet<Self>;
	type WeightInfo = pallet_contracts::weights::SubstrateWeight<Self>;
	type ChainExtension = liberland_extension_runtime::LiberlandExtension;
	type Schedule = Schedule;
	type AddressGenerator = pallet_contracts::DefaultAddressGenerator;
	type MaxCodeLen = ConstU32<{ 123 * 1024 }>;
	type MaxStorageKeyLen = ConstU32<128>;
	type UnsafeUnstableInterface = ConstBool<false>;
	type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
	type RuntimeHoldReason = RuntimeHoldReason;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type Migrations = ();
	#[cfg(feature = "runtime-benchmarks")]
	type Migrations = pallet_contracts::migration::codegen::BenchMigrations;
	type MaxDelegateDependencies = ConstU32<32>;
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type Debug = ();
	type Environment = ();
}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
	/// We prioritize im-online heartbeats over election solution submission.
	pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
	pub const MaxAuthorities: u32 = 100;
	pub const MaxKeys: u32 = 10_000;
	pub const MaxPeerInHeartbeats: u32 = 10_000;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		public: <Signature as traits::Verify>::Signer,
		account: AccountId,
		nonce: Nonce,
	) -> Option<(RuntimeCall, <UncheckedExtrinsic as traits::Extrinsic>::SignaturePayload)> {
		// take the biggest period possible.
		let period =
			BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;
		let current_block = System::block_number()
			.saturated_into::<u64>()
			// The `System::block_number` is initialized with `n+1`,
			// so the actual block number is `n`.
			.saturating_sub(1);
		let era = Era::mortal(period, current_block);
		let extra = (
			frame_system::CheckNonZeroSender::<Runtime>::new(),
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(era),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_asset_conversion_tx_payment::ChargeAssetTxPayment::<Runtime>::from(0, None),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				log::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let address = <Runtime as frame_system::Config>::Lookup::unlookup(account);
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (address, signature, extra)))
	}
}

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as traits::Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	RuntimeCall: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = RuntimeCall;
}

impl pallet_im_online::Config for Runtime {
	type AuthorityId = ImOnlineId;
	type RuntimeEvent = RuntimeEvent;
	type NextSessionRotation = Babe;
	type ValidatorSet = Historical;
	type ReportUnresponsiveness = Offences;
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
	type MaxKeys = MaxKeys;
	type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
}

impl pallet_offences::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
	type OnOffenceHandler = Staking;
}

impl pallet_authority_discovery::Config for Runtime {
	type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
	pub const MaxSetIdSessionEntries: u32 = BondingDuration::get() * SessionsPerEra::get();
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
	type MaxNominators = MaxNominatorRewardedPerValidator;
	type MaxSetIdSessionEntries = MaxSetIdSessionEntries;
	type KeyOwnerProof = <Historical as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
	type EquivocationReportSystem =
		pallet_grandpa::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

parameter_types! {
	pub const BasicDeposit: Balance = 1 * CENTS;
	pub const FieldDeposit: Balance = 100 * MILLICENTS;
	pub const SubAccountDeposit: Balance = 500 * MILLICENTS;
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = CouncilAccount;
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type RegistrarOrigin = EnsureRootOrHalfCouncil;
	type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
	type Citizenship = LLM;
}


parameter_types! {
	pub const AssetDeposit: Balance = 100 * DOLLARS;
	pub const ApprovalDeposit: Balance = 1 * DOLLARS;
	pub const StringLimit: u32 = 1028;
	pub const MetadataDepositBase: Balance = 10 * DOLLARS;
	pub const MetadataDepositPerByte: Balance = 1 * DOLLARS;
}

impl pallet_assets::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u128;
	type AssetId = u32;
	type AssetIdParameter = codec::Compact<u32>;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = ConstU128<DOLLARS>;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type CallbackHandle = ();
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	type RemoveItemsLimit = ConstU32<1000>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

ord_parameter_types! {
	pub const AssetConversionOrigin: AccountId = AccountIdConversion::<AccountId>::into_account_truncating(&AssetConversionPalletId::get());
}

impl pallet_assets::Config<Instance2> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u128;
	type AssetId = u32;
	type AssetIdParameter = codec::Compact<u32>;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSignedBy<AssetConversionOrigin, AccountId>>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = ConstU128<DOLLARS>;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	type RemoveItemsLimit = ConstU32<1000>;
	type CallbackHandle = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

parameter_types! {
	pub const AssetConversionPalletId: PalletId = PalletId(*b"py/ascon");
	pub AllowMultiAssetPools: bool = true;
	pub const PoolSetupFee: Balance = 1 * DOLLARS; // should be more or equal to the existential deposit
	pub const MintMinLiquidity: Balance = 100;  // 100 is good enough when the main currency has 10-12 decimals.
	pub const LiquidityWithdrawalFee: Permill = Permill::from_parts(0);
}

impl pallet_asset_conversion::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type AssetBalance = <Self as pallet_balances::Config>::Balance;
	type HigherPrecisionBalance = sp_core::U256;
	type Assets = Assets;
	type Balance = Balance;
	type PoolAssets = PoolAssets;
	type AssetId = <Self as pallet_assets::Config>::AssetId;
	type MultiAssetId = NativeOrAssetId<u32>;
	type PoolAssetId = <Self as pallet_assets::Config<Instance2>>::AssetId;
	type PalletId = AssetConversionPalletId;
	type LPFee = ConstU32<5>; // means 0.5%;
	type PoolSetupFee = PoolSetupFee;
	type PoolSetupFeeReceiver = AssetConversionOrigin;
	type LiquidityWithdrawalFee = LiquidityWithdrawalFee;
	type WeightInfo = pallet_asset_conversion::weights::SubstrateWeight<Runtime>;
	type AllowMultiAssetPools = AllowMultiAssetPools;
	type MaxSwapPathLength = ConstU32<4>;
	type MintMinLiquidity = MintMinLiquidity;
	type MultiAssetIdConverter = NativeOrAssetIdConverter<u32>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

parameter_types! {
	pub const TOTALLLM: Balance      = 70_000_000u128 * GRAINS_IN_LLM;
	pub const PRERELEASELLM: Balance = 13_300_000u128 * GRAINS_IN_LLM;
	pub const CitizenshipMinimum: Balance = 5_000u128 * GRAINS_IN_LLM;
	pub const UnlockFactor: Permill = Permill::from_parts(8742);
	pub const AssetId: u32 = 1;
	pub const InflationEventInterval: BlockNumber = 30 * DAYS;
	pub const InflationEventReleaseFactor: Perbill = Perbill::from_parts(8741611);
}

impl pallet_liberland_initializer::Config for Runtime {}

impl pallet_llm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type TotalSupply = TOTALLLM; //70 million in hardcap
	type PreReleasedAmount = PRERELEASELLM; // PreRelease 7 million
	type CitizenshipMinimumPooledLLM = CitizenshipMinimum;
	type UnlockFactor = UnlockFactor;
	type AssetId = AssetId;
	type AssetName = AssetName;
	type AssetSymbol = AssetSymbol;
	type InflationEventInterval = InflationEventInterval;
	type InflationEventReleaseFactor = InflationEventReleaseFactor;
	type SenateOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		EnsureSenateMajority
	>;
	type OnLLMPoliticsUnlock = OnLLMPoliticsUnlock;
	type WeightInfo = ();
	type MaxCourts = ConstU32<2>;
}

parameter_types! {
	pub const CollectionDeposit: Balance = 100 * DOLLARS;
	pub const ItemDeposit: Balance = 1 * DOLLARS;
	pub const ApprovalsLimit: u32 = 20;
	pub const ItemAttributesApprovalsLimit: u32 = 20;
	pub const MaxTips: u32 = 10;
	pub const MaxDeadlineDuration: BlockNumber = 12 * 30 * DAYS;
	pub Features: PalletFeatures = PalletFeatures::all_enabled();
	pub const MaxAttributesPerCall: u32 = 10;
	pub const LLCoords: (impls::Coords, impls::Coords) = (
		impls::Coords {
			lat: 45_7686480,
			long: 18_8821180,
		},
		impls::Coords {
			lat: 45_7785989,
			long: 18_8892809,
		},
	);
}

impl pallet_nfts::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = u32;
	type ItemId = u32;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type CollectionDeposit = CollectionDeposit;
	type ItemDeposit = ItemDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type AttributeDepositBase = MetadataDepositBase;
	type DepositPerByte = MetadataDepositPerByte;
	type StringLimit = ConstU32<256>;
	type KeyLimit = ConstU32<64>;
	type ValueLimit = ConstU32<256>;
	type ApprovalsLimit = ApprovalsLimit;
	type ItemAttributesApprovalsLimit = ItemAttributesApprovalsLimit;
	type MaxTips = MaxTips;
	type MaxDeadlineDuration = MaxDeadlineDuration;
	type MaxAttributesPerCall = MaxAttributesPerCall;
	type Features = Features;
	type OffchainSignature = Signature;
	type OffchainPublic = <Signature as traits::Verify>::Signer;
	type WeightInfo = pallet_nfts::weights::SubstrateWeight<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type Locker = ();
	type Citizenship = LLM;
	type MetadataValidator = impls::LandMetadataValidator<LLCoords>;
}

impl pallet_transaction_storage::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type RuntimeCall = RuntimeCall;
	type FeeDestination = ();
	type WeightInfo = pallet_transaction_storage::weights::SubstrateWeight<Runtime>;
	type MaxBlockTransactions =
		ConstU32<{ pallet_transaction_storage::DEFAULT_MAX_BLOCK_TRANSACTIONS }>;
	type MaxTransactionSize =
		ConstU32<{ pallet_transaction_storage::DEFAULT_MAX_TRANSACTION_SIZE }>;
}

impl pallet_liberland_legislation::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Citizenship = LLM;
	type ConstitutionOrigin = pallet_democracy::EnsureReferendumProportionAtLeast<Self, 2, 3>;
	type InternationalTreatyOrigin = EnsureRootOrHalfCouncil;
	type LowTierDeleteOrigin = EitherOf<
		EnsureRoot<AccountId>,
		EnsureSenateMajority
	>;
	type LLInitializer = LiberlandInitializer;
	type WeightInfo = pallet_liberland_legislation::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub CompanyRegistryReserveIdentifier: &'static [u8; 8] = b"compregi";
	pub CompanyRegistryMaxRegistrars: u32 = 10u32;
	pub CompanyRegistryBaseDeposit: Balance = 1 * CENTS;
	pub CompanyRegistryByteDeposit: Balance = 10 * MILLICENTS;
	pub CouncilAccountId: AccountId = PalletId(*b"regcounc").into_account_truncating();
}

type EnsureMembersAsAccountId<I, A> = MapSuccess<
	pallet_collective::EnsureProportionMoreThan<AccountId, I, 1, 2>, // half of collective
	ToAccountId<(), A>
>;

type RegistryEnsureRegistrar = EitherOf<
	EnsureSigned<AccountId>,
	EnsureMembersAsAccountId<CouncilCollective, CouncilAccountId>,
>;

type CompanyRegistryInstance = pallet_collective::Instance1;
impl pallet_registry::Config<CompanyRegistryInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EntityData = BoundedVec<u8, ConstU32<8192>>; // max 8KiB data per entity
	type EntityId = u32; // max 4,294,967,295 companies registrations (including removed, IDs arent reused)
	type AddRegistrarOrigin = EnsureRoot<AccountId>;
	type RegistrarOrigin = RegistryEnsureRegistrar;
	type MaxRegistrars = CompanyRegistryMaxRegistrars;
	type BaseDeposit = CompanyRegistryBaseDeposit;
	type ByteDeposit = CompanyRegistryByteDeposit;
	type EntityOrigin = EnsureSigned<AccountId>;
	type ReserveIdentifier = CompanyRegistryReserveIdentifier;
	type WeightInfo = ();
}

parameter_types! {
	pub const IdentityOfficePalletId: PalletId = PalletId(*b"off/iden");
	pub const CompanyRegistryOfficePalletId: PalletId = PalletId(*b"off/comp");
	pub const LandRegistryOfficePalletId: PalletId = PalletId(*b"off/land");
	pub const MetaverseLandRegistryOfficePalletId: PalletId = PalletId(*b"off/meta");
	pub const AssetRegistryOfficePalletId: PalletId = PalletId(*b"off/asse");
}

type IdentityOfficeInstance = pallet_office::Instance1;
type CompanyRegistryOfficeInstance = pallet_office::Instance2;
type LandRegistryOfficeInstance = pallet_office::Instance3;
type MetaverseLandRegistryOfficeInstance = pallet_office::Instance4;
type AssetRegistryOfficeInstance = pallet_office::Instance5;

impl pallet_office::Config<IdentityOfficeInstance> for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type PalletId = IdentityOfficePalletId;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AdminOrigin = EnsureSigned<AccountId>;
	type CallFilter = IdentityCallFilter;
	type WeightInfo = ();
}

impl pallet_office::Config<CompanyRegistryOfficeInstance> for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type PalletId = CompanyRegistryOfficePalletId;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AdminOrigin = EnsureSigned<AccountId>;
	type CallFilter = RegistryCallFilter;
	type WeightInfo = ();
}

impl pallet_office::Config<LandRegistryOfficeInstance> for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type PalletId = LandRegistryOfficePalletId;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AdminOrigin = EnsureSigned<AccountId>;
	type CallFilter = NftsCallFilter;
	type WeightInfo = ();
}

impl pallet_office::Config<MetaverseLandRegistryOfficeInstance> for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type PalletId = MetaverseLandRegistryOfficePalletId;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AdminOrigin = EnsureSigned<AccountId>;
	type CallFilter = NftsCallFilter;
	type WeightInfo = ();
}

impl pallet_office::Config<AssetRegistryOfficeInstance> for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type PalletId = AssetRegistryOfficePalletId;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AdminOrigin = EnsureSigned<AccountId>;
	type CallFilter = NftsCallFilter;
	type WeightInfo = ();
}

parameter_types! {
	pub const LLDBridgePalletId: PalletId = PalletId(*b"lldbridg");
	pub const LLMBridgePalletId: PalletId = PalletId(*b"llmbridg");
	pub const MaxRelays: u32 = 20;
	pub const MaxWatchers: u32 = 20;
	pub const WithdrawalDelay: BlockNumber = 60 * MINUTES;

	// * 66k LLD in single hour (1 burst + hour avg)
	// * 894k LLD in single day (1 burst + 24*hour avg)
	pub const LLDRateLimit: (Balance, Balance) = (
		// max burst and max single withdrawal
		30_000 * DOLLARS,
		// decay per block (max average withdrawal rate over infinite time)
		// 60 LLD * 600 blocks/h = max avg 36k LLD per hour
		60 * DOLLARS
	);
	pub const LLDMaxTotalLocked: Balance = 300_000 * DOLLARS;
	pub const LLDMinimumTransfer: Balance = 30 * DOLLARS;

	// * 22k LLM in single hour (1 burst + hour avg)
	// * 298k LLM in single day (1 burst + 24*hour avg)
	pub const LLMRateLimit: (Balance, Balance) = (
		// max burst and max single withdrawal
		10_000 * GRAINS_IN_LLM,
		// decay per block (max average withdrawal rate over infinite time)
		// 20 LLM * 600 blocks/h = max avg 12k LLM per hour
		20 * GRAINS_IN_LLM
	);
	pub const LLMMaxTotalLocked: Balance = 100_000 * GRAINS_IN_LLM;
	pub const LLMMinimumTransfer: Balance = 10 * GRAINS_IN_LLM;
	pub const BridgeMinimumFee: Balance = 10 * CENTS;
	pub const BridgeMaximumFee: Balance = 10 * DOLLARS;
	pub const BridgeMinimumVotesRequired: u32 = 3;
}

parameter_types! {
	pub const CouncilAccountPalletId: PalletId = PalletId(*b"councilc");
}

impl pallet_custom_account::Config<pallet_custom_account::Instance1> for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type PalletId = CouncilAccountPalletId;
	type ExecuteOrigin = EnsureCouncilMajority;
	type CallFilter = CouncilAccountCallFilter;
	type WeightInfo = ();
	type Currency = Balances;
}

parameter_types! {
	pub const SenateAccountPalletId: PalletId = PalletId(*b"lltreasu");
}

impl pallet_custom_account::Config<pallet_custom_account::Instance2> for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type PalletId = SenateAccountPalletId;
	type ExecuteOrigin = EnsureSenateMajority;
	type CallFilter = SenateAccountCallFilter;
	type WeightInfo = ();
	type Currency = Balances;
}

pub struct IntoAuthor;
impl OnUnbalanced<Credit<AccountId, Balances>> for IntoAuthor {
	fn on_nonzero_unbalanced(credit: Credit<AccountId, Balances>) {
		if let Some(author) = Authorship::author() {
			let _ = <Balances as Balanced<_>>::resolve(&author, credit);
		}
	}
}

parameter_types! {
	pub ContractRegistryReserveIdentifier: &'static [u8; 8] = b"contregi";
	pub ContractRegistryBaseDeposit: Balance = 1 * CENTS;
	pub ContractRegistryByteDeposit: Balance = 10 * MILLICENTS;
}

impl pallet_contracts_registry::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxContractContentLen = ConstU32<{ 2u32 * 1024u32 * 1024u32 }>;
	type MaxParties = ConstU32<16u32>;
	type AddJudgeOrigin = EnsureRoot<AccountId>;
	type SubmitOrigin = EnsureSigned<AccountId>;
	type WeightInfo = pallet_contracts_registry::weights::SubstrateWeight<Runtime>;
	type Currency = Balances;
	type BaseDeposit = ContractRegistryBaseDeposit;
	type ByteDeposit = ContractRegistryByteDeposit;
	type ReserveIdentifier = ContractRegistryReserveIdentifier;
}

// Sora Bridge
parameter_types! {
	pub const BridgeMaxMessagePayloadSize: u32 = 256;
	pub const BridgeMaxMessagesPerCommit: u32 = 20;
	pub const ThisNetworkId: bridge_types::GenericNetworkId = bridge_types::GenericNetworkId::Sub(bridge_types::SubNetworkId::Liberland);
	pub const BridgeMaxPeers: u32 = 50;
	// Not as important as some essential transactions (e.g. im_online or similar ones)
	pub DataSignerPriority: TransactionPriority = Perbill::from_percent(10) * TransactionPriority::max_value();
	// We don't want to have not relevant imports be stuck in transaction pool
	// for too long
	pub DataSignerLongevity: TransactionLongevity = EPOCH_DURATION_IN_BLOCKS as u64;
	pub const MinAssetBalance: u32 = 1;
	pub SoraMainnetTechAcc: AccountId = impls::get_account_id_from_string_hash("Sora");
}

// Sora Bridge
impl leaf_provider::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Hashing = Keccak256;
	type Hash = <Keccak256 as sp_runtime::traits::Hash>::Output;
	type Randomness = pallet_babe::RandomnessFromTwoEpochsAgo<Self>;
}

// Sora Bridge
impl substrate_bridge_app::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OutboundChannel = SubstrateBridgeOutboundChannel;
	type CallOrigin = dispatch::EnsureAccount<
		bridge_types::types::CallOriginOutput<bridge_types::SubNetworkId, sp_core::H256, ()>,
	>;
	type MessageStatusNotifier = SoraBridgeProvider;
	type AssetRegistry = SoraBridgeProvider;
	type AccountIdConverter = impls::SoraAccountIdConverter;
	type AssetIdConverter = impls::SoraAssetIdConverter;
	type BalancePrecisionConverter = impls::GenericBalancePrecisionConverter;
	type BridgeAssetLocker = SoraBridgeProvider;
	type WeightInfo = substrate_bridge_app::weights::SubstrateWeight<Runtime>;
}

// Sora Bridge
impl bridge_data_signer::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OutboundChannel = SubstrateBridgeOutboundChannel;
	type CallOrigin = dispatch::EnsureAccount<
		bridge_types::types::CallOriginOutput<bridge_types::SubNetworkId, sp_core::H256, ()>,
	>;
	type MaxPeers = BridgeMaxPeers;
	type UnsignedPriority = DataSignerPriority;
	type UnsignedLongevity = DataSignerLongevity;
	type WeightInfo = bridge_data_signer::weights::SubstrateWeight<Runtime>;
}

// Sora Bridge
impl multisig_verifier::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CallOrigin = dispatch::EnsureAccount<
		bridge_types::types::CallOriginOutput<bridge_types::SubNetworkId, sp_core::H256, ()>,
	>;
	type OutboundChannel = SubstrateBridgeOutboundChannel;
	type MaxPeers = BridgeMaxPeers;
	type WeightInfo = multisig_verifier::weights::SubstrateWeight<Runtime>;
	type ThisNetworkId = ThisNetworkId;
}

// Sora Bridge
impl dispatch::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OriginOutput =
		bridge_types::types::CallOriginOutput<bridge_types::SubNetworkId, sp_core::H256, ()>;
	type Origin = RuntimeOrigin;
	type MessageId = bridge_types::types::MessageId;
	type Hashing = Keccak256;
	type Call = impls::DispatchableSubstrateBridgeCall;
	type CallFilter = impls::SoraBridgeCallFilter;
	type WeightInfo = dispatch::weights::SubstrateWeight<Runtime>;
}

// Sora Bridge
impl substrate_bridge_channel::inbound::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Verifier = impls::MultiVerifier;
	type MessageDispatch = SubstrateDispatch;
	type UnsignedPriority = DataSignerPriority;
	type UnsignedLongevity = DataSignerLongevity;
	type MaxMessagePayloadSize = BridgeMaxMessagePayloadSize;
	type MaxMessagesPerCommit = BridgeMaxMessagesPerCommit;
	type ThisNetworkId = ThisNetworkId;
	type WeightInfo = substrate_bridge_channel::inbound::weights::SubstrateWeight<Runtime>;
}

// Sora Bridge
impl substrate_bridge_channel::outbound::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MessageStatusNotifier = SoraBridgeProvider;
	type MaxMessagePayloadSize = BridgeMaxMessagePayloadSize;
	type MaxMessagesPerCommit = BridgeMaxMessagesPerCommit;
	type AuxiliaryDigestHandler = LeafProvider;
	type AssetId = bridge_types::LiberlandAssetId;
	type Balance = Balance;
	type TimepointProvider = impls::GenericTimepointProvider;
	type ThisNetworkId = ThisNetworkId;
	type WeightInfo = substrate_bridge_channel::outbound::weights::SubstrateWeight<Runtime>;
}

// Sora Bridge
impl sora_liberland_bridge_provider::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MinBalance = MinAssetBalance;
	type Balances = Balances;
	type AssetId = LiberlandAssetId;
	type SoraApp = SoraBridgeApp;
	type AccountIdConverter = sp_runtime::traits::Identity;
	type TimepointProvider = impls::GenericTimepointProvider;
	type SoraMainnetTechAcc = SoraMainnetTechAcc;
}

construct_runtime!(
	pub struct Runtime
	{
		System: frame_system = 0,
		Utility: pallet_utility = 1,
		Babe: pallet_babe = 2,
		Timestamp: pallet_timestamp = 3,
		// Authorship must be before session in order to note author in the correct session and era
		// for im-online and staking.
		Authorship: pallet_authorship = 4,
		Balances: pallet_balances = 6,
		TransactionPayment: pallet_transaction_payment = 7,
		ElectionProviderMultiPhase: pallet_election_provider_multi_phase = 9,
		Democracy: pallet_democracy = 10,
		Council: pallet_collective::<Instance1> = 11,
		TechnicalCommittee: pallet_collective::<Instance2> = 12,
		TechnicalMembership: pallet_membership::<Instance1> = 13,
		Grandpa: pallet_grandpa = 14,
		Treasury: pallet_treasury = 15,
		Contracts: pallet_contracts = 16,
		Sudo: pallet_sudo = 17,
		ImOnline: pallet_im_online = 18,
		AuthorityDiscovery: pallet_authority_discovery = 19,
		Offences: pallet_offences = 20,
		Historical: pallet_session_historical::{Pallet} = 21,
		Identity: pallet_identity = 23,
		Scheduler: pallet_scheduler = 27,
		Preimage: pallet_preimage = 28,
		Proxy: pallet_proxy = 29,
		Multisig: pallet_multisig = 30,
		Assets: pallet_assets = 33,
		Nfts: pallet_nfts = 38,
		TransactionStorage: pallet_transaction_storage = 39,
		VoterList: pallet_bags_list::<Instance1> = 40,
		LLM: pallet_llm = 46,
		LiberlandLegislation: pallet_liberland_legislation = 47,
		LiberlandInitializer: pallet_liberland_initializer = 48,
		Elections: pallet_elections_phragmen = 49,
		Staking: pallet_staking = 50,
		Session: pallet_session = 51,
		CompanyRegistry: pallet_registry::<Instance1> = 52,
		IdentityOffice: pallet_office::<Instance1> = 53,
		CompanyRegistryOffice: pallet_office::<Instance2> = 54,
		LandRegistryOffice: pallet_office::<Instance3> = 55,
		MetaverseLandRegistryOffice: pallet_office::<Instance4> = 56,
		AssetRegistryOffice: pallet_office::<Instance5> = 57,
		Senate: pallet_collective::<Instance3> = 58,
		CouncilAccount: pallet_custom_account::<Instance1> = 61,
		AssetConversion: pallet_asset_conversion = 62,
		PoolAssets: pallet_assets::<Instance2> = 63,
		AssetConversionTxPayment: pallet_asset_conversion_tx_payment = 64,
		ContractsRegistry: pallet_contracts_registry = 65,
		SenateAccount: pallet_custom_account::<Instance2> = 66,

		// Sora Bridge:
		LeafProvider: leaf_provider = 80,
		SoraBridgeApp: substrate_bridge_app = 81,
		SubstrateBridgeInboundChannel: substrate_bridge_channel::inbound = 82,
		SubstrateBridgeOutboundChannel: substrate_bridge_channel::outbound = 83,
		SubstrateDispatch: dispatch = 84,
		BridgeDataSigner: bridge_data_signer = 85,
		MultisigVerifier: multisig_verifier = 86,
		SoraBridgeProvider: sora_liberland_bridge_provider = 87,
	}
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
///
/// When you change this, you **MUST** modify [`sign`] in `bin/node/testing/src/keyring.rs`!
///
/// [`sign`]: <../../testing/src/keyring.rs.html>
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_asset_conversion_tx_payment::ChargeAssetTxPayment<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	Migrations,
>;


// staking is only expected to be used by polkadot/kusama/et al., so they didn't
// bother to bump the default storage version. as such, we have V7_0_0 version
// set, but it's actually the layout of V12. Fix it before running V13 migration.
mod staking_v12 {
	use super::*;
	use frame_support::{storage_alias, traits::OnRuntimeUpgrade, pallet_prelude::*};

	#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	enum ObsoleteReleases {
		V1_0_0Ancient,
		V2_0_0,
		V3_0_0,
		V4_0_0,
		V5_0_0,
		V6_0_0,
		V7_0_0,
		V8_0_0,
		V9_0_0,
		V10_0_0,
		V11_0_0,
		V12_0_0,
	}

	impl Default for ObsoleteReleases {
		fn default() -> Self {
			Self::V12_0_0
		}
	}

	#[storage_alias]
	type StorageVersion<T: pallet_staking::Config> = StorageValue<pallet_staking::Pallet<T>, ObsoleteReleases, ValueQuery>;

	pub struct Migration<T>(sp_std::marker::PhantomData<T>);
	impl<T: pallet_staking::Config> OnRuntimeUpgrade for Migration<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
			frame_support::ensure!(
                StorageVersion::<T>::get() == ObsoleteReleases::V7_0_0,
                "Expected v7 before upgrading to v12"
            );

            Ok(Default::default())
		}

		fn on_runtime_upgrade() -> Weight {
			StorageVersion::<T>::put(ObsoleteReleases::V12_0_0);
			log::info!("Migrated pallet-staking StorageVersion to V12");
			T::DbWeight::get().reads_writes(1, 1)
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
			frame_support::ensure!(
                StorageVersion::<T>::get() == ObsoleteReleases::V12_0_0,
                "Failed to upgrade to v12"
            );
			Ok(())
		}
	}
}

// All migrations executed on runtime upgrade as a nested tuple of types implementing
// `OnRuntimeUpgrade`.
type Migrations = (
	crate::migrations::add_senate_account_pallet::Migration<Runtime>,
);

type EventRecord = frame_system::EventRecord<
	<Runtime as frame_system::Config>::RuntimeEvent,
	<Runtime as frame_system::Config>::Hash,
>;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	frame_benchmarking::define_benchmarks!(
		[frame_benchmarking, BaselineBench::<Runtime>]
		[pallet_assets, Assets]
		[pallet_babe, Babe]
		[pallet_bags_list, VoterList]
		[pallet_balances, Balances]
		[pallet_collective, Council]
		[pallet_contracts, Contracts]
		[pallet_democracy, Democracy]
		[pallet_asset_conversion, AssetConversion]
		[pallet_election_provider_multi_phase, ElectionProviderMultiPhase]
		[pallet_election_provider_support_benchmarking, EPSBench::<Runtime>]
		[pallet_elections_phragmen, Elections]
		[pallet_grandpa, Grandpa]
		[pallet_identity, Identity]
		[pallet_im_online, ImOnline]
		[pallet_membership, TechnicalMembership]
		[pallet_multisig, Multisig]
		[pallet_offences, OffencesBench::<Runtime>]
		[pallet_preimage, Preimage]
		[pallet_proxy, Proxy]
		[pallet_scheduler, Scheduler]
		[pallet_session, SessionBench::<Runtime>]
		[pallet_staking, Staking]
		[pallet_sudo, Sudo]
		[frame_system, SystemBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[pallet_transaction_storage, TransactionStorage]
		[pallet_treasury, Treasury]
		[pallet_nfts, Nfts]
		[pallet_utility, Utility]
		[pallet_registry, CompanyRegistry]
		[pallet_office, IdentityOffice]
		[pallet_liberland_legislation, LiberlandLegislation]
		[pallet_llm, LLM]
		[pallet_custom_account, CouncilAccount]
		[pallet_contracts_registry, ContractsRegistry]
	);
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> sp_consensus_grandpa::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: sp_consensus_grandpa::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Grandpa::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}

		fn generate_key_ownership_proof(
			_set_id: sp_consensus_grandpa::SetId,
			authority_id: GrandpaId,
		) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
			use codec::Encode;

			Historical::prove((sp_consensus_grandpa::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(sp_consensus_grandpa::OpaqueKeyOwnershipProof::new)
		}
	}

	impl sp_consensus_babe::BabeApi<Block> for Runtime {
		fn configuration() -> sp_consensus_babe::BabeConfiguration {
			let epoch_config = Babe::epoch_config().unwrap_or(BABE_GENESIS_EPOCH_CONFIG);
			sp_consensus_babe::BabeConfiguration {
				slot_duration: Babe::slot_duration(),
				epoch_length: EpochDuration::get(),
				c: epoch_config.c,
				authorities: Babe::authorities().to_vec(),
				randomness: Babe::randomness(),
				allowed_slots: epoch_config.allowed_slots,
			}
		}

		fn current_epoch_start() -> sp_consensus_babe::Slot {
			Babe::current_epoch_start()
		}

		fn current_epoch() -> sp_consensus_babe::Epoch {
			Babe::current_epoch()
		}

		fn next_epoch() -> sp_consensus_babe::Epoch {
			Babe::next_epoch()
		}

		fn generate_key_ownership_proof(
			_slot: sp_consensus_babe::Slot,
			authority_id: sp_consensus_babe::AuthorityId,
		) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
			use codec::Encode;

			Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
			key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Babe::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}
	}

	impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
		fn authorities() -> Vec<AuthorityDiscoveryId> {
			AuthorityDiscovery::authorities()
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl assets_api::AssetsApi<
		Block,
		AccountId,
		Balance,
		u32,
	> for Runtime
	{
		fn account_balances(account: AccountId) -> Vec<(u32, Balance)> {
			Assets::account_balances(account)
		}
	}

	impl pallet_contracts::ContractsApi<Block, AccountId, Balance, BlockNumber, Hash, EventRecord> for Runtime
	{
		fn call(
			origin: AccountId,
			dest: AccountId,
			value: Balance,
			gas_limit: Option<Weight>,
			storage_deposit_limit: Option<Balance>,
			input_data: Vec<u8>,
		) -> pallet_contracts_primitives::ContractExecResult<Balance, EventRecord> {
			let gas_limit = gas_limit.unwrap_or(RuntimeBlockWeights::get().max_block);
			Contracts::bare_call(
				origin,
				dest,
				value,
				gas_limit,
				storage_deposit_limit,
				input_data,
				pallet_contracts::DebugInfo::UnsafeDebug,
				pallet_contracts::CollectEvents::UnsafeCollect,
				pallet_contracts::Determinism::Enforced,
			)
		}

		fn instantiate(
			origin: AccountId,
			value: Balance,
			gas_limit: Option<Weight>,
			storage_deposit_limit: Option<Balance>,
			code: pallet_contracts_primitives::Code<Hash>,
			data: Vec<u8>,
			salt: Vec<u8>,
		) -> pallet_contracts_primitives::ContractInstantiateResult<AccountId, Balance, EventRecord>
		{
			let gas_limit = gas_limit.unwrap_or(RuntimeBlockWeights::get().max_block);
			Contracts::bare_instantiate(
				origin,
				value,
				gas_limit,
				storage_deposit_limit,
				code,
				data,
				salt,
				pallet_contracts::DebugInfo::UnsafeDebug,
				pallet_contracts::CollectEvents::UnsafeCollect,
			)
		}

		fn upload_code(
			origin: AccountId,
			code: Vec<u8>,
			storage_deposit_limit: Option<Balance>,
			determinism: pallet_contracts::Determinism,
		) -> pallet_contracts_primitives::CodeUploadResult<Hash, Balance>
		{
			Contracts::bare_upload_code(
				origin,
				code,
				storage_deposit_limit,
				determinism,
			)
		}

		fn get_storage(
			address: AccountId,
			key: Vec<u8>,
		) -> pallet_contracts_primitives::GetStorageResult {
			Contracts::get_storage(
				address,
				key
			)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
		Block,
		Balance,
	> for Runtime {
		fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_asset_conversion::AssetConversionApi<
		Block,
		Balance,
		u128,
		NativeOrAssetId<u32>
	> for Runtime
	{
		fn quote_price_exact_tokens_for_tokens(asset1: NativeOrAssetId<u32>, asset2: NativeOrAssetId<u32>, amount: u128, include_fee: bool) -> Option<Balance> {
			AssetConversion::quote_price_exact_tokens_for_tokens(asset1, asset2, amount, include_fee)
		}

		fn quote_price_tokens_for_exact_tokens(asset1: NativeOrAssetId<u32>, asset2: NativeOrAssetId<u32>, amount: u128, include_fee: bool) -> Option<Balance> {
			AssetConversion::quote_price_tokens_for_exact_tokens(asset1, asset2, amount, include_fee)
		}

		fn get_reserves(asset1: NativeOrAssetId<u32>, asset2: NativeOrAssetId<u32>) -> Option<(Balance, Balance)> {
			AssetConversion::get_reserves(&asset1, &asset2).ok()
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(call: RuntimeCall, len: u32) -> RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(call: RuntimeCall, len: u32) -> FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_nfts_runtime_api::NftsApi<Block, AccountId, u32, u32> for Runtime {
		fn owner(collection: u32, item: u32) -> Option<AccountId> {
			<Nfts as Inspect<AccountId>>::owner(&collection, &item)
		}

		fn collection_owner(collection: u32) -> Option<AccountId> {
			<Nfts as Inspect<AccountId>>::collection_owner(&collection)
		}

		fn attribute(
			collection: u32,
			item: u32,
			key: Vec<u8>,
		) -> Option<Vec<u8>> {
			<Nfts as Inspect<AccountId>>::attribute(&collection, &item, &key)
		}

		fn custom_attribute(
			account: AccountId,
			collection: u32,
			item: u32,
			key: Vec<u8>,
		) -> Option<Vec<u8>> {
			<Nfts as Inspect<AccountId>>::custom_attribute(
				&account,
				&collection,
				&item,
				&key,
			)
		}

		fn system_attribute(
			collection: u32,
			item: u32,
			key: Vec<u8>,
		) -> Option<Vec<u8>> {
			<Nfts as Inspect<AccountId>>::system_attribute(&collection, &item, &key)
		}

		fn collection_attribute(collection: u32, key: Vec<u8>) -> Option<Vec<u8>> {
			<Nfts as Inspect<AccountId>>::collection_attribute(&collection, &key)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here. If any of the pre/post migration checks fail, we shall stop
			// right here and right now.
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect
		) -> Weight {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check, signature_check, select).unwrap()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;

			// Trying to add benchmarks directly to the Session Pallet caused cyclic dependency
			// issues. To get around that, we separated the Session benchmarks into its own crate,
			// which is why we need these two lines below.
			use pallet_session_benchmarking::Pallet as SessionBench;
			use pallet_offences_benchmarking::Pallet as OffencesBench;
			use pallet_election_provider_support_benchmarking::Pallet as EPSBench;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch};
			use sp_storage::TrackedStorageKey;

			// Trying to add benchmarks directly to the Session Pallet caused cyclic dependency
			// issues. To get around that, we separated the Session benchmarks into its own crate,
			// which is why we need these two lines below.
			use pallet_session_benchmarking::Pallet as SessionBench;
			use pallet_offences_benchmarking::Pallet as OffencesBench;
			use pallet_election_provider_support_benchmarking::Pallet as EPSBench;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			impl pallet_session_benchmarking::Config for Runtime {}
			impl pallet_offences_benchmarking::Config for Runtime {}
			impl pallet_election_provider_support_benchmarking::Config for Runtime {}
			impl frame_system_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}

			use frame_support::traits::WhitelistedStorageKeys;
			let mut whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

			// Treasury Account
			// TODO: this is manual for now, someday we might be able to use a
			// macro for this particular key
			let treasury_key = frame_system::Account::<Runtime>::hashed_key_for(Treasury::account_id());
			whitelist.push(treasury_key.to_vec().into());

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);
			Ok(batches)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_election_provider_support::NposSolution;
	use frame_system::offchain::CreateSignedTransaction;
	use sp_runtime::UpperOf;

	#[test]
	fn validate_transaction_submitter_bounds() {
		fn is_submit_signed_transaction<T>()
		where
			T: CreateSignedTransaction<RuntimeCall>,
		{
		}

		is_submit_signed_transaction::<Runtime>();
	}

	#[test]
	fn perbill_as_onchain_accuracy() {
		type OnChainAccuracy =
			<<Runtime as pallet_election_provider_multi_phase::MinerConfig>::Solution as NposSolution>::Accuracy;
		let maximum_chain_accuracy: Vec<UpperOf<OnChainAccuracy>> = (0..MaxNominations::get())
			.map(|_| <UpperOf<OnChainAccuracy>>::from(OnChainAccuracy::one().deconstruct()))
			.collect();
		let _: UpperOf<OnChainAccuracy> =
			maximum_chain_accuracy.iter().fold(0, |acc, x| acc.checked_add(*x).unwrap());
	}

	#[test]
	fn call_size() {
		let size = core::mem::size_of::<RuntimeCall>();
		assert!(
			size <= CALL_PARAMS_MAX_SIZE,
			"size of RuntimeCall {} is more than {CALL_PARAMS_MAX_SIZE} bytes.
			 Some calls have too big arguments, use Box to reduce the size of RuntimeCall.
			 If the limit is too strong, maybe consider increase the limit.",
			size,
		);
	}
}
