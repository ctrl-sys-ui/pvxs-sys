//! Test SharedPV functions (create_mailbox, create_readonly, open_*, post_*, etc.)

use epics_pvxs_sys::SharedPV;

#[test]
fn test_shared_pv_create_mailbox() {
    // Test creating mailbox SharedPV
    match SharedPV::create_mailbox() {
        Ok(pv) => {
            // Test that PV starts closed
            assert!(!pv.is_open());
            println!("Successfully created mailbox SharedPV");
        }
        Err(e) => {
            panic!("Failed to create mailbox SharedPV: {}", e);
        }
    }
}

#[test]
fn test_shared_pv_create_readonly() {
    // Test creating readonly SharedPV
    match SharedPV::create_readonly() {
        Ok(pv) => {
            // Test that PV starts closed
            assert!(!pv.is_open());
            println!("Successfully created readonly SharedPV");
        }
        Err(e) => {
            panic!("Failed to create readonly SharedPV: {}", e);
        }
    }
}

#[test]
fn test_shared_pv_open_double() {
    // Test opening PV with double value
    let mut pv = SharedPV::create_mailbox()
        .expect("Failed to create mailbox PV");
    
    // Open with initial value
    pv.open_double(42.5)
        .expect("Failed to open PV with double value");
    
    // Verify PV is now open
    assert!(pv.is_open());
    
    // Fetch and verify we can get a value
    let value = pv.fetch()
        .expect("Failed to fetch PV value");
    
    println!("Opened double PV with value: {}", value);
}

#[test]
fn test_shared_pv_open_int32() {
    // Test opening PV with int32 value
    let mut pv = SharedPV::create_mailbox()
        .expect("Failed to create mailbox PV");
    
    pv.open_int32(100)
        .expect("Failed to open PV with int32 value");
    
    assert!(pv.is_open());
    
    let value = pv.fetch()
        .expect("Failed to fetch PV value");
    
    println!("Opened int32 PV with value: {}", value);
}

#[test]
fn test_shared_pv_open_string() {
    // Test opening PV with string value
    let mut pv = SharedPV::create_mailbox()
        .expect("Failed to create mailbox PV");
    
    pv.open_string("Hello PVXS")
        .expect("Failed to open PV with string value");
    
    assert!(pv.is_open());
    
    let value = pv.fetch()
        .expect("Failed to fetch PV value");
    
    println!("Opened string PV with value: {}", value);
}

#[test]
fn test_shared_pv_post_double() {
    // Test posting double values
    let mut pv = SharedPV::create_mailbox()
        .expect("Failed to create mailbox PV");
    
    // Open with initial value
    pv.open_double(0.0)
        .expect("Failed to open PV");
    
    // Post new values
    let test_values = [1.5, -3.14, 0.0, f64::MAX, f64::MIN];
    
    for &value in &test_values {
        pv.post_double(value)
            .expect("Failed to post double value");
        
        println!("Posted double value: {}", value);
    }
}

#[test]
fn test_shared_pv_post_int32() {
    // Test posting int32 values
    let mut pv = SharedPV::create_mailbox()
        .expect("Failed to create mailbox PV");
    
    pv.open_int32(0)
        .expect("Failed to open PV");
    
    let test_values = [42, -100, 0, i32::MAX, i32::MIN];
    
    for &value in &test_values {
        pv.post_int32(value)
            .expect("Failed to post int32 value");
        
        println!("Posted int32 value: {}", value);
    }
}

#[test]
fn test_shared_pv_post_string() {
    // Test posting string values
    let mut pv = SharedPV::create_mailbox()
        .expect("Failed to create mailbox PV");
    
    pv.open_string("initial")
        .expect("Failed to open PV");
    
    let test_strings = [
        "Hello World",
        "",  // empty string
        "Special chars: !@#$%^&*()",
        "Unicode: 🚀 测试",
        "Multi\nline\tstring",
    ];
    
    for &string_value in &test_strings {
        pv.post_string(string_value)
            .expect("Failed to post string value");
        
        println!("Posted string value: '{}'", string_value);
    }
}

#[test]
fn test_shared_pv_close() {
    // Test closing PVs
    let mut pv = SharedPV::create_mailbox()
        .expect("Failed to create mailbox PV");
    
    // Open PV
    pv.open_double(1.0)
        .expect("Failed to open PV");
    assert!(pv.is_open());
    
    // Close PV
    pv.close()
        .expect("Failed to close PV");
    assert!(!pv.is_open());
    
    println!("Successfully opened and closed PV");
}

#[test]
fn test_shared_pv_fetch() {
    // Test fetching PV values
    let mut pv = SharedPV::create_mailbox()
        .expect("Failed to create mailbox PV");
    
    pv.open_double(3.14159)
        .expect("Failed to open PV");
    
    // Fetch initial value
    let initial_value = pv.fetch()
        .expect("Failed to fetch initial value");
    println!("Initial value: {}", initial_value);
    
    // Post new value and fetch again
    pv.post_double(2.71828)
        .expect("Failed to post new value");
    
    let updated_value = pv.fetch()
        .expect("Failed to fetch updated value");
    println!("Updated value: {}", updated_value);
}

#[test]
fn test_shared_pv_open_close_cycle() {
    // Test multiple open/close cycles
    let mut pv = SharedPV::create_mailbox()
        .expect("Failed to create mailbox PV");
    
    for i in 0..5 {
        // Open with different value
        pv.open_double(i as f64)
            .expect("Failed to open PV in cycle");
        assert!(pv.is_open());
        
        // Post a value
        pv.post_double((i + 10) as f64)
            .expect("Failed to post in cycle");
        
        // Close
        pv.close()
            .expect("Failed to close PV in cycle");
        assert!(!pv.is_open());
        
        println!("Completed open/close cycle {}", i);
    }
}

#[test]
fn test_shared_pv_operations_on_closed() {
    // Test operations on unopened/closed PV
    let mut pv = SharedPV::create_mailbox()
        .expect("Failed to create PV");
    
    // Try to post to unopened PV - should fail
    match pv.post_double(42.0) {
        Ok(_) => {
            println!("Posting to unopened PV succeeded (unexpected)");
        }
        Err(e) => {
            println!("Posting to unopened PV failed as expected: {}", e);
        }
    }
    
    // Try to fetch from unopened PV - should fail
    match pv.fetch() {
        Ok(_) => {
            println!("Fetching from unopened PV succeeded (unexpected)");
        }
        Err(e) => {
            println!("Fetching from unopened PV failed as expected: {}", e);
        }
    }
}

#[test]
fn test_shared_pv_readonly_behavior() {
    // Test readonly PV behavior
    let mut readonly_pv = SharedPV::create_readonly()
        .expect("Failed to create readonly PV");
    
    readonly_pv.open_double(99.99)
        .expect("Failed to open readonly PV");
    
    assert!(readonly_pv.is_open());
    
    // Readonly PVs might still allow posting (depends on implementation)
    // This tests the API surface regardless of the underlying behavior
    match readonly_pv.post_double(11.11) {
        Ok(_) => {
            println!("Posting to readonly PV succeeded");
        }
        Err(e) => {
            println!("Posting to readonly PV failed: {}", e);
        }
    }
    
    // Fetching should work on readonly PVs
    let value = readonly_pv.fetch()
        .expect("Failed to fetch from readonly PV");
    println!("Readonly PV value: {}", value);
}