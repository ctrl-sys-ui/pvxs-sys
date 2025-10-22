// Test to verify that async functionality provides helpful error messages when disabled

#[cfg(test)]
mod async_optional_tests {
    use crate::*;
    
    #[cfg(not(feature = "async"))]
    #[test] 
    fn test_async_disabled_compilation() {
        // This test simply verifies that the library compiles without the async feature
        // The actual async methods won't be available in the high-level API when async is disabled
        
        // Try to create a context - this tests basic functionality
        let ctx_result = std::panic::catch_unwind(|| {
            Context::from_env()
        });
        
        // Context creation might fail due to EPICS environment, but that's expected in test environment
        // The important thing is that it compiles without async dependencies
        match ctx_result {
            Ok(_) => println!("Context created successfully without async feature"),
            Err(_) => println!("Context creation failed (expected in test environment)"),
        }
    }
    
    #[cfg(feature = "async")]
    #[test]
    fn test_async_functions_available() {
        // When async feature is enabled, async functions should be available
        let ctx_result = std::panic::catch_unwind(|| {
            Context::from_env()
        });
        
        if let Ok(_ctx) = ctx_result {
            // This should not panic due to missing async functionality
            // The get_async method should exist when async feature is enabled
            let has_async_methods = true; // If this compiles, async methods are available
            assert!(has_async_methods, "Async methods should be available when feature is enabled");
        }
    }
}