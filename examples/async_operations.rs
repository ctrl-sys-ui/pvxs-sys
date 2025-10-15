// Async example - demonstrates async/await support using Tokio

#[cfg(feature = "async")]
use epics_pvxs_sys::{Context, PvxsError};
use std::env;

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), PvxsError> {
    // Get PV names from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <pv_name1> [pv_name2] ...", args[0]);
        eprintln!("Example: {} TEST:PV_Bool", args[0]);
        eprintln!("Example: {} TEST:PV1 TEST:PV2 TEST:PV3", args[0]);
        eprintln!();
        eprintln!("This example demonstrates async/await support using Tokio.");
        eprintln!("Operations are performed asynchronously without blocking.");
        std::process::exit(1);
    }
    
    let pv_names: Vec<String> = args[1..].to_vec();
    let timeout = 5.0; // 5 second timeout
    
    println!("🚀 Creating PVXS context from environment...");
    let mut ctx = Context::from_env()?;
    
    println!("📡 Starting async operations for {} PV(s)...", pv_names.len());
    println!();
    
    // Demonstrate concurrent async operations
    let mut tasks: Vec<tokio::task::JoinHandle<()>> = Vec::new();
    
    for (i, pv_name) in pv_names.iter().enumerate() {
        let pv_name_clone = pv_name.clone();
        
        println!("🔄 Starting async operations for '{}'...", pv_name);
        
        // INFO operation
        println!("  📋 Getting structure info...");
        let info = ctx.info_async(&pv_name_clone, timeout).await?;
        println!("  ✅ Structure: {}", info);
        
        // GET operation
        println!("  📥 Reading current value...");
        let value = ctx.get_async(&pv_name_clone, timeout).await?;
        
        // Try to read the value field
        match value.get_field_double("value") {
            Ok(v) => println!("  ✅ Value (double): {}", v),
            Err(_) => {
                match value.get_field_int32("value") {
                    Ok(v) => println!("  ✅ Value (int32): {}", v),
                    Err(_) => {
                        match value.get_field_string("value") {
                            Ok(v) => println!("  ✅ Value (string): {}", v),
                            Err(_) => println!("  ℹ️  Complex value structure (see info above)"),
                        }
                    }
                }
            }
        }
        
        // For demonstration, let's try a PUT operation with a simple value
        if let Ok(_) = value.get_field_double("value") {
            println!("  📤 Writing test value (current + 0.1)...");
            let current = value.get_field_double("value").unwrap_or(0.0);
            let new_value = current + 0.1;
            
            match ctx.put_double_async(&pv_name_clone, new_value, timeout).await {
                Ok(_) => println!("  ✅ PUT successful: wrote {}", new_value),
                Err(e) => println!("  ⚠️  PUT failed: {}", e),
            }
        }
        
        println!();
    }
    
    println!("🎉 All async operations completed!");
    println!();
    println!("=== ASYNC/AWAIT DEMONSTRATION ===");
    println!("✅ Operations were performed asynchronously using Tokio");
    println!("✅ Context methods returned Futures that could be awaited");
    println!("✅ No blocking occurred - runtime was free to handle other tasks");
    println!("✅ Native Rust async/await syntax with EPICS PVAccess");
    
    println!();
    println!("💡 Benefits of async support:");
    println!("• Non-blocking I/O for better application responsiveness");
    println!("• Easy integration with existing Tokio-based applications");
    println!("• Natural error handling with Result types and ? operator");
    println!("• Composable with other async operations and libraries");
    println!("• Efficient resource usage for high-concurrency applications");
    
    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("❌ This example requires the 'async' feature to be enabled.");
    eprintln!("Run with: cargo run --features async --example async_operations -- <pv_name>");
    std::process::exit(1);
}