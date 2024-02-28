use sp_core::{Bytes, OpaqueMetadata};
use std::{path::PathBuf, str::FromStr};

fn main() {
	let metadata: Bytes =
		OpaqueMetadata::new(kitchensink_runtime::Runtime::metadata().into()).into();
	let out_dir = PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
		.unwrap()
		.join("src/bytes");
	std::fs::create_dir_all(&out_dir).unwrap();
	std::fs::write(out_dir.join("liberland_metadata.scale"), metadata.0).unwrap();
	let workspace_root = PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
		.unwrap()
		.parent()
		.unwrap()
		.parent()
		.unwrap()
		.to_str()
		.unwrap()
		.to_string();
	println!("cargo:rerun-if-changed={}/bin/node/runtime", workspace_root);
	println!("cargo:rerun-if-changed={}/frame", workspace_root);
	println!("cargo:rerun-if-changed={}/common", workspace_root);
}
