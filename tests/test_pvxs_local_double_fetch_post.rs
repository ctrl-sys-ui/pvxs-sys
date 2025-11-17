use epics_pvxs_sys::{Server, SharedPV, NTScalarMetadataBuilder};

#[test]
fn test_pv_local_double_fetch_post() {
    // This test creates a local pv (loc:double) on a server and gets 
    // and sets the value on server side.
    let initial_value = 3.14159;
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    let mut srv_pv_loc_double: SharedPV = loc_srv.create_pv_double("loc:double", initial_value, NTScalarMetadataBuilder::new())
        .expect("Failed to create pv:double");

    // Do a server side fetch to verify initial value
    match srv_pv_loc_double.fetch() {
        Ok(value) => assert!((value.get_field_double("value").unwrap() - initial_value).abs() < 1e-6),
        Err(e) => panic!("Failed to fetch value: {:?}", e),
    }

    // Post an int32 to simulate value conversion
    match srv_pv_loc_double.post_int32(42) {
        Ok(_) => (),
        Err(e) => panic!("Failed to post int32 value: {:?}", e),
    }

    // Fetch again to verify the converted value
    match srv_pv_loc_double.fetch() {
        Ok(value) => assert!((value.get_field_double("value").unwrap() - 42.0).abs() < 1e-6),
        Err(e) => panic!("Failed to fetch value: {:?}", e),
    }

    // Post a string to simulate value being invalid... negative test
    match srv_pv_loc_double.post_string("not_a_number") {
        Ok(_) => panic!("Expected error when posting invalid string to double pv, but got Ok"),
        Err(_) => assert!(true), // Expected error
    }
    
    // Now set a new value and do a server side post
    let new_value = 2.71828;
    match srv_pv_loc_double.post_double(new_value) {
        Ok(_) => (),
        Err(e) => panic!("Failed to post new value: {:?}", e),
    } 
    
    // Fetch again to verify the new value
    match srv_pv_loc_double.fetch() {
        Ok(value) => assert!((value.get_field_double("value").unwrap() - new_value).abs() < 1e-6),
        Err(e) => panic!("Failed to fetch value: {:?}", e),
    }
}

#[test]
fn test_pv_local_double_fetch_post_with_error_propagation() -> Result<(), Box<dyn std::error::Error>> {
    let initial_value = 123.456;
    // This test verifies that errors in get/set operations are properly propagated.
    let loc_srv = Server::create_isolated()?;

    let mut srv_pv_loc_double: SharedPV = loc_srv.create_pv_double("loc:double", initial_value, NTScalarMetadataBuilder::new())?;

    // Intentionally cause an error by trying to post an invalid value
    match srv_pv_loc_double.post_string("invalid_double") {
        Ok(_) => panic!("Expected error when posting invalid value, but got Ok"),
        Err(_) => assert!(true), // Expected error
    }

    // Verify that fetching still works after the error
    let fetched_value = srv_pv_loc_double.fetch()?;
    assert!((fetched_value.get_field_double("value")? - initial_value).abs() < 1e-6);

    // Now post a valid value and verify
    let new_value = 987.654;
    srv_pv_loc_double.post_double(new_value)?;
    let fetched_value = srv_pv_loc_double.fetch()?;
    assert!((fetched_value.get_field_double("value")? - new_value).abs() < 1e-6);

    Ok(())
}

#[test]
fn test_pv_local_double_special_values() {
    // Test handling of special floating point values
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    let mut srv_pv_loc_double: SharedPV = loc_srv.create_pv_double("loc:double", 0.0, NTScalarMetadataBuilder::new())
        .expect("Failed to create pv:double");

    // Test positive infinity
    match srv_pv_loc_double.post_double(f64::INFINITY) {
        Ok(_) => {
            let value = srv_pv_loc_double.fetch().unwrap();
            assert!(value.get_field_double("value").unwrap().is_infinite());
        },
        Err(e) => println!("Server may not support infinity: {:?}", e),
    }

    // Test negative infinity
    match srv_pv_loc_double.post_double(f64::NEG_INFINITY) {
        Ok(_) => {
            let value = srv_pv_loc_double.fetch().unwrap();
            assert!(value.get_field_double("value").unwrap().is_infinite());
        },
        Err(e) => println!("Server may not support negative infinity: {:?}", e),
    }

    // Test NaN (may not be supported by EPICS)
    match srv_pv_loc_double.post_double(f64::NAN) {
        Ok(_) => {
            let _value = srv_pv_loc_double.fetch().unwrap();
            // Note: NaN comparisons always return false, so we can't use assert_eq
            println!("NaN value posted successfully");
        },
        Err(e) => println!("Server may not support NaN: {:?}", e),
    }

    // Test very large and very small numbers
    srv_pv_loc_double.post_double(f64::MAX).expect("Failed to post max value");
    let value = srv_pv_loc_double.fetch().unwrap();
    assert_eq!(value.get_field_double("value").unwrap(), f64::MAX);

    srv_pv_loc_double.post_double(f64::MIN).expect("Failed to post min value");
    let value = srv_pv_loc_double.fetch().unwrap();
    assert_eq!(value.get_field_double("value").unwrap(), f64::MIN);
}