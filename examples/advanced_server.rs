// Advanced PVXS server example using StaticSource
// This demonstrates creating a server with multiple sources and more complex PV organization

use epics_pvxs_sys::bridge::*;
use std::time::Duration;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting advanced PVXS server example...");
    
    // Create server from environment (uses EPICS environment variables)
    let mut server = server_create_from_env()?;
    println!("Created server from environment configuration");
    
    // Create multiple static sources for different device groups
    let mut device1_source = static_source_create()?;
    let mut device2_source = static_source_create()?;
    
    println!("Created static sources");
    
    // Device 1: Temperature monitoring system
    let mut temp1_pv = shared_pv_create_readonly()?;
    let mut temp2_pv = shared_pv_create_readonly()?;
    let mut temp_alarm_pv = shared_pv_create_mailbox()?;
    
    shared_pv_open_double(temp1_pv.pin_mut(), 22.3)?;
    shared_pv_open_double(temp2_pv.pin_mut(), 23.1)?;
    shared_pv_open_string(temp_alarm_pv.pin_mut(), "OK".to_string())?;
    
    static_source_add_pv(device1_source.pin_mut(), "device1:temp1".to_string(), temp1_pv.pin_mut())?;
    static_source_add_pv(device1_source.pin_mut(), "device1:temp2".to_string(), temp2_pv.pin_mut())?;
    static_source_add_pv(device1_source.pin_mut(), "device1:alarm".to_string(), temp_alarm_pv.pin_mut())?;
    
    // Device 2: Motion control system
    let mut position_pv = shared_pv_create_mailbox()?;
    let mut velocity_pv = shared_pv_create_readonly()?;
    let mut moving_pv = shared_pv_create_readonly()?;
    
    shared_pv_open_double(position_pv.pin_mut(), 0.0)?;
    shared_pv_open_double(velocity_pv.pin_mut(), 0.0)?;
    shared_pv_open_int32(moving_pv.pin_mut(), 0)?; // 0 = stopped, 1 = moving
    
    static_source_add_pv(device2_source.pin_mut(), "device2:position".to_string(), position_pv.pin_mut())?;
    static_source_add_pv(device2_source.pin_mut(), "device2:velocity".to_string(), velocity_pv.pin_mut())?;
    static_source_add_pv(device2_source.pin_mut(), "device2:moving".to_string(), moving_pv.pin_mut())?;
    
    println!("Added PVs to static sources");
    
    // Add sources to server with different priorities (lower number = higher priority)
    server_add_source(server.pin_mut(), "temperature_monitor".to_string(), device1_source.pin_mut(), 10)?;
    server_add_source(server.pin_mut(), "motion_control".to_string(), device2_source.pin_mut(), 20)?;
    
    // Also add some PVs directly to the server's built-in source
    let mut global_status_pv = shared_pv_create_mailbox()?;
    let mut uptime_pv = shared_pv_create_readonly()?;
    
    shared_pv_open_string(global_status_pv.pin_mut(), "STARTING".to_string())?;
    shared_pv_open_int32(uptime_pv.pin_mut(), 0)?;
    
    server_add_pv(server.pin_mut(), "system:status".to_string(), global_status_pv.pin_mut())?;
    server_add_pv(server.pin_mut(), "system:uptime".to_string(), uptime_pv.pin_mut())?;
    
    println!("Added sources and global PVs to server");
    
    // Start the server
    server_start(server.pin_mut())?;
    let tcp_port = server_get_tcp_port(&server);
    let udp_port = server_get_udp_port(&server);
    
    println!("Advanced server started on TCP port {} and UDP port {}", tcp_port, udp_port);
    println!("");
    println!("Available PV groups:");
    println!("");
    println!("Temperature Monitor (device1):");
    println!("  device1:temp1      - Temperature sensor 1 (°C)");
    println!("  device1:temp2      - Temperature sensor 2 (°C)");
    println!("  device1:alarm      - Temperature alarm status");
    println!("");
    println!("Motion Control (device2):");
    println!("  device2:position   - Current position (mm)");
    println!("  device2:velocity   - Current velocity (mm/s)");
    println!("  device2:moving     - Motion status (0=stopped, 1=moving)");
    println!("");
    println!("System Status:");
    println!("  system:status      - Overall system status");
    println!("  system:uptime      - System uptime (seconds)");
    println!("");
    println!("Example commands:");
    println!("  pvget device1:temp1");
    println!("  pvput device2:position 10.5");
    println!("  pvput system:status \"OPERATIONAL\"");
    println!("");
    println!("Press Ctrl+C to stop the server");
    
    // Update system status
    shared_pv_post_string(global_status_pv.pin_mut(), "OPERATIONAL".to_string())?;
    
    // Simulation variables
    let mut uptime = 0i32;
    let mut temp1 = 22.3f64;
    let mut temp2 = 23.1f64;
    let mut position = 0.0f64;
    let mut target_position = 0.0f64;
    let mut velocity = 0.0f64;
    let mut moving = 0i32;
    
    let start_time = std::time::Instant::now();
    
    loop {
        thread::sleep(Duration::from_millis(500)); // Update every 500ms
        
        uptime = start_time.elapsed().as_secs() as i32;
        shared_pv_post_int32(uptime_pv.pin_mut(), uptime)?;
        
        // Simulate temperature fluctuations
        temp1 += (rand() * 2.0 - 1.0) * 0.1;
        temp2 += (rand() * 2.0 - 1.0) * 0.1;
        temp1 = temp1.max(20.0).min(30.0);
        temp2 = temp2.max(20.0).min(30.0);
        
        shared_pv_post_double(temp1_pv.pin_mut(), temp1)?;
        shared_pv_post_double(temp2_pv.pin_mut(), temp2)?;
        
        // Check for temperature alarm
        let max_temp = temp1.max(temp2);
        let alarm_status = if max_temp > 28.0 {
            "HIGH"
        } else if max_temp < 18.0 {
            "LOW"
        } else {
            "OK"
        };
        shared_pv_post_string(temp_alarm_pv.pin_mut(), alarm_status.to_string())?;
        
        // Simulate motion control
        // Check if we need to move towards a different target
        if (uptime % 10) == 0 && (uptime > 0) { // Change target every 10 seconds
            target_position = (rand() * 20.0 - 10.0); // Random position between -10 and 10
        }
        
        // Simple motion simulation
        let position_error = target_position - position;
        if position_error.abs() > 0.1 {
            moving = 1;
            velocity = position_error.signum() * (position_error.abs().min(2.0)); // Max velocity 2 mm/s
            position += velocity * 0.5; // 500ms timestep
        } else {
            moving = 0;
            velocity = 0.0;
        }
        
        shared_pv_post_double(position_pv.pin_mut(), position)?;
        shared_pv_post_double(velocity_pv.pin_mut(), velocity)?;
        shared_pv_post_int32(moving_pv.pin_mut(), moving)?;
        
        // Print status every 5 seconds
        if (uptime % 5) == 0 && uptime > 0 {
            println!("Status update [{}s]: temp1={:.1}°C, temp2={:.1}°C, alarm={}, pos={:.1}mm, vel={:.1}mm/s, moving={}", 
                     uptime, temp1, temp2, alarm_status, position, velocity, moving);
        }
    }
}

// Simple pseudo-random number generator for demo purposes
fn rand() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    (nanos as f64) / (u32::MAX as f64)
}