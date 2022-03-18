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

use grandpa_primitives::AuthorityId as GrandpaId;
use hex_literal::hex;
use node_runtime::{
	constants::currency::*, wasm_binary_unwrap, AuthorityDiscoveryConfig, BabeConfig,
	BalancesConfig, Block, CouncilConfig, DemocracyConfig, ElectionsConfig, GrandpaConfig,
	ImOnlineConfig, IndicesConfig, MaxNominations, SessionConfig, SessionKeys, SocietyConfig,
	StakerStatus, StakingConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_chain_spec::Properties;
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

pub use node_primitives::{AccountId, Balance, Signature};
pub use node_runtime::GenesisConfig;

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
		// (
		// 	// 5HjbsZixAwen8rQNNnkmJ5wLw1GToj1d5Rk1DkBPsf7HJD1W
		// 	hex!["fad7979de732fd3936afdf5cd36c3084a7e6499458878dc358504de9a28a971f"].into(),
		// 	// 5EeDcQXDx8oLBaKJ3mX85axwayosyJMGcnYKaxd7F8QZJYT2
		// 	hex!["7207b9f9c4b90119c3cce43da19ccf73e1760d4e6732a6bd7ac086ffe3ad025e"].into(),
		// 	// 5FrfnPEu3Vna4PGmGu3ePaHnNPgHvJhixGGTC8cbL9UH1xJx
		// 	hex!["a7c301f7e641c2281e641bf9c68d29bec671f4acb47cd91bc2a12ef7c0c114e4"]
		// 		.unchecked_into(),
		// 	// 5DCxhJuo6FetKAskDAtVUDmn1jmRcCAWqQb1ydMLGF1qoZUX
		// 	hex!["328825e2887bc2abb888d7aaf76f8e9cf1637dbd4f388d0b3d31e787c8bf962c"]
		// 		.unchecked_into(),
		// 	// 5HWJo81AUqpTg1YaT82VEYmRBaHZ3fQ22D9n7FRYdg9yJq14
		// 	hex!["f0b3eac069b83dfee7798f4b5ea3dc52aca48ebed1fda40b3916699f58833a36"]
		// 		.unchecked_into(),
		// 	// 5CqNc35wV4XiqUkABGHwysm3MaDeB3y7y6DUxA5LM5pzcGSm
		// 	hex!["22113bf86ea5e3c304ff6276fa443c65a51997895f2b63ef657cfcc2d2be5a7d"]
		// 		.unchecked_into(),
		// ),
		// // F keys
		// (
		// 	// 5FbybRPQ8J7kVm43N6iKZKrdqFGk725BkrKnTQRgkiaqajVW
		// 	hex!["9c8e48a2f6ff588533f53763b9254353ab851a719603a110c2c3c057891a5526"].into(),
		// 	// 5DLoPvM3vXBABKuXjv589mJPzVGx79xKE4kkK3p3JHmz836M
		// 	hex!["3882ce260a3b065cc220502c100d5a18a78b9b3bfc04f7b6cf14a9e7974ccb3a"].into(),
		// 	// 5HgWFpA8qspGUKHj326XfCSNVGzTNvvw2em9JbfwG3jYjVaK
		// 	hex!["f87af3161c8076a5ca40a2a89371567718ddea57dc6a26d3f61d109e7b69923f"]
		// 		.unchecked_into(),
		// 	// 5HpLsRiZHNkpntqMw8yLGmz2mNw1oY2tX3QKE7yaC7VV7WhW
		// 	hex!["fe7551174926dde66080548a5b9ef6553ea22a319dc86dc98b6faf0d29e3a301"]
		// 		.unchecked_into(),
		// 	// 5E4Pj2idwHLbJeoqmaGBP71pFhRc9AtvNXyfhb21KgQucJEK
		// 	hex!["583b6e203bb776417caf71f055e6f2fd31974dd64fb1e53b787da718701dbf22"]
		// 		.unchecked_into(),
		// 	// 5GEoFZMF3EYrL3TzcyFWeQdWuw9RcwQUxAWbu7troMExxJbG
		// 	hex!["b8a38c415dbca92f42d7ddb363a5482a624686452baa8426ae76084350a0cd1e"]
		// 		.unchecked_into(),
		// ),
		// // A keys
		// (
		// 	// 5DHkMCTkVcnYZrNnnhAfFFZq1nfHdC3eVtWL9cTwENHvfsFm
		// 	hex!["362ecfe9dc809c6f8dc1fbabe362359bfc10c9fd338878259b9597b5c98d4d5a"].into(),
		// 	// 5GRdANmvxgsf8PeTg32j6VVzyrTBFzSsEo7VATyQ2YpnoiW6
		// 	hex!["c0e54896103096ac60e30656014d8e70d927dacec6443974797dad7cf7df722d"].into(),
		// 	// 5F92DdJq8qnQNA5dzHqHa8n3LwtoHuKgg1ZXop9jmC8vKEte
		// 	hex!["87ff7de508defa1c9bee77eafd1095d4d70fe46bae815c7f9c41158a0ed9c2b0"]
		// 		.unchecked_into(),
		// 	// 5Egoqq7aNFY1kYmnN6ACVViVPiudEtP1CuSStUk1j1y6dTEN
		// 	hex!["7401735abd35ac50775a15941f8c7134a087507c2b2006d2c7cf77d75c9a6b7d"]
		// 		.unchecked_into(),
		// 	// 5F1v9k87Yk81njddDfKSiBdpyew57X2Wftr9bDnWLukZjjfU
		// 	hex!["829459ce71553a4ec94983caba9e5c0a02057210c83cac90ad89066c5543683c"]
		// 		.unchecked_into(),
		// 	// 5DyrjunWJcdNQB3z9kgzy8EqNuBYUNiJBhXUyjAHKpjQQ3RU
		// 	hex!["54c624f4da9e43a87ab7ce59cb715805c1b48b0382c763f63a813b24b6da8461"]
		// 		.unchecked_into(),
		// ),
		];
	// D keys
	// (
	// 	// 5F6gSQYmagwfFjeuAvpvux3rx5cm93Y38xGMXQQCYM5oEifn
	// 	hex!["86366aadb31d744f1c59da7eb15ce5e23b69813aab5e8fc0cefdd3145b6a631d"].into(),
	// 	// 5FHGKZRsMaXFEgJdeshdm6VvyyCwWNuEHX3QBJDGGAXBT1kU
	// 	hex!["8e48ed75232a42185be6da6bf024b35ed1b9f8a63c5093eb14a26419c1401f16"].into(),
	// 	// 5Ger91spTSRhNn3cqkZiNXvyVjFuCmr3eypJ2vErJbfs39Hg
	// 	hex!["cafb28207144e3319b4b17bf9ba4ca14d05fd6a9f9501462fa3edeb306ba9994"]
	// 		.unchecked_into(),
	// 	// 5DXCgb56mc5FYKhGDftRskbZGjjk87ayEEpmLJCJSEa1wxST
	// 	hex!["4071a6cf35750837b25aa39e1b43865da4fe3def460643e55412a53c2d216160"]
	// 		.unchecked_into(),
	// 	// 5FvJaiEPwVNDuKCpQZXF7K9vN2TWw2nXL8LJuNrqeg7QLNTW
	// 	hex!["aa889e99baa17b65b0ef76dc1ddad73c35864f836f87345dff3b2b997322454d"]
	// 		.unchecked_into(),
	// 	// 5GEb1Gih5iGQULmfN8hJgZZAJyiDXwBEs2vf72aHB8fD3LqP
	// 	hex!["b87a52b506f4b1590443904ad9a0f63c1a826f3673abe4ac7b1b5303822be325"]
	// 		.unchecked_into(),
	// ),
	// ];

	let root_key: AccountId = hex![
		// 5FLK82VHiD4f6Zwnk4ajqs2X59DNG4bVBuXzbEkc66XmDZUc
		"909c17e16dcf8bdacf31d3fc2f7d8a39d8fe46c38ebeffe5d68c10cd18d53f21"
	]
	.into();

	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	testnet_genesis(initial_authorities, vec![], root_key, Some(endowed_accounts))
}

fn properties() -> sc_chain_spec::Properties {
	let mut p = Properties::new();
	p.insert("prefix".into(), 56.into());
	p.insert("network".into(), "liberland".into());
	p.insert("displayName".into(), "Liberland Menger".into());
	p.insert("tokenSymbol".into(), "LLM".into());
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
			members: endowed_accounts
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
			authorities: vec![
				(
					hex!["328825e2887bc2abb888d7aaf76f8e9cf1637dbd4f388d0b3d31e787c8bf962c"]
					.unchecked_into(),
					1
				),
				(
					// 5Egoqq7aNFY1kYmnN6ACVViVPiudEtP1CuSStUk1j1y6dTEN
					hex!["7401735abd35ac50775a15941f8c7134a087507c2b2006d2c7cf77d75c9a6b7d"]
					.unchecked_into(),
					1
				)
			],		
			epoch_config: Some(node_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		im_online: ImOnlineConfig { keys: vec![] },
		authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
		grandpa: GrandpaConfig { authorities: vec![
				(
					// 5FrfnPEu3Vna4PGmGu3ePaHnNPgHvJhixGGTC8cbL9UH1xJx
					hex!["a7c301f7e641c2281e641bf9c68d29bec671f4acb47cd91bc2a12ef7c0c114e4"]
						.unchecked_into(),
					1
				),
				(
					// 5F92DdJq8qnQNA5dzHqHa8n3LwtoHuKgg1ZXop9jmC8vKEte
					hex!["87ff7de508defa1c9bee77eafd1095d4d70fe46bae815c7f9c41158a0ed9c2b0"]
					.unchecked_into(),
					1
				),
			] 
		},
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
	}
}

fn development_config_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![authority_keys_from_seed("Alice")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
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
				new_full_base(config, |_, _| ())?;
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
