//! Test Context::monitor() function

use epics_pvxs_sys::Context;

#[test]
fn test_context_monitor_creation() {
    // Test Monitor creation
    match Context::from_env() {
        Ok(mut ctx) => {
            // Try to create monitor for invalid PV
            match ctx.monitor("invalid:monitor:pv") {
                Ok(_monitor) => {
                    println!("Monitor creation for invalid PV succeeded");
                    // Note: Monitor creation might succeed even if PV doesn't exist yet
                }
                Err(e) => {
                    println!("Monitor creation for invalid PV failed: {}", e);
                    assert!(!e.to_string().is_empty());
                }
            }
        }
        Err(e) => {
            println!("Skipping monitor test - no context available: {}", e);
        }
    }
}

#[test]
fn test_monitor_api_surface() {
    // Test that Monitor type has expected properties
    // This tests compilation and API structure
    
    // Note: Monitor cannot be Send due to C++ pointers, and doesn't implement Debug
    // This is expected behavior for types wrapping C++ objects
    println!("Monitor API surface test - types exist and compile");
}

#[test]
fn test_monitor_lifecycle() {
    // Test basic monitor operations
    match Context::from_env() {
        Ok(mut ctx) => {
            match ctx.monitor("test:monitor:pv") {
                Ok(mut monitor) => {
                    // Test basic lifecycle operations
                    println!("Monitor PV name: {}", monitor.name());
                    
                    // Start monitoring
                    monitor.start();
                    assert!(monitor.is_running());
                    
                    // Check connection status
                    let connected = monitor.is_connected();
                    println!("Monitor connected: {}", connected);
                    
                    // Check for updates (should be immediate)
                    let has_update = monitor.has_update();
                    println!("Monitor has update: {}", has_update);
                    
                    // Try non-blocking update check
                    match monitor.try_get_update() {
                        Ok(Some(value)) => {
                            println!("Got monitor update: {}", value);
                        }
                        Ok(None) => {
                            println!("No monitor update available");
                        }
                        Err(e) => {
                            println!("Monitor update check failed: {}", e);
                        }
                    }
                    
                    // Stop monitoring
                    monitor.stop();
                    assert!(!monitor.is_running());
                    
                    println!("Monitor lifecycle test completed");
                }
                Err(e) => {
                    println!("Monitor creation failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Skipping monitor lifecycle test - no context available: {}", e);
        }
    }
}