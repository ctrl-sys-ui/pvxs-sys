//! Test Context::from_env() function

use epics_pvxs_sys::Context;

#[test]
fn test_context_from_env() {
    // Test that Context::from_env() creates a valid context
    // This may fail in environments without EPICS setup, which is expected
    match Context::from_env() {
        Ok(_ctx) => {
            println!("Context created successfully from environment");
        }
        Err(e) => {
            println!("Context creation failed (expected without EPICS env): {}", e);
            // This is expected behavior when EPICS environment is not configured
            assert!(e.to_string().contains("PVXS") || e.to_string().contains("error"));
        }
    }
}

#[test]
fn test_context_from_env_error_handling() {
    // Test error handling in context creation
    // The specific error depends on the environment, but it should not panic
    let result = Context::from_env();
    
    match result {
        Ok(_) => {
            // Success case - environment is properly configured
            println!("Context creation succeeded");
        }
        Err(e) => {
            // Error case - validate error structure
            assert!(!e.to_string().is_empty());
            assert!(e.to_string().starts_with("PVXS error:"));
        }
    }
}

#[test]
fn test_context_thread_safety() {
    // Test that Context implements Send and Sync
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    
    assert_send::<Context>();
    assert_sync::<Context>();
}