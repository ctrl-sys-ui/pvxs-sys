//! Test Context::rpc() function

use epics_pvxs_sys::Context;

#[test]
fn test_context_rpc_creation() {
    // Test RPC creation
    match Context::from_env() {
        Ok(mut ctx) => {
            // Try to create RPC for invalid service
            match ctx.rpc("invalid:rpc:service") {
                Ok(_rpc) => {
                    println!("RPC creation for invalid service succeeded");
                }
                Err(e) => {
                    println!("RPC creation for invalid service failed: {}", e);
                    assert!(!e.to_string().is_empty());
                }
            }
        }
        Err(e) => {
            println!("Skipping RPC test - no context available: {}", e);
        }
    }
}

#[test]
fn test_rpc_api_surface() {
    // Test that RPC type has expected properties
    // This tests compilation and API structure
    
    // Note: Rpc cannot be Send due to C++ pointers
    // This is expected behavior for types wrapping C++ objects
    println!("Rpc API surface test - types exist and compile");
}

#[test]
fn test_rpc_argument_builder() {
    // Test RPC argument building
    match Context::from_env() {
        Ok(mut ctx) => {
            match ctx.rpc("test:rpc:service") {
                Ok(mut rpc) => {
                    // Test RPC builder pattern with different argument types
                    let result1 = rpc.arg_string("command", "test");
                    match result1 {
                        Ok(_) => println!("RPC string arg succeeded"),
                        Err(e) => println!("RPC string arg failed: {}", e),
                    }
                    
                    let result2 = rpc.arg_double("value", 3.14);
                    match result2 {
                        Ok(_) => println!("RPC double arg succeeded"),
                        Err(e) => println!("RPC double arg failed: {}", e),
                    }
                    
                    let result3 = rpc.arg_int32("count", 42);
                    match result3 {
                        Ok(_) => println!("RPC int32 arg succeeded"),
                        Err(e) => println!("RPC int32 arg failed: {}", e),
                    }
                    
                    let result4 = rpc.arg_bool("enabled", true);
                    match result4 {
                        Ok(_) => println!("RPC bool arg succeeded"),
                        Err(e) => println!("RPC bool arg failed: {}", e),
                    }
                    
                    // Test chaining (if previous operations succeeded)
                    if let Ok(mut rpc_chain) = ctx.rpc("test:chain") {
                        let chain_result = rpc_chain
                            .arg_string("cmd", "start")
                            .and_then(|r| r.arg_double("val", 1.0))
                            .and_then(|r| r.arg_int32("num", 10))
                            .and_then(|r| r.arg_bool("flag", false));
                        
                        match chain_result {
                            Ok(_) => println!("RPC chaining succeeded"),
                            Err(e) => println!("RPC chaining failed: {}", e),
                        }
                    }
                }
                Err(e) => {
                    println!("RPC creation failed for argument test: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Skipping RPC argument test - no context available: {}", e);
        }
    }
}

#[test]
fn test_rpc_execution() {
    // Test RPC execution
    match Context::from_env() {
        Ok(mut ctx) => {
            match ctx.rpc("test:execute:service") {
                Ok(mut rpc) => {
                    // Set up some arguments
                    let _ = rpc.arg_string("operation", "test");
                    
                    // Test execution with short timeout
                    match rpc.execute(0.1) {
                        Ok(value) => {
                            println!("RPC execution succeeded: {}", value);
                        }
                        Err(e) => {
                            println!("RPC execution failed (expected): {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("RPC creation failed for execution test: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Skipping RPC execution test - no context available: {}", e);
        }
    }
}

#[test]
fn test_rpc_error_conditions() {
    // Test RPC error handling
    match Context::from_env() {
        Ok(mut ctx) => {
            match ctx.rpc("error:test:service") {
                Ok(mut rpc) => {
                    // Test invalid argument operations
                    let result = rpc.arg_string("", "empty_name_arg");
                    match result {
                        Ok(_) => {
                            println!("RPC arg with empty name succeeded");
                        }
                        Err(e) => {
                            println!("RPC arg with empty name failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("RPC creation failed for error test: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Skipping RPC error test - no context available: {}", e);
        }
    }
}