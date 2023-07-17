// FIXME this is mostly copied from src/types.rs, what license/copyright notices should we put?

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	pallet_prelude::*,
	traits::{ConstU32, Get},
	storage_alias,
	BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
use scale_info::{
	build::{Fields, Variants},
	Path, Type, TypeInfo,
};
use sp_runtime::{traits::AppendZerosInput, RuntimeDebug};
use sp_std::{fmt::Debug, iter::once, prelude::*};
use crate::{BalanceOf, Config, Pallet, Judgement, RegistrarIndex};

#[storage_alias]
pub type IdentityOf<T: Config> = StorageMap<
	Pallet<T>,
	Twox64Concat,
	<T as frame_system::Config>::AccountId,
	Registration<BalanceOf<T>, <T as Config>::MaxRegistrars, <T as Config>::MaxAdditionalFields>,
	OptionQuery,
>;

#[derive(Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen)]
pub enum Data {
	None,
	Raw(BoundedVec<u8, ConstU32<32>>),
	BlakeTwo256([u8; 32]),
	Sha256([u8; 32]),
	Keccak256([u8; 32]),
	ShaThree256([u8; 32]),
}

impl Data {
	pub fn is_none(&self) -> bool {
		self == &Data::None
	}
}

impl Decode for Data {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let b = input.read_byte()?;
		Ok(match b {
			0 => Data::None,
			n @ 1..=33 => {
				let mut r: BoundedVec<_, _> = vec![0u8; n as usize - 1]
					.try_into()
					.expect("bound checked in match arm condition; qed");
				input.read(&mut r[..])?;
				Data::Raw(r)
			},
			34 => Data::BlakeTwo256(<[u8; 32]>::decode(input)?),
			35 => Data::Sha256(<[u8; 32]>::decode(input)?),
			36 => Data::Keccak256(<[u8; 32]>::decode(input)?),
			37 => Data::ShaThree256(<[u8; 32]>::decode(input)?),
			_ => return Err(codec::Error::from("invalid leading byte")),
		})
	}
}

impl Encode for Data {
	fn encode(&self) -> Vec<u8> {
		match self {
			Data::None => vec![0u8; 1],
			Data::Raw(ref x) => {
				let l = x.len().min(32);
				let mut r = vec![l as u8 + 1; l + 1];
				r[1..].copy_from_slice(&x[..l as usize]);
				r
			},
			Data::BlakeTwo256(ref h) => once(34u8).chain(h.iter().cloned()).collect(),
			Data::Sha256(ref h) => once(35u8).chain(h.iter().cloned()).collect(),
			Data::Keccak256(ref h) => once(36u8).chain(h.iter().cloned()).collect(),
			Data::ShaThree256(ref h) => once(37u8).chain(h.iter().cloned()).collect(),
		}
	}
}
impl codec::EncodeLike for Data {}

macro_rules! data_raw_variants {
    ($variants:ident, $(($index:literal, $size:literal)),* ) => {
		$variants
		$(
			.variant(concat!("Raw", stringify!($size)), |v| v
				.index($index)
				.fields(Fields::unnamed().field(|f| f.ty::<[u8; $size]>()))
			)
		)*
    }
}

impl TypeInfo for Data {
	type Identity = Self;

	fn type_info() -> Type {
		let variants = Variants::new().variant("None", |v| v.index(0));

		// create a variant for all sizes of Raw data from 0-32
		let variants = data_raw_variants!(
			variants,
			(1, 0),
			(2, 1),
			(3, 2),
			(4, 3),
			(5, 4),
			(6, 5),
			(7, 6),
			(8, 7),
			(9, 8),
			(10, 9),
			(11, 10),
			(12, 11),
			(13, 12),
			(14, 13),
			(15, 14),
			(16, 15),
			(17, 16),
			(18, 17),
			(19, 18),
			(20, 19),
			(21, 20),
			(22, 21),
			(23, 22),
			(24, 23),
			(25, 24),
			(26, 25),
			(27, 26),
			(28, 27),
			(29, 28),
			(30, 29),
			(31, 30),
			(32, 31),
			(33, 32)
		);

		let variants = variants
			.variant("BlakeTwo256", |v| {
				v.index(34).fields(Fields::unnamed().field(|f| f.ty::<[u8; 32]>()))
			})
			.variant("Sha256", |v| {
				v.index(35).fields(Fields::unnamed().field(|f| f.ty::<[u8; 32]>()))
			})
			.variant("Keccak256", |v| {
				v.index(36).fields(Fields::unnamed().field(|f| f.ty::<[u8; 32]>()))
			})
			.variant("ShaThree256", |v| {
				v.index(37).fields(Fields::unnamed().field(|f| f.ty::<[u8; 32]>()))
			});

		Type::builder().path(Path::new("Data", module_path!())).variant(variants)
	}
}

impl Default for Data {
	fn default() -> Self {
		Self::None
	}
}

#[derive(
	CloneNoBound, Encode, Decode, Eq, MaxEncodedLen, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[codec(mel_bound())]
#[cfg_attr(test, derive(frame_support::DefaultNoBound))]
#[scale_info(skip_type_params(FieldLimit))]
pub struct IdentityInfo<FieldLimit: Get<u32>> {
	pub additional: BoundedVec<(Data, Data), FieldLimit>,
	pub display: Data,
	pub legal: Data,
	pub web: Data,
	pub riot: Data,
	pub email: Data,
	pub pgp_fingerprint: Option<[u8; 20]>,
	pub image: Data,
	pub twitter: Data,
}

#[derive(
	CloneNoBound, Encode, Eq, MaxEncodedLen, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[codec(mel_bound())]
#[scale_info(skip_type_params(MaxJudgements, MaxAdditionalFields))]
pub struct Registration<
	Balance: Encode + Decode + MaxEncodedLen + Copy + Clone + Debug + Eq + PartialEq,
	MaxJudgements: Get<u32>,
	MaxAdditionalFields: Get<u32>,
> {
	pub judgements: BoundedVec<(RegistrarIndex, Judgement<Balance>), MaxJudgements>,
	pub deposit: Balance,
	pub info: IdentityInfo<MaxAdditionalFields>,
}

impl<
		Balance: Encode + Decode + MaxEncodedLen + Copy + Clone + Debug + Eq + PartialEq,
		MaxJudgements: Get<u32>,
		MaxAdditionalFields: Get<u32>,
	> Decode for Registration<Balance, MaxJudgements, MaxAdditionalFields>
{
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let (judgements, deposit, info) = Decode::decode(&mut AppendZerosInput::new(input))?;
		Ok(Self { judgements, deposit, info })
	}
}