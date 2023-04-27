use ethers::prelude::Abigen;
use std::{env, path::Path};

fn main() {
	let out_dir = env::var_os("OUT_DIR").unwrap();
	let dest_path = Path::new(&out_dir).join("bridge_abi.rs");
	Abigen::new("BridgeABI", "Bridge.json")
		.unwrap()
		.generate()
		.unwrap()
		.write_to_file(dest_path)
		.unwrap();

	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=Bridge.json");
}
