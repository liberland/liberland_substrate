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

use super::*;
use codec::{Encode, Decode};
use frame_support::{
	BoundedVec,
	pallet_prelude::{ConstU32, PhantomData, Get, MaxEncodedLen},
	traits::{
		Currency, OnUnbalanced, InstanceFilter,
		Contains, PrivilegeCmp, EnsureOrigin,
	},
};
use sp_runtime::{RuntimeDebug, AccountId32, DispatchError, traits::{TrailingZeroInput, Morph}};
use sp_std::{vec, cmp::{max, min, Ordering}};
use scale_info::TypeInfo;
use sp_runtime::traits::Dispatchable;
use frame_support::dispatch::{DispatchInfo, PostDispatchInfo, DispatchErrorWithPostInfo, GetDispatchInfo};
use bridge_types::H256;
use sp_runtime::traits::Convert;
use bridge_types::LiberlandAssetId;
use serde::{Deserialize, Serialize};

use crate::{
	AccountId, Authorship, Balances, NegativeImbalance, RuntimeCall,
	Democracy, RuntimeOrigin,
};

pub struct Author;
impl OnUnbalanced<NegativeImbalance> for Author {
	fn on_nonzero_unbalanced(amount: NegativeImbalance) {
		if let Some(author) = Authorship::author() {
			Balances::resolve_creating(&author, amount);
		}
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
	Serialize,
	Deserialize,
)]
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
	Serialize,
	Deserialize,
)]
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
	Serialize,
	Deserialize,
)]
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

#[derive(
	Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, MaxEncodedLen, scale_info::TypeInfo, Serialize, Deserialize,
)]
pub struct SenateAccountCallFilter;

impl Contains<RuntimeCall> for SenateAccountCallFilter {
	fn contains(c: &RuntimeCall) -> bool {
		match c {
			RuntimeCall::Utility(pallet_utility::Call::batch { calls }) => calls.iter().all(|inner_call| Self::contains(inner_call)),
			RuntimeCall::Utility(pallet_utility::Call::batch_all { calls }) => calls.iter().all(|inner_call| Self::contains(inner_call)),
			_ => matches!(c,
				RuntimeCall::LLM(pallet_llm::Call::remark { .. }) |
				RuntimeCall::LLM(pallet_llm::Call::send_llm { .. }) |
				RuntimeCall::LLM(pallet_llm::Call::send_llm_to_politipool { .. }) |
				RuntimeCall::Assets(pallet_assets::Call::transfer { .. }) |
				RuntimeCall::Assets(pallet_assets::Call::transfer_keep_alive { .. }) |
				RuntimeCall::Balances(pallet_balances::Call::transfer { .. }) |
				RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive { .. })
			)
		}
	}
}

#[derive(
	Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, MaxEncodedLen, scale_info::TypeInfo, Serialize, Deserialize,
)]
pub struct CouncilAccountCallFilter;

impl CouncilAccountCallFilter {
	fn ensure_schedule(c: &RuntimeCall) -> Result<(&RuntimeCall, &BlockNumber), ()> {
		if let RuntimeCall::Scheduler(pallet_scheduler::Call::schedule { when, call, ..}) = c {
			Ok((call, when))
		} else if let RuntimeCall::Scheduler(pallet_scheduler::Call::schedule_named { when, call, ..}) = c {
			Ok((call, when))
		} else {
			Err(())
		}
	}

	fn ensure_batched(c: &RuntimeCall) -> Result<&Vec<RuntimeCall>, ()> {
		if let RuntimeCall::Utility(pallet_utility::Call::batch { calls }) = c {
			Ok(calls)
		} else if let RuntimeCall::Utility(pallet_utility::Call::batch_all { calls }) = c {
			Ok(calls)
		} else {
			Err(())
		}
	}

	fn ensure_transfer_or_remark(c: &RuntimeCall) -> Result<(), ()> {
		match c {
			RuntimeCall::LLM(pallet_llm::Call::remark { .. }) => Ok(()),
			RuntimeCall::LLM(pallet_llm::Call::send_llm { .. }) => Ok(()),
			RuntimeCall::LLM(pallet_llm::Call::send_llm_to_politipool { .. }) => Ok(()),
			RuntimeCall::Assets(pallet_assets::Call::transfer { .. }) => Ok(()),
			RuntimeCall::Assets(pallet_assets::Call::transfer_keep_alive { .. }) => Ok(()),
			RuntimeCall::Balances(pallet_balances::Call::transfer { .. }) => Ok(()),
			RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive { .. }) => Ok(()),
			_ => Err(())
		}
	}

	fn ensure_valid_council_call(c: &RuntimeCall) -> Result<(), ()> {
		let (scheduled_call, when) = Self::ensure_schedule(c)?;
		if when - System::block_number() <= 4 * DAYS {
			return Err(());
		}

		let batched_calls = Self::ensure_batched(scheduled_call)?;

		batched_calls.iter()
			.map(Self::ensure_transfer_or_remark)
			.all(|v| v.is_ok())
			.then_some(())
			.ok_or(())
	}
}

impl Contains<RuntimeCall> for CouncilAccountCallFilter {
	fn contains(c: &RuntimeCall) -> bool {
		Self::ensure_valid_council_call(c).is_ok()
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

pub struct EnsureCmp<L>(sp_std::marker::PhantomData<L>);
impl<L: EnsureOrigin<RuntimeOrigin>> PrivilegeCmp<OriginCaller> for EnsureCmp<L> {
  fn cmp_privilege(left: &OriginCaller, _: &OriginCaller) -> Option<Ordering> {
		if L::try_origin(
			<OriginCaller as Into<RuntimeOrigin>>::into(left.clone())
		).is_ok() {
			Some(Ordering::Equal)
		} else {
			None
		}
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
	Serialize,
	Deserialize,
)]
pub struct Coords {
	pub lat: i64,
	pub long: i64,
}

#[derive(
	Clone,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
	Serialize,
	Deserialize,
)]
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
		#[derive(PartialEq, Eq)]
		enum PointsOrientation {
			Clockwise,
			Counterclockwise,
			Collinear,
		}
		fn intersection(p1: Coords, q1: Coords, p2: Coords, q2: Coords) -> bool {
			fn on_segment(p: Coords, q: Coords, r: Coords) -> bool {
				q.long <= max(p.long, r.long) &&
				   q.long >= min(p.long, r.long) &&
				   q.lat <= max(p.lat, r.lat) &&
				   q.lat >= min(p.lat, r.lat)
			}
			fn orientation(p: Coords, q: Coords, r: Coords) -> PointsOrientation {
				let val = ((q.lat - p.lat) * (r.long - q.long)) - ((q.long - p.long) * (r.lat - q.lat));
				match val {
					val if val > 0 => {
						PointsOrientation::Clockwise
					}
					val if val < 0 => {
						PointsOrientation::Counterclockwise
					}
					_ => {
						PointsOrientation::Collinear
					}
				}
			}

			let o1 = orientation(p1, q1, p2);
			let o2 = orientation(p1, q1, q2);
			let o3 = orientation(p2, q2, p1);
			let o4 = orientation(p2, q2, q1);
			if (o1 != o2 && o3 != o4) ||
			   (o1 == PointsOrientation::Collinear && on_segment(p1, p2, q1)) ||
			   (o2 == PointsOrientation::Collinear && on_segment(p1, q2, q1)) ||
			   (o3 == PointsOrientation::Collinear && on_segment(p2, p1, q2)) ||
			   (o4 == PointsOrientation::Collinear && on_segment(p2, q1, q2))
			{
				return true
			}

			false
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

		let mut data = data.unwrap();

		// pop last point if it's the same as first
		if data.demarcation.len() > 1 &&
		   data.demarcation.first() == data.demarcation.last()
		{
			data.demarcation.pop();
		}

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
			for j in (i + 2)..=(lines.len() + i - 2) {
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

#[derive(
	Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, MaxEncodedLen, scale_info::TypeInfo, Serialize, Deserialize,
)]
pub struct ContractsCallFilter;

impl Contains<RuntimeCall> for ContractsCallFilter {
	fn contains(c: &RuntimeCall) -> bool {
		matches!(c,
			RuntimeCall::LLM(pallet_llm::Call::force_transfer { .. }) |
			RuntimeCall::Assets(pallet_assets::Call::approve_transfer { .. }) |
			RuntimeCall::Assets(pallet_assets::Call::cancel_approval { .. }) |
			RuntimeCall::Assets(pallet_assets::Call::transfer { .. }) |
			RuntimeCall::Assets(pallet_assets::Call::transfer_approved { .. }) |
			RuntimeCall::Assets(pallet_assets::Call::transfer_keep_alive { .. })
		)
	}
}

// Sora Bridge
pub struct GenericTimepointProvider;

impl bridge_types::traits::TimepointProvider for GenericTimepointProvider {
	fn get_timepoint() -> bridge_types::GenericTimepoint {
		bridge_types::GenericTimepoint::Sora(crate::System::block_number())
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct DispatchableSubstrateBridgeCall(bridge_types::substrate::BridgeCall);

impl Dispatchable for DispatchableSubstrateBridgeCall {
	type RuntimeOrigin = crate::RuntimeOrigin;
	type Config = crate::Runtime;
	type Info = DispatchInfo;
	type PostInfo = PostDispatchInfo;

	fn dispatch(
		self,
		origin: Self::RuntimeOrigin,
	) -> sp_runtime::DispatchResultWithInfo<Self::PostInfo> {
		match self.0 {
			bridge_types::substrate::BridgeCall::ParachainApp(_) => {
				Err(DispatchErrorWithPostInfo {
					post_info: Default::default(),
					error: DispatchError::Other("Unavailable"),
				})
			},
			bridge_types::substrate::BridgeCall::XCMApp(_) => Err(DispatchErrorWithPostInfo {
				post_info: Default::default(),
				error: DispatchError::Other("Unavailable"),
			}),
			bridge_types::substrate::BridgeCall::DataSigner(msg) => {
				let call: bridge_data_signer::Call<crate::Runtime> = msg.into();
				let call: crate::RuntimeCall = call.into();
				call.dispatch(origin)
			},
			bridge_types::substrate::BridgeCall::MultisigVerifier(msg) => {
				let call: multisig_verifier::Call<crate::Runtime> = msg.into();
				let call: crate::RuntimeCall = call.into();
				call.dispatch(origin)
			},
			bridge_types::substrate::BridgeCall::SubstrateApp(msg) => {
				let call: substrate_bridge_app::Call<crate::Runtime> = msg.try_into()?;
				let call: crate::RuntimeCall = call.into();
				call.dispatch(origin)
			},
		}
	}
}

impl GetDispatchInfo for DispatchableSubstrateBridgeCall {
	fn get_dispatch_info(&self) -> DispatchInfo {
		match &self.0 {
			bridge_types::substrate::BridgeCall::ParachainApp(_) => Default::default(),
			bridge_types::substrate::BridgeCall::XCMApp(_) => Default::default(),
			bridge_types::substrate::BridgeCall::DataSigner(msg) => {
				let call: bridge_data_signer::Call<crate::Runtime> = msg.clone().into();
				call.get_dispatch_info()
			},
			bridge_types::substrate::BridgeCall::MultisigVerifier(msg) => {
				let call: multisig_verifier::Call<crate::Runtime> = msg.clone().into();
				call.get_dispatch_info()
			},
			bridge_types::substrate::BridgeCall::SubstrateApp(msg) => {
				let call: substrate_bridge_app::Call<crate::Runtime> =
					match substrate_bridge_app::Call::try_from(msg.clone()) {
						Ok(c) => c,
						Err(_) => return Default::default(),
					};
				call.get_dispatch_info()
			},
		}
	}
}

pub struct SoraBridgeCallFilter;

impl Contains<DispatchableSubstrateBridgeCall> for SoraBridgeCallFilter {
	fn contains(call: &DispatchableSubstrateBridgeCall) -> bool {
		match &call.0 {
			bridge_types::substrate::BridgeCall::ParachainApp(_) => false,
			bridge_types::substrate::BridgeCall::XCMApp(_) => false,
			bridge_types::substrate::BridgeCall::DataSigner(_) => true,
			bridge_types::substrate::BridgeCall::MultisigVerifier(_) => true,
			bridge_types::substrate::BridgeCall::SubstrateApp(_) => true,
		}
	}
}

pub struct MultiVerifier;

#[derive(Clone, Debug, PartialEq, codec::Encode, codec::Decode, scale_info::TypeInfo)]
pub enum MultiProof {
	#[codec(index = 0)]
	Multisig(<crate::MultisigVerifier as bridge_types::traits::Verifier>::Proof),
	/// This proof is only used for benchmarking purposes
	#[cfg(feature = "runtime-benchmarks")]
	#[codec(skip)]
	Empty,
}

impl bridge_types::traits::Verifier for MultiVerifier {
	type Proof = MultiProof;

	fn verify(
		network_id: bridge_types::GenericNetworkId,
		message: H256,
		proof: &Self::Proof,
	) -> frame_support::pallet_prelude::DispatchResult {
		match proof {
			MultiProof::Multisig(proof) => {
				crate::MultisigVerifier::verify(network_id, message, proof)
			},
			#[cfg(feature = "runtime-benchmarks")]
			MultiProof::Empty => Ok(()),
		}
	}

	fn verify_weight(proof: &Self::Proof) -> Weight {
		match proof {
			MultiProof::Multisig(proof) => crate::MultisigVerifier::verify_weight(proof),
			#[cfg(feature = "runtime-benchmarks")]
			MultiProof::Empty => Default::default(),
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn valid_proof() -> Option<Self::Proof> {
		Some(MultiProof::Empty)
	}
}

pub struct SoraAssetIdConverter;
impl Convert<LiberlandAssetId, bridge_types::GenericAssetId> for SoraAssetIdConverter {
	fn convert(a: LiberlandAssetId) -> bridge_types::GenericAssetId {
		bridge_types::GenericAssetId::Liberland(a.into())
	}
}

pub struct SoraAccountIdConverter;
impl Convert<crate::AccountId, bridge_types::GenericAccount> for SoraAccountIdConverter {
	fn convert(a: crate::AccountId) -> bridge_types::GenericAccount {
		bridge_types::GenericAccount::Liberland(a)
	}
}

pub struct GenericBalancePrecisionConverter;
impl
	bridge_types::traits::BalancePrecisionConverter<
		LiberlandAssetId,
		crate::Balance,
		bridge_types::GenericBalance,
	> for GenericBalancePrecisionConverter
{
	fn from_sidechain(
		_: &LiberlandAssetId,
		_: u8,
		amount: bridge_types::GenericBalance,
	) -> Option<crate::Balance> {
		match amount {
			bridge_types::GenericBalance::Substrate(balance) => Some(balance),
			_ => None,
		}
	}

	fn to_sidechain(
		_: &LiberlandAssetId,
		_: u8,
		amount: crate::Balance,
	) -> Option<bridge_types::GenericBalance> {
		Some(bridge_types::GenericBalance::Substrate(amount))
	}
}

// a function that generates an account id from a seed string
pub fn get_account_id_from_string_hash(seed: &str) -> AccountId32 {
	let hash = H256::from_slice(&sp_io::hashing::blake2_256(seed.as_bytes()));
	let public_key = sp_core::sr25519::Public::from_h256(hash);
	AccountId32::from(public_key)
}

#[cfg(test)]
mod senate_filter_tests {
	use super::{SenateAccountCallFilter, RuntimeCall};
	use frame_support::{PalletId, traits::Contains};
	use sp_runtime::{traits::AccountIdConversion, AccountId32};

	fn accid() -> AccountId32 {
		PalletId(*b"12345678").into_account_truncating()
	}

	fn acc() -> sp_runtime::MultiAddress<AccountId32, ()> {
		accid().into()
	}

	#[test]
	fn allows_batch_all() {
		sp_io::TestExternalities::default().execute_with(|| {
			let calls = vec![
				RuntimeCall::LLM(pallet_llm::Call::remark { data: vec![].try_into().unwrap() }),
				RuntimeCall::LLM(pallet_llm::Call::send_llm { to_account: accid(), amount: 1u8.into() }),
				RuntimeCall::LLM(pallet_llm::Call::send_llm_to_politipool { to_account: accid(), amount: 1u8.into() }),
				RuntimeCall::Balances(pallet_balances::Call::transfer { dest: acc(), value: 1u8.into() }),
				RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive { dest: acc(), value: 1u8.into() }),
				RuntimeCall::Assets(pallet_assets::Call::transfer { id: 100.into(), target: acc(), amount: 1u8.into() }),
				RuntimeCall::Assets(pallet_assets::Call::transfer_keep_alive { id: 100.into(), target: acc(), amount: 1u8.into() }),
			];
			let call = RuntimeCall::Utility(pallet_utility::Call::batch_all { calls });
			assert!(SenateAccountCallFilter::contains(&call));
		});
	}

	#[test]
	fn allows_batch() {
		sp_io::TestExternalities::default().execute_with(|| {
			let calls = vec![
				RuntimeCall::LLM(pallet_llm::Call::remark { data: vec![].try_into().unwrap() }),
				RuntimeCall::LLM(pallet_llm::Call::send_llm { to_account: accid(), amount: 1u8.into() }),
				RuntimeCall::LLM(pallet_llm::Call::send_llm_to_politipool { to_account: accid(), amount: 1u8.into() }),
				RuntimeCall::Balances(pallet_balances::Call::transfer { dest: acc(), value: 1u8.into() }),
				RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive { dest: acc(), value: 1u8.into() }),
				RuntimeCall::Assets(pallet_assets::Call::transfer { id: 100.into(), target: acc(), amount: 1u8.into() }),
				RuntimeCall::Assets(pallet_assets::Call::transfer_keep_alive { id: 100.into(), target: acc(), amount: 1u8.into() }),
			];
			let call = RuntimeCall::Utility(pallet_utility::Call::batch { calls });
			assert!(SenateAccountCallFilter::contains(&call));
		});
	}

	#[test]
	fn allows_lld_transfers() {
		sp_io::TestExternalities::default().execute_with(|| {
			let c = RuntimeCall::Balances(pallet_balances::Call::transfer { dest: acc(), value: 1u8.into() });
			assert!(SenateAccountCallFilter::contains(&c));
			let c = RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive { dest: acc(), value: 1u8.into() });
			assert!(SenateAccountCallFilter::contains(&c));
		});
	}

	#[test]
	fn allows_assets_transfers() {
		sp_io::TestExternalities::default().execute_with(|| {
			let c = RuntimeCall::Assets(pallet_assets::Call::transfer { id: 100.into(), target: acc(), amount: 1u8.into() });
			assert!(SenateAccountCallFilter::contains(&c));
			let c = RuntimeCall::Assets(pallet_assets::Call::transfer_keep_alive { id: 100.into(), target: acc(), amount: 1u8.into() });
			assert!(SenateAccountCallFilter::contains(&c));
		});
	}

	#[test]
	fn allows_llm_transfers() {
		sp_io::TestExternalities::default().execute_with(|| {
			let c = RuntimeCall::LLM(pallet_llm::Call::remark { data: vec![].try_into().unwrap() });
			assert!(SenateAccountCallFilter::contains(&c));
			let c = RuntimeCall::LLM(pallet_llm::Call::send_llm { to_account: accid(), amount: 1u8.into() });
			assert!(SenateAccountCallFilter::contains(&c));
			let c = RuntimeCall::LLM(pallet_llm::Call::send_llm_to_politipool { to_account: accid(), amount: 1u8.into() });
			assert!(SenateAccountCallFilter::contains(&c));
		});
	}

	#[test]
	fn disallows_other_stuff() {
		sp_io::TestExternalities::default().execute_with(|| {
			let c = RuntimeCall::System(frame_system::Call::remark { remark: vec![].try_into().unwrap() });
			assert!(!SenateAccountCallFilter::contains(&c));
		});
	}

	#[test]
	fn disallows_batched_other_stuff() {
		sp_io::TestExternalities::default().execute_with(|| {
			let calls = vec![
				RuntimeCall::System(frame_system::Call::remark { remark: vec![].try_into().unwrap() }),
			];
			let call = RuntimeCall::Utility(pallet_utility::Call::batch { calls });
			assert!(!SenateAccountCallFilter::contains(&call));
		});
	}
}

#[cfg(test)]
mod council_filter_tests {
	use crate::DAYS;
	use super::{CouncilAccountCallFilter, RuntimeCall};
	use frame_support::{PalletId, traits::Contains};
	use sp_runtime::{traits::AccountIdConversion, AccountId32};

	fn wrap_in_scheduled_batch(calls: Vec<RuntimeCall>) -> RuntimeCall {
		RuntimeCall::Scheduler(pallet_scheduler::Call::schedule {
			when: 10 * DAYS,
			maybe_periodic: None,
			priority: 1,
			call: RuntimeCall::Utility(pallet_utility::Call::batch { calls }).into()
		})
	}

	fn remark_call() -> RuntimeCall {
		RuntimeCall::LLM(pallet_llm::Call::remark { data: vec![].try_into().unwrap() })
	}

	fn accid() -> AccountId32 {
		PalletId(*b"12345678").into_account_truncating()
	}

	fn acc() -> sp_runtime::MultiAddress<AccountId32, ()> {
		accid().into()
	}

	#[test]
	fn allows_schedule() {
		sp_io::TestExternalities::default().execute_with(|| {
			let call = wrap_in_scheduled_batch(vec![]);
			assert!(CouncilAccountCallFilter::contains(&call));
		});
	}

	#[test]
	fn allows_scheduled_batch_all() {
		sp_io::TestExternalities::default().execute_with(|| {
			let call = RuntimeCall::Scheduler(pallet_scheduler::Call::schedule {
				when: 10 * DAYS,
				maybe_periodic: None,
				priority: 1,
				call: RuntimeCall::Utility(pallet_utility::Call::batch_all { calls: vec![] }).into()
			});
			assert!(CouncilAccountCallFilter::contains(&call));
		});
	}

	#[test]
	fn allows_schedule_named() {
		sp_io::TestExternalities::default().execute_with(|| {
			let call = RuntimeCall::Scheduler(pallet_scheduler::Call::schedule_named {
				id: [0u8].repeat(32).try_into().unwrap(),
				when: 10 * DAYS,
				maybe_periodic: None,
				priority: 1,
				call: RuntimeCall::Utility(pallet_utility::Call::batch { calls: vec![] }).into()
			});
			assert!(CouncilAccountCallFilter::contains(&call));
		});
	}

	#[test]
	fn allows_scheduled_batched_balances() {
		sp_io::TestExternalities::default().execute_with(|| {
			let calls = vec![
				RuntimeCall::Balances(pallet_balances::Call::transfer { dest: acc(), value: 1u8.into() }),
				remark_call(),
				RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive { dest: acc(), value: 1u8.into() }),
				remark_call(),
			];
			let call = wrap_in_scheduled_batch(calls);
			assert!(CouncilAccountCallFilter::contains(&call));
		});
	}

	#[test]
	fn allows_scheduled_batched_llm() {
		sp_io::TestExternalities::default().execute_with(|| {
			let calls = vec![
				RuntimeCall::LLM(pallet_llm::Call::send_llm { to_account: accid(), amount: 1u8.into() }),
				remark_call(),
				RuntimeCall::LLM(pallet_llm::Call::send_llm_to_politipool { to_account: accid(), amount: 1u8.into() }),
				remark_call(),
			];
			let call = wrap_in_scheduled_batch(calls);
			assert!(CouncilAccountCallFilter::contains(&call));
		});
	}

	#[test]
	fn allows_scheduled_batched_assets() {
		sp_io::TestExternalities::default().execute_with(|| {
			let calls = vec![
				RuntimeCall::Assets(pallet_assets::Call::transfer { id: 100.into(), target: acc(), amount: 1u8.into() }),
				remark_call(),
				RuntimeCall::Assets(pallet_assets::Call::transfer_keep_alive { id: 100.into(), target: acc(), amount: 1u8.into() }),
				remark_call(),
			];
			let call = wrap_in_scheduled_batch(calls);
			assert!(CouncilAccountCallFilter::contains(&call));
		});
	}

	#[test]
	fn disallows_short_schedule() {
		sp_io::TestExternalities::default().execute_with(|| {
			let call = RuntimeCall::Scheduler(pallet_scheduler::Call::schedule {
				when: 1 * DAYS,
				maybe_periodic: None,
				priority: 1,
				call: RuntimeCall::Utility(pallet_utility::Call::batch { calls: vec![] }).into()
			});
			assert!(!CouncilAccountCallFilter::contains(&call));
		});
	}

	#[test]
	fn disallows_direct_batching() {
		sp_io::TestExternalities::default().execute_with(|| {
			let call = RuntimeCall::Utility(pallet_utility::Call::batch { calls: vec![] });
			assert!(!CouncilAccountCallFilter::contains(&call));

			let call = RuntimeCall::Utility(pallet_utility::Call::batch_all { calls: vec![] });
			assert!(!CouncilAccountCallFilter::contains(&call));
		});
	}

	#[test]
	fn disallows_direct_transfers() {
		sp_io::TestExternalities::default().execute_with(|| {
			let call = RuntimeCall::LLM(pallet_llm::Call::send_llm_to_politipool { to_account: accid(), amount: 1u8.into() });
			assert!(!CouncilAccountCallFilter::contains(&call));

			let call = RuntimeCall::LLM(pallet_llm::Call::send_llm { to_account: accid(), amount: 1u8.into() });
			assert!(!CouncilAccountCallFilter::contains(&call));

			let call = RuntimeCall::Assets(pallet_assets::Call::transfer { id: 100.into(), target: acc().into(), amount: 1u8.into() });
			assert!(!CouncilAccountCallFilter::contains(&call));

			let call = RuntimeCall::Balances(pallet_balances::Call::transfer { dest: acc(), value: 1u8.into() });
			assert!(!CouncilAccountCallFilter::contains(&call));
		});
	}
}

#[cfg(test)]
mod contracts_filter_tests {
	use super::{ContractsCallFilter, RuntimeCall};
	use frame_support::{PalletId, traits::Contains};
	use sp_runtime::{traits::AccountIdConversion, AccountId32};

	fn accid() -> AccountId32 {
		PalletId(*b"12345678").into_account_truncating()
	}

	fn acc() -> sp_runtime::MultiAddress<AccountId32, ()> {
		accid().into()
	}

	#[test]
	fn allows_llm_force_transfer() {
		sp_io::TestExternalities::default().execute_with(|| {
			let call = RuntimeCall::LLM(pallet_llm::Call::force_transfer {
				from: pallet_llm::LLMAccount::Liquid(accid()),
				to: pallet_llm::LLMAccount::Liquid(accid()),
				amount: 1u8.into(),
			});
			assert!(ContractsCallFilter::contains(&call));
		});
	}

	#[test]
	fn allows_assets_approve_transfer() {
		sp_io::TestExternalities::default().execute_with(|| {
			let call = RuntimeCall::Assets(pallet_assets::Call::approve_transfer {
				id: 0.into(),
				delegate: acc(),
				amount: 1u8.into(),
			});
			assert!(ContractsCallFilter::contains(&call));
		});
	}

	#[test]
	fn allows_assets_cancel_approval() {
		sp_io::TestExternalities::default().execute_with(|| {
			let call = RuntimeCall::Assets(pallet_assets::Call::cancel_approval {
				id: 0.into(),
				delegate: acc(),
			});
			assert!(ContractsCallFilter::contains(&call));
		});
	}

	#[test]
	fn allows_assets_transfer() {
		sp_io::TestExternalities::default().execute_with(|| {
			let call = RuntimeCall::Assets(pallet_assets::Call::transfer {
				id: 0.into(),
				target: acc(),
				amount: 1u8.into(),
			});
			assert!(ContractsCallFilter::contains(&call));
		});
	}

	#[test]
	fn allows_assets_transfer_approved() {
		sp_io::TestExternalities::default().execute_with(|| {
			let call = RuntimeCall::Assets(pallet_assets::Call::transfer_approved {
				id: 0.into(),
				owner: acc(),
				destination: acc(),
				amount: 1u8.into(),
			});
			assert!(ContractsCallFilter::contains(&call));
		});
	}

	#[test]
	fn allows_assets_transfer_keep_alive() {
		sp_io::TestExternalities::default().execute_with(|| {
			let call = RuntimeCall::Assets(pallet_assets::Call::transfer_keep_alive {
				id: 0.into(),
				target: acc(),
				amount: 1u8.into(),
			});
			assert!(ContractsCallFilter::contains(&call));
		});
	}

	#[test]
	fn disallows_other_stuff() {
		sp_io::TestExternalities::default().execute_with(|| {
			let call = RuntimeCall::System(frame_system::Call::remark { remark: vec![].try_into().unwrap() });
			assert!(!ContractsCallFilter::contains(&call));
		});
	}
}

#[cfg(test)]
mod multiplier_tests {
	use frame_support::{
		dispatch::DispatchClass,
		weights::{Weight, WeightToFee},
	};
	use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
	use sp_runtime::{
		assert_eq_error_rate,
		traits::{Convert, One, Zero},
		BuildStorage, FixedPointNumber,
	};

	use crate::{
		constants::{currency::*, time::*},
		AdjustmentVariable, MaximumMultiplier, MinimumMultiplier, Runtime,
		RuntimeBlockWeights as BlockWeights, System, TargetBlockFullness, TransactionPayment,
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

		let max_normal = max_normal();
		let target_weight = target();
		let normalized_weight_dimensions = (
			block_weight.ref_time() as f64 / max_normal.ref_time() as f64,
			block_weight.proof_size() as f64 / max_normal.proof_size() as f64,
		);

		let (normal, max, target) =
			if normalized_weight_dimensions.0 < normalized_weight_dimensions.1 {
				(block_weight.proof_size(), max_normal.proof_size(), target_weight.proof_size())
			} else {
				(block_weight.ref_time(), max_normal.ref_time(), target_weight.ref_time())
			};

		// maximum tx weight
		let m = max as f64;
		// block weight always truncated to max weight
		let block_weight = (normal as f64).min(m);
		let v: f64 = AdjustmentVariable::get().to_float();

		// Ideal saturation in terms of weight
		let ss = target as f64;
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
		let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.unwrap()
			.into();
		t.execute_with(|| {
			System::set_block_consumed_resources(w, 0);
			assertions()
		});
	}

	pub use node_primitives::Signature;
	use sp_core::{Public, Pair};
	use sp_runtime::traits::{IdentifyAccount, Verify};

	type AccountPublic = <Signature as Verify>::Signer;

	/// Helper function to generate a crypto pair from seed
	pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
		TPublic::Pair::from_string(&format!("//{}", seed), None)
			.expect("static values are valid; qed")
			.public()
	}

	/// Helper function to generate an account ID from seed
	pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
	where
		AccountPublic: From<<TPublic::Pair as Pair>::Public>,
	{
		AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
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
		// the block ref time is 1/100th bigger than target.
		run_with_system_weight(target().set_ref_time(target().ref_time() * 101 / 100), || {
			let next = runtime_multiplier_update(min_multiplier());
			assert!(next > min_multiplier(), "{:?} !> {:?}", next, min_multiplier());
		});

		// the block proof size is 1/100th bigger than target.
		run_with_system_weight(target().set_proof_size((target().proof_size() / 100) * 101), || {
			let next = runtime_multiplier_update(min_multiplier());
			assert!(next > min_multiplier(), "{:?} !> {:?}", next, min_multiplier());
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
		let kb_time = Weight::from_parts(1024, 0);
		let kb_size = Weight::from_parts(0, 1024);
		let mb_time = 1024u64 * kb_time;
		let max_fm = Multiplier::saturating_from_integer(i128::MAX);

		// check that for all values it can compute, correctly.
		vec![
			Weight::zero(),
			// testcases ignoring proof size part of the weight.
			Weight::from_parts(1, 0),
			Weight::from_parts(10, 0),
			Weight::from_parts(1000, 0),
			kb_time,
			10u64 * kb_time,
			100u64 * kb_time,
			mb_time,
			10u64 * mb_time,
			Weight::from_parts(2147483647, 0),
			Weight::from_parts(4294967295, 0),
			// testcases ignoring ref time part of the weight.
			Weight::from_parts(0, 100000000000),
			1000000u64 * kb_size,
			1000000000u64 * kb_size,
			Weight::from_parts(0, 18014398509481983),
			Weight::from_parts(0, 9223372036854775807),
			// test cases with both parts of the weight.
			BlockWeights::get().max_block / 1024,
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
		vec![
			t + Weight::from_parts(100, 0),
			t + Weight::from_parts(0, t.proof_size() * 2),
			t * 2,
			t * 4,
		]
		.into_iter()
		.for_each(|i| {
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

	use frame_system::{EnsureRoot, RawOrigin};
	use node_primitives::AccountId;
	use core::cmp::Ordering;
	use sp_runtime::testing::sr25519;
	use crate::{EnsureCmp, OriginCaller, sp_api_hidden_includes_construct_runtime::hidden_include::traits::PrivilegeCmp};

	#[test]
	fn ensure_cmp_works_for_root() {
		type OriginPrivilegeCmp = EnsureCmp<
				EnsureRoot<AccountId>
		>;

		assert_eq!(
			OriginPrivilegeCmp::cmp_privilege(
				&OriginCaller::system(RawOrigin::Root), 
				&OriginCaller::system(RawOrigin::Root)
			),
			Some(Ordering::Equal)
		);
	}

	#[test]
	fn ensure_cmp_do_not_works_for_account() {
		let alice = get_account_id_from_seed::<sr25519::Public>("Alice");

		type OriginPrivilegeCmp = EnsureCmp<
				EnsureRoot<AccountId>
		>;

		assert_eq!(
			OriginPrivilegeCmp::cmp_privilege(
				&OriginCaller::system(RawOrigin::Signed(alice)), 
				&OriginCaller::system(RawOrigin::Root)
			),
			None
		);
	}
}
