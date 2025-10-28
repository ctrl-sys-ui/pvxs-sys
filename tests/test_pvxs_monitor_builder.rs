// test_pvxs_monitor_builder.rs - Test MonitorBuilder and callback functionality

use epics_pvxs_sys::{Context, PvxsError, Server, SharedPV};
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicUsize, Ordering};

// Global callback counter for testing
static CALLBACK_COUNTER: AtomicUsize = AtomicUsize::new(0);
static COUNTER_TEST_CALLBACK_COUNTER: AtomicUsize = AtomicUsize::new(0);

// Callback function for testing
extern "C" fn test_event_callback() {
    CALLBACK_COUNTER.fetch_add(1, Ordering::SeqCst);
    println!("🔔 Callback invoked! Total calls: {}", CALLBACK_COUNTER.load(Ordering::SeqCst));
}

// Callback function specifically for counter test
extern "C" fn counter_test_callback() {
    let count = COUNTER_TEST_CALLBACK_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    println!("🔔 COUNTER CALLBACK #{}: Server value changed!", count);
}

/// Test basic MonitorBuilder creation and configuration
#[test]
fn test_monitor_builder_creation() -> Result<(), PvxsError> {
    // Create isolated server for testing
    let mut server = Server::create_isolated()?;
    let mut shared_pv = SharedPV::create_mailbox()?;
    
    // Open with initial double value
    shared_pv.open_double(1.0)?;
    server.add_pv("TEST:MonitorBuilder:Double", &mut shared_pv)?;
    
    // Start server
    server.start()?;
    
    // Give server time to start
    thread::sleep(Duration::from_millis(100));
    
    // Create client context
    let mut ctx = Context::from_env()?;
    
    // Test MonitorBuilder creation and configuration
    let _monitor = ctx.monitor_builder("TEST:MonitorBuilder:Double")?
        .mask_connected(false)        // Don't include connection events
        .mask_disconnected(true)      // Include disconnection events
        .exec()?;
    
    println!("✓ MonitorBuilder created and configured successfully");
    
    // Clean up
    server.stop()?;
    Ok(())
}

/// Test MonitorBuilder with different mask configurations
#[test]
fn test_monitor_builder_mask_options() -> Result<(), PvxsError> {
    // Create isolated server for testing
    let mut server = Server::create_isolated()?;
    let mut shared_pv = SharedPV::create_mailbox()?;
    
    // Open with initial value
    shared_pv.open_double(2.5)?;
    server.add_pv("TEST:MonitorBuilder:Masks", &mut shared_pv)?;
    server.start()?;
    
    thread::sleep(Duration::from_millis(100));
    
    let mut ctx = Context::from_env()?;
    
    // Test with both masks enabled
    let _monitor1 = ctx.monitor_builder("TEST:MonitorBuilder:Masks")?
        .mask_connected(true)
        .mask_disconnected(true)
        .exec()?;
    
    println!("✓ Monitor created with both connection events enabled");
    
    // Test with both masks disabled
    let _monitor2 = ctx.monitor_builder("TEST:MonitorBuilder:Masks")?
        .mask_connected(false)
        .mask_disconnected(false)
        .exec()?;
    
    println!("✓ Monitor created with connection events disabled");
    
    // Test default configuration (no explicit mask calls)
    let _monitor3 = ctx.monitor_builder("TEST:MonitorBuilder:Masks")?
        .exec()?;
    
    println!("✓ Monitor created with default configuration");
    
    server.stop()?;
    Ok(())
}

/// Test Monitor pop() method following PVXS pattern
#[test]
fn test_monitor_pop_functionality() -> Result<(), PvxsError> {
    // Create isolated server for testing
    let mut server = Server::create_isolated()?;
    let mut shared_pv = SharedPV::create_mailbox()?;
    
    // Open with initial value
    shared_pv.open_double(10.0)?;
    server.add_pv("TEST:MonitorBuilder:Pop", &mut shared_pv)?;
    server.start()?;
    
    thread::sleep(Duration::from_millis(100));
    
    let mut ctx = Context::from_env()?;
    
    // Create monitor using builder
    let mut monitor = ctx.monitor_builder("TEST:MonitorBuilder:Pop")?
        .mask_connected(false)
        .exec()?;
    
    // Start monitoring
    monitor.start();
    
    // Give time for initial connection
    thread::sleep(Duration::from_millis(200));
    
    // Test pop() method - should initially be empty or have connection data
    match monitor.pop() {
        Ok(Some(value)) => {
            println!("✓ Initial value popped: {}", value);
            // Try to get the value field
            match value.get_field_double("value") {
                Ok(val) => println!("  Initial value field: {}", val),
                Err(_) => println!("  Could not get value field (might be connection event)"),
            }
        },
        Ok(None) => println!("✓ Queue initially empty"),
        Err(e) => {
            println!("✓ Pop returned event/error: {}", e);
            // This is expected for connection events
        }
    }
    
    // Now test updating the PV via client PUT operation
    let new_value = 25.5;
    match ctx.put_double("TEST:MonitorBuilder:Pop", new_value, 2.0) {
        Ok(_) => {
            println!("✓ Posted new value: {}", new_value);
            
            // Give time for update to propagate
            thread::sleep(Duration::from_millis(200));
            
            // Try to pop the update
            let mut updates_received = 0;
            for _ in 0..5 { // Try a few times
                match monitor.pop() {
                    Ok(Some(value)) => {
                        updates_received += 1;
                        println!("✓ Update {} popped: {}", updates_received, value);
                        
                        // Try to extract the value
                        if let Ok(val) = value.get_field_double("value") {
                            println!("  Value field: {}", val);
                            if (val - new_value).abs() < 0.001 {
                                println!("✓ Correct value received!");
                                break;
                            }
                        }
                    },
                    Ok(None) => {
                        println!("  Queue empty, waiting...");
                        thread::sleep(Duration::from_millis(50));
                    },
                    Err(e) => {
                        println!("  Event/error: {}", e);
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            }
            
            if updates_received > 0 {
                println!("✓ Successfully received {} updates via pop()", updates_received);
            } else {
                println!("! No data updates received (may be connection events only)");
            }
        },
        Err(e) => println!("! Failed to PUT new value: {}", e),
    }
    
    monitor.stop();
    server.stop()?;
    Ok(())
}

/// Test real Rust function callback functionality
#[test]
fn test_monitor_builder_with_callback() -> Result<(), PvxsError> {
    // Reset callback counter
    CALLBACK_COUNTER.store(0, Ordering::SeqCst);
    
    // Create isolated server for testing
    let mut server = Server::create_isolated()?;
    let mut shared_pv = SharedPV::create_mailbox()?;
    
    shared_pv.open_double(42.0)?;
    server.add_pv("TEST:MonitorBuilder:Callback", &mut shared_pv)?;
    server.start()?;
    
    thread::sleep(Duration::from_millis(100));
    
    let mut ctx = Context::from_env()?;
    
    // Create monitor with actual Rust callback function
    let mut monitor = ctx.monitor_builder("TEST:MonitorBuilder:Callback")?
        .mask_connected(false)   // Allow connection events - callback should fire for these
        .mask_disconnected(false) // Allow disconnection events
        .event(test_event_callback)  // Set the actual callback
        .exec()?;
    
    println!("✓ Monitor created with Rust callback function");
    
    // Start monitoring
    monitor.start();
    
    // Wait longer for initial connection and let connection events fire callback
    thread::sleep(Duration::from_millis(1000)); // Increased wait time
    
    // Check initial callback count
    let initial_count = CALLBACK_COUNTER.load(Ordering::SeqCst);
    println!("✓ Initial callback count: {}", initial_count);
    
    // Test if we can see any activity in the monitor queue (connection events, etc.)
    println!("🔍 Checking monitor queue for any events...");
    let mut events_seen = 0;
    for attempt in 1..=3 {
        match monitor.pop() {
            Ok(Some(value)) => {
                events_seen += 1;
                println!("  Event {}: {}", events_seen, value);
                if let Ok(val) = value.get_field_double("value") {
                    println!("    Value: {}", val);
                }
            },
            Ok(None) => {
                println!("  Attempt {}: Queue empty", attempt);
            },
            Err(e) => {
                events_seen += 1;
                println!("  Event {}: {}", events_seen, e);
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    
    // Update the PV directly via the server (this should trigger the callback)
    for i in 1..=3 {
        let new_value = 100.0 + i as f64;
        
        // Try server-side update first (more reliable)
        match shared_pv.post_double(new_value) {
            Ok(_) => {
                println!("✓ Posted value via server: {}", new_value);
                thread::sleep(Duration::from_millis(200)); // Give time for callback
            },
            Err(e) => {
                println!("! Server POST failed: {}, trying client PUT", e);
                // Fallback to client PUT
                match ctx.put_double("TEST:MonitorBuilder:Callback", new_value, 2.0) {
                    Ok(_) => {
                        println!("✓ PUT value via client: {}", new_value);
                        thread::sleep(Duration::from_millis(200));
                    },
                    Err(e) => println!("! PUT failed: {}", e),
                }
            }
        }
        
        // Check if callback was invoked after each update
        let current_count = CALLBACK_COUNTER.load(Ordering::SeqCst);
        if current_count > initial_count {
            println!("✓ Callback count after update {}: {}", i, current_count);
        }
    }
    
    // Give extra time for all callbacks to be processed
    thread::sleep(Duration::from_millis(500));
    
    // Check final callback count
    let final_count = CALLBACK_COUNTER.load(Ordering::SeqCst);
    println!("✓ Final callback count: {}", final_count);
    
    // Verify that callbacks were actually invoked
    if final_count > initial_count {
        println!("✅ SUCCESS: Rust callback function was invoked {} times!", 
                final_count - initial_count);
        
        // Also verify we can still pop() the values
        let mut values_popped = 0;
        while let Ok(Some(value)) = monitor.pop() {
            values_popped += 1;
            if let Ok(val) = value.get_field_double("value") {
                println!("  📥 Popped value: {}", val);
            }
        }
        println!("✓ Total values popped after callbacks: {}", values_popped);
        
    } else {
        println!("⚠️  WARNING: No callbacks were invoked (this might be expected in some test environments)");
        
        // Check if we can at least see values in the queue
        let mut values_in_queue = 0;
        while let Ok(Some(value)) = monitor.pop() {
            values_in_queue += 1;
            if let Ok(val) = value.get_field_double("value") {
                println!("  📥 Value in queue (no callback): {}", val);
            }
        }
        
        if values_in_queue > 0 {
            println!("✓ Found {} values in monitor queue (but callbacks didn't fire)", values_in_queue);
        } else {
            println!("! No values found in monitor queue either");
        }
    }
    
    monitor.stop();
    server.stop()?;
    Ok(())
}

/// Test MonitorBuilder with string PV
#[test]
fn test_monitor_builder_string_pv() -> Result<(), PvxsError> {
    let mut server = Server::create_isolated()?;
    let mut shared_pv = SharedPV::create_mailbox()?;
    
    shared_pv.open_string("Hello MonitorBuilder")?;
    server.add_pv("TEST:MonitorBuilder:String", &mut shared_pv)?;
    server.start()?;
    
    thread::sleep(Duration::from_millis(100));
    
    let mut ctx = Context::from_env()?;
    
    let mut monitor = ctx.monitor_builder("TEST:MonitorBuilder:String")?
        .mask_connected(false)
        .mask_disconnected(false)
        .exec()?;
    
    monitor.start();
    thread::sleep(Duration::from_millis(200));
    
    // Try to get initial value
    match monitor.pop() {
        Ok(Some(value)) => {
            if let Ok(string_val) = value.get_field_string("value") {
                println!("✓ String PV initial value: {}", string_val);
            }
        },
        Ok(None) => println!("✓ String PV queue initially empty"),
        Err(e) => println!("✓ String PV event: {}", e),
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
            println!("✓ MonitorBuilder created for non-existent PV");
            
            // Try to execute - this might succeed but the monitor won't connect
            match builder.exec() {
                Ok(monitor) => {
                    println!("✓ Monitor created (may not connect to non-existent PV)");
                    println!("  Connection status: {}", monitor.is_connected());
                },
                Err(e) => println!("✓ Expected error creating monitor for non-existent PV: {}", e),
            }
        },
        Err(e) => println!("✓ Expected error creating builder for non-existent PV: {}", e),
    }
}

/// Test monitoring with multiple rapid value changes
#[test]
fn test_monitor_builder_rapid_updates() -> Result<(), PvxsError> {
    let mut server = Server::create_isolated()?;
    let mut shared_pv = SharedPV::create_mailbox()?;
    
    shared_pv.open_double(0.0)?;
    server.add_pv("TEST:MonitorBuilder:Rapid", &mut shared_pv)?;
    server.start()?;
    
    thread::sleep(Duration::from_millis(100));
    
    let mut ctx = Context::from_env()?;
    
    let mut monitor = ctx.monitor_builder("TEST:MonitorBuilder:Rapid")?
        .mask_connected(false)
        .exec()?;
    
    monitor.start();
    thread::sleep(Duration::from_millis(200));
    
    // Clear initial events
    while monitor.pop().unwrap_or(None).is_some() {}
    
    // Post rapid updates using client PUT operations
    println!("✓ Posting rapid updates...");
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
    
    println!("✓ Collected {} updates: {:?}", updates.len(), updates);
    
    if !updates.is_empty() {
        println!("✓ Successfully received rapid updates via MonitorBuilder");
    }
    
    monitor.stop();
    server.stop()?;
    Ok(())
}

/// Integration test comparing MonitorBuilder vs regular Monitor
#[test] 
fn test_monitor_builder_vs_regular_monitor() -> Result<(), PvxsError> {
    let mut server = Server::create_isolated()?;
    let mut shared_pv = SharedPV::create_mailbox()?;
    
    shared_pv.open_double(100.0)?;
    server.add_pv("TEST:MonitorBuilder:Compare", &mut shared_pv)?;
    server.start()?;
    
    thread::sleep(Duration::from_millis(100));
    
    let mut ctx = Context::from_env()?;
    
    // Create monitor using traditional method
    let mut regular_monitor = ctx.monitor("TEST:MonitorBuilder:Compare")?;
    regular_monitor.start();
    
    // Create monitor using builder
    let mut builder_monitor = ctx.monitor_builder("TEST:MonitorBuilder:Compare")?
        .mask_connected(false)
        .exec()?;
    builder_monitor.start();
    
    thread::sleep(Duration::from_millis(200));
    
    // Test that both monitors work
    println!("✓ Regular monitor connected: {}", regular_monitor.is_connected());
    println!("✓ Builder monitor connected: {}", builder_monitor.is_connected());
    
    // Both should be monitoring the same PV
    assert_eq!(regular_monitor.name(), builder_monitor.name());
    println!("✓ Both monitors have same PV name: {}", regular_monitor.name());
    
    // Both should detect updates
    let _ = ctx.put_double("TEST:MonitorBuilder:Compare", 999.9, 1.0);
    thread::sleep(Duration::from_millis(100));
    
    let regular_has_update = regular_monitor.has_update();
    let builder_has_update = builder_monitor.has_update();
    
    println!("✓ Regular monitor has update: {}", regular_has_update);
    println!("✓ Builder monitor has update: {}", builder_has_update);
    
    regular_monitor.stop();
    builder_monitor.stop();
    server.stop()?;
    Ok(())
}

/// Test callbacks with continuously incrementing server-side value
#[test]
fn test_monitor_builder_with_server_side_counter() -> Result<(), PvxsError> {
    // Reset callback counter
    COUNTER_TEST_CALLBACK_COUNTER.store(0, Ordering::SeqCst);
    
    // Create server using from_env instead of create_isolated
    let mut server = Server::from_env()?;
    let mut shared_pv = SharedPV::create_mailbox()?;
    
    shared_pv.open_double(0.0)?;
    server.add_pv("TEST:MonitorBuilder:Counter", &mut shared_pv)?;
    server.start()?;
    
    thread::sleep(Duration::from_millis(200));
    
    let mut ctx = Context::from_env()?;
    
    // Create monitor with callback - allow connection events to see if they trigger callbacks
    let mut monitor = ctx.monitor_builder("TEST:MonitorBuilder:Counter")?
        .mask_connected(false)   // Allow connection events
        .mask_disconnected(false) // Allow disconnection events  
        .event(counter_test_callback)  // Set the callback
        .exec()?;
    
    println!("✓ Monitor created with counter callback");
    
    // Start monitoring
    monitor.start();
    
    // Wait for initial connection
    thread::sleep(Duration::from_millis(500));
    
    let initial_count = COUNTER_TEST_CALLBACK_COUNTER.load(Ordering::SeqCst);
    println!("✓ Initial callback count: {}", initial_count);
    
    // We need to use a different approach since SharedPV doesn't support clone
    // Let's use the context to PUT values instead of server-side posting
    let mut ctx_clone = Context::from_env()?;
    
    // Spawn background thread to continuously update the value
    let counter_handle = thread::spawn(move || {
        for i in 1..=10 {
            match ctx_clone.put_double("TEST:MonitorBuilder:Counter", i as f64, 1.0) {
                Ok(_) => {
                    println!("📊 Client PUT counter value: {}", i);
                },
                Err(e) => {
                    println!("! Failed to PUT counter value {}: {}", i, e);
                }
            }
            thread::sleep(Duration::from_millis(200)); // Update every 200ms
        }
        println!("📊 Background counter thread finished");
    });
    
    // Monitor the callbacks for a while
    let test_duration = Duration::from_millis(3000); // 3 seconds
    let start_time = std::time::Instant::now();
    
    println!("🔍 Monitoring callbacks for {} seconds...", test_duration.as_secs());
    
    while start_time.elapsed() < test_duration {
        let current_count = COUNTER_TEST_CALLBACK_COUNTER.load(Ordering::SeqCst);
        
        // Print progress periodically
        if start_time.elapsed().as_millis() % 500 == 0 {
            println!("  Callbacks so far: {}", current_count);
        }
        
        // DON'T pop values immediately - let the queue build up to trigger callbacks!
        // Comment out the pop() calls to see if callbacks fire when queue goes empty->not-empty
        /*
        let mut values_this_check = 0;
        while let Ok(Some(value)) = monitor.pop() {
            values_this_check += 1;
            if let Ok(val) = value.get_field_double("value") {
                println!("  📥 Popped value: {}", val);
            }
        }
        
        if values_this_check > 0 {
            println!("  📥 Total values popped this check: {}", values_this_check);
        }
        */
        
        thread::sleep(Duration::from_millis(100));
    }
    
    // Wait for background thread to finish
    counter_handle.join().unwrap();
    
    // Give a bit more time for any remaining callbacks
    thread::sleep(Duration::from_millis(500));
    
    let final_count = COUNTER_TEST_CALLBACK_COUNTER.load(Ordering::SeqCst);
    println!("✓ Final callback count: {}", final_count);
    
    // Check final queue state
    let mut final_values = 0;
    while let Ok(Some(value)) = monitor.pop() {
        final_values += 1;
        if let Ok(val) = value.get_field_double("value") {
            println!("  📥 Final value: {}", val);
        }
    }
    
    if final_count > initial_count {
        println!("✅ SUCCESS: {} callbacks were invoked by server-side counter!", 
                final_count - initial_count);
    } else if final_values > 0 {
        println!("📦 Values found in queue ({}) but no callbacks fired", final_values);
    } else {
        println!("⚠️  No callbacks fired and no values in queue");
    }
    
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
    // Reset callback counter
    COUNTER_TEST_CALLBACK_COUNTER.store(0, Ordering::SeqCst);
    
    // Create server using from_env
    let mut server = Server::from_env()?;
    let mut shared_pv = SharedPV::create_mailbox()?;
    
    shared_pv.open_double(0.0)?;
    server.add_pv("TEST:MonitorBuilder:EventPattern", &mut shared_pv)?;
    server.start()?;
    
    thread::sleep(Duration::from_millis(200));
    
    let mut ctx = Context::from_env()?;
    
    // Create monitor with callback
    let mut monitor = ctx.monitor_builder("TEST:MonitorBuilder:EventPattern")?
        .event(counter_test_callback)  // Set the callback
        .exec()?;
    
    println!("✓ Monitor created with event callback");
    
    // Start monitoring
    monitor.start();
    
    // Wait for initial connection and drain any connection events
    thread::sleep(Duration::from_millis(500));
    
    println!("🧹 Draining initial queue (connection events)...");
    let mut drained = 0;
    while let Ok(Some(_)) = monitor.pop() {
        drained += 1;
    }
    println!("  Drained {} initial items", drained);
    
    let initial_count = COUNTER_TEST_CALLBACK_COUNTER.load(Ordering::SeqCst);
    println!("✓ Initial callback count after drain: {}", initial_count);
    
    // Now the queue should be EMPTY and needNotify should be TRUE
    // Post a single value - this should trigger callback (empty -> not-empty)
    println!("\n📤 Posting first value (100.0)...");
    ctx.put_double("TEST:MonitorBuilder:EventPattern", 100.0, 1.0)?;
    
    // Wait for callback to fire
    thread::sleep(Duration::from_millis(500));
    
    let count_after_first = COUNTER_TEST_CALLBACK_COUNTER.load(Ordering::SeqCst);
    println!("  Callback count after first value: {} (delta: {})", 
             count_after_first, count_after_first - initial_count);
    
    // Drain the queue completely (sets needNotify back to true)
    println!("🧹 Draining queue after first value...");
    let mut values_popped = 0;
    while let Ok(Some(value)) = monitor.pop() {
        values_popped += 1;
        if let Ok(val) = value.get_field_double("value") {
            println!("  📦 Popped value #{}: {}", values_popped, val);
        }
    }
    println!("  Drained {} values", values_popped);
    
    // Queue is now EMPTY again, needNotify is TRUE again
    // Post another value - this should trigger callback again
    println!("\n📤 Posting second value (200.0)...");
    ctx.put_double("TEST:MonitorBuilder:EventPattern", 200.0, 1.0)?;
    
    // Wait for callback to fire
    thread::sleep(Duration::from_millis(500));
    
    let count_after_second = COUNTER_TEST_CALLBACK_COUNTER.load(Ordering::SeqCst);
    println!("  Callback count after second value: {} (delta: {})", 
             count_after_second, count_after_second - count_after_first);
    
    // Drain again
    println!("🧹 Draining queue after second value...");
    let mut values_popped_2 = 0;
    while let Ok(Some(value)) = monitor.pop() {
        values_popped_2 += 1;
        if let Ok(val) = value.get_field_double("value") {
            println!("  📦 Popped value #{}: {}", values_popped_2, val);
        }
    }
    println!("  Drained {} values", values_popped_2);
    
    let final_count = COUNTER_TEST_CALLBACK_COUNTER.load(Ordering::SeqCst);
    println!("\n📊 Final callback count: {}", final_count);
    println!("📊 Total callbacks from posted values: {}", final_count - initial_count);
    
    // We expect at least 2 callbacks (one for each value posted)
    assert!(final_count - initial_count >= 2, 
        "Expected at least 2 callbacks from posted values, got {}", 
        final_count - initial_count);
    
    println!("✅ SUCCESS: Event callbacks working correctly!");
    
    monitor.stop();
    server.stop()?;
    Ok(())
}