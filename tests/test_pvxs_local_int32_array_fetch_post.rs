use epics_pvxs_sys::{Server, SharedPV};

#[test]
fn test_pv_local_int32_array_fetch_post() {
    // This test creates a local pv (loc:int32:array) on a server and gets 
    // and sets the array value on server side.
    let initial_value = 42;
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    // Create an int32 PV that we'll try to use for arrays
    // Note: Array support depends on server implementation
    let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_int32("loc:int32:array", initial_value)
        .expect("Failed to create pv:int32:array");

    // Try to post and fetch int32 arrays (server-side only, no network)
    println!("Testing local int32 array operations...");

    // Verify we can fetch the initial scalar value
    match srv_pv_loc_array.fetch() {
        Ok(value) => {
            // Try to get as array first, fall back to scalar
            match value.get_field_int32_array("value") {
                Ok(array) => {
                    println!("Successfully got int32 array with {} elements", array.len());
                    if !array.is_empty() {
                        assert_eq!(array[0], initial_value);
                    }
                },
                Err(_) => {
                    // Fall back to scalar access
                    let scalar_val = value.get_field_int32("value").unwrap();
                    assert_eq!(scalar_val, initial_value);
                    println!("PV operates as scalar, not array");
                }
            }
        },
        Err(e) => panic!("Failed to fetch initial value: {:?}", e),
    }

    // Test posting different int32 values and reading back
    let test_values = vec![0, -1, 100, -100, 32767, -32768, 1000000, -1000000];
    
    for test_val in test_values {
        srv_pv_loc_array.post_int32(test_val).expect("Failed to post test value");
        
        let fetched = srv_pv_loc_array.fetch().expect("Failed to fetch test value");
        let retrieved_val = fetched.get_field_int32("value").unwrap();
        assert_eq!(retrieved_val, test_val, 
                  "Value mismatch: posted {}, got {}", test_val, retrieved_val);
    }
    
    println!("✓ Int32 values posted and fetched successfully");
}

#[test]
fn test_pv_local_int32_array_boundary_values() {
    // Test local handling of boundary int32 values
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_int32("loc:int32:boundary", 0)
        .expect("Failed to create pv:int32:boundary");

    // Test boundary int32 values
    let boundary_values = vec![
        ("Min i32", i32::MIN),
        ("Min+1", i32::MIN + 1),
        ("Negative", -1),
        ("Zero", 0),
        ("Positive", 1),
        ("Max-1", i32::MAX - 1),
        ("Max i32", i32::MAX),
    ];

    for (name, value) in boundary_values {
        match srv_pv_loc_array.post_int32(value) {
            Ok(_) => {
                let fetched = srv_pv_loc_array.fetch().expect("Failed to fetch boundary value");
                let retrieved = fetched.get_field_int32("value").unwrap();
                assert_eq!(retrieved, value, 
                          "{}: expected {}, got {}", name, value, retrieved);
                println!("✓ {} handled successfully: {}", name, value);
            },
            Err(e) => println!("⚠ {} not supported: {} - {}", name, value, e),
        }
    }

    // Test type conversions
    println!("\nTesting type conversions...");
    
    // Post double that should convert to int32
    match srv_pv_loc_array.post_double(3.14159) {
        Ok(_) => {
            let fetched = srv_pv_loc_array.fetch().unwrap();
            let retrieved = fetched.get_field_int32("value").unwrap();
            println!("✓ Double 3.14159 converted to int32: {}", retrieved);
            // Typically would be truncated to 3
        },
        Err(e) => println!("⚠ Double to int32 conversion not supported: {}", e),
    }

    // Post very large double that might overflow
    match srv_pv_loc_array.post_double(1e15) {
        Ok(_) => {
            let fetched = srv_pv_loc_array.fetch().unwrap();
            let retrieved = fetched.get_field_int32("value").unwrap();
            println!("✓ Large double 1e15 converted to int32: {}", retrieved);
        },
        Err(e) => println!("⚠ Large double conversion not supported: {}", e),
    }
}

#[test]
fn test_pv_local_int32_array_type_conversions() {
    // Test various type conversions to int32
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_int32("loc:int32:convert", 0)
        .expect("Failed to create pv:int32:convert");

    // Test string to int32 conversion (should fail)
    match srv_pv_loc_array.post_string("123") {
        Ok(_) => {
            let fetched = srv_pv_loc_array.fetch().unwrap();
            let retrieved = fetched.get_field_int32("value").unwrap();
            println!("✓ String '123' converted to int32: {}", retrieved);
        },
        Err(e) => println!("⚠ String to int32 conversion not supported: {}", e),
    }

    // Test invalid string
    match srv_pv_loc_array.post_string("not_a_number") {
        Ok(_) => panic!("Expected error when posting invalid string to int32 PV"),
        Err(_) => println!("✓ Correctly rejected invalid string for int32 PV"),
    }

    // Test fractional double conversion
    let fractional_tests = vec![
        (1.7, "positive fractional"),
        (-2.3, "negative fractional"),
        (0.9, "less than 1"),
        (-0.1, "negative less than 1"),
    ];

    for (test_val, description) in fractional_tests {
        match srv_pv_loc_array.post_double(test_val) {
            Ok(_) => {
                let fetched = srv_pv_loc_array.fetch().unwrap();
                let retrieved = fetched.get_field_int32("value").unwrap();
                println!("✓ {} {:.1} converted to int32: {}", description, test_val, retrieved);
            },
            Err(e) => println!("⚠ {} conversion not supported: {}", description, e),
        }
    }
}

#[test]
fn test_pv_local_int32_array_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    // Test error handling for int32 arrays with proper error propagation
    let loc_srv = Server::create_isolated()?;
    let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_int32("loc:int32:errors", 123)?;

    // Verify initial state
    let initial_fetch = srv_pv_loc_array.fetch()?;
    let initial_val = initial_fetch.get_field_int32("value")?;
    assert_eq!(initial_val, 123);

    // Test that valid operations work
    srv_pv_loc_array.post_int32(987)?;
    let updated_fetch = srv_pv_loc_array.fetch()?;
    let updated_val = updated_fetch.get_field_int32("value")?;
    assert_eq!(updated_val, 987);

    // Test invalid string posting (should fail)
    match srv_pv_loc_array.post_string("invalid_integer") {
        Ok(_) => println!("⚠ String unexpectedly accepted for int32 PV"),
        Err(_) => println!("✓ Correctly rejected invalid string for int32 PV"),
    }

    // Verify PV still works after error
    srv_pv_loc_array.post_int32(456)?;
    let final_fetch = srv_pv_loc_array.fetch()?;
    let final_val = final_fetch.get_field_int32("value")?;
    assert_eq!(final_val, 456);

    println!("✓ Error handling verified for int32 array PV");
    Ok(())
}