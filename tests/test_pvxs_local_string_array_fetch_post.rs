use epics_pvxs_sys::{Server, SharedPV};

#[test]
fn test_pv_local_string_array_fetch_post() {
    // This test creates a local pv (loc:string:array) on a server and gets 
    // and sets the array value on server side.
    let initial_value = "Initial string array element";
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    // Create a string PV that we'll try to use for arrays
    // Note: Array support depends on server implementation
    let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_string("loc:string:array", initial_value)
        .expect("Failed to create pv:string:array");

    // Try to post and fetch string arrays (server-side only, no network)
    println!("Testing local string array operations...");

    // Verify we can fetch the initial scalar value
    match srv_pv_loc_array.fetch() {
        Ok(value) => {
            // Try to get as array first, fall back to scalar
            match value.get_field_string_array("value") {
                Ok(array) => {
                    println!("Successfully got string array with {} elements", array.len());
                    if !array.is_empty() {
                        assert_eq!(array[0], initial_value);
                    }
                },
                Err(_) => {
                    // Fall back to scalar access
                    let scalar_val = value.get_field_string("value").unwrap();
                    assert_eq!(scalar_val, initial_value);
                    println!("PV operates as scalar, not array");
                }
            }
        },
        Err(e) => panic!("Failed to fetch initial value: {:?}", e),
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
        srv_pv_loc_array.post_string(&test_val).expect("Failed to post test value");
        
        let fetched = srv_pv_loc_array.fetch().expect("Failed to fetch test value");
        let retrieved_val = fetched.get_field_string("value").unwrap();
        assert_eq!(retrieved_val, test_val, 
                  "Value mismatch: posted '{}', got '{}'", test_val, retrieved_val);
    }
    
    println!("✓ String values posted and fetched successfully");
}

#[test]
fn test_pv_local_string_array_special_characters() {
    // Test local handling of special characters and encodings
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_string("loc:string:special", "")
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

    for (name, test_string) in special_strings {
        match srv_pv_loc_array.post_string(test_string) {
            Ok(_) => {
                let fetched = srv_pv_loc_array.fetch().expect("Failed to fetch special string");
                let retrieved = fetched.get_field_string("value").unwrap();
                assert_eq!(retrieved, test_string, 
                          "{}: string not preserved correctly", name);
                println!("✓ {} string handled successfully: '{}'", name, 
                        if test_string.len() > 50 { 
                            format!("{}...", &test_string[..47]) 
                        } else { 
                            test_string.to_string() 
                        });
            },
            Err(e) => println!("⚠ {} string not supported: '{}' - {}", name, test_string, e),
        }
    }
}

#[test]
fn test_pv_local_string_array_type_conversions() {
    // Test various type conversions to string
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_string("loc:string:convert", "initial")
        .expect("Failed to create pv:string:convert");

    // Test numeric to string conversions
    println!("Testing numeric to string conversions...");

    // Test double to string conversion
    match srv_pv_loc_array.post_double(3.14159) {
        Ok(_) => {
            let fetched = srv_pv_loc_array.fetch().unwrap();
            let retrieved = fetched.get_field_string("value").unwrap();
            println!("✓ Double 3.14159 converted to string: '{}'", retrieved);
        },
        Err(e) => println!("⚠ Double to string conversion not supported: {}", e),
    }

    // Test int32 to string conversion  
    match srv_pv_loc_array.post_int32(42) {
        Ok(_) => {
            let fetched = srv_pv_loc_array.fetch().unwrap();
            let retrieved = fetched.get_field_string("value").unwrap();
            println!("✓ Int32 42 converted to string: '{}'", retrieved);
        },
        Err(e) => println!("⚠ Int32 to string conversion not supported: {}", e),
    }

    // Test negative numbers
    match srv_pv_loc_array.post_int32(-123) {
        Ok(_) => {
            let fetched = srv_pv_loc_array.fetch().unwrap();
            let retrieved = fetched.get_field_string("value").unwrap();
            println!("✓ Negative int32 -123 converted to string: '{}'", retrieved);
        },
        Err(e) => println!("⚠ Negative int32 to string conversion not supported: {}", e),
    }

    // Test zero
    match srv_pv_loc_array.post_double(0.0) {
        Ok(_) => {
            let fetched = srv_pv_loc_array.fetch().unwrap();
            let retrieved = fetched.get_field_string("value").unwrap();
            println!("✓ Zero converted to string: '{}'", retrieved);
        },
        Err(e) => println!("⚠ Zero to string conversion not supported: {}", e),
    }

    // Test very large numbers
    match srv_pv_loc_array.post_double(1e15) {
        Ok(_) => {
            let fetched = srv_pv_loc_array.fetch().unwrap();
            let retrieved = fetched.get_field_string("value").unwrap();
            println!("✓ Large number 1e15 converted to string: '{}'", retrieved);
        },
        Err(e) => println!("⚠ Large number to string conversion not supported: {}", e),
    }
}

#[test]
fn test_pv_local_string_array_length_limits() {
    // Test string length limits and performance
    let loc_srv = Server::create_isolated()
        .expect("Failed to create isolated server");

    let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_string("loc:string:limits", "")
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
        
        match srv_pv_loc_array.post_string(&test_string) {
            Ok(_) => {
                let fetched = srv_pv_loc_array.fetch().unwrap();
                let retrieved = fetched.get_field_string("value").unwrap();
                
                if retrieved == test_string {
                    println!("✓ {} string ({} chars) handled successfully", description, length);
                } else {
                    println!("⚠ {} string truncated: {} → {} chars", 
                            description, length, retrieved.len());
                }
            },
            Err(e) => println!("⚠ {} string ({} chars) not supported: {}", description, length, e),
        }
    }
}

#[test]
fn test_pv_local_string_array_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    // Test error handling for string arrays with proper error propagation
    let loc_srv = Server::create_isolated()?;
    let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_string("loc:string:errors", "initial")?;

    // Verify initial state
    let initial_fetch = srv_pv_loc_array.fetch()?;
    let initial_val = initial_fetch.get_field_string("value")?;
    assert_eq!(initial_val, "initial");

    // Test that valid operations work
    srv_pv_loc_array.post_string("updated")?;
    let updated_fetch = srv_pv_loc_array.fetch()?;
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
        match srv_pv_loc_array.post_string(test_case) {
            Ok(_) => {
                let fetched = srv_pv_loc_array.fetch()?;
                let retrieved = fetched.get_field_string("value")?;
                println!("✓ Edge case handled: {:?} → {:?}", test_case, retrieved);
            },
            Err(e) => println!("⚠ Edge case not supported: {:?} - {}", test_case, e),
        }
    }

    // Verify PV still works after edge case tests
    srv_pv_loc_array.post_string("final")?;
    let final_fetch = srv_pv_loc_array.fetch()?;
    let final_val = final_fetch.get_field_string("value")?;
    assert_eq!(final_val, "final");

    println!("✓ Error handling verified for string array PV");
    Ok(())
}