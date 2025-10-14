// Simple GET example - demonstrates reading a PV value

use epics_pvxs_sys::{Context, PvxsError};
use std::env;

fn main() -> Result<(), PvxsError> {
    // Get PV name from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <pv_name>", args[0]);
        eprintln!("Example: {} my:pv:name", args[0]);
        std::process::exit(1);
    }
    
    let pv_name = &args[1];
    let timeout = 5.0; // 5 second timeout
    
    println!("Creating PVXS context from environment...");
    let mut ctx = Context::from_env()?;
    
    println!("Getting value of '{}'...", pv_name);
    let value = ctx.get(pv_name, timeout)?;
    
    // Print the entire value structure
    println!("\nComplete value structure:");
    println!("{}", value);
    
    // Try to access specific fields
    println!("\nAccessing specific fields:");
    
    // Try to get the main value field
    match value.get_field_double("value") {
        Ok(v) => println!("  value (double): {}", v),
        Err(e) => {
            // Maybe it's an integer?
            match value.get_field_int32("value") {
                Ok(v) => println!("  value (int32): {}", v),
                Err(_) => {
                    // Maybe it's a string?
                    match value.get_field_string("value") {
                        Ok(v) => println!("  value (string): {}", v),
                        Err(_) => println!("  value: Could not read as double, int32, or string: {}", e),
                    }
                }
            }
        }
    }
    
    // Try to get alarm information
    match value.get_field_int32("alarm.severity") {
        Ok(severity) => {
            println!("  alarm.severity: {}", severity);
            let severity_str = match severity {
                0 => "NO_ALARM",
                1 => "MINOR",
                2 => "MAJOR",
                3 => "INVALID",
                _ => "UNKNOWN",
            };
            println!("    ({})", severity_str);
        }
        Err(_) => println!("  alarm.severity: not available"),
    }
    
    match value.get_field_string("alarm.message") {
        Ok(msg) => println!("  alarm.message: {}", msg),
        Err(_) => println!("  alarm.message: not available"),
    }
    
    // Try to get timestamp
    match value.get_field_int32("timeStamp.secondsPastEpoch") {
        Ok(seconds) => println!("  timeStamp.secondsPastEpoch: {}", seconds),
        Err(_) => println!("  timeStamp: not available"),
    }
    
    println!("\nGET operation completed successfully!");
    
    Ok(())
}
