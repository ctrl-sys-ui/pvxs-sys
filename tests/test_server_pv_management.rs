//! Test Server PV management functions (add_pv, remove_pv, create_pv_*)

use epics_pvxs_sys::Server;

#[test]
fn test_server_create_pv_double() {
    // Test creating double PVs through server
    let server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    let pv = server.create_pv_double("test_double", 42.5)
        .expect("Failed to create double PV");
    
    assert!(pv.is_open());
    
    // Verify we can fetch the initial value
    let value = pv.fetch()
        .expect("Failed to fetch PV value");
    
    println!("Created double PV with value: {}", value);
}

#[test]
fn test_server_create_pv_int32() {
    // Test creating int32 PVs through server
    let server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    let pv = server.create_pv_int32("test_int", 100)
        .expect("Failed to create int32 PV");
    
    assert!(pv.is_open());
    
    let value = pv.fetch()
        .expect("Failed to fetch PV value");
    
    println!("Created int32 PV with value: {}", value);
}

#[test]
fn test_server_create_pv_string() {
    // Test creating string PVs through server
    let server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    let pv = server.create_pv_string("test_string", "Hello PVXS")
        .expect("Failed to create string PV");
    
    assert!(pv.is_open());
    
    let value = pv.fetch()
        .expect("Failed to fetch PV value");
    
    println!("Created string PV with value: {}", value);
}

#[test]
fn test_server_create_readonly_pv_double() {
    // Test creating readonly double PVs
    let server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    let pv = server.create_readonly_pv_double("readonly_test", 99.99)
        .expect("Failed to create readonly double PV");
    
    assert!(pv.is_open());
    
    let value = pv.fetch()
        .expect("Failed to fetch readonly PV value");
    
    println!("Created readonly double PV with value: {}", value);
}

#[test]
fn test_server_add_pv() {
    // Test adding PVs to server
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    let mut pv = server.create_pv_double("counter", 0.0)
        .expect("Failed to create PV");
    
    // Add PV to server
    server.add_pv("test:counter", &mut pv)
        .expect("Failed to add PV to server");
    
    println!("Successfully added PV to server");
}

#[test]
fn test_server_remove_pv() {
    // Test removing PVs from server
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    let mut pv = server.create_pv_double("temp", 23.5)
        .expect("Failed to create PV");
    
    // Add PV first
    server.add_pv("test:temperature", &mut pv)
        .expect("Failed to add PV to server");
    
    // Remove PV
    server.remove_pv("test:temperature")
        .expect("Failed to remove PV from server");
    
    println!("Successfully removed PV from server");
}

#[test]
fn test_server_pv_lifecycle() {
    // Test complete PV lifecycle with server
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    // Create multiple PVs of different types
    let mut double_pv = server.create_pv_double("voltage", 3.3)
        .expect("Failed to create double PV");
    let mut int_pv = server.create_pv_int32("count", 42)
        .expect("Failed to create int PV");
    let mut string_pv = server.create_pv_string("status", "IDLE")
        .expect("Failed to create string PV");
    
    // Add all PVs to server
    server.add_pv("device:voltage", &mut double_pv)
        .expect("Failed to add voltage PV");
    server.add_pv("device:count", &mut int_pv)
        .expect("Failed to add count PV");
    server.add_pv("device:status", &mut string_pv)
        .expect("Failed to add status PV");
    
    // Start server
    server.start()
        .expect("Failed to start server");
    
    println!("Server started with {} PVs", 3);
    
    // Update PV values
    double_pv.post_double(5.0)
        .expect("Failed to update voltage");
    int_pv.post_int32(100)
        .expect("Failed to update count");
    string_pv.post_string("RUNNING")
        .expect("Failed to update status");
    
    println!("Updated all PV values");
    
    // Remove PVs one by one
    server.remove_pv("device:voltage")
        .expect("Failed to remove voltage PV");
    server.remove_pv("device:count")
        .expect("Failed to remove count PV");
    server.remove_pv("device:status")
        .expect("Failed to remove status PV");
    
    println!("Removed all PVs from server");
    
    // Stop server
    server.stop()
        .expect("Failed to stop server");
    
    println!("PV lifecycle test completed");
}

#[test]
fn test_server_pv_name_conflicts() {
    // Test adding PVs with duplicate names
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    let mut pv1 = server.create_pv_double("first", 1.0)
        .expect("Failed to create first PV");
    let mut pv2 = server.create_pv_double("second", 2.0)
        .expect("Failed to create second PV");
    
    // Add first PV
    server.add_pv("conflict:test", &mut pv1)
        .expect("Failed to add first PV");
    
    // Try to add second PV with same name
    match server.add_pv("conflict:test", &mut pv2) {
        Ok(_) => {
            println!("Adding duplicate PV name succeeded (overwrite behavior)");
            // Clean up - remove the PV
            let _ = server.remove_pv("conflict:test");
        }
        Err(e) => {
            println!("Adding duplicate PV name failed as expected: {}", e);
            // Clean up first PV only
            server.remove_pv("conflict:test")
                .expect("Failed to remove first PV");
        }
    }
}

#[test]
fn test_server_remove_nonexistent_pv() {
    // Test removing non-existent PV
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    match server.remove_pv("nonexistent:pv") {
        Ok(_) => {
            println!("Removing non-existent PV succeeded (idempotent behavior)");
        }
        Err(e) => {
            println!("Removing non-existent PV failed as expected: {}", e);
            assert!(!e.to_string().is_empty());
        }
    }
}