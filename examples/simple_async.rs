// examples/simple_async.rs
//! Simple Async Example - Demonstrates async/await functionality
//! 
//! This example shows:
//! - Using async Context methods with tokio
//! - Reading different data types asynchronously
//! - Writing values asynchronously
//! - Getting PV info asynchronously
//! - Concurrent async operations
//! 
//! Usage:
//!   cargo run --features async --example simple_async -- PV_NAME [PV_NAME2 ...]
//!   cargo run --features async --example simple_async -- TEST:DOUBLE
//!   cargo run --features async --example simple_async -- TEST:COUNTER TEST:RANDOM TEST:TEMPERATURE

#[cfg(feature = "async")]
use epics_pvxs_sys::Context;
#[cfg(feature = "async")]
use std::env;
#[cfg(feature = "async")]
use tokio::time::{timeout, Duration};

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <PV_NAME> [PV_NAME2] [PV_NAME3] ...", args[0]);
        eprintln!("Example: {} TEST:DOUBLE", args[0]);
        eprintln!("Example: {} TEST:COUNTER TEST:RANDOM TEST:TEMPERATURE", args[0]);
        eprintln!();
        eprintln!("This example demonstrates async/await functionality.");
        eprintln!("Requires the 'async' feature to be enabled.");
        std::process::exit(1);
    }
    
    let pv_names: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
    
    println!("=== Simple Async Example ===");
    println!("Testing async operations with PVs: {:?}", pv_names);
    println!();
    
    // Create context
    let mut ctx = Context::from_env()?;
    println!("✓ Created PVXS context");
    
    // Test each PV with comprehensive async operations
    for (i, pv_name) in pv_names.iter().enumerate() {
        println!("\n--- Testing PV #{}: {} ---", i + 1, pv_name);
        
        // Test async operations for this PV
        if let Err(e) = test_pv_async(&mut ctx, pv_name).await {
            eprintln!("✗ Error testing {}: {}", pv_name, e);
        }
    }
    
    // Test concurrent operations if we have multiple PVs
    if pv_names.len() > 1 {
        println!("\n--- Testing Concurrent Async Operations ---");
        test_concurrent_operations(&mut ctx, &pv_names).await?;
    }
    
    println!("\n=== Async Operations Complete ===");
    Ok(())
}

#[cfg(feature = "async")]
async fn test_pv_async(ctx: &mut Context, pv_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let timeout_duration = 5.0;
    
    // 1. Test async INFO operation
    println!("  📋 Testing info_async...");
    match timeout(Duration::from_secs(6), ctx.info_async(pv_name, timeout_duration)).await {
        Ok(Ok(info)) => {
            println!("     ✓ INFO successful - got structure info");
            // Show basic structure information
            println!("     📝 Structure type detected");
        }
        Ok(Err(e)) => println!("     ✗ INFO failed: {}", e),
        Err(_) => println!("     ✗ INFO timed out"),
    }
    
    // 2. Test async GET operation
    println!("  📥 Testing get_async...");
    match timeout(Duration::from_secs(6), ctx.get_async(pv_name, timeout_duration)).await {
        Ok(Ok(value)) => {
            println!("     ✓ GET successful");
            
            // Try to extract different data types
            test_value_extraction(&value);
        }
        Ok(Err(e)) => println!("     ✗ GET failed: {}", e),
        Err(_) => println!("     ✗ GET timed out"),
    }
    
    // 3. Test async PUT operation (only for writable PVs)
    if pv_name.contains("SETPOINT") || pv_name.contains("OUT") || pv_name.contains("DOUBLE") {
        println!("  📤 Testing put_double_async...");
        let test_value = 123.456;
        
        match timeout(Duration::from_secs(6), ctx.put_double_async(pv_name, test_value, timeout_duration)).await {
            Ok(Ok(())) => {
                println!("     ✓ PUT successful - wrote {}", test_value);
                
                // Verify the write with another GET
                tokio::time::sleep(Duration::from_millis(100)).await;
                match ctx.get_async(pv_name, timeout_duration).await {
                    Ok(verify_value) => {
                        if let Ok(read_back) = verify_value.get_field_double("value") {
                            println!("     ✓ Verification read: {}", read_back);
                            if (read_back - test_value).abs() < 0.001 {
                                println!("     ✓ Write/read verification successful!");
                            } else {
                                println!("     ⚠ Value mismatch - may be read-only or modified by IOC");
                            }
                        }
                    }
                    Err(e) => println!("     ⚠ Verification read failed: {}", e),
                }
            }
            Ok(Err(e)) => println!("     ✗ PUT failed: {} (may be read-only)", e),
            Err(_) => println!("     ✗ PUT timed out"),
        }
    } else {
        println!("  📤 Skipping PUT test (PV appears to be read-only)");
    }
    
    Ok(())
}

#[cfg(feature = "async")]
fn test_value_extraction(value: &epics_pvxs_sys::Value) {
    // Try different field access patterns
    println!("     🔍 Extracting value data:");
    
    // Try double
    if let Ok(double_val) = value.get_field_double("value") {
        println!("       • Double value: {}", double_val);
    }
    
    // Try int32
    if let Ok(int_val) = value.get_field_int32("value") {
        println!("       • Int32 value: {}", int_val);
    }
    
    // Try string
    if let Ok(string_val) = value.get_field_string("value") {
        println!("       • String value: \"{}\"", string_val);
    }
    
    // Try timestamp if available
    if let Ok(timestamp) = value.get_field_double("timeStamp.secondsPastEpoch") {
        println!("       • Timestamp: {}", timestamp);
    }
    
    // Try alarm severity
    if let Ok(severity) = value.get_field_int32("alarm.severity") {
        let severity_str = match severity {
            0 => "NO_ALARM",
            1 => "MINOR",
            2 => "MAJOR",
            3 => "INVALID",
            _ => "UNKNOWN",
        };
        println!("       • Alarm severity: {} ({})", severity, severity_str);
    }
    
    // Try alarm status
    if let Ok(status) = value.get_field_int32("alarm.status") {
        println!("       • Alarm status: {}", status);
    }
}

#[cfg(feature = "async")]
async fn test_concurrent_operations(ctx: &mut Context, pv_names: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running {} sequential async operations...", pv_names.len());
    
    let start = std::time::Instant::now();
    
    // Sequential async operations (since Context requires &mut self)
    let mut results = Vec::new();
    for &pv_name in pv_names {
        println!("  🚀 Starting async GET for: {}", pv_name);
        let result = timeout(
            Duration::from_secs(5),
            ctx.get_async(pv_name, 3.0)
        ).await;
        
        match result {
            Ok(Ok(value)) => {
                if let Ok(val) = value.get_field_double("value") {
                    results.push((pv_name, Ok(val)));
                    println!("     ✓ {}: {}", pv_name, val);
                } else if let Ok(val) = value.get_field_int32("value") {
                    results.push((pv_name, Ok(val as f64)));
                    println!("     ✓ {}: {}", pv_name, val);
                } else {
                    results.push((pv_name, Ok(0.0)));
                    println!("     ✓ {}: <non-numeric>", pv_name);
                }
            }
            Ok(Err(e)) => {
                results.push((pv_name, Err(format!("{}", e))));
                println!("     ✗ {}: {}", pv_name, e);
            }
            Err(_) => {
                results.push((pv_name, Err("Timeout".to_string())));
                println!("     ✗ {}: Timeout", pv_name);
            }
        }
    }
    
    let duration = start.elapsed();
    println!("Completed {} operations in {:.2}s", results.len(), duration.as_secs_f64());
    
    // Summary
    let successful = results.iter().filter(|(_, result)| result.is_ok()).count();
    println!("Success rate: {}/{} ({:.1}%)", 
        successful, 
        results.len(), 
        (successful as f64 / results.len() as f64) * 100.0
    );
    
    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("This example requires the 'async' feature to be enabled.");
    eprintln!("Run with: cargo run --features async --example simple_async -- PV_NAME");
    std::process::exit(1);
}

#[cfg(all(test, feature = "async"))]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_async_context_creation() {
        // Test that we can create a context in an async context
        let result = Context::from_env();
        assert!(result.is_ok(), "Failed to create context: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_async_operations_timeout() {
        // Test that async operations respect timeouts
        let mut ctx = Context::from_env().expect("Failed to create context");
        
        // Try to access a non-existent PV with short timeout
        let result = timeout(
            Duration::from_secs(2),
            ctx.get_async("NON_EXISTENT_PV", 1.0)
        ).await;
        
        // Should either timeout or return an error
        match result {
            Ok(Ok(_)) => panic!("Unexpected success for non-existent PV"),
            Ok(Err(_)) => println!("Got expected error for non-existent PV"),
            Err(_) => println!("Got expected timeout for non-existent PV"),
        }
    }
}