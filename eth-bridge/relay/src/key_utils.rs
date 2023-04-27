use crate::{cmdline, settings, Result};
use ethers::{
	core::{rand::thread_rng, utils::to_checksum},
	prelude::LocalWallet,
	signers::Signer,
};
use sp_core::{sr25519::Pair as SubstratePair, Pair};
use std::path::Path;
use subxt::{tx::PairSigner, SubstrateConfig};

fn _print_keys(keys: &settings::Keys) -> Result<()> {
	let eth_wallet: LocalWallet = keys.ethereum_private_key.parse()?;
	let address = to_checksum(&eth_wallet.address(), None);
	println!("  Ethereum address:     \t{}", address);
	println!("  Ethereum private key: \t{}", keys.ethereum_private_key);
	let (pair, _) = SubstratePair::from_string_with_seed(&keys.substrate_secret_seed, None)?;
	let sub_signer: PairSigner<SubstrateConfig, SubstratePair> = PairSigner::new(pair);
	let address = format!("{}", sub_signer.account_id());
	println!("  Liberland address:    \t{}", address);
	println!("  Liberland secret seed:\t{}", keys.substrate_secret_seed);
	Ok(())
}

pub fn print(config: &Path) -> Result<()> {
	let config = settings::parse(config)?;
	for relay_keys in config.relays.iter() {
		println!("Relay:");
		_print_keys(&relay_keys.keys)?;
	}
	for watcher_keys in config.watchers.iter() {
		println!("Watcher:");
		_print_keys(watcher_keys)?;
	}
	Ok(())
}

fn gen_keys_ethereum() -> Result<()> {
	let wallet = LocalWallet::new(&mut thread_rng());
	let address = to_checksum(&wallet.address(), None);
	println!("Ethereum address:     \t{}", address);
	let signer = wallet.signer();
	println!("Ethereum private key: \t0x{}", hex::encode(signer.to_bytes()));
	Ok(())
}

fn gen_keys_liberland() -> Result<()> {
	let (pair, seed) = SubstratePair::generate();
	let sub_signer: PairSigner<SubstrateConfig, SubstratePair> = PairSigner::new(pair);
	let address = format!("{}", sub_signer.account_id());
	println!("Liberland address:    \t{}", address);
	println!("Liberland secret seed:\t0x{}", hex::encode(seed));
	Ok(())
}

pub fn gen_keys(key_type: cmdline::KeyType) -> Result<()> {
	match key_type {
		cmdline::KeyType::Ethereum => gen_keys_ethereum(),
		cmdline::KeyType::Liberland => gen_keys_liberland(),
	}
}
