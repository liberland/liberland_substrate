#![cfg(test)]

pub use crate as pallet_federated_bridge;
use frame_support::{
	parameter_types,
	traits::{ConstU32, ConstU64, GenesisBuild},
	weights::Weight,
	PalletId,
};
use frame_system::EnsureRoot;
use sp_core::{ConstU16, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Bridge: pallet_federated_bridge,
	}
);

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(
			Weight::from_parts(frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
		);
}
impl frame_system::Config for Test {
	type AccountData = pallet_balances::AccountData<u64>;
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
	type MaxReserves = ConstU32<1>;
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
	type HoldIdentifier = ();
	type MaxHolds = ();
}

parameter_types! {
	pub const BridgePalletId: PalletId = PalletId(*b"defbridg");
	pub const RateLimit: (u64, u64) = (1000, 10);
}

impl pallet_federated_bridge::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type Token = Balances;
	type PalletId = BridgePalletId;
	type MaxRelays = ConstU32<10>;
	type MaxWatchers = ConstU32<10>;
	type WithdrawalDelay = ConstU64<10>;
	type WithdrawalRateLimit = RateLimit;
	type ForceOrigin = EnsureRoot<Self::AccountId>;
	type MaxTotalLocked = ConstU64<10000>;
	type MinimumTransfer = ConstU64<2>;
	type MinimumFee = ConstU64<10>;
	type MaximumFee = ConstU64<100>;
	type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let balances =
		vec![(0, 100), (1, 100), (2, 100), (3, 100), (4, 100), (5, 100), (6, 100), (200, 20000)];
	pallet_balances::GenesisConfig::<Test> { balances: balances.clone() }
		.assimilate_storage(&mut t)
		.unwrap();

	BridgeConfig {
		relays: vec![0, 1, 2].try_into().unwrap(),
		watchers: vec![0, 3].try_into().unwrap(),
		votes_required: 2,
		fee: 4,
		state: pallet_federated_bridge::BridgeState::Active,
		admin: Some(100),
		super_admin: Some(101),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}
