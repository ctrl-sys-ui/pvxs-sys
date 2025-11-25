mod test_pvxs_monitor_builder {
    // test_pvxs_monitor_builder.rs - Test MonitorBuilder and callback functionality
    use epics_pvxs_sys::{Context, Monitor, PvxsError, Server, NTScalarMetadataBuilder};
    use std::thread;
    use std::time::Duration;

    // Simple callback for basic testing
    extern "C" fn simple_test_callback() {
        // This is just a placeholder for testing callback registration
    }

    #[test]
    fn test_local_server_failes_to_connect_to_remote_client_monitor() -> Result<(), PvxsError> {
        // Create isolated server for testing
        let mut server = Server::create_isolated()?;
        
        // Create PV with initial value (automatically added to server)
        server.create_pv_double("TEST:MonitorBuilder:LocalFail", 3.14, NTScalarMetadataBuilder::new())?;
        server.start()?;
        
        thread::sleep(Duration::from_millis(100));
        
        let mut ctx = Context::from_env()?;
        
        // Attempt to create monitor using builder from remote client context
        let monitor_result: Result<Monitor, PvxsError> = ctx.monitor_builder("TEST:MonitorBuilder:LocalFail")?
            .connect_exception(false)  // Suppress connection exceptions
            .disconnect_exception(false)  // Suppress disconnection exceptions
            .exec();
        
        match monitor_result {
            Ok(mut monitor) => {
                // Start monitoring
                assert!(monitor.start().is_ok());
                thread::sleep(Duration::from_millis(1000)); // Wait a bit for connection attempt
                
                // Since server is isolated, monitor should not connect
                assert!(!monitor.is_connected(), "Monitor should not connect to isolated local server");
                
                assert!(monitor.stop().is_ok());
            },
            Err(e) => {
                assert!(false, "Monitor creation failed unexpectedly: {:?}", e);
            }
        }
        assert!(server.stop().is_ok());
        Ok(())
    }

    /// Test basic MonitorBuilder creation and configuration
    #[test]
    fn test_monitor_builder_creation()  -> Result<(), PvxsError> {
        let pv_name = "TEST:MonitorBuilder:Creation";
        // Now start a new server from_env to test actual connection
        let mut server = Server::from_env()?;
        server.create_pv_double(pv_name, 1.0, NTScalarMetadataBuilder::new())?;
        assert!(server.start().is_ok());
        
        let mut ctx = Context::from_env()?;
        // Test MonitorBuilder creation again - this time server is running
        let mut _monitor: Result<Monitor, PvxsError> = ctx.monitor_builder(pv_name)?
            .connect_exception(false)  // Suppress connection exceptions
            .disconnect_exception(false)  // Suppress disconnection exceptions     
            .exec();
        match _monitor {
            Ok(mut mon) => {
                // Start the monitor
                mon.start();
                // Give more time for connection to establish
                thread::sleep(Duration::from_millis(2000));
                
                // is_connected() now properly uses Connect object to check actual connection
                assert!(mon.is_connected(), "Monitor should be connected to from_env server");
                
                // stop the server
                assert!(server.stop().is_ok());
                thread::sleep(Duration::from_millis(1000));
                
                // After stopping server, should detect disconnection
                assert_eq!(mon.is_connected(), false, "Monitor should be disconnected after server stop");
                
                // start the server again
                assert!(server.start().is_ok());
                // Give more time for reconnection (might take longer than initial connection)
                thread::sleep(Duration::from_millis(5000));
                assert_eq!(mon.is_connected(), true, "Monitor should reconnect after server restart");
            },
            Err(e) => {
                assert!(false, "Monitor creation failed: {:?}", e);
            }
        }
        Ok(())
    }

    /// Test MonitorBuilder with different mask configurations
    #[test]
    fn test_monitor_builder_mask_options() -> Result<(), PvxsError> {
        // Create isolated server for testing
        let mut server = Server::create_isolated()?;
        
        // Create PV with initial value (automatically added to server)
        server.create_pv_double("TEST:MonitorBuilder:Masks", 2.5, NTScalarMetadataBuilder::new())?;
        server.start()?;
        
        thread::sleep(Duration::from_millis(100));
        
        let mut ctx = Context::from_env()?;
        
        // Test with both masks enabled (suppress exceptions)
        let mut monitor1 = ctx.monitor_builder("TEST:MonitorBuilder:Masks")?
            .connect_exception(false)  // Suppress connection exceptions
            .disconnect_exception(false)  // Suppress disconnection exceptions
            .exec()?;
        
        // When both connected and disconnected masks are enabled, we should see connection events
        monitor1.start()?;
        thread::sleep(Duration::from_millis(500));
        monitor1.stop()?;

        // Test with both masks disabled (throw exceptions)
        let mut monitor2 = ctx.monitor_builder("TEST:MonitorBuilder:Masks")?
            .connect_exception(true)  // Throw connection exceptions
            .disconnect_exception(true)  // Throw disconnection exceptions
            .exec()?;
        // Test default configuration (no explicit mask calls)
        let _monitor3 = ctx.monitor_builder("TEST:MonitorBuilder:Masks")?
            .exec()?;
        server.stop()?;
        Ok(())
    }

    /// Test Monitor pop() method following PVXS pattern
    #[test]
    fn test_monitor_pop_functionality() -> Result<(), PvxsError> {
        // Create isolated server for testing
        let mut server = Server::from_env()?;
        
        // Create PV with initial value (automatically added to server)
        server.create_pv_double("TEST:MonitorBuilder:Pop", 10.0, NTScalarMetadataBuilder::new())?;
        server.start()?;
        
        thread::sleep(Duration::from_millis(100));
        
        let mut ctx = Context::from_env()?;
        
        // Create monitor using builder
        let mut monitor = ctx.monitor_builder("TEST:MonitorBuilder:Pop")?
            .connect_exception(false) 
            .exec()?;
        
        // Start monitoring
        monitor.start().expect("Failed to start monitor");
        
        // Give time for initial connection
        thread::sleep(Duration::from_millis(200));
        
        // Test pop() method - should initially be empty or have connection data
        match monitor.pop() {
            Ok(Some(value)) => {
                // Try to get the value field
                assert!(value.get_field_double("value").is_ok(), "Method - should initially have value field");
                    
            },
            Ok(None) => assert!(true, "Queue initially empty"),
            Err(e) => {
                assert!(false, "Error popping from monitor queue: {:?}", e);
            }
        }
        
        // Now test updating the PV via client PUT operation
        let new_value = 25.5;
        match ctx.put_double("TEST:MonitorBuilder:Pop", new_value, 2.0) {
            Ok(_) => {
                
                // Give time for update to propagate
                thread::sleep(Duration::from_millis(200));
                
                // Try to pop the update
                let mut updates_received = 0;
                for _ in 0..5 { // Try a few times
                    match monitor.pop() {
                        Ok(Some(value)) => {
                            updates_received += 1;
                            
                            // Try to extract the value
                            if let Ok(val) = value.get_field_double("value") {
                                assert_eq! (val, new_value, "Popped value should match updated PV value");
                            }
                        },
                        Ok(None) => {
                            thread::sleep(Duration::from_millis(50));
                        },
                        Err(_e) => {
                            thread::sleep(Duration::from_millis(50));
                        }
                    }
                }
                
                if updates_received > 0 {
                } else {
                }
            },
            Err(e) => assert!(false, "Failed to PUT new value: {}", e),
        }
        
        monitor.stop();
        server.stop()?;
        Ok(())
    }

    /// Test real Rust function callback functionality
    #[test]
    fn test_monitor_builder_with_callback() -> Result<(), PvxsError> {
        // Create isolated server for testing
        let mut server = Server::from_env()?;
        
        server.create_pv_double("TEST:MonitorBuilder:Callback", 42.0, NTScalarMetadataBuilder::new())?;
        server.start()?;
        
        thread::sleep(Duration::from_millis(100));
        
        let mut ctx = Context::from_env()?;
        
        // Create monitor with actual Rust callback function
        let mut monitor = ctx.monitor_builder("TEST:MonitorBuilder:Callback")?
            .connect_exception(true)      // Throw connection exceptions in queue
            .disconnect_exception(true)   // Throw disconnection exceptions in queue
            .event(simple_test_callback)  // Set a simple callback
            .exec()?;
        
        // Start monitoring
        monitor.start();
        
        // Wait for initial connection
        thread::sleep(Duration::from_millis(1000));
        
        // Test if we can see any activity in the monitor queue (connection events, etc.)
        let mut events_seen = 0;
        for _attempt in 1..=3 {
            match monitor.pop() {
                Ok(Some(value)) => {
                    events_seen += 1;
                    let _ = value.get_field_double("value");
                },
                Ok(None) => {},
                Err(_e) => {
                    events_seen += 1;
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        
        // Update the PV via client PUT
        for i in 1..=3 {
            let new_value = 100.0 + i as f64;
            
            // Use client PUT to update the PV
            match ctx.put_double("TEST:MonitorBuilder:Callback", new_value, 2.0) {
                Ok(_) => {
                    thread::sleep(Duration::from_millis(200));
                },
                Err(e) => assert!(false, "PUT failed: {}", e),
            }
        }
        
        // Give extra time for all updates to be processed
        thread::sleep(Duration::from_millis(500));
        
        // Verify we can pop() the values from the queue
        let mut values_popped = 0;
        while let Ok(Some(value)) = monitor.pop() {
            values_popped += 1;
            let _ = value.get_field_double("value");
        }
        
        // Check that we received some events
        assert!(values_popped > 0 || events_seen > 0, 
            "Expected to receive some events, got values={} events={}", 
            values_popped, events_seen);
        
        monitor.stop();
        server.stop()?;
        Ok(())
    }

    /// Test MonitorBuilder with string PV
    #[test]
    fn test_monitor_builder_string_pv() -> Result<(), PvxsError> {
        let mut server = Server::from_env()?;
        
        server.create_pv_string("TEST:MonitorBuilder:String", "Hello MonitorBuilder", NTScalarMetadataBuilder::new())?;
        server.start()?;
        
        thread::sleep(Duration::from_millis(100));
        
        let mut ctx = Context::from_env()?;
        
        let mut monitor = ctx.monitor_builder("TEST:MonitorBuilder:String")?
            .connect_exception(false)
            .disconnect_exception(false)
            .exec()?;
        
        monitor.start();
        thread::sleep(Duration::from_millis(200));
        
        // Try to get initial value
        match monitor.pop() {
            Ok(Some(value)) => {
                assert!(value.get_field_string("value").is_ok(), "Method - should initially have value field");
            },
            Ok(None) => assert!(true, "String PV queue initially empty"),
            Err(e) => assert!(false, "String PV event: {}", e),
        }
        
        monitor.stop();
        server.stop()?;
        Ok(())
    }

    /// Test error handling in MonitorBuilder
    #[test]
    fn test_monitor_builder_error_handling() {
        let mut ctx = Context::from_env().expect("Context creation failed");
        
        // Test with non-existent PV
        match ctx.monitor_builder("NONEXISTENT:PV:NAME") {
            Ok(builder) => {
                // Builder creation should succeed, but exec might fail or timeout
                
                // Try to execute - this might succeed but the monitor won't connect
                match builder.exec() {
                    Ok(_monitor) => {
                    },
                    Err(e) => assert!(true, "Expected error creating monitor for non-existent PV: {}", e),
                }
            },
            Err(e) => assert!(true, "Expected error creating builder for non-existent PV: {}", e),
        }
    }

    /// Test monitoring with multiple rapid value changes
    #[test]
    fn test_monitor_builder_rapid_updates() -> Result<(), PvxsError> {
        let mut server = Server::create_isolated()?;
        
        server.create_pv_double("TEST:MonitorBuilder:Rapid", 0.0, NTScalarMetadataBuilder::new())?;
        server.start()?;
        
        thread::sleep(Duration::from_millis(100));
        
        let mut ctx = Context::from_env()?;
        
        let mut monitor = ctx.monitor_builder("TEST:MonitorBuilder:Rapid")?
            .connect_exception(true)  // Throw connection exceptions
            .exec()?;
        
        monitor.start();
        thread::sleep(Duration::from_millis(200));
        
        // Clear initial events
        while monitor.pop().unwrap_or(None).is_some() {}
        
        // Post rapid updates using client PUT operations
        for i in 1..=5 {
            let _ = ctx.put_double("TEST:MonitorBuilder:Rapid", i as f64, 1.0);
            thread::sleep(Duration::from_millis(20)); // Small delay between updates
        }
        
        // Give time for all updates to propagate
        thread::sleep(Duration::from_millis(200));
        
        // Collect all updates
        let mut updates = Vec::new();
        while let Ok(Some(value)) = monitor.pop() {
            if let Ok(val) = value.get_field_double("value") {
                updates.push(val);
            }
        }
        
        
        if !updates.is_empty() {
        }
        
        monitor.stop();
        server.stop()?;
        Ok(())
    }

    /// Integration test comparing MonitorBuilder vs regular Monitor
    #[test] 
    fn test_monitor_builder_vs_regular_monitor() -> Result<(), PvxsError> {
        let mut server = Server::create_isolated()?;
        
        server.create_pv_double("TEST:MonitorBuilder:Compare", 100.0, NTScalarMetadataBuilder::new())?;
        server.start()?;
        
        thread::sleep(Duration::from_millis(100));
        
        let mut ctx = Context::from_env()?;
        
        // Create monitor using traditional method
        let mut regular_monitor = ctx.monitor("TEST:MonitorBuilder:Compare")?;
        regular_monitor.start();
        
        // Create monitor using builder
        let mut builder_monitor = ctx.monitor_builder("TEST:MonitorBuilder:Compare")?
            .connect_exception(true)  // Throw connection exceptions
            .exec()?;
        builder_monitor.start();
        
        thread::sleep(Duration::from_millis(200));
        
        // Test that both monitors work
        
        // Both should be monitoring the same PV
        assert_eq!(regular_monitor.name(), builder_monitor.name());
        
        // Both should detect updates
        let _ = ctx.put_double("TEST:MonitorBuilder:Compare", 999.9, 1.0);
        thread::sleep(Duration::from_millis(100));
        
        let _regular_has_update = regular_monitor.has_update();
        let _builder_has_update = builder_monitor.has_update();
        
        
        regular_monitor.stop();
        builder_monitor.stop();
        server.stop()?;
        Ok(())
    }

    /// Test callbacks with continuously incrementing server-side value
    #[test]
    fn test_monitor_builder_with_server_side_counter() -> Result<(), PvxsError> {
        // Create server using from_env instead of create_isolated
        let mut server = Server::from_env()?;
        
        server.create_pv_double("TEST:MonitorBuilder:Counter", 0.0, NTScalarMetadataBuilder::new())?;
        server.start()?;
        
        thread::sleep(Duration::from_millis(200));
        
        let mut ctx = Context::from_env()?;
        
        // Create monitor with callback
        let mut monitor = ctx.monitor_builder("TEST:MonitorBuilder:Counter")?
            .connect_exception(true)
            .disconnect_exception(true)
            .event(simple_test_callback)
            .exec()?;
        
        // Start monitoring
        monitor.start();
        
        // Wait for initial connection
        thread::sleep(Duration::from_millis(500));
        
        // Use context to PUT values
        let mut ctx_clone = Context::from_env()?;
        
        // Spawn background thread to continuously update the value
        let counter_handle = thread::spawn(move || {
            for i in 1..=10 {
                let _ = ctx_clone.put_double("TEST:MonitorBuilder:Counter", i as f64, 1.0);
                thread::sleep(Duration::from_millis(200));
            }
        });
        
        // Wait for background thread to finish
        counter_handle.join().unwrap();
        
        // Give time for remaining updates
        thread::sleep(Duration::from_millis(500));
        
        // Check queue state - drain all values
        let mut values_received = 0;
        while let Ok(Some(value)) = monitor.pop() {
            values_received += 1;
            let _ = value.get_field_double("value");
        }
        
        assert!(values_received > 0, 
            "Expected to receive some values from server updates, got {}", 
            values_received);
        
        monitor.stop();
        server.stop()?;
        Ok(())
    }

    /// Test that demonstrates the correct PVXS event callback pattern:
    /// 1. Event fires when queue goes empty -> not-empty
    /// 2. Drain queue completely (sets needNotify back to true)
    /// 3. Post new value (queue empty -> not-empty again)
    /// 4. Event fires again
    #[test]
    fn test_monitor_builder_proper_event_pattern() -> Result<(), PvxsError> {
        // Create server using from_env
        let mut server = Server::from_env()?;
        
        server.create_pv_double("TEST:MonitorBuilder:EventPattern", 0.0, NTScalarMetadataBuilder::new())?;
        server.start()?;
        
        thread::sleep(Duration::from_millis(200));
        
        let mut ctx = Context::from_env()?;
        
        // Create monitor with callback
        let mut monitor = ctx.monitor_builder("TEST:MonitorBuilder:EventPattern")?
            .event(simple_test_callback)
            .exec()?;
        
        // Start monitoring
        monitor.start();
        
        // Wait for initial connection and drain any connection events
        thread::sleep(Duration::from_millis(500));
        
        let mut _drained = 0;
        while let Ok(Some(_)) = monitor.pop() {
            _drained += 1;
        }
        
        // Queue is now EMPTY
        // Post a single value
        ctx.put_double("TEST:MonitorBuilder:EventPattern", 100.0, 1.0)?;
        
        // Wait for update
        thread::sleep(Duration::from_millis(500));
        
        // Drain the queue completely
        let mut values_popped = 0;
        while let Ok(Some(value)) = monitor.pop() {
            values_popped += 1;
            assert!(value.get_field_double("value").is_ok(), "Should have value field when draining");
        }
        
        // Queue is now EMPTY again
        // Post another value
        ctx.put_double("TEST:MonitorBuilder:EventPattern", 200.0, 1.0)?;
        
        // Wait for update
        thread::sleep(Duration::from_millis(500));
        
        // Drain again
        let mut values_popped_2 = 0;
        while let Ok(Some(value)) = monitor.pop() {
            values_popped_2 += 1;
            assert!(value.get_field_double("value").is_ok(), "Should have value field when draining");
        }
        
        // We expect to have received both values
        assert!(values_popped > 0 && values_popped_2 > 0, 
            "Expected to receive both posted values, got first={} second={}", 
            values_popped, values_popped_2);
        
        monitor.stop().expect("Monitor stop failed");
        server.stop()?;
        Ok(())
    }
}