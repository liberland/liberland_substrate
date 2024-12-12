
//! Autogenerated weights for pallet_llm
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-06-21, STEPS: `20`, REPEAT: `10`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `kacper-HP-ProBook-445-G7`, CPU: `AMD Ryzen 7 4700U with Radeon Graphics`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// target/release/substrate-node
// benchmark
// pallet
// --pallet=pallet_llm
// --wasm-execution=compiled
// --steps=20
// --repeat=10
// --output=substrate/frame/llm/src/weights.rs
// --extrinsic=*
// --template=./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_llm.
pub trait WeightInfo {
	fn politics_lock() -> Weight;
	fn politics_unlock() -> Weight;
	fn treasury_llm_transfer() -> Weight;
	fn treasury_llm_transfer_to_politipool() -> Weight;
	fn send_llm_to_politipool() -> Weight;
	fn send_llm() -> Weight;
	fn treasury_lld_transfer() -> Weight;
	fn remark(l: u32, ) -> Weight;
	fn force_transfer() -> Weight;
	fn set_courts(l: u32, ) -> Weight;
}

/// Weights for pallet_llm using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(210), added: 2685, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:2 w:2)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(134), added: 2609, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `LLM::LLMPolitics` (r:1 w:1)
	/// Proof: `LLM::LLMPolitics` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn politics_lock() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1584`
		//  Estimated: `6208`
		// Minimum execution time: 80_249_000 picoseconds.
		Weight::from_parts(81_297_000, 6208)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: `LLM::LLMPolitics` (r:1 w:1)
	/// Proof: `LLM::LLMPolitics` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LLM::Withdrawlock` (r:1 w:1)
	/// Proof: `LLM::Withdrawlock` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(210), added: 2685, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:2 w:2)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(134), added: 2609, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `LLM::WithdrawlockDuration` (r:1 w:0)
	/// Proof: `LLM::WithdrawlockDuration` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `LLM::ElectionlockDuration` (r:1 w:0)
	/// Proof: `LLM::ElectionlockDuration` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Democracy::VotingOf` (r:1 w:0)
	/// Proof: `Democracy::VotingOf` (`max_values`: None, `max_size`: Some(3803), added: 6278, mode: `MaxEncodedLen`)
	/// Storage: `LLM::Electionlock` (r:0 w:1)
	/// Proof: `LLM::Electionlock` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn politics_unlock() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1565`
		//  Estimated: `7268`
		// Minimum execution time: 98_618_000 picoseconds.
		Weight::from_parts(99_526_000, 7268)
			.saturating_add(T::DbWeight::get().reads(9_u64))
			.saturating_add(T::DbWeight::get().writes(7_u64))
	}
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(210), added: 2685, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:2 w:2)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(134), added: 2609, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn treasury_llm_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `878`
		//  Estimated: `6208`
		// Minimum execution time: 61_461_000 picoseconds.
		Weight::from_parts(62_370_000, 6208)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(210), added: 2685, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:3 w:3)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(134), added: 2609, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `LLM::LLMPolitics` (r:1 w:1)
	/// Proof: `LLM::LLMPolitics` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn treasury_llm_transfer_to_politipool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1415`
		//  Estimated: `8817`
		// Minimum execution time: 118_943_000 picoseconds.
		Weight::from_parts(123_971_000, 8817)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(6_u64))
	}
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(210), added: 2685, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:3 w:3)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(134), added: 2609, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `LLM::LLMPolitics` (r:1 w:1)
	/// Proof: `LLM::LLMPolitics` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn send_llm_to_politipool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1689`
		//  Estimated: `8817`
		// Minimum execution time: 131_444_000 picoseconds.
		Weight::from_parts(132_142_000, 8817)
			.saturating_add(T::DbWeight::get().reads(7_u64))
			.saturating_add(T::DbWeight::get().writes(7_u64))
	}
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(210), added: 2685, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:2 w:2)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(134), added: 2609, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn send_llm() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1151`
		//  Estimated: `6208`
		// Minimum execution time: 73_544_000 picoseconds.
		Weight::from_parts(74_523_000, 6208)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn treasury_lld_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `312`
		//  Estimated: `6196`
		// Minimum execution time: 64_814_000 picoseconds.
		Weight::from_parts(66_280_000, 6196)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// The range of component `l` is `[0, 64]`.
	fn remark(l: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 10_476_000 picoseconds.
		Weight::from_parts(11_854_364, 0)
			// Standard Error: 1_546
			.saturating_add(Weight::from_parts(3_034, 0).saturating_mul(l.into()))
	}
	/// Storage: `LLM::Courts` (r:1 w:0)
	/// Proof: `LLM::Courts` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `LLM::LLMPolitics` (r:1 w:1)
	/// Proof: `LLM::LLMPolitics` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(210), added: 2685, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:2 w:2)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(134), added: 2609, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn force_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1431`
		//  Estimated: `6208`
		// Minimum execution time: 83_322_000 picoseconds.
		Weight::from_parts(84_929_000, 6208)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: `LLM::Courts` (r:0 w:1)
	/// Proof: `LLM::Courts` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `l` is `[1, 2]`.
	fn set_courts(l: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 6_076_000 picoseconds.
		Weight::from_parts(6_768_812, 0)
			// Standard Error: 73_598
			.saturating_add(Weight::from_parts(241_677, 0).saturating_mul(l.into()))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(210), added: 2685, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:2 w:2)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(134), added: 2609, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `LLM::LLMPolitics` (r:1 w:1)
	/// Proof: `LLM::LLMPolitics` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn politics_lock() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1584`
		//  Estimated: `6208`
		// Minimum execution time: 80_249_000 picoseconds.
		Weight::from_parts(81_297_000, 6208)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(5_u64))
	}
	/// Storage: `LLM::LLMPolitics` (r:1 w:1)
	/// Proof: `LLM::LLMPolitics` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LLM::Withdrawlock` (r:1 w:1)
	/// Proof: `LLM::Withdrawlock` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(210), added: 2685, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:2 w:2)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(134), added: 2609, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `LLM::WithdrawlockDuration` (r:1 w:0)
	/// Proof: `LLM::WithdrawlockDuration` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `LLM::ElectionlockDuration` (r:1 w:0)
	/// Proof: `LLM::ElectionlockDuration` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Democracy::VotingOf` (r:1 w:0)
	/// Proof: `Democracy::VotingOf` (`max_values`: None, `max_size`: Some(3803), added: 6278, mode: `MaxEncodedLen`)
	/// Storage: `LLM::Electionlock` (r:0 w:1)
	/// Proof: `LLM::Electionlock` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn politics_unlock() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1565`
		//  Estimated: `7268`
		// Minimum execution time: 98_618_000 picoseconds.
		Weight::from_parts(99_526_000, 7268)
			.saturating_add(RocksDbWeight::get().reads(9_u64))
			.saturating_add(RocksDbWeight::get().writes(7_u64))
	}
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(210), added: 2685, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:2 w:2)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(134), added: 2609, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn treasury_llm_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `878`
		//  Estimated: `6208`
		// Minimum execution time: 61_461_000 picoseconds.
		Weight::from_parts(62_370_000, 6208)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(210), added: 2685, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:3 w:3)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(134), added: 2609, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `LLM::LLMPolitics` (r:1 w:1)
	/// Proof: `LLM::LLMPolitics` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn treasury_llm_transfer_to_politipool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1415`
		//  Estimated: `8817`
		// Minimum execution time: 118_943_000 picoseconds.
		Weight::from_parts(123_971_000, 8817)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(6_u64))
	}
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(210), added: 2685, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:3 w:3)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(134), added: 2609, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `LLM::LLMPolitics` (r:1 w:1)
	/// Proof: `LLM::LLMPolitics` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn send_llm_to_politipool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1689`
		//  Estimated: `8817`
		// Minimum execution time: 131_444_000 picoseconds.
		Weight::from_parts(132_142_000, 8817)
			.saturating_add(RocksDbWeight::get().reads(7_u64))
			.saturating_add(RocksDbWeight::get().writes(7_u64))
	}
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(210), added: 2685, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:2 w:2)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(134), added: 2609, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn send_llm() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1151`
		//  Estimated: `6208`
		// Minimum execution time: 73_544_000 picoseconds.
		Weight::from_parts(74_523_000, 6208)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(5_u64))
	}
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn treasury_lld_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `312`
		//  Estimated: `6196`
		// Minimum execution time: 64_814_000 picoseconds.
		Weight::from_parts(66_280_000, 6196)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// The range of component `l` is `[0, 64]`.
	fn remark(l: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 10_476_000 picoseconds.
		Weight::from_parts(11_854_364, 0)
			// Standard Error: 1_546
			.saturating_add(Weight::from_parts(3_034, 0).saturating_mul(l.into()))
	}
	/// Storage: `LLM::Courts` (r:1 w:0)
	/// Proof: `LLM::Courts` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `LLM::LLMPolitics` (r:1 w:1)
	/// Proof: `LLM::LLMPolitics` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(210), added: 2685, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:2 w:2)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(134), added: 2609, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn force_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1431`
		//  Estimated: `6208`
		// Minimum execution time: 83_322_000 picoseconds.
		Weight::from_parts(84_929_000, 6208)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(5_u64))
	}
	/// Storage: `LLM::Courts` (r:0 w:1)
	/// Proof: `LLM::Courts` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `l` is `[1, 2]`.
	fn set_courts(l: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 6_076_000 picoseconds.
		Weight::from_parts(6_768_812, 0)
			// Standard Error: 73_598
			.saturating_add(Weight::from_parts(241_677, 0).saturating_mul(l.into()))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}
