use crate as pallet_liberland_legislation;
use frame_support::{
	ord_parameter_types, parameter_types,
	pallet_prelude::Weight,
	traits::{EqualPrivilegeOnly, AsEnsureOriginWithArg, ConstU16, ConstU32, ConstU64, EitherOfDiverse, GenesisBuild},
};
use frame_system as system;
use frame_system::{EnsureRoot, EnsureSigned, EnsureSignedBy};
use pallet_balances::AccountData;
use sp_core::H256;
use sp_runtime::{
	Perbill,
	Permill,
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances,
		Assets: pallet_assets,
		Identity: pallet_identity,
		LiberlandLegislation: pallet_liberland_legislation,
		LLM: pallet_llm,
		LiberlandInitializer: pallet_liberland_initializer,
		Scheduler: pallet_scheduler,
		Democracy: pallet_democracy,
	}
);

impl system::Config for Test {
	type AccountData = AccountData<u64>;
	type AccountId = u64;
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockHashCount = ConstU64<250>;
	type BlockLength = ();
	type BlockNumber = u64;
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = ConstU16<42>;
	type SystemWeightInfo = ();
	type Version = ();
}

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(frame_support::weights::constants::WEIGHT_PER_SECOND.set_proof_size(u64::MAX));
}
parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * BlockWeights::get().max_block;
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

impl pallet_democracy::Config for Test {
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
	type ExternalOrigin = EnsureSignedBy<Two, u64>;
	type ExternalMajorityOrigin = EnsureSignedBy<Two, u64>;
	type ExternalDefaultOrigin = EnsureSignedBy<One, u64>;
	type FastTrackOrigin = EnsureSignedBy<Two, u64>;
	type CancellationOrigin = EnsureSignedBy<Two, u64>;
	type BlacklistOrigin = EnsureRoot<u64>;
	type CancelProposalOrigin = EnsureRoot<u64>;
	type VetoOrigin = EnsureSignedBy<Two, u64>;
	type CooloffPeriod = ConstU64<2>;
	type InstantOrigin = EnsureSignedBy<Two, u64>;
	type InstantAllowed = ();
	type Scheduler = Scheduler;
	type MaxVotes = ConstU32<100>;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
	type MaxProposals = ConstU32<100>;
	type Preimages = ();
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Citizenship = LLM;
	type LLM = LLM;
	type LLInitializer = LiberlandInitializer;
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
}

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
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>;
	type RemoveItemsLimit = ConstU32<1000>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

parameter_types! {
	pub const MaxAdditionalFields: u32 = 2;
	pub const MaxRegistrars: u32 = 20;
}

ord_parameter_types! {
	pub const One: u64 = 1;
	pub const Two: u64 = 2;
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

impl pallet_liberland_initializer::Config for Test {}

parameter_types! {
	pub const TOTALLLM: u64 = 70000000u64;
	pub const PRERELEASELLM: u64 = 7000000u64;
	pub const CitizenshipMinimum: u64 = 5000u64;
	pub const UnlockFactor: Permill = Permill::from_percent(10);
	pub const AssetId: u32 = 1;
	pub const AssetName: &'static str = "LiberTest Merit";
	pub const AssetSymbol: &'static str = "LTM";
	pub const InflationEventInterval: u64 = 1000;
}

impl pallet_llm::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type TotalSupply = TOTALLLM;
	type PreReleasedAmount = PRERELEASELLM;
	type CitizenshipMinimumPooledLLM = CitizenshipMinimum;
	type UnlockFactor = UnlockFactor;
	type AssetId = AssetId;
	type AssetName = AssetName;
	type AssetSymbol = AssetSymbol;
	type InflationEventInterval = InflationEventInterval;
}

impl pallet_liberland_legislation::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Citizenship = LLM;
	type ConstitutionOrigin = pallet_democracy::EnsureReferendumProportionAtLeast<Self, 3, 4>;
	type InternationalTreatyOrigin = EnsureSignedBy<One, u64>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_llm::GenesisConfig::<Test>::default().assimilate_storage(&mut t).unwrap();
	pallet_liberland_initializer::GenesisConfig::<Test> {
		citizenship_registrar: Some(0),
		initial_citizens: vec![(1, 5000, 5000), (2, 5000, 5000), (3, 5000, 5000)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	let mut ext: sp_io::TestExternalities = t.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
