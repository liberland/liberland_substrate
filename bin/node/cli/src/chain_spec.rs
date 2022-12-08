// This file is part of Substrate.

// Copyright (C) 2018-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Substrate chain configurations.

// File has been modified by Liberland in 2022. All modifications by Liberland are distributed under the MIT license.

// You should have received a copy of the MIT license along with this program. If not, see https://opensource.org/licenses/MIT

use grandpa_primitives::AuthorityId as GrandpaId;
use kitchensink_runtime::{
	constants::currency::*, wasm_binary_unwrap, AuthorityDiscoveryConfig, BabeConfig,
	BalancesConfig, Block, CouncilConfig, DemocracyConfig, ElectionsConfig, GrandpaConfig,
	ImOnlineConfig, IndicesConfig, MaxNominations, SessionConfig,
	SessionKeys, SocietyConfig, StakerStatus, StakingConfig, SudoConfig, SystemConfig,
	TechnicalCommitteeConfig, LiberlandInitializerConfig,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::{ChainSpecExtension, Properties};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};

pub use kitchensink_runtime::GenesisConfig;
pub use node_primitives::{AccountId, Balance, Signature};

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
	/// The light sync state extension used by the sync-state rpc.
	pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;
/// Flaming Fir testnet generator
pub fn flaming_fir_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/flaming-fir.json")[..])
}

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

fn staging_testnet_config_genesis() -> GenesisConfig {
	#[rustfmt::skip]
	// stash, controller, session-key
	// generated with secret:
	// for i in 1 2 3 4 ; do for j in stash controller; do subkey inspect "$secret"/fir/$j/$i; done; done
	//
	// and
	//
	// for i in 1 2 3 4 ; do for j in session; do subkey --ed25519 inspect "$secret"//fir//$j//$i; done; done

	let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = vec![
		// F
		(
		// 5DRUP4qyVHqPJoybR6XUE2HZ5GDuyBJ67VK5cP5ULgCF3RTJ
			array_bytes::hex_n_into_unchecked("3c1305e6cfd61af1ca30dd3532705c614ebcc4ee8431bbae97fbadf778fdaa7e"),
			// 5EvVVsUhTfaSocMkp2S2MTiBa8JBem8d7nhdJkeNVv1kYf95
			array_bytes::hex_n_into_unchecked("7e71226a60ac69915ad04bc7fad2235452533ceb714396196b9aac939ad65c74"),
			// grandpa:5FB4Mqt26dYN4vHDFwZfKEZhGsrqJNTBop6cTCR87ys1vmn7
			array_bytes::hex2array_unchecked("898d312c59c405816d2435f2fe8141f051be1ee7285418ac738d3886911f0332").unchecked_into(),
			// babe:  5FCkVeUbr34jiwQXZtrJhzJpmKJKS4iMKyRUkhPdiGJyqfSu
			array_bytes::hex2array_unchecked("8ad78ada77b349e5d4dcb5989bfbf430de021256a314239aaea1fc9e201f0746").unchecked_into(),
			// imol:  5G3kiRAZLJ5gwDpo5oR6NySSnuuxzyEuiLtfzYpjmNpM2qCJ
			array_bytes::hex2array_unchecked("b0374ef3a33e8b9efb6eb531ca0956d984f9e8c1d50d5ddd39a133fb46e6de34").unchecked_into(),
			// audi:  5DyvCLAVWuat381u3DHgo9aq8cwSjzEtRXxx72PZiZ9sdiMh
			array_bytes::hex2array_unchecked("54d1c6d509ed9abea92323cc243528dff84accfb6a8aaed6652c2553e29f0e30").unchecked_into(),
		),
		(
			// 5GznkA2FEC2c9rJqbX5zA4Fyrwrb24KYtHDHBAoY7KpgNdwv
			array_bytes::hex_n_into_unchecked("da30a3a0b2ae75542a2cb84ecd230417ab68fc9df607f5b1e22c10cb5ae57637"),
			// 5EZP5Hd2beWc1C4dJ4iWYAifWsiJ2qeupNHCaBRaCbXRK6VB
			array_bytes::hex_n_into_unchecked("6e575b3749789938e140bbb0b933def97c99a25cdf4968cf85b6ae09b3296b5b"),
			// grandpa:5HAnjVEhxzMfHS6WKH9iX4mzn8A3HxMbXJCmLXU3JaV9AQhA
			array_bytes::hex2array_unchecked("e1d10ded56af2b86e8e3651d928c22506d34843d63c9686d76aab55a548c4ce0").unchecked_into(),
			// babe:  5DyVvqYH5kETsw9CYicR8ujbHw4wYjvcJHMtwRK7QtC5kPj8
			array_bytes::hex2array_unchecked("5480161452b1b86f588c98e1019c80d57a28beb867d116a8713dc3301a582802").unchecked_into(),
			// imol:  5GpC3UWCLD38gfDh1uDRDXtMz26UCspbUmoeKnBktJ13ZFSn
			array_bytes::hex2array_unchecked("d21b5eaac541cf7b51a0be6b0164572ad7f2d5dc599ea3151f3830465dd37148").unchecked_into(),
			// audi:  5D57EqeRzTyeoMpFZViVRGC6wzC8rFg2joxBTK2fciZb9maj
			array_bytes::hex2array_unchecked("2c8af1f4a2043a08170164ad6eb0d45c15005204074c7cb99a0d36190d7f1a7a").unchecked_into(),
		),
//		(
//			// 5CrJUdQtoVLDwQHf9P9GjXh21mn5cpWisBtyBVkGgzi3aNd8
//			array_bytes::hex_n_into_unchecked("22c695a1919847596698b7fc8f8efc23bf96f4156d14472f42736d6475c3280b"),
//			// 5DAvZ3ynu3wQbEJBbxWy8MDa7T8LuUjpRmf5ooCJWFeLrbhw
//			array_bytes::hex_n_into_unchecked("30fa7200854b9d8db626e8dc0721b49286a2e9eec4323547b5ea64a5d07caf33"),
//			// grandpa:5EnR7f7T1ZxKHPjs6FKm9xGzzvT3bMPCtMVHg8PNN5Jnai2U
//			array_bytes::hex2array_unchecked("78486ab057148ea8082d40703ce950b7611e74582100d4b3febfee20b0ba9de8").unchecked_into(),
//			// babe:  5CccJe3SYWD4Syw93BruVn3AMN3GV14cRDZ5PdFv2cTpE7Uv
//			array_bytes::hex2array_unchecked("18552997b0f38c215d599420bd0e3c464dc28dcb95af1f5e5c95a95cb1ae5e3c").unchecked_into(),
//			// imol:  5Cqt26giAT8pKkdmsPpF9Lrjhzg7fH9RSeBG2dX49YmzESFV
//			array_bytes::hex2array_unchecked("227440e662ccff1316b02ac03a1a70e7e33f5a4e6ed2f10c983c7ba7bce46f78").unchecked_into(),
//			// audi:  5HR2VDzZutnsGyys4bE3hSSspdYjNmpRXysTcP1fnLVPaW1Y
//			array_bytes::hex2array_unchecked("ecacca5b4a5408b3d191bf6d22ad78a04fd27af249ffaa07441dcc5b2a5fff69").unchecked_into(),
//		),
	];

	let root_key: AccountId = array_bytes::hex_n_into_unchecked(
		// F new
		//5GZXCJvjfniCCLmKiyqzXLdwgcSgiQNUtsuFVhrpvfjopShL
		"c6eb294494e9afe9cc64eac8f24e70b775cfc6d3e34b0bedec9273325603bd3d"
	);

	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	testnet_genesis(initial_authorities, vec![], root_key, Some(endowed_accounts), Some(vec![]), None, vec![])
}

fn properties() -> sc_chain_spec::Properties {
	let mut p = Properties::new();
	p.insert("prefix".into(), 56.into());
	p.insert("network".into(), "liberland".into());
	p.insert("displayName".into(), "Liberland Hazlitt".into());
	p.insert("tokenSymbol".into(), "LLD".into());
	p.insert("tokenDecimals".into(), 12.into());
	p.insert("standardAccount".into(), "*25519".into());
	p.insert("ss58Format".into(), 56.into());
	p.insert("website".into(), "https://liberland.org".into());
	p
}
/// Staging testnet config.
pub fn staging_testnet_config() -> ChainSpec {
	let boot_nodes = vec![];
	ChainSpec::from_genesis(
		"Staging Testnet",
		"staging_testnet",
		ChainType::Live,
		staging_testnet_config_genesis,
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Staging telemetry url is valid; qed"),
		),
		None,
		None,
		Some(properties()),
		Default::default(),
	)
}

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

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(
	seed: &str,
) -> (AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
	council_group: Option<Vec<AccountId>>,
	citizenship_registrar: Option<AccountId>,
	initial_citizens: Vec<(AccountId, Balance)>,
) -> GenesisConfig {
	let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		]
	});

	let council_group: Vec<AccountId> = council_group.unwrap_or(vec![
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		get_account_id_from_seed::<sr25519::Public>("Bob"),
		get_account_id_from_seed::<sr25519::Public>("Charlie"),
	]);

	// endow all authorities and nominators.
	initial_authorities
		.iter()
		.map(|x| &x.0)
		.chain(initial_nominators.iter())
		.for_each(|x| {
			if !endowed_accounts.contains(x) {
				endowed_accounts.push(x.clone())
			}
		});

	// stakers: all validators and nominators.
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (MaxNominations::get() as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.into_iter()
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), STASH, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	let num_endowed_accounts = endowed_accounts.len();

	const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
	const STASH: Balance = ENDOWMENT / 1000;

	// Add Prefunded accounts
	let f_ac: Vec<AccountId> = vec![
		array_bytes::hex_n_into_unchecked("061a7f0a43e35d16f330e64c1a4e5000db4ba064fc3630cc4a9e2027899a5a6f"), //F
		array_bytes::hex_n_into_unchecked("b86373a2dff0a7b5741fd7e1857de41353fca3b924f14eae5f4c70d69e949150"), // N
		array_bytes::hex_n_into_unchecked("ba14fb5a00f052330c9c09e0467bce1d7896edefe92851b893e777aade53f921"), // D
		array_bytes::hex_n_into_unchecked("f874b8c112a9bb565e0798d9b5dcfee0fdbd54dd0fcc865c1251a75bd3faee45"), // M
		array_bytes::hex_n_into_unchecked("52fd11392742ccf58bcff90c33ca15bdf4bd3416aabcd5d51a654c1f387b6d18"),
	];
	// rewrite, not to use for loop
	for ac in f_ac.iter() {
		endowed_accounts.push(ac.clone());
	}

	GenesisConfig {
		system: SystemConfig { code: wasm_binary_unwrap().to_vec() },
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect(),
		},
		indices: IndicesConfig { indices: vec![] },
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers,
			..Default::default()
		},
		democracy: DemocracyConfig::default(),
		elections: ElectionsConfig {
			members: council_group
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.map(|member| (member, STASH))
				.collect(),
		},
		council: CouncilConfig::default(),
		technical_committee: TechnicalCommitteeConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			phantom: Default::default(),
		},
		sudo: SudoConfig { key: Some(root_key) },
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(kitchensink_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		im_online: ImOnlineConfig { keys: vec![] },
		authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
		grandpa: GrandpaConfig { authorities: vec![] },
		technical_membership: Default::default(),
		treasury: Default::default(),
		society: SocietyConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			pot: 0,
			max_members: 999,
		},
		vesting: Default::default(),
		assets: Default::default(),
		gilt: Default::default(),
		transaction_storage: Default::default(),
		transaction_payment: Default::default(),
		llm: Default::default(),
		liberland_initializer: LiberlandInitializerConfig {
			citizenship_registrar, initial_citizens
		},
	}
}

fn development_config_genesis() -> GenesisConfig {
	let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
	testnet_genesis(
		vec![authority_keys_from_seed("Alice")],
		vec![],
		alice.clone(),
		None,
		None,
		Some(alice.clone()),
		vec![
			(alice, 1000),
			(get_account_id_from_seed::<sr25519::Public>("Bob"), 1000),
			(get_account_id_from_seed::<sr25519::Public>("Charlie"), 1000),
		],
	)
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Development",
		"dev",
		ChainType::Development,
		development_config_genesis,
		vec![],
		None,
		None,
		None,
		Some(properties()),
		Default::default(),
	)
}

fn local_testnet_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		None,
		None,
		vec![],
	)
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		ChainType::Local,
		local_testnet_genesis,
		vec![],
		None,
		None,
		None,
		Some(properties()),
		Default::default(),
	)
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use crate::service::{new_full_base, NewFullBase};
	use sc_service_test;
	use sp_runtime::BuildStorage;

	fn local_testnet_genesis_instant_single() -> GenesisConfig {
		testnet_genesis(
			vec![authority_keys_from_seed("Alice")],
			vec![],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			None,
			None,
			None,
			vec![],
		)
	}

	/// Local testnet config (single validator - Alice)
	pub fn integration_test_config_with_single_authority() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis_instant_single,
			vec![],
			None,
			None,
			None,
			None,
			Default::default(),
		)
	}

	/// Local testnet config (multivalidator Alice + Bob)
	pub fn integration_test_config_with_two_authorities() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis,
			vec![],
			None,
			None,
			None,
			None,
			Default::default(),
		)
	}

	#[test]
	#[ignore]
	fn test_connectivity() {
		sp_tracing::try_init_simple();

		sc_service_test::connectivity(integration_test_config_with_two_authorities(), |config| {
			let NewFullBase { task_manager, client, network, transaction_pool, .. } =
				new_full_base(config, false, |_, _| ())?;
			Ok(sc_service_test::TestNetComponents::new(
				task_manager,
				client,
				network,
				transaction_pool,
			))
		});
	}

	#[test]
	fn test_create_development_chain_spec() {
		development_config().build_storage().unwrap();
	}

	#[test]
	fn test_create_local_testnet_chain_spec() {
		local_testnet_config().build_storage().unwrap();
	}

	#[test]
	fn test_staging_test_net_chain_spec() {
		staging_testnet_config().build_storage().unwrap();
	}
}
