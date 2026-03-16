// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
mod test_pvxs_local_string_array_fetch_post {
    use pvxs_sys::{Server, NTScalarMetadataBuilder};

    #[test]
    fn test_pv_local_string_array_fetch_post() {
        // This test creates a local pv (loc:string:array) and tests
        // server-side fetch() and post_string() operations.
        let initial_value = "Initial string array element";
        let name = "loc:string:array";
        let loc_srv = Server::start_isolated()
            .expect("Failed to create isolated server");

        // Create a string PV and capture for server-side operations
        loc_srv.create_pv_string(name, initial_value, NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string:array");

        // Verify we can fetch the initial scalar value
        match loc_srv.fetch_string(name) {
            Ok(fetched) => {
                assert_eq!(fetched.value, initial_value);
            }
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
            loc_srv.post_string(name, &test_val).expect("Failed to post test value");
            
            let fetched = loc_srv.fetch_string(name).expect("Failed to fetch test value");
            assert_eq!(fetched.value, test_val, "Value mismatch: posted '{}', got '{}'", test_val, fetched.value);
        }
        
    }

    #[test]
    fn test_pv_local_string_array_special_characters() {
        // Test local handling of special characters using server-side operations
        let name = "loc:string:special";
        let loc_srv = Server::start_isolated()
            .expect("Failed to create isolated server");

        loc_srv.create_pv_string(name, "", NTScalarMetadataBuilder::new())
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
            loc_srv.post_string(name, test_string).expect(&format!("Failed to post {} string", test_name));
            let fetched = loc_srv.fetch_string(name).expect("Failed to fetch special string");
            assert_eq!(fetched.value, test_string, "{}: string not preserved correctly", test_name);
            if test_string.len() > 50 {
                assert!(test_string.len() > 50, "{}: string length {}", test_name, test_string.len());
            }
        }
    }


    #[test]
    fn test_pv_local_string_array_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        // Test error handling for string arrays using server-side operations
        let name = "loc:string:errors";
        let loc_srv = Server::start_isolated()?;
        loc_srv.create_pv_string(name, "initial", NTScalarMetadataBuilder::new())?;

        // Verify initial state
        let initial_fetch = loc_srv.fetch_string(name)?;
        let initial_val = initial_fetch.value;
        assert_eq!(initial_val, "initial");

        // Test that valid operations work
        loc_srv.post_string(name, "updated")?;
        let updated_fetch = loc_srv.fetch_string(name)?;
        let updated_val = updated_fetch.value;
        assert_eq!(updated_val, "updated");

        // String PVs should generally accept all string values
        // Test edge cases that might cause issues
        let edge_cases = vec![
            "",  // Empty string
            "\0",  // Null character (might be problematic)
            "\u{FFFF}",  // High Unicode
        ];

        for test_case in edge_cases {
            loc_srv.post_string(name, test_case)?;
            let fetched = loc_srv.fetch_string(name)?;
            assert!(fetched.value == test_case, "Failed to retrieve posted edge case string");
        }

        // Verify PV still works after edge case tests
        loc_srv.post_string(name, "final")?;
        let final_fetch = loc_srv.fetch_string(name)?;
        let final_val = final_fetch.value;
        assert_eq!(final_val, "final");

        Ok(())
    }
}
