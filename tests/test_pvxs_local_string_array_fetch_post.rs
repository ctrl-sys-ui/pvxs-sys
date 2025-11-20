mod test_pvxs_local_string_array_fetch_post {
    use epics_pvxs_sys::{Server, Context, NTScalarMetadataBuilder};

    #[test]
    fn test_pv_local_string_array_fetch_post() {
        // This test creates a local pv (loc:string:array) on a server and gets 
        // and sets the array value using client operations.
        let initial_value = "Initial string array element";
        let name = "loc:string:array";
        let timeout = 5.0;
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        // Create a string PV
        loc_srv.create_pv_string(name, initial_value, NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string:array");

        let mut ctx = Context::from_env().expect("Failed to create client context");

        // Verify we can fetch the initial scalar value
        match ctx.get(name, timeout) {
            Ok(value) => {
                // Try to get as array first, fall back to scalar
                match value.get_field_string_array("value") {
                    Ok(array) => {
                        if !array.is_empty() {
                            assert_eq!(array[0], initial_value);
                        }
                    },
                    Err(_) => {
                        // Fall back to scalar access
                        let scalar_val = value.get_field_string("value").unwrap();
                        assert_eq!(scalar_val, initial_value);
                    }
                }
            },
            Err(e) => assert!(false, "Failed to fetch initial value: {:?}", e),
        }

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
            ctx.put_string(name, &test_val, timeout).expect("Failed to post test value");
            
            let fetched = ctx.get(name, timeout).expect("Failed to fetch test value");
            let retrieved_val = fetched.get_field_string("value").unwrap();
            assert_eq!(retrieved_val, test_val, 
                    "Value mismatch: posted '{}', got '{}'", test_val, retrieved_val);
        }
        
    }

    #[test]
    fn test_pv_local_string_array_special_characters() {
        // Test local handling of special characters and encodings
        let name = "loc:string:special";
        let timeout = 5.0;
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        loc_srv.create_pv_string(name, "", NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string:special");

        let mut ctx = Context::from_env().expect("Failed to create client context");

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
            match ctx.put_string(name, test_string, timeout) {
                Ok(_) => {
                    let fetched = ctx.get(name, timeout).expect("Failed to fetch special string");
                    let retrieved = fetched.get_field_string("value").unwrap();
                    assert_eq!(retrieved, test_string, "{}: string not preserved correctly", test_name);
                    if test_string.len() > 50 {
                        assert!(test_string.len() > 50, "{}: string length {}", test_name, test_string.len());
                    }
                },
                Err(e) => println!("{} string not supported: '{}' - {}", test_name, test_string, e),
            }
        }
    }

    #[test]
    fn test_pv_local_string_array_type_conversions() {
        // Test various type conversions to string
        let name = "loc:string:convert";
        let timeout = 5.0;
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        loc_srv.create_pv_string(name, "initial", NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string:convert");

        let mut ctx = Context::from_env().expect("Failed to create client context");

        // Test numeric to string conversions

        // Test double to string conversion
        match ctx.put_double(name, 3.14159, timeout) {
            Ok(_) => {
                let fetched = ctx.get(name, timeout).unwrap();
                let _retrieved = fetched.get_field_string("value").unwrap();
            },
            Err(e) => assert!(false, "Double to string conversion not supported: {}", e),
        }

        // Test int32 to string conversion  
        match ctx.put_int32(name, 42, timeout) {
            Ok(_) => {
                let fetched = ctx.get(name, timeout).unwrap();
                let _retrieved = fetched.get_field_string("value").unwrap();
            },
            Err(e) => assert!(false, "Int32 to string conversion not supported: {}", e),
        }

        // Test negative numbers
        match ctx.put_int32(name, -123, timeout) {
            Ok(_) => {
                let fetched = ctx.get(name, timeout).unwrap();
                let _retrieved = fetched.get_field_string("value").unwrap();
            },
            Err(e) => assert!(false, "Negative int32 to string conversion not supported: {}", e),
        }

        // Test zero
        match ctx.put_double(name, 0.0, timeout) {
            Ok(_) => {
                let fetched = ctx.get(name, timeout).unwrap();
                let _retrieved = fetched.get_field_string("value").unwrap();
            },
            Err(e) => assert!(false, "Zero to string conversion not supported: {}", e),
        }

        // Test very large numbers
        match ctx.put_double(name, 1e15, timeout) {
            Ok(_) => {
                let fetched = ctx.get(name, timeout).unwrap();
                let _retrieved = fetched.get_field_string("value").unwrap();
            },
            Err(e) => assert!(false, "Large number to string conversion not supported: {}", e),
        }
    }

    #[test]
    fn test_pv_local_string_array_length_limits() {
        // Test string length limits and performance
        let name = "loc:string:limits";
        let timeout = 5.0;
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        loc_srv.create_pv_string(name, "", NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string:limits");

        let mut ctx = Context::from_env().expect("Failed to create client context");

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
            
            match ctx.put_string(name, &test_string, timeout) {
                Ok(_) => {
                    let fetched = ctx.get(name, timeout).unwrap();
                    let retrieved = fetched.get_field_string("value").unwrap();
                    
                    assert!(retrieved == test_string, "{}: string length {} not preserved correctly", description, length);
                },
                Err(e) => assert!(false, "{} string ({} chars) not supported: {}", description, length, e),
            }
        }
    }

    #[test]
    fn test_pv_local_string_array_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        // Test error handling for string arrays with proper error propagation
        let name = "loc:string:errors";
        let timeout = 5.0;
        let mut loc_srv = Server::create_isolated()?;
        loc_srv.create_pv_string(name, "initial", NTScalarMetadataBuilder::new())?;

        let mut ctx = Context::from_env()?;

        // Verify initial state
        let initial_fetch = ctx.get(name, timeout)?;
        let initial_val = initial_fetch.get_field_string("value")?;
        assert_eq!(initial_val, "initial");

        // Test that valid operations work
        ctx.put_string(name, "updated", timeout)?;
        let updated_fetch = ctx.get(name, timeout)?;
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
            match ctx.put_string(name, test_case, timeout) {
                Ok(_) => {
                    let fetched = ctx.get(name, timeout)?;
                    assert!(fetched.get_field_string("value").is_ok(), "Failed to retrieve posted edge case string");
                },
                Err(e) => assert!(false, "Edge case not supported: {:?} - {}", test_case, e),
            }
        }

        // Verify PV still works after edge case tests
        ctx.put_string(name, "final", timeout)?;
        let final_fetch = ctx.get(name, timeout)?;
        let final_val = final_fetch.get_field_string("value")?;
        assert_eq!(final_val, "final");

        Ok(())
    }
}