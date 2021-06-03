
//! Autogenerated weights for frame_system
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-05-08, STEPS: `[50, ]`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("/tmp/artemis-benchmark-bNy/spec.json"), DB CACHE: 128

// Executed Command:
// target/release/artemis
// benchmark
// --chain
// /tmp/artemis-benchmark-bNy/spec.json
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// frame_system
// --extrinsic
// *
// --repeat
// 20
// --steps
// 50
// --output
// runtime/rococo/src/weights/frame_system_weights.rs


#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for frame_system.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> frame_system::WeightInfo for WeightInfo<T> {
	fn remark(_b: u32, ) -> Weight {
		(1_421_000 as Weight)
	}
	fn remark_with_event(b: u32, ) -> Weight {
		(21_068_000 as Weight)
			// Standard Error: 0
			.saturating_add((1_000 as Weight).saturating_mul(b as Weight))
	}
	fn set_heap_pages() -> Weight {
		(2_120_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_changes_trie_config() -> Weight {
		(9_051_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn set_storage(i: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 0
			.saturating_add((839_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(i as Weight)))
	}
	fn kill_storage(i: u32, ) -> Weight {
		(6_255_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((638_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(i as Weight)))
	}
	fn kill_prefix(p: u32, ) -> Weight {
		(2_310_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((1_211_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(p as Weight)))
	}
}
