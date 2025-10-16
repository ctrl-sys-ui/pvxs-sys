// examples/simple_monitor.rs
//! Simple Monitor Example - Demonstrates the working monitor functionality
//! 
//! This example shows:
//! - Creating monitors for one or more PVs
//! - Receiving real-time value updates
//! - Counting value changes
//! 
//! Usage:
//!   cargo run --example simple_monitor -- PV_NAME [PV_NAME2 ...]
//!   cargo run --example simple_monitor -- TEST:COUNTER
//!   cargo run --example simple_monitor -- TEST:COUNTER TEST:RANDOM TEST:TEMPERATURE

use epics_pvxs_sys::Context;
use std::env;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <PV_NAME> [PV_NAME2] [PV_NAME3] ...", args[0]);
        eprintln!("Example: {} TEST:COUNTER", args[0]);
        eprintln!("Example: {} TEST:COUNTER TEST:RANDOM TEST:TEMPERATURE", args[0]);
        std::process::exit(1);
    }
    
    let pv_names: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
    
    println!("=== Simple Monitor Example ===");
    println!("Monitoring PVs: {:?}", pv_names);
    
    // Create context and monitors
    let mut ctx = Context::from_env()?;
    let mut monitors = Vec::new();
    
    // Create monitors for each PV
    for pv_name in &pv_names {
        match ctx.monitor(pv_name) {
            Ok(mut monitor) => {
                monitor.start();
                monitors.push((pv_name.to_string(), monitor));
                println!("✓ Started monitoring: {}", pv_name);
            }
            Err(e) => {
                eprintln!("✗ Failed to create monitor for {}: {}", pv_name, e);
            }
        }
    }
    
    if monitors.is_empty() {
        eprintln!("No monitors created successfully");
        std::process::exit(1);
    }
    
    println!("Monitoring for 10 seconds...");
    println!("Press Ctrl+C to stop early");
    
    // Collect updates for 10 seconds
    let start = Instant::now();
    let mut total_updates = 0;
    let mut pv_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    
    // Initialize counters
    for (pv_name, _) in &monitors {
        pv_counts.insert(pv_name.clone(), 0);
    }
    
    while start.elapsed() < Duration::from_secs(10) {
        for (pv_name, monitor) in &mut monitors {
            if let Ok(Some(update)) = monitor.try_get_update() {
                let count = pv_counts.get_mut(pv_name).unwrap();
                *count += 1;
                total_updates += 1;
                
                // Try to get the value (handle different field names)
                let value_str = if let Ok(val) = update.get_field_double("value") {
                    format!("{:.3}", val)
                } else if let Ok(val) = update.get_field_int32("value") {
                    format!("{}", val)
                } else if let Ok(val) = update.get_field_string("value") {
                    val
                } else {
                    "N/A".to_string()
                };
                
                println!("[{:>8.1}s] {}: {} (update #{})", 
                    start.elapsed().as_secs_f32(), 
                    pv_name, 
                    value_str, 
                    count
                );
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    
    // Stop all monitors
    for (_, mut monitor) in monitors {
        monitor.stop();
    }
    
    println!("\n=== Summary ===");
    println!("Total updates received: {}", total_updates);
    for (pv_name, count) in &pv_counts {
        println!("  {}: {} updates", pv_name, count);
    }
    
    if total_updates > 0 {
        println!("✓ Monitor is working!");
    } else {
        println!("⚠ No updates received - check IOC is running and PVs exist");
    }
    
    Ok(())
}