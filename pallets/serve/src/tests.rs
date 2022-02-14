use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test1() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(Serve::registered_server_collection(Origin::signed(1)));
		// collection_id: u32,
		// serve_types: ServeTypes,
		// serve_ways: ServeWays,
		// serve_state: ServeState,
		// serve_switch: bool,
		// serve_version: Vec<u8>,
		// serve_name: Vec<u8>,
		// serve_description: Vec<u8>,
		// serve_url: Vec<u8>,
		// serve_price: BalanceOf<T>,
		// server_limit_time: Option<TimeTypes>,
		// server_limit_times: Option<u32>
		assert_ok!(Serve::add_serve(
			Origin::signed(1),
			0,
			ServeTypes::RecordTimes,
			ServeWays::RESTFUL,
			ServeState::Dev,
			true,
			b"2".to_vec(),
			b"2".to_vec(),
			b"2".to_vec(),
			b"2".to_vec(),
			23,
			Some(TimeTypes::Day),
			Some(20),
		));
		assert_ok!(Serve::registered_use_server_certificate(Origin::signed(1),0,0,0));
		// Read pallet storage and assert an expected result.
		// assert_eq!(TemplateModule::something(), Some(42));
	});
}

