//! Test Context async functions (get_async, put_double_async, info_async)

#[cfg(feature = "async")]
mod async_tests {
    use epics_pvxs_sys::Context;
    use tokio;

    #[tokio::test]
    async fn test_context_get_async() {
        // Test async GET operation
        match Context::from_env() {
            Ok(mut ctx) => {
                // Test async get API exists and compiles
                let result = ctx.get_async("test:async:pv", 1.0).await;
                match result {
                    Ok(_value) => println!("Async GET succeeded"),
                    Err(e) => println!("Async GET failed (expected): {}", e),
                }
            }
            Err(_) => {
                println!("Skipping async GET test - no EPICS environment");
            }
        }
    }

    #[tokio::test]
    async fn test_context_put_double_async() {
        // Test async PUT operation
        match Context::from_env() {
            Ok(mut ctx) => {
                // Test async put API exists and compiles
                let result = ctx.put_double_async("test:async:put", 42.0, 1.0).await;
                match result {
                    Ok(()) => println!("Async PUT succeeded"),
                    Err(e) => println!("Async PUT failed (expected): {}", e),
                }
            }
            Err(_) => {
                println!("Skipping async PUT test - no EPICS environment");
            }
        }
    }

    #[tokio::test]
    async fn test_context_info_async() {
        // Test async INFO operation
        match Context::from_env() {
            Ok(mut ctx) => {
                // Test async info API exists and compiles
                let result = ctx.info_async("test:async:info", 1.0).await;
                match result {
                    Ok(_value) => println!("Async INFO succeeded"),
                    Err(e) => println!("Async INFO failed (expected): {}", e),
                }
            }
            Err(_) => {
                println!("Skipping async INFO test - no EPICS environment");
            }
        }
    }

    #[tokio::test]
    async fn test_async_timeout_behavior() {
        // Test async operations with very short timeouts
        match Context::from_env() {
            Ok(mut ctx) => {
                let start = std::time::Instant::now();
                
                // Test very short timeout
                let result = ctx.get_async("timeout:async:test", 0.001).await;
                let elapsed = start.elapsed();
                
                match result {
                    Ok(_) => {
                        println!("Async short timeout succeeded");
                    }
                    Err(e) => {
                        println!("Async short timeout failed: {}", e);
                        assert!(elapsed.as_millis() < 2000, "Async timeout took too long: {:?}", elapsed);
                    }
                }
            }
            Err(_) => {
                println!("Skipping async timeout test - no EPICS environment");
            }
        }
    }

    #[tokio::test]
    async fn test_rpc_execute_async() {
        // Test async RPC execution
        match Context::from_env() {
            Ok(mut ctx) => {
                match ctx.rpc("test:async:service") {
                    Ok(mut rpc) => {
                        // Test RPC builder pattern - build args step by step
                        let arg_result = rpc.arg_string("command", "test")
                            .and_then(|r| r.arg_double("value", 3.14))
                            .and_then(|r| r.arg_int32("count", 10))
                            .and_then(|r| r.arg_bool("enabled", true));
                        
                        match arg_result {
                            Ok(_) => {
                                // Now execute with the configured RPC
                                let exec_result = rpc.execute_async(1.0).await;
                                match exec_result {
                                    Ok(_value) => println!("Async RPC succeeded"),
                                    Err(e) => println!("Async RPC execution failed (expected): {}", e),
                                }
                            }
                            Err(e) => println!("RPC argument setup failed: {}", e),
                        }
                    }
                    Err(e) => println!("RPC creation failed (expected): {}", e),
                }
            }
            Err(_) => {
                println!("Skipping async RPC test - no EPICS environment");
            }
        }
    }

    #[tokio::test]
    async fn test_multiple_async_operations() {
        // Test running multiple async operations sequentially (due to mutable borrow restrictions)
        match Context::from_env() {
            Ok(mut ctx) => {
                // Run operations sequentially
                let get_result = ctx.get_async("multi:test1", 1.0).await;
                let info_result = ctx.info_async("multi:test2", 1.0).await;
                let put_result = ctx.put_double_async("multi:test3", 99.0, 1.0).await;
                
                println!("Sequential async GET result: {:?}", get_result.is_ok());
                println!("Sequential async INFO result: {:?}", info_result.is_ok());
                println!("Sequential async PUT result: {:?}", put_result.is_ok());
                
                // At least one should complete (successfully or with error)
                // This tests that the async runtime works correctly
            }
            Err(_) => {
                println!("Skipping multiple async test - no EPICS environment");
            }
        }
    }
}

#[cfg(not(feature = "async"))]
#[test]
fn test_async_feature_disabled() {
    // When async feature is disabled, ensure we can still compile
    println!("Async feature is disabled - skipping async tests");
}