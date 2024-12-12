// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
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
	constants::currency::*, constants::llm::*, wasm_binary_unwrap,
	BabeConfig, BalancesConfig, Block, CouncilConfig,
	DemocracyConfig, ElectionsConfig, ImOnlineConfig,
	MaxNominations, SessionConfig, SessionKeys,
	StakerStatus, StakingConfig, SudoConfig, SystemConfig,
	TechnicalCommitteeConfig, LiberlandInitializerConfig,
	CompanyRegistryOfficePalletId, CompanyRegistryOfficeConfig,
	LandRegistryOfficeConfig, IdentityOfficeConfig, CompanyRegistryConfig,
	IdentityOfficePalletId, AssetRegistryOfficeConfig,
	LandRegistryOfficePalletId, AssetRegistryOfficePalletId,
	MetaverseLandRegistryOfficeConfig, MetaverseLandRegistryOfficePalletId,
	SenateConfig, MinistryOfFinanceOfficeConfig,
	impls::{RegistryCallFilter, IdentityCallFilter, NftsCallFilter},
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
	traits::{IdentifyAccount, Verify, AccountIdConversion},
	Perbill,
};

pub use kitchensink_runtime::RuntimeGenesisConfig;
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
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig, Extensions>;

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

/// Mainnet config.
pub fn mainnet_config() -> ChainSpec {
	ChainSpec::from_json_bytes(&include_bytes!("../../../../specs/mainnet.raw.json")[..]).expect("Broken mainnet chain spec")
}

/// Bastiat testnet config.
pub fn bastiat_testnet_config() -> ChainSpec {
	ChainSpec::from_json_bytes(&include_bytes!("../../../../specs/bastiat.raw.json")[..]).expect("Broken bastiat chain spec")
}

fn staging_testnet_config_genesis() -> RuntimeGenesisConfig {
	#[rustfmt::skip]
	let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = vec![
		// Liberland Node 1
		(
			AccountId::from_ss58check("5DLfiq37tePrZoaVNJDPFyDkRa2Lbr7faPMre2omsoNytoq4").unwrap(),
			AccountId::from_ss58check("5FyJBpWan9YzAyjwEzKcns4SJYrcJcAb3PKRB7rb8cymgryX").unwrap(),
			array_bytes::hex2array_unchecked("04fd9f3ff2040a822c4ce0275431f9ee5f4c9e5e781742116fa8546c1e0bdb7b").unchecked_into(),
			array_bytes::hex2array_unchecked("d0b72f9ee9b0eeb67450e0a8d71d00e3234ee3b195904e767b60e7a6f589dd55").unchecked_into(),
			array_bytes::hex2array_unchecked("1a6296323683419f4178e64b68476510f2b212261d5c78e28f00ee5e56a42130").unchecked_into(),
			array_bytes::hex2array_unchecked("2062cfe4b566703a1aca8777a40debae985eb0f2dc91dd9a4973c72509f09113").unchecked_into(),
		),
		// Liberland Node 2
		(
			AccountId::from_ss58check("5CiYBzVkYAJKZ9oa38hnpFuLSiiLxeYyfTb6dzReS1YNaoMy").unwrap(),
			AccountId::from_ss58check("5Df7LyLkNq8BymLP22G7Z696kxao1bMqYLMnGKmPZKqZhrbh").unwrap(),
			array_bytes::hex2array_unchecked("ab73ab7de09146f1af36151e2124076eee1ba327aafac0a995c0bc87d14d1407").unchecked_into(),
			array_bytes::hex2array_unchecked("a8bc8e6b4751db3f3555e2a5e18fdcf71444123460726f1c407645c3ebd0ab4c").unchecked_into(),
			array_bytes::hex2array_unchecked("cccd35bc9fbd981e82ab6217d13fdc361e01be17c7fa6747b479e293aad1d661").unchecked_into(),
			array_bytes::hex2array_unchecked("9af294f1fff0fffa37a0b972edb027a4115ae7567ceeb1da2174bb900a41fc15").unchecked_into(),
		),
		// Liberland Node 3
		(
			AccountId::from_ss58check("5FnpJZSHMrCCTwukFsbEHVHsVYo5vXGSHnYvhahhtew2jDJL").unwrap(),
			AccountId::from_ss58check("5CLUTtAS3w6zLsj7ffZSb7stKKczVUJXHztstmRq1aUSMzHT").unwrap(),
			array_bytes::hex2array_unchecked("054252cb5db71765eac20eed2f40797af88996e88f9168fe136cf78a782af651").unchecked_into(),
			array_bytes::hex2array_unchecked("d08f9c727fe6d7e3fc8132fd021cd673e350a82ef4eda9843bf1d19e038a3d53").unchecked_into(),
			array_bytes::hex2array_unchecked("0ef088bf4da2148f2d119376f31a36f7f60958cecf46f67e8763a57c7f8a3f3b").unchecked_into(),
			array_bytes::hex2array_unchecked("d0f97065a403e04429f9bdefed008b201c171b548f3a166e9aad9f139205076b").unchecked_into(),
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

	let min_citizenship_llm = 5000 * GRAINS_IN_LLM;
	let mut citizens_with_balance: Vec<(AccountId, Balance, Balance)> = citizens.iter().map(|id| (id.clone(), 0, 0)).collect();
	citizens_with_balance.extend(vec![
		// Nodes 1-3
		(AccountId::from_ss58check("5FyJBpWan9YzAyjwEzKcns4SJYrcJcAb3PKRB7rb8cymgryX").unwrap(), min_citizenship_llm, min_citizenship_llm),
		(AccountId::from_ss58check("5Df7LyLkNq8BymLP22G7Z696kxao1bMqYLMnGKmPZKqZhrbh").unwrap(), min_citizenship_llm, min_citizenship_llm),
		(AccountId::from_ss58check("5CLUTtAS3w6zLsj7ffZSb7stKKczVUJXHztstmRq1aUSMzHT").unwrap(), min_citizenship_llm, min_citizenship_llm)
	]);

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
		citizens_with_balance,
		Some(technical_committee),
		None,
		vec![],
	)
}

fn properties() -> sc_chain_spec::Properties {
	let mut p = Properties::new();
	p.insert("prefix".into(), 56.into());
	p.insert("network".into(), "liberland".into());
	p.insert("displayName".into(), "Liberland PowellGoHome".into());
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
		"PowellGoHome",
		"powell_go_home",
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

/// Helper function to generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed.
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

/// Helper function to create RuntimeGenesisConfig for testing.
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
	initial_citizens: Vec<(AccountId, Balance, Balance)>,
	technical_committee: Option<Vec<AccountId>>,
	offices_admin: Option<AccountId>,
	offices_clerks: Vec<AccountId>,
) -> RuntimeGenesisConfig {
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
			LandRegistryOfficePalletId::get().into_account_truncating(),
			MetaverseLandRegistryOfficePalletId::get().into_account_truncating(),
			AssetRegistryOfficePalletId::get().into_account_truncating(),
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
		.map(|x| (x.0.clone(), x.0.clone(), STASH, StakerStatus::Validator))
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
	const INITIAL_STAKE: Balance = 5000 * GRAINS_IN_LLM;


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

	let identity_clerks = offices_clerks.iter().map(|acc| (acc.clone(), IdentityCallFilter::Judgement)).collect();
	let registry_clerks = offices_clerks.iter().map(|acc| (acc.clone(), RegistryCallFilter::RegisterOnly)).collect();
	let nfts_clerks: Vec<(AccountId, NftsCallFilter)> = offices_clerks.iter().map(|acc| (acc.clone(), NftsCallFilter::ManageItems)).collect();

	RuntimeGenesisConfig {
		system: SystemConfig { code: wasm_binary_unwrap().to_vec(), ..Default::default() },
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect(),
		},
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
			citizenship_required: false,
			..Default::default()
		},
		democracy: DemocracyConfig::default(),
		elections: ElectionsConfig {
			members: council_group
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.map(|member| (member, INITIAL_STAKE))
				.collect(),
		},
		council: CouncilConfig::default(),
		senate: SenateConfig::default(),
		technical_committee: TechnicalCommitteeConfig {
			members: technical_committee,
			phantom: Default::default(),
		},
		sudo: SudoConfig { key: Some(root_key.clone()) },
		babe: BabeConfig {
			epoch_config: Some(kitchensink_runtime::BABE_GENESIS_EPOCH_CONFIG),
			..Default::default()
		},
		im_online: ImOnlineConfig { keys: vec![] },
		authority_discovery: Default::default(),
		grandpa: Default::default(),
		technical_membership: Default::default(),
		treasury: Default::default(),
		assets: pallet_assets::GenesisConfig {
			// This asset is used by the NIS pallet as counterpart currency.
			assets: vec![(9, get_account_id_from_seed::<sr25519::Public>("Alice"), true, 1)],
			..Default::default()
		},
		pool_assets: Default::default(),
		transaction_storage: Default::default(),
		transaction_payment: Default::default(),
		llm: Default::default(),
		liberland_initializer: LiberlandInitializerConfig {
			citizenship_registrar: Some(IdentityOfficePalletId::get().into_account_truncating()),
			initial_citizens,
			land_registrar: Some(LandRegistryOfficePalletId::get().into_account_truncating()),
			metaverse_land_registrar: Some(MetaverseLandRegistryOfficePalletId::get().into_account_truncating()),
			asset_registrar: Some(AssetRegistryOfficePalletId::get().into_account_truncating()),
		},
		company_registry: CompanyRegistryConfig {
			registries: vec![
				CompanyRegistryOfficePalletId::get().into_account_truncating()
			].try_into().unwrap(),
			entities: vec![],
		},
		identity_office: IdentityOfficeConfig {
			admin: offices_admin.clone(),
			clerks: identity_clerks,
		},
		company_registry_office: CompanyRegistryOfficeConfig {
			admin: offices_admin.clone(),
			clerks: registry_clerks,
		},
		land_registry_office: LandRegistryOfficeConfig {
			admin: offices_admin.clone(),
			clerks: nfts_clerks.clone(),
		},
		metaverse_land_registry_office: MetaverseLandRegistryOfficeConfig {
			admin: offices_admin.clone(),
			clerks: nfts_clerks.clone(),
		},
		ministry_of_finance_office: MinistryOfFinanceOfficeConfig {
			admin: offices_admin.clone(),
			clerks: vec![],
		},
		asset_registry_office: AssetRegistryOfficeConfig {
			admin: offices_admin,
			clerks: nfts_clerks,
		},
		substrate_bridge_outbound_channel: Default::default(),
		sora_bridge_app: Default::default(),
	}
}

fn development_config_genesis() -> RuntimeGenesisConfig {
	let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
	let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
	let total_llm = 6000 * GRAINS_IN_LLM;
	let locked_llm = 5000 * GRAINS_IN_LLM;
	testnet_genesis(
		vec![authority_keys_from_seed("Alice")],
		vec![],
		alice.clone(),
		None,
		None,
		vec![
			(alice.clone(), total_llm, locked_llm),
			(bob.clone(), total_llm, locked_llm),
			(get_account_id_from_seed::<sr25519::Public>("Charlie"), total_llm, locked_llm),
			(AccountId::from_ss58check("5G3uZjEpvNAQ6U2eUjnMb66B8g6d8wyB68x6CfkRPNcno8eR").unwrap(), total_llm, locked_llm), // Citizen1
			(AccountId::from_ss58check("5GGgzku3kHSnAjxk7HBNeYzghSLsQQQGGznZA7u3h6wZUseo").unwrap(), total_llm, locked_llm), // Dorian
			(AccountId::from_ss58check("5GZXCJvjfniCCLmKiyqzXLdwgcSgiQNUtsuFVhrpvfjopShL").unwrap(), total_llm, locked_llm), // Laissez sudo
			(AccountId::from_ss58check("5GjYePC6HKJGGnEzEZzSvimy6uctuMat4Kr2tjACtKyY9nhT").unwrap(), total_llm, locked_llm), // Web3_Test1
			(AccountId::from_ss58check("5EqhBxsfDdbddFxcdRPhDBx8V3N2QyQspV5FNfQeT8nFQtj8").unwrap(), total_llm, locked_llm), // Web3_Test2
			(AccountId::from_ss58check("5CkYuVwK6bRjjaqam76VkPG4xXb1TsmbSQzWrMwaFnQ1nu6z").unwrap(), total_llm, locked_llm), // Web3_Test3
		],
		None,
		Some(alice.clone()),
		vec![bob.clone()],
	)
}

/// Development config (single validator Alice).
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

fn local_testnet_genesis() -> RuntimeGenesisConfig {
	let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
	let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
	let total_llm = 6000 * GRAINS_IN_LLM;
	let locked_llm = 5000 * GRAINS_IN_LLM;
	testnet_genesis(
		vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob"), authority_keys_from_seed("Charlie")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		None,
		vec![
			(alice.clone(), total_llm, locked_llm),
			(bob.clone(), total_llm, locked_llm),
			(get_account_id_from_seed::<sr25519::Public>("Charlie"), total_llm, locked_llm),
		],
		None,
		Some(alice.clone()),
		vec![bob.clone()],
	)
}

/// Local testnet config (multivalidator Alice + Bob).
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

	fn local_testnet_genesis_instant_single() -> RuntimeGenesisConfig {
		testnet_genesis(
			vec![authority_keys_from_seed("Alice")],
			vec![],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			None,
			None,
			vec![],
			None,
			None,
			vec![],
		)
	}

	/// Local testnet config (single validator - Alice).
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

	/// Local testnet config (multivalidator Alice + Bob).
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
			let NewFullBase { task_manager, client, network, sync, transaction_pool, .. } =
				new_full_base(config, false, |_, _| ())?;
			Ok(sc_service_test::TestNetComponents::new(
				task_manager,
				client,
				network,
				sync,
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

	#[test]
	fn test_bastiat_test_net_chain_spec() {
		bastiat_testnet_config().build_storage().unwrap();
	}

	#[test]
	fn test_mainnet_chain_spec() {
		mainnet_config().build_storage().unwrap();
	}
}
