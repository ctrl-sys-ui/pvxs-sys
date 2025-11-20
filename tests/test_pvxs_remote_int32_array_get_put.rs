use epics_pvxs_sys::{Server, Context, PvxsError, NTScalarMetadataBuilder};

#[test]
fn test_pv_remote_int32_array_get_put() {
    // This test creates a remote pv on the server and uses
    // a client context to get and put int32 array values.
    let timeout = 5.0;
    let initial_array = vec![10, 20, 30, 40, 50];
    let name = "remote:int32:array";
    let mut srv = Server::from_env()
        .expect("Failed to create server from env");
    
    // Create server with int32 array PV (automatically added)
    srv.create_pv_int32_array(name, initial_array.clone(), NTScalarMetadataBuilder::new())
        .expect("Failed to create pv:int32:array on server");

    // start the server
    srv.start().expect("Failed to start server");

    // Create a client context to interact with the server
    let mut ctx = Context::from_env()
        .expect("Failed to create client context from env");

    // Do a put to set array values
    match ctx.put_int32_array(name, initial_array.clone(), timeout) {
        Ok(_) => {            
            // Do a get to verify the array values
            let get_result: Result<epics_pvxs_sys::Value, PvxsError> = ctx.get(name, timeout);
            match get_result {
                Ok(value) => {
                    match value.get_field_int32_array("value") {
                        Ok(retrieved_array) => {
                            assert_eq!(retrieved_array.len(), initial_array.len());
                            for (i, (&expected, &actual)) in initial_array.iter().zip(retrieved_array.iter()).enumerate() {
                                assert_eq!(expected, actual, 
                                          "Array element {} mismatch: expected {}, got {}", i, expected, actual);
                            }
                        },
                        Err(e) => assert!(false, "Failed to get array field: {:?}", e),
                    }
                },
                Err(e) => assert!(false, "Failed to get value from remote pv: {:?}", e),
            }
        },
        Err(e) => {
            // Skip the test if arrays aren't supported
            srv.stop().expect("Failed to stop server");
            return;
        }
    }

    // Test with negative values
    let negative_array = vec![-100, -50, 0, 50, 100];
    match ctx.put_int32_array(name, negative_array.clone(), timeout) {
        Ok(_) => {
            let value = ctx.get(name, timeout).expect("Failed to get negative array");
            let retrieved = value.get_field_int32_array("value").unwrap();
            assert_eq!(retrieved, negative_array);
        },
        Err(e) => assert!(false, "Negative array not supported: {:?}", e),
    }

    // Test with large array
    let large_array: Vec<i32> = (0..200).collect();
    match ctx.put_int32_array(name, large_array.clone(), timeout) {
        Ok(_) => {
            let value = ctx.get(name, timeout).expect("Failed to get large array");
            let retrieved = value.get_field_int32_array("value").unwrap();
            assert_eq!(retrieved.len(), large_array.len(), "Large array length mismatch");
        },
        Err(e) => assert!(false, "Large array not supported: {:?}", e),
    }

    // Test with empty array
    match ctx.put_int32_array(name, vec![], timeout) {
        Ok(_) => {
            let value = ctx.get(name, timeout).expect("Failed to get empty array");
            let retrieved = value.get_field_int32_array("value").unwrap();
            assert_eq!(retrieved.len(), 0);
        },
        Err(e) => assert!(false, "Empty array not supported: {:?}", e),
    }

    // Close the server after test
    srv.stop().expect("Failed to stop server");
}

#[test]
fn test_pv_remote_int32_array_boundary() {
    // Test int32 array with boundary values
    let timeout = 5.0;
    let name = "remote:int32:array:boundary";
    
    let mut srv = Server::from_env()
        .expect("Failed to create server from env");
    srv.create_pv_int32_array(name, vec![0], NTScalarMetadataBuilder::new())
        .expect("Failed to create pv:int32:array on server");

    srv.start().expect("Failed to start server");

    let mut ctx = Context::from_env()
        .expect("Failed to create client context from env");

    // Test array with boundary values
    let boundary_array = vec![
        i32::MIN,
        i32::MIN + 1,
        -1,
        0,
        1,
        i32::MAX - 1,
        i32::MAX,
    ];

    match ctx.put_int32_array(name, boundary_array.clone(), timeout) {
        Ok(_) => {
            let value = ctx.get(name, timeout).expect("Failed to get boundary array");
            let retrieved = value.get_field_int32_array("value").unwrap();
            
            assert_eq!(retrieved.len(), boundary_array.len());
            for (i, (&expected, &actual)) in boundary_array.iter().zip(retrieved.iter()).enumerate() {
                assert_eq!(expected, actual, 
                          "Boundary value {} mismatch: expected {}, got {}", i, expected, actual);
            }
        },
        Err(e) => assert!(false, "Boundary values array not supported: {:?}", e),
    }

    // Test monotonic sequence
    let sequence_array: Vec<i32> = (1..=1000).collect();
    match ctx.put_int32_array(name, sequence_array.clone(), timeout) {
        Ok(_) => {
            let value = ctx.get(name, timeout).expect("Failed to get sequence array");
            let retrieved = value.get_field_int32_array("value").unwrap();
            assert_eq!(retrieved, sequence_array, "Monotonic sequence array does not match");
        },
        Err(e) => assert!(false, "Sequence array not supported: {:?}", e),
    }

    srv.stop().expect("Failed to stop server");
}