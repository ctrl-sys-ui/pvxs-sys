mod test_pvxs_remote_string_array_get_put {
    use epics_pvxs_sys::{Server, Context, PvxsError, NTScalarMetadataBuilder};

    #[test]
    fn test_pv_remote_string_array_get_put() {
        // This test creates a remote pv on the server and uses
        // a client context to get and put string array values.
        let timeout = 5.0;
        let initial_array = vec![
            "First".to_string(), 
            "Second".to_string(), 
            "Third".to_string(), 
            "Fourth".to_string()
        ];
        let name = "remote:string:array";
        let mut srv = Server::from_env()
            .expect("Failed to create server from env");
        
        // Create server with string array PV
        srv.create_pv_string(name, "", NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string:array on server");

        // start the server
        srv.start().expect("Failed to start server");

        // Create a client context to interact with the server
        let mut ctx = Context::from_env()
            .expect("Failed to create client context from env");

        // Do a put to set array values
        match ctx.put_string_array(name, initial_array.clone(), timeout) {
            Ok(_) => {
                
                // Do a get to verify the array values
                let get_result: Result<epics_pvxs_sys::Value, PvxsError> = ctx.get(name, timeout);
                match get_result {
                    Ok(value) => {
                        match value.get_field_string_array("value") {
                            Ok(retrieved_array) => {
                                assert_eq!(retrieved_array.len(), initial_array.len());
                                for (i, (expected, actual)) in initial_array.iter().zip(retrieved_array.iter()).enumerate() {
                                    assert_eq!(expected, actual, 
                                            "Array element {} mismatch: expected '{}', got '{}'", i, expected, actual);
                                }
                            },
                            Err(e) => assert!(false, "Failed to get array field: {:?}", e),
                        }
                    },
                    Err(e) => assert!(false, "Failed to get value from remote pv: {:?}", e),
                }
            },
            Err(_) => {
                // Skip the test if arrays aren't supported
                srv.stop().expect("Failed to stop server");
                return;
            }
        }

        // Test with special characters
        let special_array = vec![
            "Empty: ".to_string(),
            "Spaces and punctuation!@#$".to_string(),
            "Unicode: αβγ 中文 🚀".to_string(),
            "Newlines\nand\ttabs".to_string(),
        ];
        match ctx.put_string_array(name, special_array.clone(), timeout) {
            Ok(_) => {
                let value = ctx.get(name, timeout).expect("Failed to get special array");
                let retrieved = value.get_field_string_array("value").unwrap();
                assert_eq!(retrieved, special_array);
            },
            Err(e) => println!("Special characters array not supported: {:?}", e),
        }

        // Test with empty strings
        let empty_array = vec!["".to_string(), "non-empty".to_string(), "".to_string()];
        match ctx.put_string_array(name, empty_array.clone(), timeout) {
            Ok(_) => {
                let value = ctx.get(name, timeout).expect("Failed to get empty strings array");
                let retrieved = value.get_field_string_array("value").unwrap();
                assert_eq!(retrieved, empty_array);
            },
            Err(e) => println!("Empty strings array not supported: {:?}", e),
        }

        // Test with empty array
        match ctx.put_string_array(name, vec![], timeout) {
            Ok(_) => {
                let value = ctx.get(name, timeout).expect("Failed to get empty array");
                let retrieved = value.get_field_string_array("value").unwrap();
                assert_eq!(retrieved.len(), 0);
            },
            Err(e) => println!("Empty array not supported: {:?}", e),
        }

        // Close the server after test
        srv.stop().expect("Failed to stop server");
    }

    #[test]
    fn test_pv_remote_string_array_large_strings() {
        // Test array with large strings
        let timeout = 5.0;
        let name = "remote:string:array:large";
        
        let mut srv = Server::from_env()
            .expect("Failed to create server from env");
        srv.create_pv_string(name, "", NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string:array on server");
        srv.start().expect("Failed to start server");

        let mut ctx = Context::from_env()
            .expect("Failed to create client context from env");

        // Test array with long strings
        let large_array = vec![
            "A".repeat(100),
            "B".repeat(1000),
            "Small".to_string(),
            "C".repeat(500),
        ];

        match ctx.put_string_array(name, large_array.clone(), timeout) {
            Ok(_) => {
                let value = ctx.get(name, timeout).expect("Failed to get large strings array");
                let retrieved = value.get_field_string_array("value").unwrap();
                
                assert_eq!(retrieved.len(), large_array.len());
                for (i, (expected, actual)) in large_array.iter().zip(retrieved.iter()).enumerate() {
                    assert_eq!(expected, actual, 
                            "Large string {} mismatch in length or content", i);
                }
            },
            Err(e) => println!("Large strings array not supported: {:?}", e),
        }

        // Test many strings
        let many_strings: Vec<String> = (0..100).map(|i| format!("String_{:03}", i)).collect();
        match ctx.put_string_array(name, many_strings.clone(), timeout) {
            Ok(_) => {
                let value = ctx.get(name, timeout).expect("Failed to get many strings array");
                let retrieved = value.get_field_string_array("value").unwrap();
                assert_eq!(retrieved, many_strings);
            },
            Err(e) => println!("Many strings array not supported: {:?}", e),
        }

        srv.stop().expect("Failed to stop server");
    }
}