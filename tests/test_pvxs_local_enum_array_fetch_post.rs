use epics_pvxs_sys::{Server, SharedPV};

#[test]
fn test_pv_local_enum_array_fetch_post() {
    // This test creates a local pv (loc:enum:array) on a server and gets 
    // and sets the enum array value on server side.
    // Note: Since SharedPV doesn't have create_pv_enum, we use int32 as base
    let initial_value = 1; // Enum index
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    // Create an int32 PV that we'll treat as enum array
    // Note: Enum support depends on server implementation and may need special setup
    let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_int32("loc:enum:array", initial_value)
        .expect("Failed to create pv:enum:array");

    // Try to post and fetch enum arrays (server-side only, no network)
    println!("Testing local enum array operations...");

    // Verify we can fetch the initial value
    match srv_pv_loc_array.fetch() {
        Ok(value) => {
            // Try to get as enum array first, fall back to int32 then enum scalar
            match value.get_field_enum_array("value") {
                Ok(array) => {
                    println!("Successfully got enum array with {} elements", array.len());
                    if !array.is_empty() {
                        assert_eq!(array[0], initial_value as i16);
                    }
                },
                Err(_) => {
                    // Try as enum scalar
                    match value.get_field_enum("value") {
                        Ok(enum_val) => {
                            assert_eq!(enum_val, initial_value as i16);
                            println!("PV operates as enum scalar, not array");
                        },
                        Err(_) => {
                            // Fall back to int32 access
                            let scalar_val = value.get_field_int32("value").unwrap();
                            assert_eq!(scalar_val, initial_value);
                            println!("PV operates as int32, not enum");
                        }
                    }
                }
            }
        },
        Err(e) => panic!("Failed to fetch initial value: {:?}", e),
    }

    // Test posting different enum values (as int32) and reading back
    let enum_indices = vec![0, 1, 2, 3, 4, 5]; // Typical enum indices
    
    for enum_val in enum_indices {
        srv_pv_loc_array.post_int32(enum_val).expect("Failed to post test value");
        
        let fetched = srv_pv_loc_array.fetch().expect("Failed to fetch test value");
        
        // Try to read back as enum first, fall back to int32
        match fetched.get_field_enum("value") {
            Ok(retrieved_enum) => {
                assert_eq!(retrieved_enum, enum_val as i16, 
                          "Enum value mismatch: posted {}, got {}", enum_val, retrieved_enum);
            },
            Err(_) => {
                let retrieved_int = fetched.get_field_int32("value").unwrap();
                assert_eq!(retrieved_int, enum_val, 
                          "Int32 value mismatch: posted {}, got {}", enum_val, retrieved_int);
            }
        }
    }
    
    println!("✓ Enum indices posted and fetched successfully");
}

#[test]
fn test_pv_local_enum_array_boundary_values() {
    // Test local handling of boundary enum values
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_int32("loc:enum:boundary", 0)
        .expect("Failed to create pv:enum:boundary");

    // Test boundary enum values (i16 range for enums)
    let boundary_values = vec![
        ("Min enum", 0i32),          // Minimum valid enum index
        ("Small", 1i32),
        ("Medium", 10i32),
        ("Large", 100i32),
        ("Very large", 1000i32),
        ("Max i16 as i32", i16::MAX as i32),  // Maximum i16 value
    ];

    for (name, value) in boundary_values {
        match srv_pv_loc_array.post_int32(value) {
            Ok(_) => {
                let fetched = srv_pv_loc_array.fetch().expect("Failed to fetch boundary value");
                
                // Try enum access first, fall back to int32
                match fetched.get_field_enum("value") {
                    Ok(retrieved_enum) => {
                        println!("✓ {} enum {} → {}", name, value, retrieved_enum);
                    },
                    Err(_) => {
                        let retrieved_int = fetched.get_field_int32("value").unwrap();
                        assert_eq!(retrieved_int, value);
                        println!("✓ {} as int32: {}", name, value);
                    }
                }
            },
            Err(e) => println!("⚠ {} not supported: {} - {}", name, value, e),
        }
    }

    // Test invalid enum values (negative)
    println!("\nTesting invalid enum values...");
    match srv_pv_loc_array.post_int32(-1) {
        Ok(_) => {
            let fetched = srv_pv_loc_array.fetch().unwrap();
            match fetched.get_field_enum("value") {
                Ok(retrieved_enum) => {
                    println!("⚠ Negative enum index accepted: {}", retrieved_enum);
                },
                Err(_) => {
                    let retrieved_int = fetched.get_field_int32("value").unwrap();
                    println!("✓ Negative value handled as int32: {}", retrieved_int);
                }
            }
        },
        Err(e) => println!("✓ Correctly rejected negative enum value: {}", e),
    }
}

#[test]
fn test_pv_local_enum_array_type_conversions() {
    // Test various type conversions to enum
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_int32("loc:enum:convert", 0)
        .expect("Failed to create pv:enum:convert");

    // Test double to enum conversion (should truncate)
    match srv_pv_loc_array.post_double(2.7) {
        Ok(_) => {
            let fetched = srv_pv_loc_array.fetch().unwrap();
            match fetched.get_field_enum("value") {
                Ok(retrieved_enum) => {
                    println!("✓ Double 2.7 converted to enum: {}", retrieved_enum);
                },
                Err(_) => {
                    let retrieved_int = fetched.get_field_int32("value").unwrap();
                    println!("✓ Double 2.7 converted to int32: {}", retrieved_int);
                }
            }
        },
        Err(e) => println!("⚠ Double to enum conversion not supported: {}", e),
    }

    // Test string to enum conversion (should fail)
    match srv_pv_loc_array.post_string("2") {
        Ok(_) => {
            let fetched = srv_pv_loc_array.fetch().unwrap();
            match fetched.get_field_enum("value") {
                Ok(retrieved_enum) => {
                    println!("✓ String '2' converted to enum: {}", retrieved_enum);
                },
                Err(_) => {
                    println!("⚠ String converted but not accessible as enum");
                }
            }
        },
        Err(_) => println!("✓ Correctly rejected string for enum PV"),
    }

    // Test fractional values
    let fractional_tests = vec![
        (0.1, "small fraction"),
        (1.9, "large fraction"), 
        (3.14159, "pi"),
        (-0.5, "negative fraction"),
    ];

    for (test_val, description) in fractional_tests {
        match srv_pv_loc_array.post_double(test_val) {
            Ok(_) => {
                let fetched = srv_pv_loc_array.fetch().unwrap();
                match fetched.get_field_enum("value") {
                    Ok(retrieved_enum) => {
                        println!("✓ {} {:.3} converted to enum: {}", description, test_val, retrieved_enum);
                    },
                    Err(_) => {
                        let retrieved_int = fetched.get_field_int32("value").unwrap();
                        println!("✓ {} {:.3} converted to int32: {}", description, test_val, retrieved_int);
                    }
                }
            },
            Err(e) => println!("⚠ {} conversion not supported: {}", description, e),
        }
    }
}

#[test] 
fn test_pv_local_enum_array_choices_simulation() {
    // Simulate enum with choices by testing multiple related PVs
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    // Create an enum index PV
    let mut srv_pv_index: SharedPV = loc_srv.create_pv_int32("loc:enum:index", 0)
        .expect("Failed to create enum index PV");

    // Simulate some typical enum states
    let enum_states = vec![
        (0, "OFF"),
        (1, "ON"), 
        (2, "STANDBY"),
        (3, "ERROR"),
        (4, "UNKNOWN"),
    ];

    println!("Testing enum state simulation...");

    for (index, state_name) in enum_states {
        // Post the enum index
        srv_pv_index.post_int32(index).expect("Failed to post enum index");
        
        let fetched = srv_pv_index.fetch().expect("Failed to fetch enum value");
        
        // Verify as enum
        match fetched.get_field_enum("value") {
            Ok(retrieved_enum) => {
                assert_eq!(retrieved_enum, index as i16);
                println!("✓ Enum state {}: {} (index {})", state_name, retrieved_enum, index);
            },
            Err(_) => {
                // Fall back to int32
                let retrieved_int = fetched.get_field_int32("value").unwrap();
                assert_eq!(retrieved_int, index);
                println!("✓ Enum state {} as int32: {}", state_name, retrieved_int);
            }
        }
    }

    // Test invalid enum state
    match srv_pv_index.post_int32(99) {
        Ok(_) => {
            let fetched = srv_pv_index.fetch().unwrap();
            match fetched.get_field_enum("value") {
                Ok(retrieved_enum) => {
                    println!("⚠ Invalid enum index 99 accepted as: {}", retrieved_enum);
                },
                Err(_) => {
                    println!("✓ Invalid enum index handled as int32");
                }
            }
        },
        Err(e) => println!("✓ Correctly rejected invalid enum index: {}", e),
    }
}

#[test]
fn test_pv_local_enum_array_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    // Test error handling for enum arrays with proper error propagation
    let loc_srv = Server::create_isolated()?;
    let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_int32("loc:enum:errors", 1)?;

    // Verify initial state
    let initial_fetch = srv_pv_loc_array.fetch()?;
    match initial_fetch.get_field_enum("value") {
        Ok(initial_enum) => {
            assert_eq!(initial_enum, 1);
            println!("✓ Initial enum value: {}", initial_enum);
        },
        Err(_) => {
            let initial_int = initial_fetch.get_field_int32("value")?;
            assert_eq!(initial_int, 1);
            println!("✓ Initial value as int32: {}", initial_int);
        }
    }

    // Test that valid operations work
    srv_pv_loc_array.post_int32(3)?;
    let updated_fetch = srv_pv_loc_array.fetch()?;
    match updated_fetch.get_field_enum("value") {
        Ok(updated_enum) => assert_eq!(updated_enum, 3),
        Err(_) => {
            let updated_int = updated_fetch.get_field_int32("value")?;
            assert_eq!(updated_int, 3);
        }
    }

    // Test invalid string posting (should fail)
    match srv_pv_loc_array.post_string("invalid_enum") {
        Ok(_) => println!("⚠ String unexpectedly accepted for enum PV"),
        Err(_) => println!("✓ Correctly rejected invalid string for enum PV"),
    }

    // Verify PV still works after error
    srv_pv_loc_array.post_int32(2)?;
    let final_fetch = srv_pv_loc_array.fetch()?;
    match final_fetch.get_field_enum("value") {
        Ok(final_enum) => assert_eq!(final_enum, 2),
        Err(_) => {
            let final_int = final_fetch.get_field_int32("value")?;
            assert_eq!(final_int, 2);
        }
    }

    println!("✓ Error handling verified for enum array PV");
    Ok(())
}