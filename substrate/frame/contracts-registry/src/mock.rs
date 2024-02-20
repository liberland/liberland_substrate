#![cfg(test)]
pub use crate as pallet_contracts_registry;

use frame_support::{
	ord_parameter_types, parameter_types,
	traits::{ConstU32, ConstU64},
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_balances::AccountData;
use sp_core::{ConstU16, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

ord_parameter_types! {
	pub const One: u64 = 1;
	pub const Two: u64 = 2;
}
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		ContractsRegistry: pallet_contracts_registry
	}
);

impl frame_system::Config for Test {
	type AccountData = AccountData<u64>;
	type AccountId = u64;
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockHashCount = ConstU64<250>;
	type BlockLength = ();
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = H256;
	type Block = Block;
	type Nonce = u64;
	type Hashing = BlakeTwo256;
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
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxHolds = ();
}

parameter_types! {
	pub const ReserveIdentifier: &'static [u8; 8] = b"cregistr";
}

impl pallet_contracts_registry::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaxContractContentLen = ConstU32<{ 2u32 * 1024u32 * 1024u32 }>;
	type MaxSignatures = ConstU32<16u32>;
	type AddJudgeOrigin = EnsureRoot<Self::AccountId>;
	type SubmitOrigin = EnsureSigned<Self::AccountId>;
	type WeightInfo = ();
	type Currency = Balances;
	type BaseDeposit = ConstU64<1>;
	type ByteDeposit = ConstU64<2>;
	type ReserveIdentifier = ReserveIdentifier;
}

// Build genesis storage according to the mock runtime.

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let balances = vec![(1, 100), (2, 100), (3, 100), (4, 40), (5, 50)];
	pallet_balances::GenesisConfig::<Test> { balances: balances.clone() }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}
