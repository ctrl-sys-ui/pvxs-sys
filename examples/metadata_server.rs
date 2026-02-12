use pvxs_sys::{AlarmSeverity, AlarmStatus, ControlMetadata, DisplayMetadata, NTScalarMetadataBuilder, Server};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting EPICS PVA server with managed PVs and metadata...");
    // Suppress benign TCP disconnect errors (socket error 10054) that occur when clients close connection after one-shot operations
    pvxs_sys::set_logger_level("pvxs.tcp.io", "CRIT")?;

    // Configure PVXS logging to suppress benign TCP disconnect errors
    // These occur when clients (like pvput) close connection after one-shot operations
    // You can also use environment variable: PVXS_LOG="pvxs.tcp.io=CRIT"
    pvxs_sys::set_logger_level("pvxs.tcp.io", "CRIT").ok();
    
    // Alternative: read from PVXS_LOG environment variable
    // pvxs_sys::configure_logging_from_env().ok();
    
    // Create managed server with automatic alarm handling
    let server = Server::start_from_env()?;
    println!("Server started successfully!");
    println!("TCP port: {}", server.tcp_port());
    println!("UDP port: {}", server.udp_port());
    
    // Create metadata with control limits and alarm thresholds
    let metadata = NTScalarMetadataBuilder::new()
        .display( DisplayMetadata {
            limit_low: -10,
            limit_high : 110,
            description: "Temperature Sensor".to_string(), 
            units: "°C".to_string(), 
            precision: 3,
        })
        .control(ControlMetadata {
            limit_low: -10.0,
            limit_high: 110.0,
            min_step: 0.1,
        })
        .alarm_metadata(
            pvxs_sys::AlarmMetadata {
                active: true,
                low_alarm_limit: 20.0,
                low_warning_limit: 30.0,
                high_warning_limit: 90.0,
                high_alarm_limit: 105.0,
                low_alarm_severity: pvxs_sys::AlarmSeverity::Major,
                low_warning_severity: pvxs_sys::AlarmSeverity::Minor,
                high_warning_severity: pvxs_sys::AlarmSeverity::Minor,
                high_alarm_severity: pvxs_sys::AlarmSeverity::Major,
                hysteresis: 1,
            }
        )
        .with_form(true);
    
    // Create PV with automatic alarm computation based on initial value
    // Initial value 25.5 is in the low warning range (20 < 25.5 < 30)
    // so it will automatically get Minor/Low status
    server.create_pv_double("temperature:sensor1", 25.5, metadata)?;
    println!("Created PV: temperature:sensor1 (initial value = 25.5)");
    
    println!("\nTest with:");
    println!("  pvget temperature:sensor1");
    println!("  pvput temperature:sensor1 75.0");
    println!("  pvinfo temperature:sensor1");
    println!("\nAlarm ranges:");
    println!("  < 20.0: MAJOR/LOLO (low alarm)");
    println!("  20.0 - 30.0: MINOR/LOW (low warning)");
    println!("  30.0 - 90.0: NO_ALARM (normal)");
    println!("  90.0 - 105.0: MINOR/HIGH (high warning)");
    println!("  > 105.0: MAJOR/HIHI (high alarm)");
    println!("\nPress Ctrl+C to stop...\n");
    
    // Simulate temperature changes

    loop {
        thread::sleep(Duration::from_secs(2));
        let mut temp = server.fetch_double("temperature:sensor1")?;
        
        if temp.alarm_status >= AlarmStatus::RecordStatus && 
            temp.alarm_severity >= AlarmSeverity::Invalid && 
            temp.alarm_message.contains("OUT_OF_CONTROL_LIMITS") {
            temp.value -= 1.014; // Drop back below high alarm limit to demonstrate hysteresis (105 - 1 hysteresis = 104)
        }
        else {
            temp.value += 1.014; // Continue increasing to demonstrate alarm progression
        }
        server.post_double("temperature:sensor1", temp.value)?;
        let precision = temp.display_metadata.as_ref().map_or(2, |d| d.precision as usize);
        let units = temp.display_metadata.as_ref().map_or("", |d| d.units.as_str());
        println!("Updated temperature to: {:.precision$}{}", temp.value, units);
    }
}
