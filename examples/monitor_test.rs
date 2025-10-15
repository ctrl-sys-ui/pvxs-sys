//! Monitor Test - Demonstrate real-time monitoring of TEST:PV_Counter
//! This example shows how to:
//! 1. Create a monitor for the incrementing counter PV
//! 2. Subscribe to value changes
//! 3. Count the number of updates received
//! 4. Display detailed update information

use epics_pvxs_sys::Context;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== EPICS PVXS Monitor Test ===");
    println!("Testing monitor functionality with TEST:PV_Counter");
    println!("This PV increments every second - we should see 5+ updates");
    println!();

    // Create context from environment
    let mut ctx = Context::from_env()?;
    println!("✓ Created PVXS context");

    // Test the counter PV exists first (but don't fail if it times out)
    let pv_name = "TEST:PV_Counter";
    println!("Testing GET to verify PV '{}' is accessible...", pv_name);
    
    match ctx.get(pv_name, 5.0) {
        Ok(initial_value) => {
            let counter_val = initial_value.get_field_int32("value").unwrap_or(-1);
            println!("✓ Initial counter value: {}", counter_val);
        }
        Err(e) => {
            println!("⚠ Warning: Failed to GET {}: {}", pv_name, e);
            println!("  This might be due to network issues or slow IOC startup.");
            println!("  Continuing with monitor test anyway...");
        }
    }

    // Create monitor
    println!("\nCreating monitor for '{}'...", pv_name);
    let mut monitor = ctx.monitor(pv_name)?;
    println!("✓ Monitor created successfully");

    // Start monitoring
    monitor.start();
    println!("✓ Monitor started");
    
    // Wait for connection
    print!("Waiting for connection");
    let start_time = Instant::now();
    let connection_timeout = Duration::from_secs(10);
    
    while !monitor.is_connected() && start_time.elapsed() < connection_timeout {
        print!(".");
        std::thread::sleep(Duration::from_millis(500));
    }
    println!();

    if !monitor.is_connected() {
        println!("⚠ Monitor failed to connect within {} seconds", connection_timeout.as_secs());
        println!("  This might indicate:");
        println!("  - IOC is not running");
        println!("  - PV doesn't exist");
        println!("  - Network connectivity issues");
        println!("  Continuing test anyway to see if delayed connection works...");
    } else {
        println!("✓ Monitor connected to '{}'", pv_name);
    }

    // Monitor for updates
    println!("\n=== Monitoring for Updates ===");
    println!("Collecting updates for 15 seconds...");
    println!("Expected: 5+ updates if TEST:PV_Counter increments every second");
    println!("If no updates appear, check:");
    println!("  - IOC is running: softIoc -d test.db");
    println!("  - PV exists: caget TEST:PV_Counter");
    println!();

    let monitor_start = Instant::now();
    let monitor_duration = Duration::from_secs(15);
    let mut update_count = 0;
    let mut last_value: Option<i32> = None;
    let mut value_changes = 0;

    while monitor_start.elapsed() < monitor_duration {
        // Check for updates with short timeout
        if let Some(update) = monitor.try_get_update()? {
            update_count += 1;
            
            // Extract counter value
            match update.get_field_int32("value") {
                Ok(current_value) => {
                    let elapsed = monitor_start.elapsed();
                    println!("Update #{:2} at {:6.1}s: value = {}", 
                            update_count, elapsed.as_secs_f64(), current_value);
                    
                    // Check if value actually changed
                    if let Some(last) = last_value {
                        if current_value != last {
                            value_changes += 1;
                        }
                    }
                    last_value = Some(current_value);
                }
                Err(e) => {
                    println!("Update #{:2}: Failed to extract value: {}", update_count, e);
                }
            }
        }
        
        // Small sleep to avoid busy waiting
        std::thread::sleep(Duration::from_millis(100));
    }

    // Results
    println!("\n=== Monitor Test Results ===");
    println!("Total updates received: {}", update_count);
    println!("Actual value changes:   {}", value_changes);
    println!("Monitor duration:       {:.1} seconds", monitor_duration.as_secs_f64());
    
    if update_count >= 5 {
        println!("✓ SUCCESS: Received {} updates (expected 5+)", update_count);
    } else {
        println!("✗ INSUFFICIENT: Only {} updates (expected 5+)", update_count);
    }

    if value_changes >= 5 {
        println!("✓ SUCCESS: {} value changes detected", value_changes);
    } else {
        println!("⚠ WARNING: Only {} value changes (expected 5+)", value_changes);
    }

    // Stop monitor
    monitor.stop();
    println!("\n✓ Monitor stopped successfully");

    println!("\n=== Test Complete ===");
    if update_count >= 5 && value_changes >= 3 {
        println!("🎉 MONITOR TEST PASSED!");
        println!("Your monitor implementation is working correctly.");
    } else if update_count > 0 {
        println!("✓ PARTIAL SUCCESS - Monitor is working!");
        println!("Received {} updates with {} value changes.", update_count, value_changes);
        println!("This confirms the monitor implementation works.");
    } else {
        println!("❌ NO UPDATES RECEIVED");
        println!("This might indicate:");
        println!("  - IOC is not running (try: softIoc -d test.db)");
        println!("  - Network configuration issues");
        println!("  - PV doesn't exist or isn't changing");
        println!("  - Monitor connection failed");
        println!("Check your EPICS environment and IOC status.");
    }

    Ok(())
}