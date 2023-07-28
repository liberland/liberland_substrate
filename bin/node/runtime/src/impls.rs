// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Some configurable implementations as associated type for the substrate runtime.

use crate::{
	AccountId, Assets, Authorship, Balances, NegativeImbalance, Runtime, Balance, RuntimeCall,
	Democracy, RuntimeOrigin,
};
use codec::{Encode, Decode};
use frame_support::{
	BoundedVec,
	pallet_prelude::{ConstU32, PhantomData, Get, MaxEncodedLen},
	RuntimeDebug,
	traits::{
		fungibles::{Balanced, Credit},
		Currency, OnUnbalanced, InstanceFilter,
		Contains,
	},
};
use sp_runtime::{AccountId32, DispatchError, traits::{TrailingZeroInput, Morph}};
use pallet_asset_tx_payment::HandleCredit;
use sp_staking::{EraIndex, OnStakerSlash};
use sp_std::{vec, collections::btree_map::BTreeMap};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub struct Author;
impl OnUnbalanced<NegativeImbalance> for Author {
	fn on_nonzero_unbalanced(amount: NegativeImbalance) {
		if let Some(author) = Authorship::author() {
			Balances::resolve_creating(&author, amount);
		}
	}
}

/// A `HandleCredit` implementation that naively transfers the fees to the block author.
/// Will drop and burn the assets in case the transfer fails.
pub struct CreditToBlockAuthor;
impl HandleCredit<AccountId, Assets> for CreditToBlockAuthor {
	fn handle_credit(credit: Credit<AccountId, Assets>) {
		if let Some(author) = pallet_authorship::Pallet::<Runtime>::author() {
			// Drop the result which will trigger the `OnDrop` of the imbalance in case of error.
			let _ = Assets::resolve(&author, credit);
		}
	}
}

pub struct OnStakerSlashNoop;
impl OnStakerSlash<AccountId, Balance> for OnStakerSlashNoop {
	fn on_slash(_stash: &AccountId, _slashed_active: Balance, _slashed_ongoing: &BTreeMap<EraIndex, Balance>) {
		// do nothing
	}
}

pub struct ToAccountId<T, R> {
	_phantom: PhantomData<T>,
	_phantom2: PhantomData<R>,
}

impl<T, R> Morph<T> for ToAccountId<T, R>
where
	R: Get<AccountId>,
{
	type Outcome = AccountId;

	fn morph(_: T) -> Self::Outcome {
		R::get()
	}
}

#[derive(
	Clone,
	Eq,
	PartialEq,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum IdentityCallFilter {
	Manager, // set_fee, set_account_id, set_fields, provide_judgement
	Judgement, // provide_judgement
}

impl Default for IdentityCallFilter {
	fn default() -> Self {
		IdentityCallFilter::Judgement
	}
}

impl InstanceFilter<RuntimeCall> for IdentityCallFilter {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			IdentityCallFilter::Manager =>
				matches!(c,
					RuntimeCall::LLM(pallet_llm::Call::send_llm_to_politipool { .. }) |
					RuntimeCall::Balances(pallet_balances::Call::transfer { .. }) |
					RuntimeCall::Identity(pallet_identity::Call::set_fee { .. }) |
					RuntimeCall::Identity(pallet_identity::Call::set_fields { .. }) |
					RuntimeCall::Identity(pallet_identity::Call::set_account_id { .. }) |
					RuntimeCall::Identity(pallet_identity::Call::provide_judgement { .. })
				),
			IdentityCallFilter::Judgement =>
				matches!(c,
					RuntimeCall::LLM(pallet_llm::Call::send_llm_to_politipool { .. }) |
					RuntimeCall::Balances(pallet_balances::Call::transfer { .. }) |
					RuntimeCall::Identity(pallet_identity::Call::provide_judgement { .. }) |
					RuntimeCall::System(frame_system::Call::remark { .. }) // for benchmarking
				)
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(IdentityCallFilter::Manager, _) => true,
			(_, IdentityCallFilter::Manager) => false,
			_ => false,
		}
	}
}

#[derive(
	Clone,
	Eq,
	PartialEq,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum RegistryCallFilter {
	All, // registry_entity, set_registered_entity, unregister
	RegisterOnly, // register_entity
}

impl Default for RegistryCallFilter {
	fn default() -> Self {
		RegistryCallFilter::RegisterOnly
	}
}

impl InstanceFilter<RuntimeCall> for RegistryCallFilter {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			RegistryCallFilter::All =>
				matches!(c,
					RuntimeCall::CompanyRegistry(pallet_registry::Call::register_entity { .. }) |
					RuntimeCall::CompanyRegistry(pallet_registry::Call::set_registered_entity { .. }) |
					RuntimeCall::CompanyRegistry(pallet_registry::Call::unregister { .. })
				),
			RegistryCallFilter::RegisterOnly =>
				matches!(c, RuntimeCall::CompanyRegistry(pallet_registry::Call::register_entity { .. }))
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(RegistryCallFilter::All, _) => true,
			(_, RegistryCallFilter::All) => false,
			_ => false,
		}
	}
}

#[derive(
	Clone,
	Eq,
	PartialEq,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum NftsCallFilter {
	Manager,
	ManageItems,
}

impl Default for NftsCallFilter {
	fn default() -> Self {
		NftsCallFilter::ManageItems
	}
}

impl InstanceFilter<RuntimeCall> for NftsCallFilter {
	fn filter(&self, c: &RuntimeCall) -> bool {	
		let matches_manage_items = matches!(c, 
			RuntimeCall::Nfts(pallet_nfts::Call::mint { .. }) |
			RuntimeCall::Nfts(pallet_nfts::Call::force_mint { .. }) |
			RuntimeCall::Nfts(pallet_nfts::Call::burn { .. }) |
			RuntimeCall::Nfts(pallet_nfts::Call::redeposit { .. }) |
			RuntimeCall::Nfts(pallet_nfts::Call::approve_transfer { .. }) |
			RuntimeCall::Nfts(pallet_nfts::Call::cancel_approval { .. }) |
			RuntimeCall::Nfts(pallet_nfts::Call::clear_all_transfer_approvals { .. }) |
			RuntimeCall::Nfts(pallet_nfts::Call::set_attribute { .. }) |
			RuntimeCall::Nfts(pallet_nfts::Call::clear_attribute { .. }) |
			RuntimeCall::Nfts(pallet_nfts::Call::approve_item_attributes { .. }) |
			RuntimeCall::Nfts(pallet_nfts::Call::cancel_item_attributes_approval { .. }) |
			RuntimeCall::Nfts(pallet_nfts::Call::set_metadata { .. }) |
			RuntimeCall::Nfts(pallet_nfts::Call::clear_metadata { .. })
		);
		match self {
			NftsCallFilter::Manager => matches_manage_items || matches!(c,
					RuntimeCall::Nfts(pallet_nfts::Call::destroy { .. }) |
					RuntimeCall::Nfts(pallet_nfts::Call::lock_item_transfer { .. }) |
					RuntimeCall::Nfts(pallet_nfts::Call::unlock_item_transfer { .. }) |
					RuntimeCall::Nfts(pallet_nfts::Call::lock_collection { .. }) |
					RuntimeCall::Nfts(pallet_nfts::Call::transfer_ownership { .. }) |
					RuntimeCall::Nfts(pallet_nfts::Call::set_team { .. }) |
					RuntimeCall::Nfts(pallet_nfts::Call::lock_item_properties { .. }) |
					RuntimeCall::Nfts(pallet_nfts::Call::set_collection_metadata { .. }) |
					RuntimeCall::Nfts(pallet_nfts::Call::clear_collection_metadata { .. }) |
					RuntimeCall::Nfts(pallet_nfts::Call::set_accept_ownership { .. }) |
					RuntimeCall::Nfts(pallet_nfts::Call::set_collection_max_supply { .. }) |
					RuntimeCall::Nfts(pallet_nfts::Call::update_mint_settings { .. }) |
					RuntimeCall::Nfts(pallet_nfts::Call::set_citizenship_required { .. })
				),
			NftsCallFilter::ManageItems => matches_manage_items,
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(NftsCallFilter::Manager, _) => true,
			(_, NftsCallFilter::Manager) => false,
			_ => false,
		}
	}
}

pub struct ContainsMember<T, I>(
    PhantomData<(T, I)>,
);

impl<T, I> Contains<T::AccountId> for ContainsMember<T, I>
where
	T: frame_system::Config + pallet_collective::Config<I>,
	I: 'static
{
	fn contains(a: &T::AccountId) -> bool {
		pallet_collective::Pallet::<T, I>::members().contains(a)
	}
}

use pallet_democracy::Voting;
pub struct OnLLMPoliticsUnlock;
impl liberland_traits::OnLLMPoliticsUnlock<AccountId32> for OnLLMPoliticsUnlock
{
	fn on_llm_politics_unlock(account_id: &AccountId32) -> Result<(), DispatchError> {
		let origin = RuntimeOrigin::signed(account_id.clone());

		match Democracy::voting_of(account_id.clone()) {
			Voting::Direct { votes, .. } => {
				for (index, _) in votes {
					Democracy::remove_vote(origin.clone(), index)?;
				}
			},
			Voting::Delegating { .. } => {
				Democracy::undelegate(origin.clone()).map_err(|e| e.error)?;
			}
		};

		Ok(())
	}
}

#[derive(
	Clone,
	Copy,
	Eq,
	PartialEq,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Coords {
	pub lat: u64,
	pub long: u64,
}

#[derive(
	Clone,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Metadata<MaxCoords: Get<u32>, MaxString: Get<u32>> {
	demarcation: BoundedVec<Coords, MaxCoords>,
	r#type: BoundedVec<u8, MaxString>,
	status: BoundedVec<u8, MaxString>,
}

pub struct LandMetadataValidator<CoordsBounds: Get<(Coords, Coords)>>(PhantomData<CoordsBounds>);

impl<CoordsBounds: Get<(Coords, Coords)>, StringLimit>
	pallet_nfts::traits::MetadataValidator<u32, u32, StringLimit>
	for LandMetadataValidator<CoordsBounds>
{
	fn validate_metadata(collection: u32, _: u32, data: &BoundedVec<u8, StringLimit>) -> bool {
		// https://www.geeksforgeeks.org/check-if-two-given-line-segments-intersect/
		fn intersection(a: Coords, b: Coords, c: Coords, d: Coords) -> bool {
			fn ccw(a: Coords, b: Coords, c: Coords) -> bool {
				(b.long - a.long) * (c.lat - b.lat) > (b.lat - a.lat) * (c.long - b.long)
			}
			ccw(a, c, d) != ccw(b, c, d) && ccw(a, b, c) != ccw(a, b, d)
		}

		if collection != 0 && collection != 1 {
			return true
		}

		let data = data.clone();
		let data = Metadata::<ConstU32<100>, ConstU32<100>>::decode(&mut TrailingZeroInput::new(
			&data[..],
		));
		if data.is_err() {
			return false
		}

		let data = data.unwrap();

		// does it have at least 3 points
		if data.demarcation.len() < 3 {
			return false
		}

		// is it roughly in the good place
		let (a, b) = CoordsBounds::get();
		for c in data.demarcation.iter() {
			if c.lat < a.lat || c.lat > b.lat || c.long < a.long || c.long > b.long {
				return false
			}
		}

		// check self intersection. we do it by checking if any of line segments
		// intersect with any other non-neighboring segment in the plot
		let mut lines = vec![];
		for i in 1..data.demarcation.len() {
			lines.push((data.demarcation[i - 1].clone(), data.demarcation[i].clone()))
		}
		lines.push((data.demarcation[data.demarcation.len() - 1], data.demarcation[0]));

		for i in 0..lines.len() {
			let a = lines[i];
			for j in i + 2..lines.len() + i - 1 {
				let j = j % lines.len();
				let b = lines[j];
				if intersection(a.0, a.1, b.0, b.1) {
					return false
				}
			}
		}

		true
	}
}

#[cfg(test)]
mod multiplier_tests {
	use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
	use sp_runtime::{
		assert_eq_error_rate,
		traits::{Convert, One, Zero},
		FixedPointNumber,
	};

	use crate::{
		constants::{currency::*, time::*},
		AdjustmentVariable, MaximumMultiplier, MinimumMultiplier, Runtime,
		RuntimeBlockWeights as BlockWeights, System, TargetBlockFullness, TransactionPayment,
	};
	use frame_support::{
		dispatch::DispatchClass,
		weights::{Weight, WeightToFee},
	};

	fn max_normal() -> Weight {
		BlockWeights::get()
			.get(DispatchClass::Normal)
			.max_total
			.unwrap_or_else(|| BlockWeights::get().max_block)
	}

	fn min_multiplier() -> Multiplier {
		MinimumMultiplier::get()
	}

	fn target() -> Weight {
		TargetBlockFullness::get() * max_normal()
	}

	// update based on runtime impl.
	fn runtime_multiplier_update(fm: Multiplier) -> Multiplier {
		TargetedFeeAdjustment::<
			Runtime,
			TargetBlockFullness,
			AdjustmentVariable,
			MinimumMultiplier,
			MaximumMultiplier,
		>::convert(fm)
	}

	// update based on reference impl.
	fn truth_value_update(block_weight: Weight, previous: Multiplier) -> Multiplier {
		let accuracy = Multiplier::accuracy() as f64;
		let previous_float = previous.into_inner() as f64 / accuracy;
		// bump if it is zero.
		let previous_float = previous_float.max(min_multiplier().into_inner() as f64 / accuracy);

		// maximum tx weight
		let m = max_normal().ref_time() as f64;
		// block weight always truncated to max weight
		let block_weight = (block_weight.ref_time() as f64).min(m);
		let v: f64 = AdjustmentVariable::get().to_float();

		// Ideal saturation in terms of weight
		let ss = target().ref_time() as f64;
		// Current saturation in terms of weight
		let s = block_weight;

		let t1 = v * (s / m - ss / m);
		let t2 = v.powi(2) * (s / m - ss / m).powi(2) / 2.0;
		let next_float = previous_float * (1.0 + t1 + t2);
		Multiplier::from_float(next_float)
	}

	fn run_with_system_weight<F>(w: Weight, assertions: F)
	where
		F: Fn() -> (),
	{
		let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap()
			.into();
		t.execute_with(|| {
			System::set_block_consumed_resources(w, 0);
			assertions()
		});
	}

	#[test]
	fn truth_value_update_poc_works() {
		let fm = Multiplier::saturating_from_rational(1, 2);
		let test_set = vec![
			(Weight::zero(), fm),
			(Weight::from_parts(100, 0), fm),
			(Weight::from_parts(1000, 0), fm),
			(target(), fm),
			(max_normal() / 2, fm),
			(max_normal(), fm),
		];
		test_set.into_iter().for_each(|(w, fm)| {
			run_with_system_weight(w, || {
				assert_eq_error_rate!(
					truth_value_update(w, fm),
					runtime_multiplier_update(fm),
					// Error is only 1 in 100^18
					Multiplier::from_inner(100),
				);
			})
		})
	}

	#[test]
	fn multiplier_can_grow_from_zero() {
		// if the min is too small, then this will not change, and we are doomed forever.
		// the weight is 1/100th bigger than target.
		run_with_system_weight(target().set_ref_time(target().ref_time() * 101 / 100), || {
			let next = runtime_multiplier_update(min_multiplier());
			assert!(next > min_multiplier(), "{:?} !>= {:?}", next, min_multiplier());
		})
	}

	#[test]
	fn multiplier_cannot_go_below_limit() {
		// will not go any further below even if block is empty.
		run_with_system_weight(Weight::zero(), || {
			let next = runtime_multiplier_update(min_multiplier());
			assert_eq!(next, min_multiplier());
		})
	}

	#[test]
	fn time_to_reach_zero() {
		// blocks per 24h in substrate-node: 28,800 (k)
		// s* = 0.1875
		// The bound from the research in an empty chain is:
		// v <~ (p / k(0 - s*))
		// p > v * k * -0.1875
		// to get p == -1 we'd need
		// -1 > 0.00001 * k * -0.1875
		// 1 < 0.00001 * k * 0.1875
		// 10^9 / 1875 < k
		// k > 533_333 ~ 18,5 days.
		run_with_system_weight(Weight::zero(), || {
			// start from 1, the default.
			let mut fm = Multiplier::one();
			let mut iterations: u64 = 0;
			loop {
				let next = runtime_multiplier_update(fm);
				fm = next;
				if fm == min_multiplier() {
					break
				}
				iterations += 1;
			}
			assert!(iterations > 533_333);
		})
	}

	#[test]
	fn min_change_per_day() {
		run_with_system_weight(max_normal(), || {
			let mut fm = Multiplier::one();
			// See the example in the doc of `TargetedFeeAdjustment`. are at least 0.234, hence
			// `fm > 1.234`.
			// we use 2 days, as our avg block time is 6sec instead of default 3 sec
			for _ in 0..(2*DAYS) {
				let next = runtime_multiplier_update(fm);
				fm = next;
			}
			assert!(fm > Multiplier::saturating_from_rational(1234, 1000));
		})
	}

	#[test]
	#[ignore]
	fn congested_chain_simulation() {
		// `cargo test congested_chain_simulation -- --nocapture` to get some insight.

		// almost full. The entire quota of normal transactions is taken.
		let block_weight = BlockWeights::get().get(DispatchClass::Normal).max_total.unwrap() -
			Weight::from_parts(100, 0);

		// Default substrate weight.
		let tx_weight = frame_support::weights::constants::ExtrinsicBaseWeight::get();

		run_with_system_weight(block_weight, || {
			// initial value configured on module
			let mut fm = Multiplier::one();
			assert_eq!(fm, TransactionPayment::next_fee_multiplier());

			let mut iterations: u64 = 0;
			loop {
				let next = runtime_multiplier_update(fm);
				// if no change, panic. This should never happen in this case.
				if fm == next {
					panic!("The fee should ever increase");
				}
				fm = next;
				iterations += 1;
				let fee =
					<Runtime as pallet_transaction_payment::Config>::WeightToFee::weight_to_fee(
						&tx_weight,
					);
				let adjusted_fee = fm.saturating_mul_acc_int(fee);
				println!(
					"iteration {}, new fm = {:?}. Fee at this point is: {} units / {} millicents, \
					{} cents, {} dollars",
					iterations,
					fm,
					adjusted_fee,
					adjusted_fee / MILLICENTS,
					adjusted_fee / CENTS,
					adjusted_fee / DOLLARS,
				);
			}
		});
	}

	#[test]
	fn stateless_weight_mul() {
		let fm = Multiplier::saturating_from_rational(1, 2);
		run_with_system_weight(target() / 4, || {
			let next = runtime_multiplier_update(fm);
			assert_eq_error_rate!(
				next,
				truth_value_update(target() / 4, fm),
				Multiplier::from_inner(100),
			);

			// Light block. Multiplier is reduced a little.
			assert!(next < fm);
		});

		run_with_system_weight(target() / 2, || {
			let next = runtime_multiplier_update(fm);
			assert_eq_error_rate!(
				next,
				truth_value_update(target() / 2, fm),
				Multiplier::from_inner(100),
			);
			// Light block. Multiplier is reduced a little.
			assert!(next < fm);
		});
		run_with_system_weight(target(), || {
			let next = runtime_multiplier_update(fm);
			assert_eq_error_rate!(
				next,
				truth_value_update(target(), fm),
				Multiplier::from_inner(100),
			);
			// ideal. No changes.
			assert_eq!(next, fm)
		});
		run_with_system_weight(target() * 2, || {
			// More than ideal. Fee is increased.
			let next = runtime_multiplier_update(fm);
			assert_eq_error_rate!(
				next,
				truth_value_update(target() * 2, fm),
				Multiplier::from_inner(100),
			);

			// Heavy block. Fee is increased a little.
			assert!(next > fm);
		});
	}

	#[test]
	fn weight_mul_grow_on_big_block() {
		run_with_system_weight(target() * 2, || {
			let mut original = Multiplier::zero();
			let mut next = Multiplier::default();

			(0..1_000).for_each(|_| {
				next = runtime_multiplier_update(original);
				assert_eq_error_rate!(
					next,
					truth_value_update(target() * 2, original),
					Multiplier::from_inner(100),
				);
				// must always increase
				assert!(next > original, "{:?} !>= {:?}", next, original);
				original = next;
			});
		});
	}

	#[test]
	fn weight_mul_decrease_on_small_block() {
		run_with_system_weight(target() / 2, || {
			let mut original = Multiplier::saturating_from_rational(1, 2);
			let mut next;

			for _ in 0..100 {
				// decreases
				next = runtime_multiplier_update(original);
				assert!(next < original, "{:?} !<= {:?}", next, original);
				original = next;
			}
		})
	}

	#[test]
	fn weight_to_fee_should_not_overflow_on_large_weights() {
		let kb = Weight::from_parts(1024, 0);
		let mb = 1024u64 * kb;
		let max_fm = Multiplier::saturating_from_integer(i128::MAX);

		// check that for all values it can compute, correctly.
		vec![
			Weight::zero(),
			Weight::from_parts(1, 0),
			Weight::from_parts(10, 0),
			Weight::from_parts(1000, 0),
			kb,
			10u64 * kb,
			100u64 * kb,
			mb,
			10u64 * mb,
			Weight::from_parts(2147483647, 0),
			Weight::from_parts(4294967295, 0),
			BlockWeights::get().max_block / 2,
			BlockWeights::get().max_block,
			Weight::MAX / 2,
			Weight::MAX,
		]
		.into_iter()
		.for_each(|i| {
			run_with_system_weight(i, || {
				let next = runtime_multiplier_update(Multiplier::one());
				let truth = truth_value_update(i, Multiplier::one());
				assert_eq_error_rate!(truth, next, Multiplier::from_inner(50_000_000));
			});
		});

		// Some values that are all above the target and will cause an increase.
		let t = target();
		vec![t + Weight::from_parts(100, 0), t * 2, t * 4].into_iter().for_each(|i| {
			run_with_system_weight(i, || {
				let fm = runtime_multiplier_update(max_fm);
				// won't grow. The convert saturates everything.
				assert_eq!(fm, max_fm);
			})
		});
	}

	use super::{Coords, LandMetadataValidator, Metadata};
	use codec::Encode;
	use frame_support::{parameter_types, BoundedVec};
	use pallet_nfts::traits::MetadataValidator;
	use sp_core::ConstU32;

	parameter_types! {
		pub const TestCoords: (Coords, Coords) = (
			Coords {
				lat: 45_7686480,
				long: 18_8821180,
			},
			Coords {
				lat: 45_7785989,
				long: 18_8892809,
			},
		);
	}
	#[test]
	fn land_metadata_validator_is_sane() {
		let good: BoundedVec<u8, ConstU32<1000>> = Metadata::<ConstU32<10>, ConstU32<10>> {
			r#type: Default::default(),
			status: Default::default(),
			demarcation: vec![
				Coords { lat: 45_7723532, long: 18_8870918 },
				Coords { lat: 45_7721717, long: 18_8871917 },
				Coords { lat: 45_7723330, long: 18_8877504 },
				Coords { lat: 45_7724976, long: 18_8876859 },
				Coords { lat: 45_7725234, long: 18_8876738 },
				Coords { lat: 45_7723532, long: 18_8870918 },
			]
			.try_into()
			.unwrap(),
		}
		.encode()
		.try_into()
		.unwrap();
		let not_enough_coords: BoundedVec<u8, ConstU32<1000>> =
			Metadata::<ConstU32<10>, ConstU32<10>> {
				r#type: Default::default(),
				status: Default::default(),
				demarcation: vec![
					Coords { lat: 45_7723532, long: 18_8870918 },
					Coords { lat: 45_7721717, long: 18_8871917 },
				]
				.try_into()
				.unwrap(),
			}
			.encode()
			.try_into()
			.unwrap();
		let invalid_coord: BoundedVec<u8, ConstU32<1000>> =
			Metadata::<ConstU32<10>, ConstU32<10>> {
				r#type: Default::default(),
				status: Default::default(),
				demarcation: vec![
					Coords { lat: 45_7723532, long: 18_8870918 },
					Coords { lat: 45_7721717, long: 18_8871917 },
					Coords { lat: 5_7723330, long: 18_8877504 },
				]
				.try_into()
				.unwrap(),
			}
			.encode()
			.try_into()
			.unwrap();
		let self_intersecting: BoundedVec<u8, ConstU32<1000>> =
			Metadata::<ConstU32<10>, ConstU32<10>> {
				r#type: Default::default(),
				status: Default::default(),
				demarcation: vec![
					Coords { lat: 45_7723532, long: 18_8870918 },
					Coords { lat: 45_7723330, long: 18_8877504 },
					Coords { lat: 45_7721717, long: 18_8871917 },
					Coords { lat: 45_7724976, long: 18_8876859 },
				]
				.try_into()
				.unwrap(),
			}
			.encode()
			.try_into()
			.unwrap();

		assert!(LandMetadataValidator::<TestCoords>::validate_metadata(1, 1, &good));
		assert!(LandMetadataValidator::<TestCoords>::validate_metadata(99, 0, &not_enough_coords));
		assert!(!LandMetadataValidator::<TestCoords>::validate_metadata(0, 1, &not_enough_coords));
		assert!(!LandMetadataValidator::<TestCoords>::validate_metadata(1, 1, &not_enough_coords));
		assert!(!LandMetadataValidator::<TestCoords>::validate_metadata(1, 1, &invalid_coord));
		assert!(!LandMetadataValidator::<TestCoords>::validate_metadata(1, 1, &self_intersecting));
	}
}
