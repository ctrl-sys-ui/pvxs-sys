// Simple PUT example - demonstrates writing a PV value

use epics_pvxs_sys::{Context, PvxsError};
use std::env;

fn main() -> Result<(), PvxsError> {
    // Get PV name and value from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <pv_name> <value>", args[0]);
        eprintln!("Example: {} my:pv:name 42.5", args[0]);
        std::process::exit(1);
    }
    
    let pv_name = &args[1];
    let value_str = &args[2];
    let timeout = 5.0; // 5 second timeout
    
    // Parse the value as a float
    let new_value: f64 = value_str.parse().map_err(|e| {
        PvxsError::new(format!("Failed to parse value '{}' as a number: {}", value_str, e))
    })?;
    
    println!("Creating PVXS context from environment...");
    let mut ctx = Context::from_env()?;
    
    // First, get the current value
    println!("\nReading current value of '{}'...", pv_name);
    match ctx.get(pv_name, timeout) {
        Ok(current) => {
            println!("Current value structure:");
            println!("{}", current);
            
            // Try to show the current numeric value
            if let Ok(v) = current.get_field_double("value") {
                println!("\nCurrent value (double): {}", v);
            } else if let Ok(v) = current.get_field_int32("value") {
                println!("\nCurrent value (int32): {}", v);
            }
        }
        Err(e) => {
            println!("Warning: Could not read current value: {}", e);
        }
    }
    
    // Now perform the PUT operation
    println!("\nWriting new value {} to '{}'...", new_value, pv_name);
    ctx.put_double(pv_name, new_value, timeout)?;
    println!("PUT operation completed successfully!");
    
    // Verify by reading back
    println!("\nVerifying by reading back...");
    let updated = ctx.get(pv_name, timeout)?;
    
    match updated.get_field_double("value") {
        Ok(v) => {
            println!("New value (double): {}", v);
            if (v - new_value).abs() < 1e-10 {
                println!("✓ Value confirmed!");
            } else {
                println!("⚠ Warning: Read back value {} differs from written value {}", v, new_value);
            }
        }
        Err(_) => {
            match updated.get_field_int32("value") {
                Ok(v) => {
                    println!("New value (int32): {}", v);
                    if (v as f64 - new_value).abs() < 1.0 {
                        println!("✓ Value confirmed (converted to integer)!");
                    }
                }
                Err(e) => {
                    println!("Could not read back value: {}", e);
                }
            }
        }
    }
    
    println!("\nPUT example completed!");
    
    Ok(())
}
