//! Test StaticSource functions (create, add_pv, remove_pv, close_all)

use epics_pvxs_sys::{StaticSource, SharedPV};

#[test]
fn test_static_source_create() {
    // Test creating StaticSource
    match StaticSource::create() {
        Ok(_source) => {
            println!("Successfully created StaticSource");
        }
        Err(e) => {
            panic!("Failed to create StaticSource: {}", e);
        }
    }
}

#[test]
fn test_static_source_add_pv() {
    // Test adding PV to StaticSource
    let mut source = StaticSource::create()
        .expect("Failed to create StaticSource");
    
    let mut pv = SharedPV::create_mailbox()
        .expect("Failed to create PV");
    
    pv.open_double(42.0)
        .expect("Failed to open PV");
    
    // Add PV to source
    source.add_pv("test:source:pv", &mut pv)
        .expect("Failed to add PV to StaticSource");
    
    println!("Successfully added PV to StaticSource");
}

#[test]
fn test_static_source_remove_pv() {
    // Test removing PV from StaticSource
    let mut source = StaticSource::create()
        .expect("Failed to create StaticSource");
    
    let mut pv = SharedPV::create_mailbox()
        .expect("Failed to create PV");
    
    pv.open_double(1.0)
        .expect("Failed to open PV");
    
    // Add PV first
    source.add_pv("removable:pv", &mut pv)
        .expect("Failed to add PV to source");
    
    // Remove PV
    source.remove_pv("removable:pv")
        .expect("Failed to remove PV from source");
    
    println!("Successfully removed PV from StaticSource");
}

#[test]
fn test_static_source_multiple_pvs() {
    // Test StaticSource with multiple PVs
    let mut source = StaticSource::create()
        .expect("Failed to create StaticSource");
    
    // Create multiple PVs of different types
    let mut double_pv = SharedPV::create_mailbox()
        .expect("Failed to create double PV");
    let mut int_pv = SharedPV::create_readonly()
        .expect("Failed to create int PV");
    let mut string_pv = SharedPV::create_mailbox()
        .expect("Failed to create string PV");
    
    // Open PVs with initial values
    double_pv.open_double(3.14)
        .expect("Failed to open double PV");
    int_pv.open_int32(42)
        .expect("Failed to open int PV");
    string_pv.open_string("test")
        .expect("Failed to open string PV");
    
    // Add all PVs to source
    source.add_pv("group:voltage", &mut double_pv)
        .expect("Failed to add double PV");
    source.add_pv("group:count", &mut int_pv)
        .expect("Failed to add int PV");
    source.add_pv("group:status", &mut string_pv)
        .expect("Failed to add string PV");
    
    println!("Added 3 PVs to StaticSource");
    
    // Remove PVs one by one
    source.remove_pv("group:voltage")
        .expect("Failed to remove voltage PV");
    source.remove_pv("group:count")
        .expect("Failed to remove count PV");
    source.remove_pv("group:status")
        .expect("Failed to remove status PV");
    
    println!("Removed all PVs from StaticSource");
}

#[test]
fn test_static_source_close_all() {
    // Test closing all PVs in StaticSource
    let mut source = StaticSource::create()
        .expect("Failed to create StaticSource");
    
    // Create and add multiple PVs
    let mut pv1 = SharedPV::create_mailbox()
        .expect("Failed to create PV1");
    let mut pv2 = SharedPV::create_mailbox()
        .expect("Failed to create PV2");
    
    pv1.open_double(1.0)
        .expect("Failed to open PV1");
    pv2.open_int32(2)
        .expect("Failed to open PV2");
    
    source.add_pv("close_test:pv1", &mut pv1)
        .expect("Failed to add PV1");
    source.add_pv("close_test:pv2", &mut pv2)
        .expect("Failed to add PV2");
    
    // Verify PVs are open
    assert!(pv1.is_open());
    assert!(pv2.is_open());
    
    // Close all PVs in source
    source.close_all()
        .expect("Failed to close all PVs in source");
    
    println!("Closed all PVs in StaticSource");
}

#[test]
fn test_static_source_hierarchical_names() {
    // Test StaticSource with hierarchical PV names
    let mut source = StaticSource::create()
        .expect("Failed to create StaticSource");
    
    let mut temp_pv = SharedPV::create_readonly()
        .expect("Failed to create temperature PV");
    let mut pressure_pv = SharedPV::create_readonly()
        .expect("Failed to create pressure PV");
    let mut flow_pv = SharedPV::create_mailbox()
        .expect("Failed to create flow PV");
    
    temp_pv.open_double(23.5)
        .expect("Failed to open temperature PV");
    pressure_pv.open_double(1013.25)
        .expect("Failed to open pressure PV");
    flow_pv.open_double(15.7)
        .expect("Failed to open flow PV");
    
    // Add PVs with hierarchical names
    source.add_pv("facility:building1:room101:temperature", &mut temp_pv)
        .expect("Failed to add temperature sensor");
    source.add_pv("facility:building1:room101:pressure", &mut pressure_pv)
        .expect("Failed to add pressure sensor");
    source.add_pv("facility:building1:room101:airflow", &mut flow_pv)
        .expect("Failed to add flow control");
    
    println!("Added hierarchical sensor PVs to StaticSource");
    
    // Update the writable PV
    flow_pv.post_double(20.0)
        .expect("Failed to update flow setpoint");
    
    // Clean up by closing all
    source.close_all()
        .expect("Failed to close all sensor PVs");
}

#[test]
fn test_static_source_remove_nonexistent() {
    // Test removing PV that was never added
    let mut source = StaticSource::create()
        .expect("Failed to create StaticSource");
    
    match source.remove_pv("never:added") {
        Ok(_) => {
            println!("Removing non-existent PV from source succeeded (idempotent)");
        }
        Err(e) => {
            println!("Removing non-existent PV from source failed: {}", e);
            assert!(!e.to_string().is_empty());
        }
    }
}

#[test]
fn test_static_source_add_unopened_pv() {
    // Test adding unopened PV to source
    let mut source = StaticSource::create()
        .expect("Failed to create StaticSource");
    
    let mut unopened_pv = SharedPV::create_mailbox()
        .expect("Failed to create unopened PV");
    
    // PV should not be open
    assert!(!unopened_pv.is_open());
    
    match source.add_pv("unopened:pv", &mut unopened_pv) {
        Ok(_) => {
            println!("Adding unopened PV to source succeeded");
            // Clean up
            let _ = source.remove_pv("unopened:pv");
        }
        Err(e) => {
            println!("Adding unopened PV to source failed: {}", e);
        }
    }
}

#[test]
fn test_static_source_close_all_empty() {
    // Test closing all PVs when source is empty
    let mut source = StaticSource::create()
        .expect("Failed to create StaticSource");
    
    // Source has no PVs - close_all should still work
    source.close_all()
        .expect("Failed to close all PVs in empty source");
    
    println!("close_all on empty StaticSource succeeded");
}

#[test]
fn test_static_source_pv_name_patterns() {
    // Test various PV name patterns in StaticSource
    let mut source = StaticSource::create()
        .expect("Failed to create StaticSource");
    
    let mut pv = SharedPV::create_mailbox()
        .expect("Failed to create test PV");
    
    pv.open_double(1.0)
        .expect("Failed to open test PV");
    
    // Test different name patterns
    let test_names = [
        "simple",
        "with:colons",
        "with_underscores",
        "with-dashes",
        "with.dots",
        "MixedCase",
        "numbers123",
        "special!@#chars",
    ];
    
    for name in &test_names {
        match source.add_pv(name, &mut pv) {
            Ok(_) => {
                println!("Added PV with name '{}' succeeded", name);
                // Remove it for next test
                let _ = source.remove_pv(name);
            }
            Err(e) => {
                println!("Adding PV with name '{}' failed: {}", name, e);
            }
        }
    }
}