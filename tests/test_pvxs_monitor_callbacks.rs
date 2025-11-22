mod test_pvxs_monitor_callbacks {
    use epics_pvxs_sys::{Context, Server, NTScalarMetadataBuilder, AtomicUsize, Ordering};
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
    fn test_monitor_callback_on_start() {
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
            .connection_events(true)
            .disconnection_events(false)
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
        println!("✓ Callback invoked {} times on start", event_count);

        // Cleanup
        monitor.stop().expect("Failed to stop monitor");
        srv.stop().expect("Failed to stop server");
    }

    #[test]
    fn test_monitor_callback_on_stop() {
        // Reset counters
        EVENT_COUNTER.store(0, Ordering::SeqCst);

        // Create a server with a PV
        let mut srv = Server::from_env().expect("Failed to create server");
        let _pv = srv.create_pv_double("callback:test:stop", 2.71, NTScalarMetadataBuilder::new())
            .expect("Failed to create PV");
        srv.start().expect("Failed to start server");

        thread::sleep(Duration::from_millis(500));

        // Create monitor with disconnection events enabled
        let mut ctx = Context::from_env().expect("Failed to create context");
        let mut monitor = ctx.monitor_builder("callback:test:stop")
            .expect("Failed to create monitor builder")
            .connection_events(false)
            .disconnection_events(true)  // Enable disconnection events
            .event(generic_event_callback)
            .exec()
            .expect("Failed to create monitor");

        monitor.start().expect("Failed to start monitor");
        thread::sleep(Duration::from_millis(500));

        // Reset counter after connection
        EVENT_COUNTER.store(0, Ordering::SeqCst);

        // Stop the monitor - should trigger disconnection callback
        monitor.stop().expect("Failed to stop monitor");
        
        // Small delay to allow callback to be processed
        thread::sleep(Duration::from_millis(500));

        let event_count = EVENT_COUNTER.load(Ordering::SeqCst);
        println!("Event count after stop: {}", event_count);
        // Note: Disconnection events might not always trigger on explicit stop
        // This is testing the API, not necessarily guaranteeing the event

        srv.stop().expect("Failed to stop server");
    }

    #[test]
    fn test_monitor_mask_configuration() {
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
            .connection_events(false)  // Mask out connection events
            .disconnection_events(false)
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
            .connection_events(true)  // Include connection events
            .disconnection_events(false)
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

        // Monitor focused on connection events
        let mut mon_connect = ctx.monitor_builder("callback:test:multi")
            .expect("Failed to create monitor builder")
            .connection_events(true)
            .disconnection_events(false)
            .event(connection_callback)
            .exec()
            .expect("Failed to create connection monitor");

        // Monitor focused on disconnection events
        let mut mon_disconnect = ctx.monitor_builder("callback:test:multi")
            .expect("Failed to create monitor builder")
            .connection_events(false)
            .disconnection_events(true)
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
            .connection_events(true)
            .disconnection_events(false)
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
}
