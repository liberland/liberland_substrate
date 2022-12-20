use crate as pallet_liberland_legislation;
use frame_support::{
	ord_parameter_types, parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU16, ConstU32, ConstU64, EitherOfDiverse, GenesisBuild},
};
use frame_system as system;
use frame_system::{EnsureRoot, EnsureSigned, EnsureSignedBy};
use pallet_balances::AccountData;
use sp_core::H256;
use sp_runtime::{
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
}

impl pallet_liberland_initializer::Config for Test {}

parameter_types! {
	pub const TOTALLLM: u64 = 70000000u64;
	pub const PREMINTLLM: u64 = 7000000u64;
	pub const ASSETID: u32 = 0u32;
}

impl pallet_llm::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type TotalSupply = TOTALLLM;
	type PreMintedAmount = PREMINTLLM;
	type AssetId = u32;
}

impl pallet_liberland_legislation::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Citizenship = LLM;
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
