// Simple INFO example - demonstrates querying PV type information

use epics_pvxs_sys::{Context, PvxsError};
use std::env;

fn main() -> Result<(), PvxsError> {
    // Get PV name from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <pv_name>", args[0]);
        eprintln!("Example: {} my:pv:name", args[0]);
        eprintln!();
        eprintln!("This example queries the type information and structure");
        eprintln!("of a process variable without reading its current value.");
        std::process::exit(1);
    }
    
    let pv_name = &args[1];
    let timeout = 5.0; // 5 second timeout
    
    println!("Creating PVXS context from environment...");
    let mut ctx = Context::from_env()?;
    
    println!("Getting type information for '{}'...", pv_name);
    let info = ctx.info(pv_name, timeout)?;
    
    println!("\n=== PV Type Information ===");
    println!("PV Name: {}", pv_name);
    println!("\nComplete structure definition:");
    println!("{}", info);
    
    // Try to analyze the structure by checking for common fields
    println!("\n=== Structure Analysis ===");
    
    // Check if it has a value field
    match info.get_field_double("value") {
        Ok(_) => println!("✓ Has 'value' field (double type)"),
        Err(_) => {
            match info.get_field_int32("value") {
                Ok(_) => println!("✓ Has 'value' field (int32 type)"),
                Err(_) => {
                    match info.get_field_string("value") {
                        Ok(_) => println!("✓ Has 'value' field (string type)"),
                        Err(_) => println!("✗ No standard 'value' field detected"),
                    }
                }
            }
        }
    }
    
    // Check for alarm fields
    match info.get_field_int32("alarm.severity") {
        Ok(_) => println!("✓ Has alarm information (alarm.severity)"),
        Err(_) => println!("✗ No alarm.severity field"),
    }
    
    match info.get_field_string("alarm.message") {
        Ok(_) => println!("✓ Has alarm message (alarm.message)"),
        Err(_) => println!("✗ No alarm.message field"),
    }
    
    // Check for timestamp fields
    match info.get_field_int32("timeStamp.secondsPastEpoch") {
        Ok(_) => println!("✓ Has timestamp information (timeStamp.secondsPastEpoch)"),
        Err(_) => println!("✗ No timeStamp.secondsPastEpoch field"),
    }
    
    // Check for control fields
    match info.get_field_double("control.limitLow") {
        Ok(_) => println!("✓ Has control limits (control.limitLow)"),
        Err(_) => println!("✗ No control.limitLow field"),
    }
    
    match info.get_field_double("control.limitHigh") {
        Ok(_) => println!("✓ Has control limits (control.limitHigh)"),
        Err(_) => println!("✗ No control.limitHigh field"),
    }
    
    // Check for display fields
    match info.get_field_string("display.units") {
        Ok(_) => println!("✓ Has engineering units (display.units)"),
        Err(_) => println!("✗ No display.units field"),
    }
    
    match info.get_field_string("display.description") {
        Ok(_) => println!("✓ Has description (display.description)"),
        Err(_) => println!("✗ No display.description field"),
    }
    
    println!("\n=== Summary ===");
    println!("This INFO operation retrieved the structure definition for '{}'", pv_name);
    println!("without actually reading the current value. This is useful for:");
    println!("• Discovering what fields are available in a PV");
    println!("• Understanding the data type of the main value");
    println!("• Checking what metadata (alarms, timestamps, etc.) is provided");
    println!("• Building dynamic clients that adapt to PV structure");
    
    println!("\nINFO operation completed successfully!");
    
    Ok(())
}