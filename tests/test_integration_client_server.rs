//! Integration tests for client-server communication using high-level API

use epics_pvxs_sys::{Server, Context, SharedPV, StaticSource};
use std::thread;
use std::time::Duration;

#[test]
fn test_server_client_isolated_communication() {
    // Test basic client-server communication with isolated server
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    // Create a test PV
    let mut test_pv = server.create_pv_double("pi", 3.14159)
        .expect("Failed to create test PV");
    
    // Add PV to server
    server.add_pv("math:pi", &mut test_pv)
        .expect("Failed to add PV to server");
    
    // Start server
    server.start()
        .expect("Failed to start server");
    
    let tcp_port = server.tcp_port();
    let udp_port = server.udp_port();
    
    println!("Isolated server started on TCP:{}, UDP:{}", tcp_port, udp_port);
    
    // Give server a moment to fully start
    thread::sleep(Duration::from_millis(100));
    
    // Update PV value through server-side API
    test_pv.post_double(2.71828)
        .expect("Failed to update PV value");
    
    println!("Updated PV value on server side");
    
    // Try client connection (may fail for isolated servers without special config)
    match Context::from_env() {
        Ok(mut ctx) => {
            match ctx.get("math:pi", 2.0) {
                Ok(value) => {
                    println!("Successfully got value from isolated server: {}", value);
                }
                Err(e) => {
                    println!("GET from isolated server failed (expected for isolated): {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to create client context: {}", e);
        }
    }
    
    // Stop server
    server.stop()
        .expect("Failed to stop server");
    
    println!("Isolated server-client communication test completed");
}

#[test]
fn test_server_multiple_pv_types() {
    // Test server with multiple PV types
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    // Create PVs of different types using server convenience methods
    let mut voltage_pv = server.create_pv_double("voltage", 3.3)
        .expect("Failed to create voltage PV");
    let mut counter_pv = server.create_pv_int32("counter", 0)
        .expect("Failed to create counter PV");
    let mut status_pv = server.create_pv_string("status", "IDLE")
        .expect("Failed to create status PV");
    let mut readonly_pv = server.create_readonly_pv_double("constant", 299792458.0)
        .expect("Failed to create readonly PV");
    
    // Add all PVs to server
    server.add_pv("device:voltage", &mut voltage_pv)
        .expect("Failed to add voltage PV");
    server.add_pv("device:counter", &mut counter_pv)
        .expect("Failed to add counter PV");
    server.add_pv("device:status", &mut status_pv)
        .expect("Failed to add status PV");
    server.add_pv("physics:lightspeed", &mut readonly_pv)
        .expect("Failed to add readonly PV");
    
    // Start server
    server.start()
        .expect("Failed to start server");
    
    println!("Server started with 4 different PV types");
    
    // Update writable PVs with various values
    voltage_pv.post_double(5.0)
        .expect("Failed to update voltage");
    counter_pv.post_int32(42)
        .expect("Failed to update counter");
    status_pv.post_string("RUNNING")
        .expect("Failed to update status");
    
    println!("Updated all writable PVs");
    
    // Test fetching values from all PVs
    for (name, pv) in [
        ("voltage", &voltage_pv),
        ("counter", &counter_pv),
        ("status", &status_pv),
        ("lightspeed", &readonly_pv),
    ] {
        match pv.fetch() {
            Ok(value) => {
                println!("Fetched {} value: {}", name, value);
            }
            Err(e) => {
                println!("Failed to fetch {} value: {}", name, e);
            }
        }
    }
    
    // Stop server
    server.stop()
        .expect("Failed to stop server");
    
    println!("Multiple PV types test completed");
}

#[test]
fn test_server_with_static_source() {
    // Test server using StaticSource for organization
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    let mut source = StaticSource::create()
        .expect("Failed to create StaticSource");
    
    // Create sensor PVs using direct SharedPV creation
    let mut temp_sensor = SharedPV::create_readonly()
        .expect("Failed to create temperature sensor");
    let mut pressure_sensor = SharedPV::create_readonly()
        .expect("Failed to create pressure sensor");
    let mut flow_control = SharedPV::create_mailbox()
        .expect("Failed to create flow control");
    
    // Initialize sensor values
    temp_sensor.open_double(25.0)
        .expect("Failed to open temperature sensor");
    pressure_sensor.open_double(1013.25)
        .expect("Failed to open pressure sensor");
    flow_control.open_double(15.5)
        .expect("Failed to open flow control");
    
    // Add sensors to source with hierarchical naming
    source.add_pv("room1:temperature", &mut temp_sensor)
        .expect("Failed to add temperature to source");
    source.add_pv("room1:pressure", &mut pressure_sensor)
        .expect("Failed to add pressure to source");
    source.add_pv("room1:airflow", &mut flow_control)
        .expect("Failed to add flow control to source");
    
    // Add source to server with priority
    server.add_source("environmental_controls", &mut source, 10)
        .expect("Failed to add source to server");
    
    // Start server
    server.start()
        .expect("Failed to start server");
    
    println!("Server started with StaticSource containing 3 sensors");
    
    // Simulate control system operation
    flow_control.post_double(18.0)  // Increase airflow
        .expect("Failed to update flow control");
    
    println!("Updated flow control setpoint");
    
    // Test server status
    let tcp_port = server.tcp_port();
    let udp_port = server.udp_port();
    
    assert_ne!(tcp_port, 0, "Server should have assigned TCP port");
    assert_ne!(udp_port, 0, "Server should have assigned UDP port");
    
    println!("Server running on TCP:{}, UDP:{}", tcp_port, udp_port);
    
    // Clean shutdown
    server.stop()
        .expect("Failed to stop server");
    
    source.close_all()
        .expect("Failed to close source PVs");
    
    println!("StaticSource integration test completed");
}

#[test]
fn test_concurrent_server_operations() {
    // Test concurrent operations on server and PVs
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    // Create multiple PVs for concurrent testing
    let mut pvs = Vec::new();
    for i in 0..5 {
        let pv = server.create_pv_double(&format!("concurrent{}", i), i as f64)
            .expect("Failed to create concurrent PV");
        pvs.push(pv);
    }
    
    // Add all PVs to server
    for (i, pv) in pvs.iter_mut().enumerate() {
        let pv_name = format!("concurrent:pv{}", i);
        server.add_pv(&pv_name, pv)
            .expect("Failed to add PV to server");
    }
    
    // Start server
    server.start()
        .expect("Failed to start server");
    
    println!("Server started with {} concurrent PVs", pvs.len());
    
    // Update all PVs rapidly
    for (i, pv) in pvs.iter_mut().enumerate() {
        let new_value = (i as f64) * 10.0 + 100.0;
        pv.post_double(new_value)
            .expect("Failed to post concurrent update");
    }
    
    println!("Posted concurrent updates to all PVs");
    
    // Verify all PVs are still operational
    for (i, pv) in pvs.iter().enumerate() {
        assert!(pv.is_open(), "PV {} should still be open", i);
        
        match pv.fetch() {
            Ok(value) => {
                println!("PV {} current value: {}", i, value);
            }
            Err(e) => {
                println!("Failed to fetch PV {} value: {}", i, e);
            }
        }
    }
    
    // Remove PVs one by one while server is running
    for i in 0..pvs.len() {
        let pv_name = format!("concurrent:pv{}", i);
        server.remove_pv(&pv_name)
            .expect("Failed to remove concurrent PV");
    }
    
    println!("Removed all concurrent PVs from running server");
    
    // Stop server
    server.stop()
        .expect("Failed to stop server");
    
    println!("Concurrent operations test completed");
}

#[test]
fn test_server_lifecycle_with_pvs() {
    // Test complete server lifecycle with PV management
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    // Test adding PVs to stopped server
    let mut pv1 = server.create_pv_double("test1", 1.0)
        .expect("Failed to create PV1");
    
    server.add_pv("lifecycle:test1", &mut pv1)
        .expect("Failed to add PV to stopped server");
    
    // Start server
    server.start()
        .expect("Failed to start server");
    
    // Add PV to running server
    let mut pv2 = server.create_pv_int32("test2", 2)
        .expect("Failed to create PV2");
    
    server.add_pv("lifecycle:test2", &mut pv2)
        .expect("Failed to add PV to running server");
    
    // Update PVs while server is running
    pv1.post_double(10.0)
        .expect("Failed to update PV1");
    pv2.post_int32(20)
        .expect("Failed to update PV2");
    
    // Remove one PV while running
    server.remove_pv("lifecycle:test1")
        .expect("Failed to remove PV from running server");
    
    // Stop server
    server.stop()
        .expect("Failed to stop server");
    
    // Remove remaining PV from stopped server
    server.remove_pv("lifecycle:test2")
        .expect("Failed to remove PV from stopped server");
    
    // Restart server (should be empty now)
    server.start()
        .expect("Failed to restart server");
    
    server.stop()
        .expect("Failed to stop restarted server");
    
    println!("Complete server lifecycle test completed");
}

#[test]
fn test_error_recovery_scenarios() {
    // Test server error recovery scenarios
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    // Try operations in wrong order
    match server.remove_pv("nonexistent") {
        Ok(_) => println!("Removing nonexistent PV succeeded (idempotent)"),
        Err(e) => println!("Removing nonexistent PV failed: {}", e),
    }
    
    // Create PV with problematic values
    let mut extreme_pv = server.create_pv_double("extreme", f64::INFINITY)
        .expect("Failed to create extreme value PV");
    
    server.add_pv("test:extreme", &mut extreme_pv)
        .expect("Failed to add extreme PV");
    
    // Start server
    server.start()
        .expect("Failed to start server");
    
    // Post various extreme values
    let extreme_values = [f64::NEG_INFINITY, 0.0, f64::MAX, f64::MIN];
    
    for &value in &extreme_values {
        match extreme_pv.post_double(value) {
            Ok(_) => println!("Posted extreme value {} succeeded", value),
            Err(e) => println!("Posted extreme value {} failed: {}", value, e),
        }
    }
    
    // Test NaN separately
    match extreme_pv.post_double(f64::NAN) {
        Ok(_) => println!("Posted NaN succeeded"),
        Err(e) => println!("Posted NaN failed: {}", e),
    }
    
    // Clean shutdown
    server.stop()
        .expect("Failed to stop server");
    
    println!("Error recovery scenarios test completed");
}