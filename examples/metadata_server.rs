use pvxs_sys::{Server, StaticSource, DisplayMetadata, ControlMetadata, 
                      ValueAlarmMetadata, NTScalarMetadataBuilder};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting EPICS PVA server with rich NTScalar metadata...");
    
    // Create server with isolated configuration
    let mut server = Server::from_env()?;
    println!("Server created in isolated mode");
    
    // Create a static source
    let mut source = StaticSource::create()?;
    println!("Static source created");
    
    // Create metadata with display limits and units
    let metadata = NTScalarMetadataBuilder::new()
        .alarm(0, 0, "OK")
        .display(DisplayMetadata {
            limit_low: 0,
            limit_high: 100,
            description: "Temperature sensor reading".to_string(),
            units: "DegC".to_string(),
            precision: 2,
        })
        .control(ControlMetadata {
            limit_low: -10.0,
            limit_high: 110.0,
            min_step: 0.1,
        })
        .value_alarm(ValueAlarmMetadata {
            active: true,
            low_alarm_limit: 5.0,
            low_warning_limit: 10.0,
            high_warning_limit: 90.0,
            high_alarm_limit: 95.0,
            low_alarm_severity: 2,
            low_warning_severity: 1,
            high_warning_severity: 1,
            high_alarm_severity: 2,
            hysteresis: 1,
        })
        .with_form(true);
    
    // Create PV with metadata
    let mut pv = server.create_pv_double("temp_sensor", 25.5, metadata)?;
    println!("Created PV with metadata: initial value = 25.5°C");
    
    // Add to source
    source.add_pv("temperature:sensor1", &mut pv)?;
    
    // Add source to server
    server.add_source("static", &mut source, 0)?;
    
    // Start the server
    server.start()?;
    println!("\nServer started successfully!");
    println!("PV available at: temperature:sensor1");
    println!("\nTest with:");
    println!("  pvget temperature:sensor1");
    println!("  pvinfo temperature:sensor1");
    println!("\nPress Ctrl+C to stop...\n");
    
    // Keep running
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
