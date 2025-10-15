// RPC Example - Demonstrates Remote Procedure Call functionality
// This example shows how to call server-side functions with arguments

use epics_pvxs_sys::{Context, PvxsError};
use std::env;

fn main() -> Result<(), PvxsError> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <rpc_service_name> [arguments...]", args[0]);
        println!("Example: {} calculator:add a=10.0 b=5.0", args[0]);
        println!("Example: {} system:control command=start enabled=true", args[0]);
        println!();
        println!("This example demonstrates EPICS RPC (Remote Procedure Call) functionality.");
        println!("RPC allows calling server-side functions with typed arguments.");
        println!();
        println!("Argument formats:");
        println!("  string_arg=value     -> String argument");
        println!("  double_arg=3.14      -> Double argument");
        println!("  int_arg=42           -> Integer argument");
        println!("  bool_arg=true        -> Boolean argument");
        return Ok(());
    }
    
    let service_name = &args[1];
    
    println!("🚀 Creating PVXS context from environment...");
    let mut ctx = Context::from_env()?;
    
    println!("📞 Creating RPC call to '{}'...", service_name);
    let mut rpc = ctx.rpc(service_name)?;
    
    // Parse and add arguments from command line
    if args.len() > 2 {
        println!("📝 Adding arguments:");
        for arg in &args[2..] {
            if let Some((name, value)) = arg.split_once('=') {
                match parse_and_add_argument(&mut rpc, name, value) {
                    Ok(arg_type) => println!("  ✅ {} = {} ({})", name, value, arg_type),
                    Err(e) => {
                        println!("  ❌ Failed to add argument '{}': {}", arg, e);
                        return Err(e);
                    }
                }
            } else {
                println!("  ⚠️  Skipping invalid argument format: '{}' (use name=value)", arg);
            }
        }
    } else {
        println!("📝 No arguments provided");
    }
    
    println!();
    println!("⚡ Executing RPC call...");
    
    // Execute with 10 second timeout
    let result = rpc.execute(10.0)?;
    
    println!("✅ RPC call completed successfully!");
    println!();
    println!("📊 Result:");
    println!("{}", result);
    
    // Try to extract common result fields
    println!();
    println!("🔍 Analyzing result structure:");
    
    // Try common field names that might be in the result
    let common_fields = ["result", "value", "status", "message", "data", "output"];
    
    for field in &common_fields {
        // Try different types for each field
        if let Ok(val) = result.get_field_string(field) {
            println!("  📝 {}: \"{}\" (string)", field, val);
        } else if let Ok(val) = result.get_field_double(field) {
            println!("  🔢 {}: {} (double)", field, val);
        } else if let Ok(val) = result.get_field_int32(field) {
            println!("  🔢 {}: {} (int32)", field, val);
        }
    }
    
    println!();
    println!("🎉 RPC demonstration completed!");
    
    Ok(())
}

/// Parse command line argument and add it to RPC with appropriate type
fn parse_and_add_argument(rpc: &mut epics_pvxs_sys::Rpc, name: &str, value: &str) -> Result<&'static str, PvxsError> {
    // Try to infer type from value format
    
    // Boolean values
    if value.eq_ignore_ascii_case("true") {
        rpc.arg_bool(name, true)?;
        return Ok("bool");
    } else if value.eq_ignore_ascii_case("false") {
        rpc.arg_bool(name, false)?;
        return Ok("bool");
    }
    
    // Try parsing as integer first (more specific)
    if let Ok(int_val) = value.parse::<i32>() {
        rpc.arg_int32(name, int_val)?;
        return Ok("int32");
    }
    
    // Try parsing as double
    if let Ok(double_val) = value.parse::<f64>() {
        rpc.arg_double(name, double_val)?;
        return Ok("double");
    }
    
    // Default to string
    rpc.arg_string(name, value)?;
    Ok("string")
}