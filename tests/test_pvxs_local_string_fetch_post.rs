use epics_pvxs_sys::{Server, SharedPV};

#[test]
fn test_pv_local_string_fetch_post() {
    // This test creates a local pv (loc:string) on a server and gets 
    // and sets the value on server side.
    let initial_value = "Hello, EPICS!";
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    let mut srv_pv_loc_string: SharedPV = loc_srv.create_pv_string("loc:string", initial_value)
        .expect("Failed to create pv:string");

    // Do a server side fetch to verify initial value
    match srv_pv_loc_string.fetch() {
        Ok(value) => assert_eq!(value.get_field_string("value").unwrap(), initial_value),
        Err(e) => panic!("Failed to fetch value: {:?}", e),
    }

    // Post a double to simulate string conversion
    match srv_pv_loc_string.post_double(42.5) {
        Ok(_) => {
            // Some servers might convert number to string
            let value = srv_pv_loc_string.fetch().unwrap();
            let string_val = value.get_field_string("value").unwrap();
            println!("Double converted to string: '{}'", string_val);
            // The exact format depends on the server implementation
        },
        Err(_) => {
            // Some servers might reject numeric values for string PVs
            println!("Server rejected double value for string PV (expected behavior)");
        }
    }

    // Post an int32 to simulate string conversion  
    match srv_pv_loc_string.post_int32(123) {
        Ok(_) => {
            let value = srv_pv_loc_string.fetch().unwrap();
            let string_val = value.get_field_string("value").unwrap();
            println!("Int32 converted to string: '{}'", string_val);
        },
        Err(_) => {
            println!("Server rejected int32 value for string PV (expected behavior)");
        }
    }
    
    // Now set a new string value and do a server side post
    let new_value = "Updated string value";
    match srv_pv_loc_string.post_string(new_value) {
        Ok(_) => (),
        Err(e) => panic!("Failed to post new value: {:?}", e),
    } 
    
    // Fetch again to verify the new value
    match srv_pv_loc_string.fetch() {
        Ok(value) => assert_eq!(value.get_field_string("value").unwrap(), new_value),
        Err(e) => panic!("Failed to fetch value: {:?}", e),
    }
}

#[test]
fn test_pv_local_string_fetch_post_with_error_propagation() -> Result<(), Box<dyn std::error::Error>> {
    let initial_value = "Initial string";
    // This test verifies that errors in get/set operations are properly propagated.
    let loc_srv = Server::create_isolated()?;

    let mut srv_pv_loc_string: SharedPV = loc_srv.create_pv_string("loc:string", initial_value)?;

    // Verify initial value
    let fetched_value = srv_pv_loc_string.fetch()?;
    assert_eq!(fetched_value.get_field_string("value")?, initial_value);

    // Post a valid string value and verify
    let new_value = "New string value";
    srv_pv_loc_string.post_string(new_value)?;
    let fetched_value = srv_pv_loc_string.fetch()?;
    assert_eq!(fetched_value.get_field_string("value")?, new_value);

    Ok(())
}

#[test]
fn test_pv_local_string_special_characters() {
    // Test handling of special characters in strings
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    let mut srv_pv_loc_string: SharedPV = loc_srv.create_pv_string("loc:string", "")
        .expect("Failed to create pv:string");

    // Test empty string
    srv_pv_loc_string.post_string("").expect("Failed to post empty string");
    let value = srv_pv_loc_string.fetch().unwrap();
    assert_eq!(value.get_field_string("value").unwrap(), "");

    // Test string with spaces and punctuation
    let special_string = "Hello, World! @#$%^&*()";
    srv_pv_loc_string.post_string(special_string).expect("Failed to post special characters");
    let value = srv_pv_loc_string.fetch().unwrap();
    assert_eq!(value.get_field_string("value").unwrap(), special_string);

    // Test string with newlines and tabs
    let whitespace_string = "Line 1\nLine 2\tTabbed";
    srv_pv_loc_string.post_string(whitespace_string).expect("Failed to post whitespace string");
    let value = srv_pv_loc_string.fetch().unwrap();
    assert_eq!(value.get_field_string("value").unwrap(), whitespace_string);

    // Test Unicode characters
    let unicode_string = "Unicode: αβγ δεζ 中文 🚀";
    srv_pv_loc_string.post_string(unicode_string).expect("Failed to post unicode string");
    let value = srv_pv_loc_string.fetch().unwrap();
    assert_eq!(value.get_field_string("value").unwrap(), unicode_string);

    // Test very long string
    let long_string = "A".repeat(1000);
    srv_pv_loc_string.post_string(&long_string).expect("Failed to post long string");
    let value = srv_pv_loc_string.fetch().unwrap();
    assert_eq!(value.get_field_string("value").unwrap(), long_string);
}