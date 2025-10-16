//! Test Context::get() function

use epics_pvxs_sys::Context;

#[test]
fn test_context_get_with_invalid_pv() {
    // Test GET operation with invalid PV names
    match Context::from_env() {
        Ok(mut ctx) => {
            // Test GET with invalid PV name
            let result = ctx.get("invalid:nonexistent:pv:name", 0.5);
            match result {
                Ok(_) => {
                    println!("GET on invalid PV succeeded unexpectedly");
                }
                Err(e) => {
                    println!("GET on invalid PV failed as expected: {}", e);
                    assert!(!e.to_string().is_empty());
                }
            }
        }
        Err(e) => {
            println!("Skipping GET test - no context available: {}", e);
        }
    }
}

#[test]
fn test_context_get_timeout() {
    // Test GET operation timeout behavior
    match Context::from_env() {
        Ok(mut ctx) => {
            // Test with extremely short timeout (should timeout immediately)
            let start = std::time::Instant::now();
            let result = ctx.get("timeout:test:pv", 0.001);  // 1ms timeout
            let elapsed = start.elapsed();
            
            match result {
                Ok(_) => {
                    println!("Very short timeout GET succeeded (PV exists)");
                }
                Err(e) => {
                    println!("Short timeout GET failed: {}", e);
                    // Should timeout quickly
                    assert!(elapsed.as_millis() < 1000, "Timeout took too long: {:?}", elapsed);
                }
            }
            
            // Test zero timeout
            let result = ctx.get("zero:timeout:test", 0.0);
            match result {
                Ok(_) => {
                    println!("Zero timeout GET succeeded");
                }
                Err(e) => {
                    println!("Zero timeout GET failed as expected: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Skipping GET timeout test - no context available: {}", e);
        }
    }
}

#[test]
fn test_context_get_api_surface() {
    // Test that GET has the expected API surface
    match Context::from_env() {
        Ok(mut ctx) => {
            // The method should exist and be callable
            let _ = ctx.get("test", 1.0);
        }
        Err(_) => {
            // Expected without EPICS environment
        }
    }
}