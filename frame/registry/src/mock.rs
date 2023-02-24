pub use crate as pallet_registry;
use frame_support::{
	parameter_types,
	traits::{ConstU32, ConstU64},
	weights::Weight,
};
use frame_system::EnsureRoot;
use sp_core::{ConstU16, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BoundedVec,
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
		Registry: pallet_registry::{Pallet, Call, Storage, Config<T>, Event<T>},
		SecondRegistry: pallet_registry::<Instance2>,
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
}

parameter_types! {
	pub ReserveIdentifier: &'static [u8; 8] = b"registry";
}

impl pallet_registry::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EntityData = BoundedVec<u8, ConstU32<5>>;
	type MaxRegistrars = ConstU32<2>;
	type BaseDeposit = ConstU64<1>;
	type ByteDeposit = ConstU64<2>;
	type RegistrarOrigin = EnsureRoot<u64>;
	type ReserveIdentifier = ReserveIdentifier;
}

impl pallet_registry::Config<pallet_registry::Instance2> for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EntityData = BoundedVec<u8, ConstU32<5>>;
	type MaxRegistrars = ConstU32<2>;
	type BaseDeposit = ConstU64<1>;
	type ByteDeposit = ConstU64<2>;
	type RegistrarOrigin = EnsureRoot<u64>;
	type ReserveIdentifier = ReserveIdentifier;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let balances = vec![(0, 100), (1, 100), (2, 100), (3, 3)];
	pallet_balances::GenesisConfig::<Test> { balances: balances.clone() }
		.assimilate_storage(&mut t)
		.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}
