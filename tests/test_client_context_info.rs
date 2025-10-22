//! Test Context::info() function

use epics_pvxs_sys::Context;

#[test]
fn test_context_info_with_invalid_pv() {
    // Test INFO operation with invalid PV names
    match Context::from_env() {
        Ok(mut ctx) => {
            // Test INFO with invalid PV name
            let result = ctx.info("invalid:nonexistent:pv:name", 0.5);
            match result {
                Ok(_) => {
                    println!("INFO on invalid PV succeeded unexpectedly");
                }
                Err(e) => {
                    println!("INFO on invalid PV failed as expected: {}", e);
                    assert!(!e.to_string().is_empty());
                }
            }
        }
        Err(e) => {
            println!("Skipping INFO test - no context available: {}", e);
        }
    }
}

#[test]
fn test_context_info_timeout() {
    // Test INFO operation timeout behavior
    match Context::from_env() {
        Ok(mut ctx) => {
            // Test with extremely short timeout
            let start = std::time::Instant::now();
            let result = ctx.info("timeout:info:test", 0.001);  // 1ms timeout
            let elapsed = start.elapsed();
            
            match result {
                Ok(_) => {
                    println!("Very short timeout INFO succeeded");
                }
                Err(e) => {
                    println!("Short timeout INFO failed: {}", e);
                    // Should timeout quickly
                    assert!(elapsed.as_millis() < 1000, "INFO timeout took too long: {:?}", elapsed);
                }
            }
        }
        Err(e) => {
            println!("Skipping INFO timeout test - no context available: {}", e);
        }
    }
}

#[test]
fn test_context_info_api_surface() {
    // Test that INFO has the expected API surface
    match Context::from_env() {
        Ok(mut ctx) => {
            // The method should exist and be callable
            let _ = ctx.info("test", 1.0);
        }
        Err(_) => {
            // Expected without EPICS environment
        }
    }
}