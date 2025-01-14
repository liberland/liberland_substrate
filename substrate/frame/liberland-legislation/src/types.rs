use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use sp_core::ConstU32;

pub type LegislationSection = u32;
pub type LegislationContent = BoundedVec<u8, ConstU32<20480>>;

#[derive(
	Encode, MaxEncodedLen, Decode, TypeInfo, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug,
)]
pub enum LegislationTier {
	Constitution = 0,
	InternationalTreaty,
	Law,
	Tier3, // FIXME proper names
	Tier4,
	Tier5,
	Decision,
	// If adding anything, update From trait implementation
	InvalidTier, // keep this last
}

#[derive(Encode, MaxEncodedLen, Decode, TypeInfo, Clone, Copy, PartialEq, Eq, Debug)]
pub struct LegislationId {
	pub year: u32,
	pub index: u32,
}

impl From<(u32, u32)> for LegislationId {
	fn from(id: (u32, u32)) -> Self {
		Self { year: id.0, index: id.1 }
	}
}
