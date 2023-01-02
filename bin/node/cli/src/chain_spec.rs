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
	constants::currency::*, constants::llm::*, wasm_binary_unwrap, AuthorityDiscoveryConfig,BabeConfig,
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
use sp_core::{crypto::{Ss58Codec, UncheckedInto}, sr25519, Pair, Public};
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
	let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = vec![
		// Liberland Node 1
		(
			AccountId::from_ss58check("5FxyS2TCjXUNEP1FMJZMgMvGeUSNZnm5QjGLZ1o2ti1xvmib").unwrap(),
			AccountId::from_ss58check("5GqJ6qrVubs5kzyBdv2sYvHXwy85S7wLA2yghYqu3EaNvYj3").unwrap(),
			array_bytes::hex2array_unchecked("8e63d55a14700c735baa7c18bf1f6fb4cbe454387374ed662a739cdf4c15ad73").unchecked_into(),
			array_bytes::hex2array_unchecked("55185a442c58e9d91a880fe45af3413631815b18effd845169faf0a4b1c12497").unchecked_into(),
			array_bytes::hex2array_unchecked("d7275a3d0df56b87b4f230f1cc6bf430aca2ad4953fff701fb37dd5556664dbe").unchecked_into(),
			array_bytes::hex2array_unchecked("2eb75e355660c82e30c8e3429b9469c067abcd83b7ab637d46ca34ab25041f6b").unchecked_into(),
		),
		// Liberland Node 2
		(
			AccountId::from_ss58check("5HaWo4ZFPsq8hKQrZKCb1C7izD3V2ZHCjaj9nLMrGNZzrkVy").unwrap(),
			AccountId::from_ss58check("5D7xXnKkqWZ6vJZQBsVdDaoUcDvnV15M4Vhg8U6keupjNoYc").unwrap(),
			array_bytes::hex2array_unchecked("e365887ead354db2e19807e7a8ffbb5b68321cfd7654df82186ea1f817ed2904").unchecked_into(),
			array_bytes::hex2array_unchecked("699c1c18aa7f444f3e81b58c4b218e3e440f0b77e27c90b4e764af665bca1f23").unchecked_into(),
			array_bytes::hex2array_unchecked("5ad7f1e8ce3bf11a2dde58a05903168c361e3fa2538fccd50b524bac116bb747").unchecked_into(),
			array_bytes::hex2array_unchecked("a3667217050f77a2810a7ef1166a8ecea8552c235b4be28cf998be738350040f").unchecked_into(),
		),
		// Liberland Node 3
		(
			AccountId::from_ss58check("5EKjSVVbUtJRF2p6X6X48TgewWsBkK8BeCfAdh4vrM1dSZ97").unwrap(),
			AccountId::from_ss58check("5E2AuKNFwMPKfJnm8kJhjfDRqs3gTFjuCih4vjxcXtopsEM1").unwrap(),
			array_bytes::hex2array_unchecked("bdda118513e66d74ed49c809261cb043c220093115a2b808cc89cec2f76f603d").unchecked_into(),
			array_bytes::hex2array_unchecked("dc5f32a7c37160ae457080060d6642cb134673a47d50b614a1d365b3e5ac7c0c").unchecked_into(),
			array_bytes::hex2array_unchecked("62ec31fc4e453371432bc0abd320eb6de2a326e0fc7839723d4ea67a2a43a46c").unchecked_into(),
			array_bytes::hex2array_unchecked("142a51556ba55259d45e66aed6424a5d3e965b3d4141779be971ab6b9fa094df").unchecked_into(),
		),
	];


	let citizens = vec![
		// F
		AccountId::from_ss58check("5CCi1rPi7cphC6iE9mWkYvbLf57b9N233nFG8hM5zjvYZpLi").unwrap(),
		// V
		AccountId::from_ss58check("5DwWxf1NzMpp4D3jv1KY176DwYRRkKDguprmMw4BjieCX2ZK").unwrap(),
		// N
		AccountId::from_ss58check("5GEUDCyZrzPy1A6Kn288pHZFDtVhfYWvYmU1iTUPMg6YSVTE").unwrap(),
		// Dorian
		AccountId::from_ss58check("5GGgzku3kHSnAjxk7HBNeYzghSLsQQQGGznZA7u3h6wZUseo").unwrap(),
		// M
		AccountId::from_ss58check("5HgUQWZ4HHmivA2kqcXb8TTQVjH11FRphsRj4BBEhBzwUbS8").unwrap(),
		// Citizen 1
		AccountId::from_ss58check("5G3uZjEpvNAQ6U2eUjnMb66B8g6d8wyB68x6CfkRPNcno8eR").unwrap(),
		// Web3_Test1
		AccountId::from_ss58check("5GjYePC6HKJGGnEzEZzSvimy6uctuMat4Kr2tjACtKyY9nhT").unwrap(),
		// Web3_Test2
		AccountId::from_ss58check("5EqhBxsfDdbddFxcdRPhDBx8V3N2QyQspV5FNfQeT8nFQtj8").unwrap(),
		// Web3_Test3
		AccountId::from_ss58check("5CkYuVwK6bRjjaqam76VkPG4xXb1TsmbSQzWrMwaFnQ1nu6z").unwrap(),
		// Kacper
		AccountId::from_ss58check("5CDpDTBeDdg2KtpgG9WGS92fN4HxpMrSpwtbS6xXke8qU8Xr").unwrap(),
	];

	let registrar_key = AccountId::from_ss58check("5G96noBmnpNgpsaVXMsEs7961NU1zUNqQractuCp5R1hKejm").unwrap();
	let root_key: AccountId = AccountId::from_ss58check("5GZXCJvjfniCCLmKiyqzXLdwgcSgiQNUtsuFVhrpvfjopShL").unwrap();

	let mut endowed_accounts: Vec<AccountId> = vec![root_key.clone(), registrar_key.clone(),];
	endowed_accounts.append(&mut citizens.clone());

	let technical_committee = vec![
		// F
		AccountId::from_ss58check("5CCi1rPi7cphC6iE9mWkYvbLf57b9N233nFG8hM5zjvYZpLi").unwrap(),
		// Dorian
		AccountId::from_ss58check("5GGgzku3kHSnAjxk7HBNeYzghSLsQQQGGznZA7u3h6wZUseo").unwrap(),
		// Kacper
		AccountId::from_ss58check("5CDpDTBeDdg2KtpgG9WGS92fN4HxpMrSpwtbS6xXke8qU8Xr").unwrap(),
	];

	testnet_genesis(
		initial_authorities,
		vec![],
		root_key,
		Some(endowed_accounts),
		Some(vec![]),
		registrar_key.into(),
		citizens.into_iter().map(|id| (id, 0, 0)).collect(),
		Some(technical_committee),
	)
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
	initial_citizens: Vec<(AccountId, Balance, Balance)>,
	technical_committee: Option<Vec<AccountId>>,
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
		array_bytes::hex_n_into_unchecked("061a7f0a43e35d16f330e64c1a4e5000db4ba064fc3630cc4a9e2027899a5a6f"), // F
		array_bytes::hex_n_into_unchecked("b86373a2dff0a7b5741fd7e1857de41353fca3b924f14eae5f4c70d69e949150"), // N
		array_bytes::hex_n_into_unchecked("ba14fb5a00f052330c9c09e0467bce1d7896edefe92851b893e777aade53f921"), // D
		array_bytes::hex_n_into_unchecked("f874b8c112a9bb565e0798d9b5dcfee0fdbd54dd0fcc865c1251a75bd3faee45"), // M
		array_bytes::hex_n_into_unchecked("52fd11392742ccf58bcff90c33ca15bdf4bd3416aabcd5d51a654c1f387b6d18"), // V
	];

	// rewrite, not to use for loop
	for ac in f_ac.iter() {
		if !endowed_accounts.contains(ac) {
			endowed_accounts.push(ac.clone());
		}
	}

	// endow all citizens.
	initial_citizens.iter().map(|x| &x.0)
		.for_each(|x| {
			if !endowed_accounts.contains(x) {
				endowed_accounts.push(x.clone())
			}
		});

	let technical_committee = technical_committee.unwrap_or(
		endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect());


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
			members: technical_committee,
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
	let total_llm = 6000 * GRAINS_IN_LLM;
	let locked_llm = 5000 * GRAINS_IN_LLM;
	testnet_genesis(
		vec![authority_keys_from_seed("Alice")],
		vec![],
		alice.clone(),
		None,
		None,
		Some(alice.clone()),
		vec![
			(alice, total_llm, locked_llm),
			(get_account_id_from_seed::<sr25519::Public>("Bob"), total_llm, locked_llm),
			(get_account_id_from_seed::<sr25519::Public>("Charlie"), total_llm, locked_llm),
			(AccountId::from_ss58check("5G3uZjEpvNAQ6U2eUjnMb66B8g6d8wyB68x6CfkRPNcno8eR").unwrap(), total_llm, locked_llm), // Citizen1
			(AccountId::from_ss58check("5GGgzku3kHSnAjxk7HBNeYzghSLsQQQGGznZA7u3h6wZUseo").unwrap(), total_llm, locked_llm), // Dorian
			(AccountId::from_ss58check("5GZXCJvjfniCCLmKiyqzXLdwgcSgiQNUtsuFVhrpvfjopShL").unwrap(), total_llm, locked_llm), // Laissez sudo
			(AccountId::from_ss58check("5GjYePC6HKJGGnEzEZzSvimy6uctuMat4Kr2tjACtKyY9nhT").unwrap(), total_llm, locked_llm), // Web3_Test1
			(AccountId::from_ss58check("5EqhBxsfDdbddFxcdRPhDBx8V3N2QyQspV5FNfQeT8nFQtj8").unwrap(), total_llm, locked_llm), // Web3_Test2
			(AccountId::from_ss58check("5CkYuVwK6bRjjaqam76VkPG4xXb1TsmbSQzWrMwaFnQ1nu6z").unwrap(), total_llm, locked_llm), // Web3_Test3
		],
		None,
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
		None,
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
			None,
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
