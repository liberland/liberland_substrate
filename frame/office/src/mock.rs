/*
Copyright © 2023 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

#![cfg(test)]

pub use crate as pallet_office;
use codec::MaxEncodedLen;
use frame_support::{
	parameter_types,
	traits::{ConstU64, GenesisBuild, InstanceFilter},
	weights::Weight,
	PalletId,
};
use frame_system::{EnsureRoot, EnsureSigned};
use sp_core::{ConstU16, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Office: pallet_office,
		GenesisOffice: pallet_office::<Instance1>,
	}
);

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(
			Weight::from_parts(frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
		);
}
impl frame_system::Config for Test {
	type AccountData = ();
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
	pub const OfficePalletId: PalletId = PalletId(*b"office  ");
}

use codec::{Decode, Encode};
use frame_support::RuntimeDebug;

#[derive(
	Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, MaxEncodedLen, scale_info::TypeInfo,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum OfficeCallFilter {
	Any,
	Remark,
}

impl Default for OfficeCallFilter {
	fn default() -> Self {
		OfficeCallFilter::Any
	}
}

impl InstanceFilter<RuntimeCall> for OfficeCallFilter {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			OfficeCallFilter::Any => true,
			OfficeCallFilter::Remark => {
				matches!(c, RuntimeCall::System(frame_system::Call::remark { .. }))
			},
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(OfficeCallFilter::Any, _) => true,
			(_, OfficeCallFilter::Any) => false,
			_ => false,
		}
	}
}

impl pallet_office::Config for Test {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type PalletId = OfficePalletId;
	type ForceOrigin = EnsureRoot<u64>;
	type AdminOrigin = EnsureSigned<u64>;
	type CallFilter = OfficeCallFilter;
	type WeightInfo = ();
}

impl pallet_office::Config<pallet_office::Instance1> for Test {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type PalletId = OfficePalletId;
	type ForceOrigin = EnsureRoot<u64>;
	type AdminOrigin = EnsureSigned<u64>;
	type CallFilter = OfficeCallFilter;
	type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	OfficeConfig { admin: Some(0), clerks: vec![] }
		.assimilate_storage(&mut t)
		.unwrap();

	GenesisOfficeConfig {
		admin: Some(99),
		clerks: vec![(100, OfficeCallFilter::Any), (101, OfficeCallFilter::Remark)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}
