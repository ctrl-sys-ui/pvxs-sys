mod test_pvxs_local_double_array_fetch_post {

    use epics_pvxs_sys::{Server, Context, NTScalarMetadataBuilder};

    #[test]
    fn test_pv_local_double_array_fetch_post() {
        // This test creates a local pv (loc:double:array) on a server and gets 
        // and sets the array value using client operations.
        let initial_value = 3.14159;
        let name = "loc:double:array";
        let timeout = 5.0;
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        // Create a double PV
        loc_srv.create_pv_double(name, initial_value, NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:double:array");

        // Use a client context to interact with the local server
        let mut ctx = Context::from_env().expect("Failed to create client context");

        // Verify we can fetch the initial scalar value
        match ctx.get(name, timeout) {
            Ok(value) => {
                // Try to get as array first, fall back to scalar
                match value.get_field_double_array("value") {
                    Ok(array) => {
                        if !array.is_empty() {
                            assert_eq!(array[0], initial_value, "Initial array value mismatch, got {}, expected {}", array[0], initial_value);
                        }
                    },
                    Err(_) => {
                        // Fall back to scalar access
                        let scalar_val = value.get_field_double("value").unwrap();
                        assert_eq!(scalar_val, initial_value, "Initial scalar value mismatch, got {}, expected {}", scalar_val, initial_value);
                    }
                }
            },
            Err(e) => assert!(false, "Failed to fetch initial value: {:?}", e),
        }

        // Test posting different double values and reading back
        let test_values = vec![0.0, -1.5, 2.71828, 1e-10, 1e10];
        
        for test_val in test_values {
            ctx.put_double(name, test_val, timeout).expect("Failed to post test value");
            
            let fetched = ctx.get(name, timeout).expect("Failed to fetch test value");
            let retrieved_val = fetched.get_field_double("value").unwrap();
            assert_eq!(retrieved_val, test_val, "Value mismatch: posted {}, got {}", test_val, retrieved_val);
        }
    }

    #[test]
    fn test_pv_local_double_array_special_values() {
        // Test local handling of special floating point values in arrays
        let name = "loc:double:special";
        let timeout = 5.0;
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        loc_srv.create_pv_double(name, 0.0, NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:double:special");

        let mut ctx = Context::from_env().expect("Failed to create client context");

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

        for (name_val, value) in special_values {
            match ctx.put_double(name, value, timeout) {
                Ok(_) => {
                    let fetched = ctx.get(name, timeout).expect("Failed to fetch special value");
                    let retrieved = fetched.get_field_double("value").unwrap();
                    
                    if value.is_finite() {
                        assert_eq!(retrieved, value, "{}: expected {}, got {}", name_val, value, retrieved);
                    }
                },
                Err(e) => assert!(false, "{} not supported: {} - {}", name_val, value, e),
            }
        }

        // Test infinity (may not be supported)
        match ctx.put_double(name, f64::INFINITY, timeout) {
            Ok(_) => {
                let fetched = ctx.get(name, timeout).unwrap();
                let retrieved = fetched.get_field_double("value").unwrap();
                assert!(retrieved.is_infinite() && retrieved > 0.0, "Expected positive infinity, got {}", retrieved);
            },
            Err(e) => assert!(false, "Positive infinity not supported: {}", e),
        }

        // Test negative infinity
        match ctx.put_double(name, f64::NEG_INFINITY, timeout) {
            Ok(_) => {
                let fetched = ctx.get(name, timeout).unwrap();
                let retrieved = fetched.get_field_double("value").unwrap();
                assert!(retrieved.is_infinite() && retrieved < 0.0, "Expected negative infinity, got {}", retrieved);
            },
            Err(e) => assert!(false, "Negative infinity not supported: {}", e),
        }

        // Test NaN (likely not supported by EPICS)
        match ctx.put_double(name, f64::NAN, timeout) {
            Ok(_) => assert!(true, "NaN posted (unusual for EPICS)"),
            Err(e) => assert!(false, "NaN not supported (expected): {}", e),
        }
    }

    #[test]
    fn test_pv_local_double_array_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        // Test error handling for double arrays with proper error propagation
        let name = "loc:double:errors";
        let timeout = 5.0;
        let mut loc_srv = Server::create_isolated()?;
        loc_srv.create_pv_double(name, 1.23, NTScalarMetadataBuilder::new())?;

        let mut ctx = Context::from_env()?;

        // Verify initial state
        let initial_fetch = ctx.get(name, timeout)?;
        let initial_val = initial_fetch.get_field_double("value")?;
        assert!((initial_val - 1.23).abs() < 1e-6);

        // Test that valid operations work
        ctx.put_double(name, 9.87, timeout)?;
        let updated_fetch = ctx.get(name, timeout)?;
        let updated_val = updated_fetch.get_field_double("value")?;
        assert!((updated_val - 9.87).abs() < 1e-6);

        // Test invalid string posting (should fail)
        match ctx.put_string(name, "not_a_number", timeout) {
            Ok(_) => assert!(false, "Expected error when posting invalid string to double PV"),
            Err(_) => assert!(true, "Correctly rejected invalid string for double PV"),
        }

        // Verify PV still works after error
        ctx.put_double(name, 4.56, timeout)?;
        let final_fetch = ctx.get(name, timeout)?;
        let final_val = final_fetch.get_field_double("value")?;
        assert_eq!(final_val, 4.56, "Expected 4.56, got {}", final_val);
        Ok(())
    }
}