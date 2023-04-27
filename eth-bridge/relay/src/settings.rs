use crate::Result;
use config::Config;
pub use ethers::{
	core::types::Address as EthAddress,
	signers::LocalWallet as EthLocalWallet,
	utils::{format_units, parse_units, ParseUnits},
};
use eyre::WrapErr;
use inquire::{required, validator::Validation, Confirm, CustomType, Text};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519::Pair as SubstratePair, Pair};
use std::{
	fs::File,
	io::Write,
	path::{Path, PathBuf},
};
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
pub struct Keys {
	pub substrate_secret_seed: String,
	pub ethereum_private_key: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Relay {
	#[serde(flatten)]
	pub keys: Keys,

	pub minimum_balance: String,
	pub claim_rewards_threshold: String,
	pub withdraw_rewards_threshold: String,
	pub rewards_address: EthAddress,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Settings {
	pub db_path: PathBuf,
	pub substrate_ws_url: Url,
	pub ethereum_ws_url: Url,
	pub max_gas_price_in_gwei: String,
	pub relays: Vec<Relay>,
	pub watchers: Vec<Keys>,
	pub lld_bridge_contract: EthAddress,
	pub llm_bridge_contract: EthAddress,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			db_path: PathBuf::from("sqlite.db"),
			substrate_ws_url: "ws://localhost:9944".parse().unwrap(),
			ethereum_ws_url: "ws://localhost:8545".parse().unwrap(),
			relays: vec![],
			watchers: vec![],
			lld_bridge_contract: Default::default(),
			llm_bridge_contract: Default::default(),
			max_gas_price_in_gwei: "100".to_string(),
		}
	}
}

pub fn interactive_init(config: &Path) -> Result<()> {
	if config.exists() {
		println!("Config file {} already exists.", config.display());
		if !Confirm::new("Do you want to overwrite it?").with_default(false).prompt()? {
			return Ok(())
		}
	}

	let defaults = if let Ok(s) = parse(config) { s } else { Settings::default() };

	let validate_ws_url = |input: &Url| {
		if input.scheme() == "ws" || input.scheme() == "wss" {
			Ok(Validation::Valid)
		} else {
			Ok(Validation::Invalid("You must provide websocket address!".into()))
		}
	};

	let db_path = CustomType {
		message: "Database file path:",
		formatter: &|i: PathBuf| i.display().to_string(),
		default_value_formatter: &|i| i.display().to_string(),
		default: Some(defaults.db_path),
		validators: vec![],
		placeholder: None,
		error_message: "Please enter valid path.".into(),
		help_message: None,
		parser: &|i| Ok(PathBuf::from(i)),
		render_config: inquire::ui::RenderConfig::default(),
	}
	.prompt()?;

	let validate_substrate_key =
		|input: &str| match SubstratePair::from_string_with_seed(input, None) {
			Ok(_) => Ok(Validation::Valid),
			Err(e) => Ok(Validation::Invalid(e.to_string().into())),
		};

	let validate_ethereum_key = |input: &str| match input.parse::<EthLocalWallet>() {
		Ok(_) => Ok(Validation::Valid),
		Err(e) => Ok(Validation::Invalid(e.to_string().into())),
	};

	let substrate_ws_url = CustomType::new("WebSocket URL to your Substrate archive node:")
		.with_validator(validate_ws_url)
		.with_default(defaults.substrate_ws_url)
		.prompt()?;

	let ethereum_ws_url = CustomType::new("WebSocket URL to your Ethereum EL node: ")
		.with_validator(validate_ws_url)
		.with_default(defaults.ethereum_ws_url)
		.prompt()?;

	let mut llm_bridge_contract_prompt = CustomType::new("Contract address for LLM/LKN Bridge:");
	if defaults.llm_bridge_contract != Default::default() {
		llm_bridge_contract_prompt =
			llm_bridge_contract_prompt.with_default(defaults.llm_bridge_contract);
	}
	let llm_bridge_contract = llm_bridge_contract_prompt.prompt()?;

	let mut lld_bridge_contract_prompt = CustomType::new("Contract address for LLD/LDN Bridge:");
	if defaults.lld_bridge_contract != Default::default() {
		lld_bridge_contract_prompt =
			lld_bridge_contract_prompt.with_default(defaults.lld_bridge_contract);
	}
	let lld_bridge_contract = lld_bridge_contract_prompt.prompt()?;

	let relays = if Confirm::new("Are you running a relay?").with_default(true).prompt()? {
		let substrate_secret_seed = Text::new("Provide relay's substrate secret seed:")
			.with_validator(validate_substrate_key)
			.with_validator(required!())
			.prompt()?;
		let ethereum_private_key = Text::new("Provide relay's ethereum private key:")
			.with_validator(validate_ethereum_key)
			.with_validator(required!())
			.prompt()?;
		let rewards_address =
			CustomType::new("Ethereum address for withdrawing rewards:").prompt()?;

		vec![Relay {
			keys: Keys { substrate_secret_seed, ethereum_private_key },
			rewards_address,
			minimum_balance: "0.1".to_string(),
			claim_rewards_threshold: "0.1".to_string(),
			withdraw_rewards_threshold: "0.2".to_string(),
		}]
	} else {
		vec![]
	};

	let watchers = if Confirm::new("Are you running a watcher?").with_default(false).prompt()? {
		let substrate_secret_seed = Text::new("Provide watcher's substrate secret seed:")
			.with_validator(required!())
			.with_validator(validate_substrate_key)
			.prompt()?;
		let ethereum_private_key = Text::new("Provide watcher's ethereum private key:")
			.with_validator(required!())
			.with_validator(validate_ethereum_key)
			.prompt()?;
		vec![Keys { substrate_secret_seed, ethereum_private_key }]
	} else {
		vec![]
	};

	let conf = toml::to_string_pretty(&Settings {
		db_path,
		substrate_ws_url,
		ethereum_ws_url,
		llm_bridge_contract,
		lld_bridge_contract,
		relays,
		watchers,
		max_gas_price_in_gwei: "100".to_string(),
	})?;

	let mut file = File::create(config)?;
	file.write_all(conf.as_bytes())?;

	println!("Config written! You can now start the daemon.");

	Ok(())
}

pub fn parse(config: &Path) -> Result<Settings> {
	let s = Config::builder()
		.add_source(config::File::with_name(config.to_str().unwrap()))
		.add_source(config::Environment::with_prefix("LIBERLAND_RELAY"))
		.build()?;
	s.try_deserialize().wrap_err_with(|| "Failure reading configuration")
}
