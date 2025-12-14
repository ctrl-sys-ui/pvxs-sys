mod test_pvxs_local_double_array_fetch_post {

    use pvxs_sys::{Server, NTScalarMetadataBuilder};

    #[test]
    fn test_pv_local_double_array_fetch_post() {
        // This test creates a local pv (loc:double:array) on a server and
        // tests server-side fetch() and post_double() operations.
        let initial_value = 3.14159;
        let name = "loc:double:array";
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        // Create a double PV and capture it for server-side operations
        let mut srv_pv = loc_srv.create_pv_double(name, initial_value, NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:double:array");

        // Verify we can fetch the initial scalar value using server-side fetch
        let value = srv_pv.fetch().expect("Failed to fetch initial value");
        let scalar_val = value.get_field_double("value").unwrap();
        assert_eq!(scalar_val, initial_value, "Initial scalar value mismatch, got {}, expected {}", scalar_val, initial_value);

        // Test posting different double values and reading back using server-side operations
        let test_values = vec![0.0, -1.5, 2.71828, 1e-10, 1e10];
        
        for test_val in test_values {
            srv_pv.post_double(test_val).expect("Failed to post test value");
            
            let fetched = srv_pv.fetch().expect("Failed to fetch test value");
            let retrieved_val = fetched.get_field_double("value").unwrap();
            assert_eq!(retrieved_val, test_val, "Value mismatch: posted {}, got {}", test_val, retrieved_val);
        }
    }

    #[test]
    fn test_pv_local_double_array_special_values() {
        // Test local handling of special floating point values using server-side operations
        let name = "loc:double:special";
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        let mut srv_pv = loc_srv.create_pv_double(name, 0.0, NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:double:special");

        // Test special double values using server-side post/fetch
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
            srv_pv.post_double(value).expect(&format!("Failed to post {}", name_val));
            let fetched = srv_pv.fetch().expect("Failed to fetch special value");
            let retrieved = fetched.get_field_double("value").unwrap();
            
            if value.is_finite() {
                assert_eq!(retrieved, value, "{}: expected {}, got {}", name_val, value, retrieved);
            }
        }

        // Test infinity
        srv_pv.post_double(f64::INFINITY).expect("Failed to post infinity");
        let fetched = srv_pv.fetch().unwrap();
        let retrieved = fetched.get_field_double("value").unwrap();
        assert!(retrieved.is_infinite() && retrieved > 0.0, "Expected positive infinity, got {}", retrieved);

        // Test negative infinity
        srv_pv.post_double(f64::NEG_INFINITY).expect("Failed to post negative infinity");
        let fetched = srv_pv.fetch().unwrap();
        let retrieved = fetched.get_field_double("value").unwrap();
        assert!(retrieved.is_infinite() && retrieved < 0.0, "Expected negative infinity, got {}", retrieved);

        // Test NaN
        srv_pv.post_double(f64::NAN).expect("Failed to post NaN");
        let fetched = srv_pv.fetch().unwrap();
        let retrieved = fetched.get_field_double("value").unwrap();
        assert!(retrieved.is_nan(), "Expected NaN, got {}", retrieved);
    }

    #[test]
    fn test_pv_local_double_array_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        // Test server-side operations with proper error propagation
        let name = "loc:double:errors";
        let mut loc_srv = Server::create_isolated()?;
        let mut srv_pv = loc_srv.create_pv_double(name, 1.23, NTScalarMetadataBuilder::new())?;

        // Verify initial state using server-side fetch
        let initial_fetch = srv_pv.fetch()?;
        let initial_val = initial_fetch.get_field_double("value")?;
        assert!((initial_val - 1.23).abs() < 1e-6);

        // Test that valid operations work using server-side post
        srv_pv.post_double(9.87)?;
        let updated_fetch = srv_pv.fetch()?;
        let updated_val = updated_fetch.get_field_double("value")?;
        assert!((updated_val - 9.87).abs() < 1e-6);

        // Test another valid operation
        srv_pv.post_double(4.56)?;
        let final_fetch = srv_pv.fetch()?;
        let final_val = final_fetch.get_field_double("value")?;
        assert_eq!(final_val, 4.56, "Expected 4.56, got {}", final_val);
        Ok(())
    }
}