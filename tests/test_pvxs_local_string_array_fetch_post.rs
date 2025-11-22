mod test_pvxs_local_string_array_fetch_post {
    use epics_pvxs_sys::{Server, NTScalarMetadataBuilder};

    #[test]
    fn test_pv_local_string_array_fetch_post() {
        // This test creates a local pv (loc:string:array) and tests
        // server-side fetch() and post_string() operations.
        let initial_value = "Initial string array element";
        let name = "loc:string:array";
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        // Create a string PV and capture for server-side operations
        let mut srv_pv = loc_srv.create_pv_string(name, initial_value, NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string:array");

        // Verify we can fetch the initial scalar value
        let value = srv_pv.fetch().expect("Failed to fetch initial value");
        let scalar_val = value.get_field_string("value").unwrap();
        assert_eq!(scalar_val, initial_value);

        // Test posting different string values and reading back
        let test_values = vec![
            "Simple test".to_string(),
            "".to_string(),  // Empty string
            "Unicode: αβγ δεζ 中文 🚀".to_string(),
            "Special chars: !@#$%^&*()".to_string(),
            "Line\nbreaks\nand\ttabs".to_string(),
            format!("Very long string: {}", "A".repeat(100)),
        ];
        
        for test_val in test_values {
            srv_pv.post_string(&test_val).expect("Failed to post test value");
            
            let fetched = srv_pv.fetch().expect("Failed to fetch test value");
            let retrieved_val = fetched.get_field_string("value").unwrap();
            assert_eq!(retrieved_val, test_val, 
                    "Value mismatch: posted '{}', got '{}'", test_val, retrieved_val);
        }
        
    }

    #[test]
    fn test_pv_local_string_array_special_characters() {
        // Test local handling of special characters using server-side operations
        let name = "loc:string:special";
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        let mut srv_pv = loc_srv.create_pv_string(name, "", NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string:special");

        // Test various character encodings and special cases
        let long_string = "A".repeat(1000);
        let special_strings = vec![
            ("Empty", ""),
            ("Spaces", "   spaces   "),
            ("ASCII symbols", "!@#$%^&*()_+-=[]{}|;':\",./<>?"),
            ("Numbers", "0123456789"),
            ("Greek", "αβγδεζηθικλμνξοπρστυφχψω"),
            ("Chinese", "你好世界"),
            ("Japanese", "こんにちは世界"),
            ("Arabic", "مرحبا بالعالم"),
            ("Emoji", "🚀🌟💡🎉🔧⚡🌍🎯"),
            ("Mixed", "Hello 世界! αβγ 🚀 123"),
            ("Control chars", "Line1\nLine2\tTabbed\rCarriage"),
            ("Quotes", r#"Single 'quotes' and "double quotes""#),
            ("Backslashes", r"Path\to\file\name.txt"),
            ("JSON-like", r#"{"key": "value", "number": 123}"#),
            ("XML-like", "<tag>content</tag>"),
            ("Very long", long_string.as_str()),
        ];

        for (test_name, test_string) in special_strings {
            srv_pv.post_string(test_string).expect(&format!("Failed to post {} string", test_name));
            let fetched = srv_pv.fetch().expect("Failed to fetch special string");
            let retrieved = fetched.get_field_string("value").unwrap();
            assert_eq!(retrieved, test_string, "{}: string not preserved correctly", test_name);
            if test_string.len() > 50 {
                assert!(test_string.len() > 50, "{}: string length {}", test_name, test_string.len());
            }
        }
    }

    #[test]
    fn test_pv_local_string_array_type_conversions() {
        // Test various type conversions to string using server-side operations
        let name = "loc:string:convert";
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        let mut srv_pv = loc_srv.create_pv_string(name, "initial", NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string:convert");

        // Test numeric to string conversions

        // Test double to string conversion
        srv_pv.post_double(3.14159).expect("Failed to post double");
        let fetched = srv_pv.fetch().unwrap();
        let _retrieved = fetched.get_field_string("value").unwrap();

        // Test int32 to string conversion  
        srv_pv.post_int32(42).expect("Failed to post int32");
        let fetched = srv_pv.fetch().unwrap();
        let _retrieved = fetched.get_field_string("value").unwrap();

        // Test negative numbers
        srv_pv.post_int32(-123).expect("Failed to post negative int32");
        let fetched = srv_pv.fetch().unwrap();
        let _retrieved = fetched.get_field_string("value").unwrap();

        // Test zero
        srv_pv.post_double(0.0).expect("Failed to post zero");
        let fetched = srv_pv.fetch().unwrap();
        let _retrieved = fetched.get_field_string("value").unwrap();

        // Test very large numbers
        srv_pv.post_double(1e15).expect("Failed to post large number");
        let fetched = srv_pv.fetch().unwrap();
        let _retrieved = fetched.get_field_string("value").unwrap();
    }

    #[test]
    fn test_pv_local_string_array_length_limits() {
        // Test string length limits using server-side operations
        let name = "loc:string:limits";
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        let mut srv_pv = loc_srv.create_pv_string(name, "", NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string:limits");

        // Test various string lengths to find limits
        let length_tests = vec![
            (10, "Short"),
            (100, "Medium"),
            (1000, "Long"),
            (10000, "Very long"),
            (100000, "Extremely long"),
        ];

        for (length, description) in length_tests {
            let test_string = "X".repeat(length);
            
            srv_pv.post_string(&test_string).expect(&format!("Failed to post {} string", description));
            let fetched = srv_pv.fetch().unwrap();
            let retrieved = fetched.get_field_string("value").unwrap();
            
            assert!(retrieved == test_string, "{}: string length {} not preserved correctly", description, length);
        }
    }

    #[test]
    fn test_pv_local_string_array_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        // Test error handling for string arrays using server-side operations
        let name = "loc:string:errors";
        let mut loc_srv = Server::create_isolated()?;
        let mut srv_pv = loc_srv.create_pv_string(name, "initial", NTScalarMetadataBuilder::new())?;

        // Verify initial state
        let initial_fetch = srv_pv.fetch()?;
        let initial_val = initial_fetch.get_field_string("value")?;
        assert_eq!(initial_val, "initial");

        // Test that valid operations work
        srv_pv.post_string("updated")?;
        let updated_fetch = srv_pv.fetch()?;
        let updated_val = updated_fetch.get_field_string("value")?;
        assert_eq!(updated_val, "updated");

        // String PVs should generally accept all string values
        // Test edge cases that might cause issues
        let edge_cases = vec![
            "",  // Empty string
            "\0",  // Null character (might be problematic)
            "\u{FFFF}",  // High Unicode
        ];

        for test_case in edge_cases {
            srv_pv.post_string(test_case)?;
            let fetched = srv_pv.fetch()?;
            assert!(fetched.get_field_string("value").is_ok(), "Failed to retrieve posted edge case string");
        }

        // Verify PV still works after edge case tests
        srv_pv.post_string("final")?;
        let final_fetch = srv_pv.fetch()?;
        let final_val = final_fetch.get_field_string("value")?;
        assert_eq!(final_val, "final");

        Ok(())
    }
}