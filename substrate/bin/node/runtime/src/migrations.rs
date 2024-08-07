use super::*;
use frame_support::traits::OnRuntimeUpgrade;

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

type DbWeight = <Runtime as frame_system::Config>::DbWeight;

pub mod add_onchain_identities {
	use super::*;
	use crate::Identity;
	use pallet_identity::{Data, IdentityInfo, Judgement};
	use sp_core::crypto::Ss58Codec;

	pub struct Migration<T>(sp_std::marker::PhantomData<T>);

	impl OnRuntimeUpgrade for Migration<Runtime> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			Ok(().encode())
		}

		fn on_runtime_upgrade() -> Weight {
			let identities = vec![
				("5EYCAe5hvejUE1BUTDSnxDfCqVkADRicSKqbcJrduV1KCDmk", b"Vault".to_vec()),
				("5EYCAe5hveooUENA5d7dwq3caqM4LLBzktNumMKmhNRXu4JE", b"Senate".to_vec()),
				(
					"5EYCAe5iXF2YZuVZv1vig4xvf1CcDVocZCWYrv3TVSXpMTYA",
					b"Citizenship Office".to_vec(),
				),
				//("5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z", b"Treasury"), - skipping this as Polkadot.js Apps have it builtin
				("5EYCAe5g8CDuMsTief7QBxfvzDFEfws6ueXTUhsbx5V81nGH", b"Congress".to_vec()),
				("5GmkwXZ94cuLMMbtE5VtBaLpFDehoEpy6MZnJhkCSicxePs2", b"SORA Bridge".to_vec()),
				("5DSfG3S7qSZzrDMj3F3qYybXAy1BLsVpRG5CNwRdwwNPjgVm", b"MEXC".to_vec()),
				("5HEX1wk33NHAeEJV3B6goDHMJTqhy411znCUmfEAxKkQeqds", b"Coinstore".to_vec()),
				("5EYCAe5iXF2YZiuxDWAcwtPMDaG7ihsYzPxajcNKRpNyD1Zi", b"Company Registry Office".to_vec()),
				("5EYCAe5iXF2YZzoQHqGhj9dtcsUNB4puM5GqR1BwVHZyaWxM", b"Land Registry Office".to_vec()),
				("5EYCAe5iXF2Ya2c2iKjeFWUAXAEcMyoKCBPvhRy8YprHTLNd", b"Metaverse Land Registry Office".to_vec()),
				("5EYCAe5iXF2YZfPZ4arKGSYZvQruDacGHkuw4qAsQSQsKpMK", b"Asset Registry Office".to_vec()),
				("5EYCAe5iXF2YZpCZr7ALYUUYaNpMXde3NUXxYn1Sc1YRM4gV", b"Ministry of Finance Office".to_vec()),
				("5FBQRNJfzsttYvw1XnSwxwSUmb7A3EYm4q8aiscADREqvzYz", b"Wrapped LLD Token Contract".to_vec()),
			];
			let mut weight = DbWeight::get().reads(0);
			for (addr, display) in identities {
				let account = AccountId::from_ss58check(addr).unwrap();
				let id = IdentityInfo {
					twitter: Data::None,
					additional: vec![].try_into().unwrap(),
					display: Data::Raw(display.try_into().unwrap()),
					legal: Data::None,
					web: Data::None,
					riot: Data::None,
					email: Data::None,
					pgp_fingerprint: None,
					image: Data::None,
				};
				let judgements = vec![(0, Judgement::KnownGood)].try_into().unwrap();
				let set_identity_weight =
					Identity::set_identity_no_deposit(&account, judgements, id);
				weight = weight.saturating_add(set_identity_weight);
			}

			weight
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
			Ok(())
		}
	}
}
