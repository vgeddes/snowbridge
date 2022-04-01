use crate::mock::*;
use crate::{FinalizedHeaders, FinalizedHeadersBySlot};
use frame_support::assert_ok;
use hex_literal::hex;

#[test]
fn it_syncs_from_an_initial_checkpoint() {
	let initial_sync = get_initial_sync();

	new_tester().execute_with(|| {
		assert_ok!(EthereumBeaconLightClient::initial_sync(
			Origin::signed(1),
			initial_sync.clone(),
		));

		assert!(<FinalizedHeaders<Test>>::contains_key(initial_sync.header.body_root));
		assert!(<FinalizedHeadersBySlot<Test>>::contains_key(initial_sync.header.slot));
	});
}

#[test]
fn it_updates_a_committee_period_sync_update() {
	let update = get_committee_sync_period_update();

	new_tester().execute_with(|| {
		assert_ok!(EthereumBeaconLightClient::sync_committee_period_update(
			Origin::signed(1),
			update,
		));
	});
}

#[test]
fn it_converts_to_binary() {
	let result = EthereumBeaconLightClient::convert_to_binary(vec![10, 33]);

	assert_eq!(result, vec![1, 0, 1, 0, 1, 0, 0, 0, 0, 1]);

	let result = EthereumBeaconLightClient::convert_to_binary(hex!("fffffffffffffffffffffffffffffffffffffffdfffffffffffffffffffffffffff7ffffffbfffffffffffffffefffffffffffbffffffffffbffffffffffffff").to_vec());

	assert_eq!(
		result,
		vec![
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		]
	);
}

#[test]
pub fn test_get_sync_committee_sum() {
	new_tester().execute_with(|| {
		assert_eq!(
			EthereumBeaconLightClient::get_sync_committee_sum(vec![0, 1, 0, 1, 1, 0, 1, 0, 1]),
			5
		);
	});
}

#[test]
pub fn test_compute_domain() {
	new_tester().execute_with(|| {
		let domain = EthereumBeaconLightClient::compute_domain(
			hex!("05000000").into(),
			hex!("00000001").into(),
			hex!("5dec7ae03261fde20d5b024dfabce8bac3276c9a4908e23d50ba8c9b50b0adff").into(),
		);

		assert_ok!(&domain);
        assert_eq!(
            domain.unwrap(),
            hex!("0500000046324489ceb6ada6d118eacdbe94f49b1fcb49d5481a685979670c7c").into()
        );
	});
}

#[test]
pub fn test_is_valid_merkle_proof() {
	new_tester().execute_with(|| {
		assert_eq!(
			EthereumBeaconLightClient::is_valid_merkle_branch(
				hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
				vec![
					hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
					hex!("5f6f02af29218292d21a69b64a794a7c0873b3e0f54611972863706e8cbdf371").into(),
					hex!("e7125ff9ab5a840c44bedb4731f440a405b44e15f2d1a89e27341b432fabe13d").into(),
					hex!("002c1fe5bc0bd62db6f299a582f2a80a6d5748ccc82e7ed843eaf0ae0739f74a").into(),
					hex!("d2dc4ba9fd4edff6716984136831e70a6b2e74fca27b8097a820cbbaa5a6e3c3").into(),
					hex!("91f77a19d8afa4a08e81164bb2e570ecd10477b3b65c305566a6d2be88510584").into(),
				],
				6,
				41,
				hex!("e46559327592741956f6beaa0f52e49625eb85dce037a0bd2eff333c743b287f").into()
			),
			true
		);
	});
}
