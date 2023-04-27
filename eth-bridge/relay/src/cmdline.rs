use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
	#[clap(flatten)]
	pub verbose: Verbosity<InfoLevel>,

	#[arg(short, long, value_name = "FILE")]
	/// Path to the config file. Use init command to initialize a new one.
	pub config: PathBuf,

	#[command(subcommand)]
	pub command: Commands,
}

pub fn parse() -> Args {
	Args::parse()
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum KeyType {
	Ethereum,
	Liberland,
}

#[derive(Subcommand, Debug, Clone, Copy)]
pub enum Commands {
	/// Initialize new config file
	Init,
	/// Start daemon
	Run,
	/// Print private and public keys
	Keys,
	/// Generate new keys
	GenKeys {
		#[arg(short, long)]
		key_type: KeyType,
	},
}
