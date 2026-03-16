// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs_sys::{NTScalarMetadataBuilder, Server};

#[test]
fn test_pv_local_fetch_post() {
    // This test creates a local pv (loc:int) on a server and gets
    // and sets the value on server side.
    let initial_value = 100;
    let name = "loc:int";
    let loc_srv = Server::start_isolated().expect("Failed to create isolated server");

    loc_srv
        .create_pv_int32(name, initial_value, NTScalarMetadataBuilder::new())
        .expect("Failed to create pv:int");

    // Do a server side fetch to verify initial value
    match loc_srv.fetch_int32(name) {
        Ok(fetched) => assert!(fetched.value == initial_value),
        Err(e) => assert!(false, "Failed to fetch value: {:?}", e),
    }

    // Post a double to simulate type mismatch, negative test
    match loc_srv.post_double(name, 3.14) {
        Ok(_) => assert!(
            false,
            "Expected error when posting double to int pv, but got Ok"
        ),
        Err(_) => assert!(true), // Expected error
    }

    // Post a string to simulate type mismatch, negative test
    match loc_srv.post_string(name, "invalid") {
        Ok(_) => assert!(
            false,
            "Expected error when posting string to int pv, but got Ok"
        ),
        Err(_) => assert!(true), // Expected error
    }

    // Now set a new value and do a server side post
    let new_value = 200;
    match loc_srv.post_int32(name, new_value) {
        Ok(_) => (),
        Err(e) => assert!(false, "Failed to post new value: {:?}", e),
    }

    // Fetch again to verify the new value
    match loc_srv.fetch_int32(name) {
        Ok(fetched) => assert!(fetched.value == new_value),
        Err(e) => assert!(false, "Failed to fetch value: {:?}", e),
    }
}

#[test]
fn test_pv_local_fetch_post_with_error_propagation() -> Result<(), Box<dyn std::error::Error>> {
    let initial_value = 1234;
    // This test verifies that errors in get/set operations are properly propagated.
    let loc_srv = Server::start_isolated()?;
    let name = "loc:int";

    loc_srv.create_pv_int32(name, initial_value, NTScalarMetadataBuilder::new())?;

    // Intentionally cause an error by trying to post an invalid value
    match loc_srv.post_string(name, "invalid_value") {
        Ok(_) => assert!(
            false,
            "Expected error when posting invalid value, but got Ok"
        ),
        Err(_) => assert!(true), // Expected error
    }

    // Verify that fetching still works after the error
    let fetched = loc_srv.fetch_int32(name)?;
    assert_eq!(fetched.value, initial_value);

    // Now post a valid value and verify
    let new_value = 5678;
    loc_srv.post_int32(name, new_value)?;
    let fetched = loc_srv.fetch_int32(name)?;
    assert_eq!(fetched.value, new_value);

    Ok(())
}
