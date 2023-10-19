// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
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
	pallet_prelude::{PhantomData, Get, MaxEncodedLen},
	RuntimeDebug,
	traits::{
		fungibles::{Balanced, CreditOf},
		Currency, OnUnbalanced, InstanceFilter,
		Contains,
	},
	dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo, Dispatchable, DispatchInfo, GetDispatchInfo},
};
use sp_runtime::{AccountId32, DispatchError, DispatchResult, traits::Morph};
use pallet_asset_tx_payment::HandleCredit;
use sp_staking::{EraIndex, OnStakerSlash};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
use sp_core::H256;
use scale_info::TypeInfo;
use frame_support::pallet_prelude::Weight;
use bridge_types::GenericNetworkId;
use sp_runtime::traits::Convert;

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
	fn handle_credit(credit: CreditOf<AccountId, Assets>) {
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
					RuntimeCall::Identity(pallet_identity::Call::set_fee { .. }) |
					RuntimeCall::Identity(pallet_identity::Call::set_fields { .. }) |
					RuntimeCall::Identity(pallet_identity::Call::set_account_id { .. }) |
					RuntimeCall::Identity(pallet_identity::Call::provide_judgement { .. })
				),
			IdentityCallFilter::Judgement =>
				matches!(c,
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

pub struct LiberlandMessageStatusNotifier;

impl bridge_types::traits::MessageStatusNotifier<u32, AccountId, Balance> for LiberlandMessageStatusNotifier {
	fn update_status(
		_: bridge_types::GenericNetworkId, 
		_: H256, 
		_: bridge_types::types::MessageStatus, 
		_: bridge_types::GenericTimepoint
	) { 
	}

	fn inbound_request(
		_: bridge_types::GenericNetworkId, 
		_: H256, 
		_: bridge_types::GenericAccount, 
		_: AccountId, 
		_: u32, 
		_: Balance, 
		_: bridge_types::GenericTimepoint, 
		_: bridge_types::types::MessageStatus
	) { 
	}
	
	fn outbound_request(
		_: bridge_types::GenericNetworkId, 
		_: H256, 
		_: AccountId, 
		_: bridge_types::GenericAccount, 
		_: u32, 
		_: Balance, 
		_: bridge_types::types::MessageStatus
	) { 
	}
}

impl bridge_types::traits::BridgeAssetRegistry<AccountId, u32> for LiberlandMessageStatusNotifier {
	type AssetName = ();
	type AssetSymbol =  ();

	fn register_asset(
		_: GenericNetworkId, 
		_: <Self as bridge_types::traits::BridgeAssetRegistry<AccountId, u32>>::AssetName, 
		_: <Self as bridge_types::traits::BridgeAssetRegistry<AccountId, u32>>::AssetSymbol
	) -> Result<u32, DispatchError> { 
		// todo!()
		// Asset::create()
		Err(DispatchError::Other("NOT AVAILIBLE"))
	}

	fn manage_asset(
		_: GenericNetworkId, 
		_: u32
	) -> Result<(), DispatchError> { 
		Ok(())
	}

	fn get_raw_info(
		asset_id: u32
	) -> bridge_types::types::RawAssetInfo { 
		// let a = <crate::Assets as crate::Runtime>::Metadata::get(asset_id);
		// let a = crate::Assets::Metadata::get(asset_id);
		// let a = crate::Asset;
		// todo!();
		bridge_types::types::RawAssetInfo {
			name: Vec::new(),
			symbol: Vec::new(),
			precision: 0,
		}
	}
}

impl bridge_types::traits::BridgeAssetLocker<AccountId> for LiberlandMessageStatusNotifier {
	type AssetId = u32;
	type Balance = Balance;

	fn lock_asset(
			network_id: GenericNetworkId,
			asset_kind: bridge_types::types::AssetKind,
			who: &AccountId,
			asset_id: &Self::AssetId,
			amount: &Self::Balance,
		) -> DispatchResult {
		// todo!()
		Ok(())
	}

	fn unlock_asset(
			network_id: GenericNetworkId,
			asset_kind: bridge_types::types::AssetKind,
			who: &AccountId,
			asset_id: &Self::AssetId,
			amount: &Self::Balance,
		) -> DispatchResult {
		// todo!()
		Ok(())
	}
}

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
        frame_support::log::debug!("Dispatching SubstrateBridgeCall: {:?}", self.0);
        match self.0 {
            bridge_types::substrate::BridgeCall::ParachainApp(msg) => Err(DispatchErrorWithPostInfo {
                post_info: Default::default(),
                error: DispatchError::Other("Unavailable"),
            }),
            bridge_types::substrate::BridgeCall::XCMApp(_msg) => Err(DispatchErrorWithPostInfo {
                post_info: Default::default(),
                error: DispatchError::Other("Unavailable"),
            }),
            bridge_types::substrate::BridgeCall::DataSigner(msg) => {
                let call: bridge_data_signer::Call<crate::Runtime> = msg.into();
                let call: crate::RuntimeCall = call.into();
                call.dispatch(origin)
            }
            bridge_types::substrate::BridgeCall::MultisigVerifier(msg) => {
                let call: multisig_verifier::Call<crate::Runtime> = msg.into();
                let call: crate::RuntimeCall = call.into();
                call.dispatch(origin)
            }
            bridge_types::substrate::BridgeCall::SubstrateApp(msg) => {
                let call: substrate_bridge_app::Call<crate::Runtime> = msg.try_into()?;
                let call: crate::RuntimeCall = call.into();
                call.dispatch(origin)
            }
        }
    }
}

impl GetDispatchInfo for DispatchableSubstrateBridgeCall {
    fn get_dispatch_info(&self) -> DispatchInfo {
        match &self.0 {
            bridge_types::substrate::BridgeCall::ParachainApp(msg) => Default::default(),
            bridge_types::substrate::BridgeCall::XCMApp(_msg) => Default::default(),
            bridge_types::substrate::BridgeCall::DataSigner(msg) => {
                let call: bridge_data_signer::Call<crate::Runtime> = msg.clone().into();
                call.get_dispatch_info()
            }
            bridge_types::substrate::BridgeCall::MultisigVerifier(msg) => {
                let call: multisig_verifier::Call<crate::Runtime> = msg.clone().into();
                call.get_dispatch_info()
            }
            bridge_types::substrate::BridgeCall::SubstrateApp(msg) => {
                let call: substrate_bridge_app::Call<crate::Runtime> =
                    match substrate_bridge_app::Call::try_from(msg.clone()) {
                        Ok(c) => c,
                        Err(_) => return Default::default(),
                    };
                call.get_dispatch_info()
            }
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
            MultiProof::Multisig(proof) => crate::MultisigVerifier::verify(network_id, message, proof),
            #[cfg(feature = "runtime-benchmarks")]
            MultiProof::Empty => Ok(()),
        }
		// todo!()
    }

    fn verify_weight(proof: &Self::Proof) -> Weight {
        match proof {
            MultiProof::Multisig(proof) => crate::MultisigVerifier::verify_weight(proof),
            #[cfg(feature = "runtime-benchmarks")]
            MultiProof::Empty => Default::default(),
        }
		// todo!()
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn valid_proof() -> Option<Self::Proof> {
        Some(MultiProof::Empty)
    }
}

pub struct SoraAssetIdConverter;
impl Convert<u32, bridge_types::GenericAssetId> for SoraAssetIdConverter {
    fn convert(a: u32) -> bridge_types::GenericAssetId {
        bridge_types::GenericAssetId::Liberland(a.into())
    }
}

pub struct SoraAccountIdConverter;
impl Convert<crate::AccountId, bridge_types::GenericAccount> for SoraAccountIdConverter {
    fn convert(a: crate::AccountId) -> bridge_types::GenericAccount {
        bridge_types::GenericAccount::Sora(a)
    }
}

pub struct GenericBalancePrecisionConverter;
impl bridge_types::traits::BalancePrecisionConverter<u32, crate::Balance, bridge_types::GenericBalance>
    for GenericBalancePrecisionConverter
{
    fn from_sidechain(
        asset_id: &u32,
        sidechain_precision: u8,
        amount: bridge_types::GenericBalance,
    ) -> Option<crate::Balance> {
        // let thischain_precision = crate::Assets::asset_infos(asset_id).2;
        // match amount {
        //     GenericBalance::Substrate(val) => BalancePrecisionConverter::convert_precision(
        //         sidechain_precision,
        //         thischain_precision,
        //         val,
        //     ),
        //     GenericBalance::EVM(_) => None,
        // }
		// todo!()
		match amount {
			bridge_types::GenericBalance::Substrate(balance) => Some(balance),
			_ => None,
		}
    }

    fn to_sidechain(
        asset_id: &u32,
        sidechain_precision: u8,
        amount: crate::Balance,
    ) -> Option<bridge_types::GenericBalance> {
        // let thischain_precision = crate::Assets::asset_infos(asset_id).2;
        // let amount = BalancePrecisionConverter::convert_precision(
        //     thischain_precision,
        //     sidechain_precision,
        //     amount,
        // )?;
        // Some(GenericBalance::Substrate(amount))
		Some(bridge_types::GenericBalance::Substrate(amount))
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
			(Weight::from_ref_time(100), fm),
			(Weight::from_ref_time(1000), fm),
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
			Weight::from_ref_time(100);

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
		let kb = Weight::from_ref_time(1024);
		let mb = 1024u64 * kb;
		let max_fm = Multiplier::saturating_from_integer(i128::MAX);

		// check that for all values it can compute, correctly.
		vec![
			Weight::zero(),
			Weight::from_ref_time(1),
			Weight::from_ref_time(10),
			Weight::from_ref_time(1000),
			kb,
			10u64 * kb,
			100u64 * kb,
			mb,
			10u64 * mb,
			Weight::from_ref_time(2147483647),
			Weight::from_ref_time(4294967295),
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
		vec![t + Weight::from_ref_time(100), t * 2, t * 4].into_iter().for_each(|i| {
			run_with_system_weight(i, || {
				let fm = runtime_multiplier_update(max_fm);
				// won't grow. The convert saturates everything.
				assert_eq!(fm, max_fm);
			})
		});
	}
}
