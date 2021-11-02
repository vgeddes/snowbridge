use frame_support::weights::Weight;

pub trait WeightInfo {
	fn submit_base_weight() -> Weight;
	fn set_reward_fraction() -> Weight;
}

impl WeightInfo for () {
	fn submit_base_weight() -> Weight {
		0
	}

	fn set_reward_fraction() -> Weight {
		0
	}
}
