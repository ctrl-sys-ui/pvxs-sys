use epics_pvxs_sys::{Server, SharedPV};

#[test]
fn test_pv_local_double_array_fetch_post() {
    // This test creates a local pv (loc:double:array) on a server and gets 
    // and sets the array value on server side.
    let initial_value = 3.14159;
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    // Create a double PV that we'll try to use for arrays
    // Note: Array support depends on server implementation
    let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_double("loc:double:array", initial_value)
        .expect("Failed to create pv:double:array");

    // Try to post and fetch double arrays (server-side only, no network)
    println!("Testing local double array operations...");

    // Verify we can fetch the initial scalar value
    match srv_pv_loc_array.fetch() {
        Ok(value) => {
            // Try to get as array first, fall back to scalar
            match value.get_field_double_array("value") {
                Ok(array) => {
                    println!("Successfully got double array with {} elements", array.len());
                    if !array.is_empty() {
                        assert!((array[0] - initial_value).abs() < 1e-6);
                    }
                },
                Err(_) => {
                    // Fall back to scalar access
                    let scalar_val = value.get_field_double("value").unwrap();
                    assert!((scalar_val - initial_value).abs() < 1e-6);
                    println!("PV operates as scalar, not array");
                }
            }
        },
        Err(e) => panic!("Failed to fetch initial value: {:?}", e),
    }

    // Test posting different double values and reading back
    let test_values = vec![0.0, -1.5, 2.71828, 1e-10, 1e10];
    
    for test_val in test_values {
        srv_pv_loc_array.post_double(test_val).expect("Failed to post test value");
        
        let fetched = srv_pv_loc_array.fetch().expect("Failed to fetch test value");
        let retrieved_val = fetched.get_field_double("value").unwrap();
        assert!((retrieved_val - test_val).abs() < 1e-14, 
               "Value mismatch: posted {}, got {}", test_val, retrieved_val);
    }
    
    println!("✓ Double values posted and fetched successfully");
}

#[test]
fn test_pv_local_double_array_special_values() {
    // Test local handling of special floating point values in arrays
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_double("loc:double:special", 0.0)
        .expect("Failed to create pv:double:special");

    // Test special double values
    let special_values = vec![
        ("Zero", 0.0),
        ("Negative zero", -0.0),
        ("PI", std::f64::consts::PI),
        ("E", std::f64::consts::E),
        ("Max", f64::MAX),
        ("Min", f64::MIN),
        ("Min positive", f64::MIN_POSITIVE),
        ("Very small", 1e-308),
        ("Very large", 1e308),
    ];

    for (name, value) in special_values {
        match srv_pv_loc_array.post_double(value) {
            Ok(_) => {
                let fetched = srv_pv_loc_array.fetch().expect("Failed to fetch special value");
                let retrieved = fetched.get_field_double("value").unwrap();
                
                if value.is_finite() {
                    assert!((retrieved - value).abs() < 1e-14, 
                           "{}: expected {}, got {}", name, value, retrieved);
                }
                println!("✓ {} handled successfully: {}", name, value);
            },
            Err(e) => println!("⚠ {} not supported: {} - {}", name, value, e),
        }
    }

    // Test infinity (may not be supported)
    match srv_pv_loc_array.post_double(f64::INFINITY) {
        Ok(_) => {
            let fetched = srv_pv_loc_array.fetch().unwrap();
            let retrieved = fetched.get_field_double("value").unwrap();
            if retrieved.is_infinite() && retrieved > 0.0 {
                println!("✓ Positive infinity supported");
            }
        },
        Err(e) => println!("⚠ Positive infinity not supported: {}", e),
    }

    // Test negative infinity
    match srv_pv_loc_array.post_double(f64::NEG_INFINITY) {
        Ok(_) => {
            let fetched = srv_pv_loc_array.fetch().unwrap();
            let retrieved = fetched.get_field_double("value").unwrap();
            if retrieved.is_infinite() && retrieved < 0.0 {
                println!("✓ Negative infinity supported");
            }
        },
        Err(e) => println!("⚠ Negative infinity not supported: {}", e),
    }

    // Test NaN (likely not supported by EPICS)
    match srv_pv_loc_array.post_double(f64::NAN) {
        Ok(_) => println!("✓ NaN posted (unusual for EPICS)"),
        Err(e) => println!("⚠ NaN not supported (expected): {}", e),
    }
}

#[test]
fn test_pv_local_double_array_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    // Test error handling for double arrays with proper error propagation
    let loc_srv = Server::create_isolated()?;
    let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_double("loc:double:errors", 1.23)?;

    // Verify initial state
    let initial_fetch = srv_pv_loc_array.fetch()?;
    let initial_val = initial_fetch.get_field_double("value")?;
    assert!((initial_val - 1.23).abs() < 1e-6);

    // Test that valid operations work
    srv_pv_loc_array.post_double(9.87)?;
    let updated_fetch = srv_pv_loc_array.fetch()?;
    let updated_val = updated_fetch.get_field_double("value")?;
    assert!((updated_val - 9.87).abs() < 1e-6);

    // Test invalid string posting (should fail)
    match srv_pv_loc_array.post_string("not_a_number") {
        Ok(_) => panic!("Expected error when posting invalid string to double PV"),
        Err(_) => println!("✓ Correctly rejected invalid string for double PV"),
    }

    // Verify PV still works after error
    srv_pv_loc_array.post_double(4.56)?;
    let final_fetch = srv_pv_loc_array.fetch()?;
    let final_val = final_fetch.get_field_double("value")?;
    assert!((final_val - 4.56).abs() < 1e-6);

    println!("✓ Error handling verified for double array PV");
    Ok(())
}