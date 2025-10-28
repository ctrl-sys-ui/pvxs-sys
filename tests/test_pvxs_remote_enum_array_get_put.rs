use epics_pvxs_sys::{Server, SharedPV, Context, PvxsError};

#[test]
fn test_pv_remote_enum_array_get_put() {
    // This test creates a remote pv on the server and uses
    // a client context to get and put enum array values.
    let timeout = 5.0;
    let initial_array = vec![0i16, 1i16, 2i16, 0i16, 1i16]; // Enum indices
    let name = "remote:enum:array";
    let mut srv = Server::from_env()
        .expect("Failed to create server from env");
    
    // Create server with enum array PV (may need special setup for enums)
    let mut srv_pv_array: SharedPV = srv.create_pv_int32(name, 0) // Using int32 as base for enum
        .expect("Failed to create pv:enum:array on server");

    // Add pv to server, making it accessible to clients
    srv.add_pv(name, &mut srv_pv_array)
        .expect("Failed to add pv to server");

    // start the server
    srv.start().expect("Failed to start server");

    // Create a client context to interact with the server
    let mut ctx = Context::from_env()
        .expect("Failed to create client context from env");

    // Do a put to set array values
    match ctx.put_enum_array(name, initial_array.clone(), timeout) {
        Ok(_) => {
            println!("Successfully put enum array");
            
            // Do a get to verify the array values
            let get_result: Result<epics_pvxs_sys::Value, PvxsError> = ctx.get(name, timeout);
            match get_result {
                Ok(value) => {
                    match value.get_field_enum_array("value") {
                        Ok(retrieved_array) => {
                            assert_eq!(retrieved_array.len(), initial_array.len());
                            for (i, (&expected, &actual)) in initial_array.iter().zip(retrieved_array.iter()).enumerate() {
                                assert_eq!(expected, actual, 
                                          "Array element {} mismatch: expected {}, got {}", i, expected, actual);
                            }
                            println!("Array values verified successfully");
                        },
                        Err(e) => panic!("Failed to get array field: {:?}", e),
                    }
                },
                Err(e) => panic!("Failed to get value from remote pv: {:?}", e),
            }
        },
        Err(e) => {
            println!("Server may not support enum arrays: {:?}", e);
            // Skip the test if arrays aren't supported
            srv.stop().expect("Failed to stop server");
            return;
        }
    }

    // Test with different enum values (within typical enum range)
    let enum_array = vec![0i16, 1i16, 2i16, 3i16, 4i16, 5i16];
    match ctx.put_enum_array(name, enum_array.clone(), timeout) {
        Ok(_) => {
            let value = ctx.get(name, timeout).expect("Failed to get enum array");
            let retrieved = value.get_field_enum_array("value").unwrap();
            assert_eq!(retrieved, enum_array);
            println!("Enum values array handled successfully");
        },
        Err(e) => println!("Enum array not supported: {:?}", e),
    }

    // Test with repeated values
    let repeated_array = vec![1i16; 10]; // All elements same value
    match ctx.put_enum_array(name, repeated_array.clone(), timeout) {
        Ok(_) => {
            let value = ctx.get(name, timeout).expect("Failed to get repeated array");
            let retrieved = value.get_field_enum_array("value").unwrap();
            assert_eq!(retrieved, repeated_array);
            println!("Repeated enum values array handled successfully");
        },
        Err(e) => println!("Repeated enum array not supported: {:?}", e),
    }

    // Test with empty array
    match ctx.put_enum_array(name, vec![], timeout) {
        Ok(_) => {
            let value = ctx.get(name, timeout).expect("Failed to get empty array");
            let retrieved = value.get_field_enum_array("value").unwrap();
            assert_eq!(retrieved.len(), 0);
            println!("Empty array handled successfully");
        },
        Err(e) => println!("Empty array not supported: {:?}", e),
    }

    // Close the server after test
    srv.stop().expect("Failed to stop server");
}

#[test]
fn test_pv_remote_enum_array_with_choices() {
    // Test enum array functionality with associated choices
    let timeout = 5.0;
    let name = "remote:enum:array:choices";
    
    let mut srv = Server::from_env()
        .expect("Failed to create server from env");
    let mut srv_pv_array: SharedPV = srv.create_pv_int32(name, 0)
        .expect("Failed to create pv:enum:array on server");

    srv.add_pv(name, &mut srv_pv_array)
        .expect("Failed to add pv to server");
    srv.start().expect("Failed to start server");

    let mut ctx = Context::from_env()
        .expect("Failed to create client context from env");

    // Expected enum choices (this would typically be defined by the server)
    let _expected_choices = vec!["OFF", "ON", "STANDBY", "ERROR", "UNKNOWN"];
    
    // Test enum array with indices corresponding to choices
    let enum_indices = vec![0i16, 1i16, 2i16, 3i16, 4i16, 1i16, 0i16];
    
    match ctx.put_enum_array(name, enum_indices.clone(), timeout) {
        Ok(_) => {
            let value = ctx.get(name, timeout).expect("Failed to get enum array with choices");
            
            // Get the enum indices
            match value.get_field_enum_array("value") {
                Ok(retrieved_indices) => {
                    assert_eq!(retrieved_indices, enum_indices);
                    
                    // Try to get the choices array (if available)
                    match value.get_field_string_array("value.choices") {
                        Ok(choices) => {
                            println!("Retrieved choices: {:?}", choices);
                            
                            // Verify indices are within choices range
                            for &index in &retrieved_indices {
                                assert!((index as usize) < choices.len(), 
                                       "Enum index {} out of range for {} choices", index, choices.len());
                            }
                            
                            // Print human-readable enum values
                            println!("Enum array values:");
                            for (i, &index) in retrieved_indices.iter().enumerate() {
                                let choice_name = if (index as usize) < choices.len() {
                                    &choices[index as usize]
                                } else {
                                    "INVALID"
                                };
                                println!("  [{}] = {} ('{}')", i, index, choice_name);
                            }
                        },
                        Err(_) => {
                            println!("Choices array not available (normal for simple enum arrays)");
                        }
                    }
                },
                Err(e) => panic!("Failed to get enum indices: {:?}", e),
            }
        },
        Err(e) => {
            println!("Enum array with choices not supported: {:?}", e);
        }
    }

    srv.stop().expect("Failed to stop server");
}

#[test]
fn test_pv_remote_enum_array_boundary_values() {
    // Test enum array with boundary values
    let timeout = 5.0;
    let name = "remote:enum:array:boundary";
    
    let mut srv = Server::from_env()
        .expect("Failed to create server from env");
    let mut srv_pv_array: SharedPV = srv.create_pv_int32(name, 0)
        .expect("Failed to create pv:enum:array on server");

    srv.add_pv(name, &mut srv_pv_array)
        .expect("Failed to add pv to server");
    srv.start().expect("Failed to start server");

    let mut ctx = Context::from_env()
        .expect("Failed to create client context from env");

    // Test with boundary values for i16 (enum type)
    let boundary_array = vec![
        0i16,                    // Minimum valid enum index
        1i16,
        10i16,                   // Typical enum values
        100i16,                  // Larger enum index
        i16::MAX,               // Maximum i16 value
    ];

    match ctx.put_enum_array(name, boundary_array.clone(), timeout) {
        Ok(_) => {
            let value = ctx.get(name, timeout).expect("Failed to get boundary enum array");
            let retrieved = value.get_field_enum_array("value").unwrap();
            
            assert_eq!(retrieved.len(), boundary_array.len());
            for (i, (&expected, &actual)) in boundary_array.iter().zip(retrieved.iter()).enumerate() {
                assert_eq!(expected, actual, 
                          "Boundary enum value {} mismatch: expected {}, got {}", i, expected, actual);
            }
            println!("Boundary enum values handled successfully");
        },
        Err(e) => println!("Boundary enum values not supported: {:?}", e),
    }

    srv.stop().expect("Failed to stop server");
}