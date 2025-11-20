use epics_pvxs_sys::{Server, SharedPV, NTScalarMetadataBuilder};

#[test]
fn test_pv_local_fetch_post(){
    // This test creates a local pv (loc:int) on a server and gets 
    // and sets the value on server side.
    let initial_value = 100;
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    let mut srv_pv_loc_int: SharedPV = loc_srv.create_pv_int32("loc:int", initial_value, NTScalarMetadataBuilder::new())
        .expect("Failed to create pv:int");

    // Do a server side fetch to verify initial value
    match srv_pv_loc_int.fetch() {
        Ok(value) => assert!(value.get_field_int32("value").unwrap() == initial_value),
        Err(e) => panic!("Failed to fetch value: {:?}", e),
    }

    // Post a double to similate value being clipped
    match srv_pv_loc_int.post_double(3.14) {
        Ok(_) => (),
        Err(e) => panic!("Failed to post double value: {:?}", e),
    }

    // Fetch again to verify the clipped value
    match srv_pv_loc_int.fetch() {
        Ok(value) => assert!(value.get_field_int32("value").unwrap() == 3),
        Err(e) => panic!("Failed to fetch value: {:?}", e),
    }

    // Post a string to similate value being invalid... negative test
    match srv_pv_loc_int.post_string("invalid") {
        Ok(_) => panic!("Expected error when posting string to int pv, but got Ok"),
        Err(_) => assert!(true), // Expected error
    }
    
    // Now set a new value and do a server side post
    let new_value = 200;
    match srv_pv_loc_int.post_int32(new_value) {
        Ok(_) => (),
        Err(e) => panic!("Failed to post new value: {:?}", e),
    } 
    
    // Fetch again to verify the new value
    match srv_pv_loc_int.fetch() {
        Ok(value) => assert!(value.get_field_int32("value").unwrap() == new_value),
        Err(e) => panic!("Failed to fetch value: {:?}", e),
    }
}

#[test]
fn test_pv_local_fetch_post_with_error_propagation() -> Result<(), Box<dyn std::error::Error>> {
    let initial_value = 1234;
    // This test verifies that errors in get/set operations are properly propagated.
    let loc_srv = Server::create_isolated()?;

    let mut srv_pv_loc_int: SharedPV = loc_srv.create_pv_int32("loc:int", initial_value, NTScalarMetadataBuilder::new())?;

    // Intentionally cause an error by trying to post an invalid value
    match srv_pv_loc_int.post_string("invalid_value") {
        Ok(_) => panic!("Expected error when posting invalid value, but got Ok"),
        Err(_) => assert!(true), // Expected error
    }

    // Verify that fetching still works after the error
    let fetched_value = srv_pv_loc_int.fetch()?;
    assert_eq!(fetched_value.get_field_int32("value")?, initial_value);

    // Now post a valid value and verify
    let new_value = 5678;
    srv_pv_loc_int.post_int32(new_value)?;
    let fetched_value = srv_pv_loc_int.fetch()?;
    assert_eq!(fetched_value.get_field_int32("value")?, new_value);

    Ok(())
}