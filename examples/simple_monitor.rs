// examples/simple_monitor.rs
//! Simple Monitor Example - Demonstrates the working monitor functionality
//! 
//! This example shows:
//! - Creating a monitor for a PV
//! - Receiving real-time value updates
//! - Counting value changes

use epics_pvxs_sys::Context;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Simple Monitor Example ===");
    
    // Create context and monitor
    let mut ctx = Context::from_env()?;
    let mut monitor = ctx.monitor("TEST:PV_Counter")?;
    
    // Start monitoring
    monitor.start();
    println!("Monitoring TEST:PV_Counter for 5 seconds...");
    
    // Collect updates for 5 seconds
    let start = Instant::now();
    let mut count = 0;
    
    while start.elapsed() < Duration::from_secs(5) {
        if let Some(update) = monitor.try_get_update()? {
            count += 1;
            let value = update.get_field_int32("value")?;
            println!("Update #{}: value = {}", count, value);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    
    monitor.stop();
    println!("Received {} updates in 5 seconds", count);
    
    if count > 0 {
        println!("✓ Monitor is working!");
    } else {
        println!("⚠ No updates received - check IOC is running");
    }
    
    Ok(())
}