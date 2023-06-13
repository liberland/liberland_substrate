mod bridge_abi;
mod cmdline;
mod daemon;
mod db;
mod key_utils;
mod liberland_api;
mod relay;
mod settings;
mod sync_managers;
mod tx_managers;
mod types;
mod utils;
mod watcher;
mod test;
pub use liberland_api::api as liberland;
use tracing_subscriber::{filter::EnvFilter, FmtSubscriber};

pub use eyre::{bail, ensure, eyre, Result};

fn level_filter(v: clap_verbosity_flag::LevelFilter) -> tracing::level_filters::LevelFilter {
	use clap_verbosity_flag::LevelFilter::*;
	use tracing::level_filters::LevelFilter;
	match v {
		Off => LevelFilter::OFF,
		Error => LevelFilter::ERROR,
		Warn => LevelFilter::WARN,
		Info => LevelFilter::INFO,
		Debug => LevelFilter::DEBUG,
		Trace => LevelFilter::TRACE,
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	let cli = cmdline::parse();

	let env_filter = EnvFilter::try_from_default_env()
		.unwrap_or_else(|_| EnvFilter::new("subxt=info,ethers=info,jsonrpsee=info"))
		.add_directive(level_filter(cli.verbose.log_level_filter()).into());

	let subscriber = FmtSubscriber::builder().with_env_filter(env_filter).finish();
	tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

	use cmdline::Commands::*;
	match cli.command {
		Init => settings::interactive_init(&cli.config),
		Keys => key_utils::print(&cli.config),
		GenKeys { key_type } => key_utils::gen_keys(key_type),
		Run => daemon::run(cli).await,
	}
}
