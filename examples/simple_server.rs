// Simple PVXS server example
// This demonstrates creating and running a basic PVXS server with some PVs

use epics_pvxs_sys::bridge::*;
use std::time::Duration;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting PVXS server example...");
    
    // Create server from environment (for proper discovery)
    let mut server = server_create_from_env()?;
    println!("Created server from environment");
    
    // Create some shared PVs
    let mut counter_pv = shared_pv_create_mailbox()?;
    let mut temperature_pv = shared_pv_create_readonly()?;
    let mut status_pv = shared_pv_create_mailbox()?;
    
    // Open PVs with initial values
    shared_pv_open_int32(counter_pv.pin_mut(), 0)?;
    shared_pv_open_double(temperature_pv.pin_mut(), 23.5)?; // Room temperature
    shared_pv_open_string(status_pv.pin_mut(), "IDLE".to_string())?;
    
    println!("Opened PVs with initial values");
    
    // Add PVs to server
    server_add_pv(server.pin_mut(), "example:counter".to_string(), counter_pv.pin_mut())?;
    server_add_pv(server.pin_mut(), "example:temperature".to_string(), temperature_pv.pin_mut())?;
    server_add_pv(server.pin_mut(), "example:status".to_string(), status_pv.pin_mut())?;
    
    println!("Added PVs to server");
    
    // Start the server
    server_start(server.pin_mut())?;
    let tcp_port = server_get_tcp_port(&server);
    let udp_port = server_get_udp_port(&server);
    println!("Server started on TCP port {} and UDP port {}", tcp_port, udp_port);
    
    println!("Server is running. Available PVs:");
    println!("  example:counter (int32)");
    println!("  example:temperature (double)");
    println!("  example:status (string)");
    println!("");
    println!("You can test with pvget/pvput:");
    println!("  pvget example:counter");
    println!("  pvput example:counter 42");
    println!("  pvget example:temperature");
    println!("  pvget example:status");
    println!("  pvput example:status \"RUNNING\"");
    println!("");
    println!("Press Ctrl+C to stop the server");
    
    // Simulate some activity - update values periodically
    let mut counter = 0i32;
    let mut temperature = 23.5f64;
    let status_messages = ["IDLE", "RUNNING", "PROCESSING", "COMPLETE"];
    let mut status_index = 0;
    
    loop {
        thread::sleep(Duration::from_secs(2));
        
        // Update counter
        counter += 1;
        if let Err(e) = shared_pv_post_int32(counter_pv.pin_mut(), counter) {
            eprintln!("Error updating counter: {}", e);
        }
        
        // Update temperature with small random variation
        temperature += (rand() * 2.0 - 1.0) * 0.5; // ±0.5°C variation
        temperature = temperature.max(20.0).min(30.0); // Keep in reasonable range
        if let Err(e) = shared_pv_post_double(temperature_pv.pin_mut(), temperature) {
            eprintln!("Error updating temperature: {}", e);
        }
        
        // Update status cyclically
        status_index = (status_index + 1) % status_messages.len();
        if let Err(e) = shared_pv_post_string(status_pv.pin_mut(), status_messages[status_index].to_string()) {
            eprintln!("Error updating status: {}", e);
        }
        
        println!("Updated: counter={}, temperature={:.1}°C, status={}", 
                 counter, temperature, status_messages[status_index]);
    }
}

// Simple pseudo-random number generator for demo purposes
fn rand() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    (nanos as f64) / (u32::MAX as f64)
}