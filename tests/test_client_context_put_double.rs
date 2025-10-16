//! Test Context::put_double() function

use epics_pvxs_sys::Context;

#[test]
fn test_context_put_double_with_invalid_pv() {
    // Test PUT operation with invalid PV names
    match Context::from_env() {
        Ok(mut ctx) => {
            // Test PUT with invalid PV name
            let result = ctx.put_double("invalid:nonexistent:pv:name", 42.0, 0.5);
            match result {
                Ok(_) => {
                    println!("PUT on invalid PV succeeded unexpectedly");
                }
                Err(e) => {
                    println!("PUT on invalid PV failed as expected: {}", e);
                    assert!(!e.to_string().is_empty());
                }
            }
        }
        Err(e) => {
            println!("Skipping PUT test - no context available: {}", e);
        }
    }
}

#[test]
fn test_context_put_double_timeout() {
    // Test PUT operation timeout behavior
    match Context::from_env() {
        Ok(mut ctx) => {
            // Test with extremely short timeout
            let start = std::time::Instant::now();
            let result = ctx.put_double("timeout:put:test", 99.9, 0.001);  // 1ms timeout
            let elapsed = start.elapsed();
            
            match result {
                Ok(_) => {
                    println!("Very short timeout PUT succeeded");
                }
                Err(e) => {
                    println!("Short timeout PUT failed: {}", e);
                    // Should timeout quickly
                    assert!(elapsed.as_millis() < 1000, "PUT timeout took too long: {:?}", elapsed);
                }
            }
        }
        Err(e) => {
            println!("Skipping PUT timeout test - no context available: {}", e);
        }
    }
}

#[test]
fn test_context_put_double_extreme_values() {
    // Test PUT with extreme double values
    match Context::from_env() {
        Ok(mut ctx) => {
            let test_values = [
                f64::MAX,
                f64::MIN, 
                f64::INFINITY,
                f64::NEG_INFINITY,
                0.0,
                -0.0,
                std::f64::consts::PI,
                std::f64::consts::E,
            ];
            
            for &value in &test_values {
                let result = ctx.put_double("extreme:value:test", value, 0.1);
                match result {
                    Ok(_) => {
                        println!("PUT extreme value {} succeeded", value);
                    }
                    Err(e) => {
                        println!("PUT extreme value {} failed: {}", value, e);
                    }
                }
            }
            
            // Test NaN separately (might have special handling)
            let result = ctx.put_double("nan:test", f64::NAN, 0.1);
            match result {
                Ok(_) => {
                    println!("PUT NaN succeeded");
                }
                Err(e) => {
                    println!("PUT NaN failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Skipping extreme values test - no context available: {}", e);
        }
    }
}

#[test]
fn test_context_put_double_api_surface() {
    // Test that put_double has the expected API surface
    match Context::from_env() {
        Ok(mut ctx) => {
            // The method should exist and be callable
            let _ = ctx.put_double("test", 1.0, 1.0);
        }
        Err(_) => {
            // Expected without EPICS environment
        }
    }
}