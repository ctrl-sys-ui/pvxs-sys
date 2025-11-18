mod test_pvxs_remote_double_array_get_put {
    use epics_pvxs_sys::{Server, SharedPV, Context, PvxsError, NTScalarMetadataBuilder};

    #[test]
    fn test_pv_remote_double_array_get_put() {
        // This test creates a remote pv on the server and uses
        // a client context to get and put array values.
        let timeout = 5.0;
        let initial_array = vec![1.1, 2.2, 3.3, 4.4, 5.5];
        let name = "remote:double:array";
        let mut srv = Server::from_env()
            .expect("Failed to create server from env");
        
        // Create server with double array PV (this may require special setup)
        // Note: Array PV creation might need different approach depending on server implementation
        let mut srv_pv_array: SharedPV = srv.create_pv_double_array(name, initial_array.clone(), NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:double:array on server");

        // Add pv to server, making it accessible to clients
        srv.add_pv(name, &mut srv_pv_array)
            .expect("Failed to add pv to server");

        // start the server
        srv.start().expect("Failed to start server");

        // Create a client context to interact with the server
        let mut ctx = Context::from_env()
            .expect("Failed to create client context from env");

        // Do a put to set array values
        match ctx.put_double_array(name, initial_array.clone(), timeout) {
            Ok(_) => {
                // Do a get to verify the array values
                let get_result: Result<epics_pvxs_sys::Value, PvxsError> = ctx.get(name, timeout);
                match get_result {
                    Ok(value) => {
                        match value.get_field_double_array("value") {
                            Ok(retrieved_array) => {
                                assert_eq!(retrieved_array.len(), initial_array.len());
                                for (i, (&expected, &actual)) in initial_array.iter().zip(retrieved_array.iter()).enumerate() {
                                    assert_eq!(expected, actual, "Array element {} mismatch: expected {}, got {}", i, expected, actual);
                                }
                            },
                            Err(e) => panic!("Failed to get array `value` field: {:?}", e),
                        }
                    },
                    Err(e) => panic!("Failed to get value from remote pv: {:?}", e),
                }
            },
            Err(e) => {
                assert!(false, "Put operation failed: {:?}", e);
            }
        }

        // Test with larger array
        let large_array: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
        match ctx.put_double_array(name, large_array.clone(), timeout) {
            Ok(_) => {
                let value = ctx.get(name, timeout).expect("Failed to get large array");
                let retrieved = value.get_field_double_array("value").unwrap();
                assert_eq!(retrieved.len(), large_array.len(), "Large array length mismatch");
            },
            Err(e) => panic!("Large array not supported: {:?}", e),
        }

        // Test with empty array
        match ctx.put_double_array(name, vec![], timeout) {
            Ok(_) => {
                let value = ctx.get(name, timeout).expect("Failed to get empty array");
                let retrieved = value.get_field_double_array("value").unwrap();
                assert_eq!(retrieved.len(), 0);
            },
            Err(e) => panic!("Empty array not supported: {:?}", e),
        }

        // Close the server after test
        srv.stop().expect("Failed to stop server");
    }

    #[test]
    fn test_pv_remote_double_array_special_values() {
        // Test array with special floating point values
        let timeout = 5.0;
        let name = "remote:double:array:special";
        
        let mut srv = Server::from_env().expect("Failed to create server from env");
        let mut srv_pv_array: SharedPV = srv.create_pv_double_array(name, vec![0.0], NTScalarMetadataBuilder::new()).expect("Failed to create pv:double:array on server");

        srv.add_pv(name, &mut srv_pv_array).expect("Failed to add pv to server");
        srv.start().expect("Failed to start server");

        let mut ctx = Context::from_env().expect("Failed to create client context from env");

        // Test array with special values
        let special_array = vec![
            0.0,
            -0.0,
            f64::MIN,
            f64::MAX,
            f64::MIN_POSITIVE,
            1e-308,  // Very small number
            1e308,   // Very large number
            std::f64::consts::PI,
            std::f64::consts::E,
        ];

        match ctx.put_double_array(name, special_array.clone(), timeout) {
            Ok(_) => {
                let value = ctx.get(name, timeout).expect("Failed to get special array");
                let retrieved = value.get_field_double_array("value").unwrap();
                
                for (i, (&expected, &actual)) in special_array.iter().zip(retrieved.iter()).enumerate() {
                    if expected.is_finite() && actual.is_finite() {
                        assert_eq!(expected, actual, "Special value {} mismatch: expected {}, got {}", i, expected, actual);
                    }
                }
            },
            Err(e) => assert!(false, "Special values array not supported: {:?}", e),
        }

        srv.stop().expect("Failed to stop server");
    }
}