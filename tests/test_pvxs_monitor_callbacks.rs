mod test_pvxs_monitor_callbacks {
    use epics_pvxs_sys::{Context, Server, NTScalarMetadataBuilder, AtomicUsize, Ordering, MonitorEvent};
    use std::thread;
    use std::time::Duration;

    // Global counters for callback testing
    static CONNECT_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static DISCONNECT_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static EVENT_COUNTER: AtomicUsize = AtomicUsize::new(0);

    extern "C" fn connection_callback() {
        CONNECT_COUNTER.fetch_add(1, Ordering::SeqCst);
        println!("Connection event detected! Count: {}", CONNECT_COUNTER.load(Ordering::SeqCst));
    }

    extern "C" fn disconnection_callback() {
        DISCONNECT_COUNTER.fetch_add(1, Ordering::SeqCst);
        println!("Disconnection event detected! Count: {}", DISCONNECT_COUNTER.load(Ordering::SeqCst));
    }

    extern "C" fn generic_event_callback() {
        EVENT_COUNTER.fetch_add(1, Ordering::SeqCst);
        println!("Event detected! Count: {}", EVENT_COUNTER.load(Ordering::SeqCst));
    }

    #[test]
    fn test_monitor_connection_and_disconnection_events_off() {
        
        // Create a server with a PV
        let mut srv = Server::from_env().expect("Failed to create server");
        let _pv = srv.create_pv_double("callback:test:stop", 2.71, NTScalarMetadataBuilder::new())
            .expect("Failed to create PV");
        srv.start().expect("Failed to start server");

        thread::sleep(Duration::from_millis(500));

        let mut ctx = Context::from_env().expect("Failed to create context");

        // Test 1: connect_exception(false) should suppress Connected exceptions (maskConnected(true))
        let mut monitor1 = ctx.monitor_builder("callback:test:stop")
            .expect("Failed to create monitor builder")
            .connect_exception(false)  // maskConnected(true) - suppresses connection exceptions
            .disconnect_exception(false)  // maskDisconnected(true) - suppresses disconnection exceptions
            .exec()
            .expect("Failed to create monitor1");

        monitor1.start().expect("Failed to start monitor1");
        thread::sleep(Duration::from_millis(500));
        
        // Pop events - we expect to get either data or Connected exception
        let mut got_connected_exception = false;
        let mut got_disconnected_exception = false;
        let mut got_finished_exception = false;
        let mut got_remote_error_exception = false;
        let mut got_generic_exception = false;

        let mut data_count1 = 0;
        for _ in 0..20 {
            match monitor1.pop() {
                Ok(Some(_)) => data_count1 += 1,
                Ok(None) => break,
                Err(MonitorEvent::Connected(_)) => {
                    got_connected_exception = true;
                },
                Err(MonitorEvent::Disconnected(_)) => {
                    got_disconnected_exception = true;
                },
                Err(MonitorEvent::Finished(_)) => {
                    got_finished_exception = true;
                },
                Err(MonitorEvent::RemoteError(_)) => {
                    got_remote_error_exception = true;
                },
                Err(MonitorEvent::ClientError(_)) => {
                    got_generic_exception = true;
                }
            }
        }
        
        monitor1.stop().expect("Failed to stop monitor1");

        // Verify: with connect_exception(false), do not get a connection exception
        assert!(!got_connected_exception, "Did not expect a connection exception with connect_exception(false)");
        assert!(!got_disconnected_exception, "Did not expect a disconnection exception with disconnect_exception(false)");
        assert!(!got_finished_exception, "Did not expect a finished exception with disconnect_exception(false)");
        assert!(!got_remote_error_exception, "Did not expect a remote error exception");
        assert!(!got_generic_exception, "Did not expect a generic client error exception");
        assert_eq!(data_count1, 1, "Expected data with as pop returns data after the initial connection");
    
    srv.stop().expect("Failed to stop server");
    }

    #[test]
    fn test_monitor_connection_on_and_disconnection_off() {
        // Create a server with a PV
        let mut srv = Server::from_env().expect("Failed to create server");
        let _pv = srv.create_pv_double("callback:test:stop", 2.71, NTScalarMetadataBuilder::new())
            .expect("Failed to create PV");
        srv.start().expect("Failed to start server");

        let mut ctx = Context::from_env().expect("Failed to create context");

        thread::sleep(Duration::from_millis(500));

        // Reset flags for next test
        let mut got_connected_exception = false;
        let mut got_disconnected_exception = false;
        let mut got_finished_exception = false;
        let mut got_remote_error_exception = false;
        let mut got_generic_exception = false;
        
        // Test 2: connect_exception(true) should queue connection exceptions as data (no exception)
        let mut monitor2 = ctx.monitor_builder("callback:test:stop")
            .expect("Failed to create monitor builder")
            .connect_exception(true)  // maskConnected(false) - exceptions queued
            .disconnect_exception(false)  // maskDisconnected(true) - throws exception
            .exec()
            .expect("Failed to create monitor2");

        monitor2.start().expect("Failed to start monitor2");
        thread::sleep(Duration::from_millis(500));
        
        let mut data_count2 = 0;
        for _ in 0..20 {
            match monitor2.pop() {
                Ok(Some(_)) => data_count2 += 1,
                Ok(None) => break,
                Err(MonitorEvent::Connected(_)) => {
                    got_connected_exception = true;
                },
                Err(MonitorEvent::Disconnected(_)) => {
                    got_disconnected_exception = true;
                },
                Err(MonitorEvent::Finished(_)) => {
                    got_finished_exception = true;
                }
                Err(MonitorEvent::RemoteError(_)) => {
                    got_remote_error_exception = true;
                },
                Err(MonitorEvent::ClientError(_)) => {
                    got_generic_exception = true;
                }
            }
        }
        
        monitor2.stop().expect("Failed to stop monitor2");

        assert!(got_connected_exception, "Expected connection exception to be queued as data with connect_exception(true)");
        assert!(!got_disconnected_exception, "Did not expect disconnection exception with disconnect_exception(false)");
        assert!(!got_finished_exception, "Did not expect finished exception");
        assert!(!got_remote_error_exception, "Did not expect a remote error exception");
        assert!(!got_generic_exception, "Did not expect a generic client error exception");
        assert!(data_count2 > 0 , "Expected data before disconnection occurred, but got {}", data_count2);

        srv.stop().expect("Failed to stop server");
    }
    
    #[test]
    fn test_monitor_connection_off_disconnection_on() {
        // Create a server with a PV
        let mut srv = Server::from_env().expect("Failed to create server");
        let _pv = srv.create_pv_double("callback:test:stop", 2.71, NTScalarMetadataBuilder::new())
            .expect("Failed to create PV");
        srv.start().expect("Failed to start server");

        let mut ctx = Context::from_env().expect("Failed to create context");
        thread::sleep(Duration::from_millis(500));

        // Reset flags for next test
        let mut got_connected_exception = false;
        let mut got_disconnected_exception = false;
        let mut got_finished_exception = false;
        let mut got_remote_error_exception = false;
        let mut got_generic_exception = false;

        // Test 3: disconnect_exception(true) should throw disconnection exceptions (maskDisconnected(false))
        let mut monitor3 = ctx.monitor_builder("callback:test:stop")
            .expect("Failed to create monitor builder")
            .connect_exception(false)  // maskConnected(true) - suppresses connection exceptions
            .disconnect_exception(true)  // maskDisconnected(false) - throws disconnection exceptions
            .exec()
            .expect("Failed to create monitor3");

        monitor3.start().expect("Failed to start monitor3");
        thread::sleep(Duration::from_millis(500));  // Wait for connection

        let mut data_count3 = 0;
        for i in 0..20 {
            match monitor3.pop() {
                Ok(Some(_)) => data_count3 += 1,
                Ok(None) => {// Do nothing, continue
                },
                Err(MonitorEvent::Connected(e)) => {
                    got_connected_exception = true;
                    println!("Unexpected connected exception: {}", e);
                },
                Err(MonitorEvent::Disconnected(e)) => {
                    got_disconnected_exception = true;
                    println!("Disconnection event detected: {}", e);
                },
                Err(MonitorEvent::Finished(e)) => {
                    got_finished_exception = true;
                    println!("Finished event detected: {}", e);
                }
                Err(MonitorEvent::RemoteError(e)) => {
                    got_remote_error_exception = true;
                    println!("Remote error event detected: {}", e);
                },
                Err(MonitorEvent::ClientError(e)) => {
                    got_generic_exception = true;
                    println!("Generic client error: {}", e);
                }
            }
            if i == 10 {
                // Stop the SERVER to trigger disconnection event
                srv.stop().expect("Failed to stop server to trigger disconnection");
                thread::sleep(Duration::from_millis(500));
            }
        }

        
        // Cleanup - server already stopped above
        thread::sleep(Duration::from_millis(500));
        
        assert!(got_disconnected_exception, "Expected disconnection exception to be thrown with disconnect_exception(true)");
        assert!(!got_connected_exception, "Did not expect a connection exception with connect_exception(false)");
        assert!(!got_finished_exception, "Did not expect finished exception");
        assert!(!got_remote_error_exception, "Did not expect a remote error exception");
        assert!(!got_generic_exception, "Did not expect a generic client error exception");
        assert!(data_count3 > 0, "Expected data before disconnection occurred, but got {}", data_count3);

    }


    #[test]
    fn test_monitor_multiple_callbacks() {
        // Test using different callbacks for different scenarios
        CONNECT_COUNTER.store(0, Ordering::SeqCst);
        DISCONNECT_COUNTER.store(0, Ordering::SeqCst);

        let mut srv = Server::from_env().expect("Failed to create server");
        let _pv = srv.create_pv_double("callback:test:multi", 4.56, NTScalarMetadataBuilder::new())
            .expect("Failed to create PV");
        srv.start().expect("Failed to start server");

        thread::sleep(Duration::from_millis(500));

        let mut ctx = Context::from_env().expect("Failed to create context");

        // Monitor focused on connection exceptions
        let mut mon_connect = ctx.monitor_builder("callback:test:multi")
            .expect("Failed to create monitor builder")
            .connect_exception(true)
            .disconnect_exception(false)
            .event(connection_callback)
            .exec()
            .expect("Failed to create connection monitor");

        // Monitor focused on disconnection exceptions
        let mut mon_disconnect = ctx.monitor_builder("callback:test:multi")
            .expect("Failed to create monitor builder")
            .connect_exception(false)
            .disconnect_exception(true)
            .event(disconnection_callback)
            .exec()
            .expect("Failed to create disconnection monitor");

        mon_connect.start().expect("Failed to start connection monitor");
        mon_disconnect.start().expect("Failed to start disconnection monitor");
        
        thread::sleep(Duration::from_millis(1000));

        let connect_count = CONNECT_COUNTER.load(Ordering::SeqCst);
        let disconnect_count = DISCONNECT_COUNTER.load(Ordering::SeqCst);

        println!("Connection callbacks: {}", connect_count);
        println!("Disconnection callbacks: {}", disconnect_count);

        // We should see connection callbacks when monitors start
        assert!(connect_count > 0, "Expected connection callbacks");

        // Cleanup
        mon_connect.stop().expect("Failed to stop connection monitor");
        mon_disconnect.stop().expect("Failed to stop disconnection monitor");
        srv.stop().expect("Failed to stop server");
    }

    #[test]
    fn test_monitor_callback_with_updates() {
        // Test that callbacks are invoked when values update
        EVENT_COUNTER.store(0, Ordering::SeqCst);

        let mut srv = Server::from_env().expect("Failed to create server");
        let mut pv = srv.create_pv_double("callback:test:updates", 0.0, NTScalarMetadataBuilder::new())
            .expect("Failed to create PV");
        srv.start().expect("Failed to start server");

        thread::sleep(Duration::from_millis(500));

        let mut ctx = Context::from_env().expect("Failed to create context");
        let mut monitor = ctx.monitor_builder("callback:test:updates")
            .expect("Failed to create monitor builder")
            .connect_exception(true)
            .disconnect_exception(false)
            .event(generic_event_callback)
            .exec()
            .expect("Failed to create monitor");

        monitor.start().expect("Failed to start monitor");
        thread::sleep(Duration::from_millis(500));

        // Reset counter after initial connection
        EVENT_COUNTER.store(0, Ordering::SeqCst);

        // Post some values to trigger callbacks
        for i in 1..=5 {
            pv.post_double(i as f64).expect("Failed to post value");
            thread::sleep(Duration::from_millis(100));
        }

        // Wait for callbacks to be processed
        thread::sleep(Duration::from_millis(500));

        let event_count = EVENT_COUNTER.load(Ordering::SeqCst);
        println!("Callbacks after {} updates: {}", 5, event_count);
        
        // We should get callbacks for the updates
        assert!(event_count > 0, "Expected callbacks for value updates");

        // Cleanup
        monitor.stop().expect("Failed to stop monitor");
        srv.stop().expect("Failed to stop server");
    }

    #[test]
    fn test_monitor_callback_on_connect_and_disconnect() {
        // Reset counters
        EVENT_COUNTER.store(0, Ordering::SeqCst);

        // Create a server with a PV
        let mut srv = Server::from_env().expect("Failed to create server");
        let _pv = srv.create_pv_double("callback:test:start", 3.14, NTScalarMetadataBuilder::new())
            .expect("Failed to create PV");
        srv.start().expect("Failed to start server");

        // Give server time to initialize
        thread::sleep(Duration::from_millis(500));

        // Create monitor with callback
        let mut ctx = Context::from_env().expect("Failed to create context");
        let mut monitor = ctx.monitor_builder("callback:test:start")
            .expect("Failed to create monitor builder")
            .connect_exception(true)
            .disconnect_exception(true)
            .event(generic_event_callback)
            .exec()
            .expect("Failed to create monitor");

        // Start the monitor - should trigger callback when connected
        monitor.start().expect("Failed to start monitor");
        
        // Wait for connection and initial value
        thread::sleep(Duration::from_millis(1000));

        // Check that callback was invoked
        let event_count = EVENT_COUNTER.load(Ordering::SeqCst);
        assert!(event_count > 0, "Expected callback to be invoked on start, got {} events", event_count);

        srv.stop().expect("Failed to stop server");
        thread::sleep(Duration::from_millis(1000));

        let event_count2 = EVENT_COUNTER.load(Ordering::SeqCst);
        assert!(event_count2 > event_count, "Expected callback to be invoked on server stop, got {} events", event_count2);

        // Cleanup
        monitor.stop().expect("Failed to stop monitor");
        // I expect no increase in counter after stop
        thread::sleep(Duration::from_millis(500));
        assert!(EVENT_COUNTER.load(Ordering::SeqCst) == event_count2, "Expected no new events after stop");
        
    }

    #[test]
    fn test_monitor_multiple_client_monitors() {
        // Test that mask configuration works correctly
        
        // Create a server
        let mut srv = Server::from_env().expect("Failed to create server");
        let _pv = srv.create_pv_double("callback:test:mask", 1.23, NTScalarMetadataBuilder::new())
            .expect("Failed to create PV");
        srv.start().expect("Failed to start server");

        thread::sleep(Duration::from_millis(500));

        let mut ctx = Context::from_env().expect("Failed to create context");

        // Test 1: With connection events masked out (should not get connection events)
        EVENT_COUNTER.store(0, Ordering::SeqCst);
        let mut monitor1 = ctx.monitor_builder("callback:test:mask")
            .expect("Failed to create monitor builder")
            .connect_exception(false)  // Mask out connection exceptions
            .disconnect_exception(false)
            .event(generic_event_callback)
            .exec()
            .expect("Failed to create monitor1");

        monitor1.start().expect("Failed to start monitor1");
        thread::sleep(Duration::from_millis(500));

        let count1 = EVENT_COUNTER.load(Ordering::SeqCst);
        println!("Events with connection masked: {}", count1);

        // Test 2: With connection events enabled
        EVENT_COUNTER.store(0, Ordering::SeqCst);
        let mut monitor2 = ctx.monitor_builder("callback:test:mask")
            .expect("Failed to create monitor builder")
            .connect_exception(true)  // Include connection exceptions
            .disconnect_exception(false)
            .event(generic_event_callback)
            .exec()
            .expect("Failed to create monitor2");

        monitor2.start().expect("Failed to start monitor2");
        thread::sleep(Duration::from_millis(500));

        let count2 = EVENT_COUNTER.load(Ordering::SeqCst);
        println!("Events with connection enabled: {}", count2);

        // With connection events enabled, we should get more events
        assert!(count2 >= count1, 
            "Expected more events with connection enabled ({}) vs masked ({})", 
            count2, count1);

        // Cleanup
        monitor1.stop().expect("Failed to stop monitor1");
        monitor2.stop().expect("Failed to stop monitor2");
        srv.stop().expect("Failed to stop server");
    }
}
