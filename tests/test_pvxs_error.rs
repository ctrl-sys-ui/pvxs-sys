//! Test PvxsError functionality

use epics_pvxs_sys::{PvxsError, Context, SharedPV};
use std::error::Error;

#[test]
fn test_pvxs_error_new() {
    // Test creating PvxsError with new()
    let error = PvxsError::new("test error message");
    
    assert_eq!(error.to_string(), "PVXS error: test error message");
    
    // Test with different message types
    let string_msg = String::from("string message");
    let error_from_string = PvxsError::new(string_msg);
    assert_eq!(error_from_string.to_string(), "PVXS error: string message");
    
    let error_from_str = PvxsError::new("str message");
    assert_eq!(error_from_str.to_string(), "PVXS error: str message");
}

#[test]
fn test_pvxs_error_display() {
    // Test Display trait implementation
    let error = PvxsError::new("display test");
    
    let display_string = format!("{}", error);
    assert_eq!(display_string, "PVXS error: display test");
    
    // Test with special characters
    let special_error = PvxsError::new("Error with \n newlines and \t tabs");
    let special_display = format!("{}", special_error);
    assert!(special_display.contains("newlines"));
    assert!(special_display.contains("tabs"));
}

#[test]
fn test_pvxs_error_debug() {
    // Test Debug trait implementation
    let error = PvxsError::new("debug test");
    
    let debug_string = format!("{:?}", error);
    assert!(debug_string.contains("PvxsError"));
    assert!(debug_string.contains("debug test"));
    
    println!("PvxsError debug: {}", debug_string);
}

#[test]
fn test_pvxs_error_clone() {
    // Test Clone trait implementation
    let original = PvxsError::new("original error");
    let cloned = original.clone();
    
    assert_eq!(original.to_string(), cloned.to_string());
    
    // Verify they're independent
    let another_clone = cloned.clone();
    assert_eq!(cloned.to_string(), another_clone.to_string());
}

#[test]
fn test_pvxs_error_as_std_error() {
    // Test that PvxsError implements std::error::Error
    let error = PvxsError::new("std error test");
    
    // Convert to trait object
    let error_trait: &dyn Error = &error;
    
    assert!(!error_trait.to_string().is_empty());
    
    // Test source() method (should return None for our simple error)
    assert!(error_trait.source().is_none());
}

#[test]
fn test_pvxs_error_from_operations() {
    // Test errors generated from actual PVXS operations
    
    // Test Context creation error (if environment is not set up)
    match Context::from_env() {
        Ok(_) => {
            println!("Context creation succeeded - environment configured");
        }
        Err(e) => {
            println!("Context creation error: {}", e);
            
            // Verify error properties
            assert!(!e.to_string().is_empty());
            assert!(e.to_string().starts_with("PVXS error:"));
            
            // Test as std::error::Error
            let error_trait: &dyn Error = &e;
            assert!(!error_trait.to_string().is_empty());
        }
    }
    
    // Test SharedPV operation error
    let mut pv = SharedPV::create_mailbox()
        .expect("Failed to create PV for error test");
    
    // Try to post to unopened PV (should generate error)
    match pv.post_double(42.0) {
        Ok(_) => {
            println!("Post to unopened PV succeeded unexpectedly");
        }
        Err(e) => {
            println!("Post to unopened PV error: {}", e);
            
            // Verify error properties
            assert!(!e.to_string().is_empty());
            assert!(e.to_string().starts_with("PVXS error:"));
        }
    }
}

#[test]
fn test_pvxs_error_empty_message() {
    // Test with empty error message
    let empty_error = PvxsError::new("");
    assert_eq!(empty_error.to_string(), "PVXS error: ");
    
    // Test with whitespace-only message
    let whitespace_error = PvxsError::new("   ");
    assert_eq!(whitespace_error.to_string(), "PVXS error:    ");
}

#[test]
fn test_pvxs_error_long_message() {
    // Test with very long error message
    let long_message = "A".repeat(1000);
    let long_error = PvxsError::new(&long_message);
    
    let error_string = long_error.to_string();
    assert!(error_string.starts_with("PVXS error:"));
    assert!(error_string.len() > 1000);
    
    println!("Long error length: {}", error_string.len());
}

#[test]
fn test_pvxs_error_unicode() {
    // Test with Unicode characters
    let unicode_error = PvxsError::new("Error with Unicode: 测试 🚀 ñoño");
    let error_string = unicode_error.to_string();
    
    assert!(error_string.contains("测试"));
    assert!(error_string.contains("🚀"));
    assert!(error_string.contains("ñoño"));
    
    println!("Unicode error: {}", error_string);
}

#[test]
fn test_result_type_alias() {
    // Test our Result<T> type alias
    use epics_pvxs_sys::Result;
    
    // Function returning success
    fn success_function() -> Result<i32> {
        Ok(42)
    }
    
    // Function returning error  
    fn error_function() -> Result<i32> {
        Err(PvxsError::new("test error"))
    }
    
    // Test success case
    match success_function() {
        Ok(value) => {
            assert_eq!(value, 42);
            println!("Success function returned: {}", value);
        }
        Err(_) => panic!("Success function should not return error"),
    }
    
    // Test error case
    match error_function() {
        Ok(_) => panic!("Error function should not return Ok"),
        Err(e) => {
            assert_eq!(e.to_string(), "PVXS error: test error");
            println!("Error function returned: {}", e);
        }
    }
    
    // Test ? operator works with our Result type
    fn chained_function() -> Result<i32> {
        let _val = success_function()?;
        error_function()?;
        Ok(100) // Should not reach here
    }
    
    match chained_function() {
        Ok(_) => panic!("Chained function should return error"),
        Err(e) => {
            println!("Chained function error: {}", e);
        }
    }
}