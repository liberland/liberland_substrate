#![cfg(test)]

use crate as pallet_llm;
use frame_support::{
	ord_parameter_types, parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU32, ConstU64, EitherOfDiverse},
	weights::Weight,
};
use frame_system::{EnsureRoot, EnsureSigned, EnsureSignedBy};
use pallet_identity::{Data, IdentityInfo};
use sp_core::{ConstU16, H256};
use sp_runtime::{
	traits::{BlakeTwo256, Hash, IdentityLookup},
	BuildStorage, Perbill, Permill,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Assets: pallet_assets,
		Identity: pallet_identity,
		LLM: pallet_llm,
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
	type Block = Block;
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Nonce = u64;
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
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxHolds = ();
}

parameter_types! {
	pub const TOTALLLM: u64 = 70000000u64;
	pub const PRERELEASELLM: u64 = 7000000u64;
	pub const CitizenshipMinimum: u64 = 5000u64;
	pub const UnlockFactor: Permill = Permill::from_parts(8742);
	pub const AssetId: u32 = 1;
	pub const AssetName: &'static str = "LiberTest Merit";
	pub const AssetSymbol: &'static str = "LTM";
	pub const InflationEventInterval: u64 = 30*24*3600/6;
	pub const InflationEventReleaseFactor: Perbill = Perbill::from_parts(8741611);
}

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
	type MaxCourts = ConstU32<3>;
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

pub fn setup_citizenships(accounts: Vec<u64>) {
	let data = Data::Raw(b"1".to_vec().try_into().unwrap());
	let eligible_on = (
		Data::Raw(b"eligible_on".to_vec().try_into().unwrap()),
		Data::Raw(vec![0].try_into().unwrap()),
	);
	let citizen = (Data::Raw(b"citizen".to_vec().try_into().unwrap()), data.clone());
	let info = IdentityInfo {
		twitter: data.clone(),
		additional: vec![eligible_on, citizen].try_into().unwrap(),
		display: data.clone(),
		legal: data.clone(),
		web: data.clone(),
		riot: data.clone(),
		email: data.clone(),
		pgp_fingerprint: Some([0; 20]),
		image: data,
	};

	Identity::add_registrar(RuntimeOrigin::root(), 0).unwrap();
	for id in accounts {
		let o = RuntimeOrigin::signed(id);
		Identity::set_identity(o, Box::new(info.clone())).unwrap();
		Identity::provide_judgement(
			RuntimeOrigin::signed(0),
			0,
			id,
			pallet_identity::Judgement::KnownGood,
			BlakeTwo256::hash_of(&info),
		)
		.unwrap();
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let treasury = LLM::get_llm_treasury_account();
	let balances = vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (treasury, 100)];
	pallet_balances::GenesisConfig::<Test> { balances: balances.clone() }
		.assimilate_storage(&mut t)
		.unwrap();
	pallet_llm::GenesisConfig::<Test> {
		unpooling_withdrawlock_duration: 180,
		unpooling_electionlock_duration: 190,
		_phantom: Default::default(),
	}
	.assimilate_storage(&mut t)
	.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
		setup_citizenships(balances.into_iter().map(|(acc, _)| acc).collect());
		LLM::transfer_from_vault(1, 6000).unwrap();
		LLM::transfer_from_vault(2, 6000).unwrap();
		LLM::set_courts(RuntimeOrigin::root(), vec![1].try_into().unwrap()).unwrap();
	});
	ext
}
