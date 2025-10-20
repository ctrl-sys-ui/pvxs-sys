// Test the PVXS int32 data type
// Creates a server side int32 PV with an initial value of 42
// Then using a client side PV to fetch and verify the value is correct.

use epics_pvxs_sys::{Server, Context};

#[test]
fn test_int32_pv() {
    let test_value = 42i32;
    let test_pv_name = "test:loc:int32";
    
    // Setup the server with an int32 PV
    let mut server = Server::from_env().expect("Failed to create server");
    let mut int_pv = server.create_pv_int32("test_int32", test_value).expect("Failed to create int32 PV");
    server.add_pv(test_pv_name, &mut int_pv).expect("Failed to add PV to server");
    
    // Start the server
    server.start().expect("Failed to start server");
    
    // Setup the client context
    let mut ctx = Context::from_env().expect("Failed to create context");
    let timeout = 5.0; // 5 second timeout
    let value = ctx.get(test_pv_name, timeout).expect("Failed to get PV");
    
    // Verify the value is correct
    let actual_value = value.get_field_int32("value").expect("Failed to get value field");
    assert_eq!(actual_value, test_value);
    
    // Clean up
    server.stop().expect("Failed to stop server");
}

#[test]
fn test_int32_pv_negative_value() {
    let test_value = -123i32;
    let test_pv_name = "test:loc:int32_negative";
    
    // Setup the server with an int32 PV with negative value
    let mut server = Server::from_env().expect("Failed to create server");
    let mut int_pv = server.create_pv_int32("test_int32_neg", test_value).expect("Failed to create int32 PV");
    server.add_pv(test_pv_name, &mut int_pv).expect("Failed to add PV to server");
    
    // Start the server
    server.start().expect("Failed to start server");
    
    // Setup the client context
    let mut ctx = Context::from_env().expect("Failed to create context");
    let timeout = 5.0; // 5 second timeout
    let value = ctx.get(test_pv_name, timeout).expect("Failed to get PV");
    
    // Verify the value is correct
    let actual_value = value.get_field_int32("value").expect("Failed to get value field");
    assert_eq!(actual_value, test_value);
    
    // Clean up
    server.stop().expect("Failed to stop server");
}

#[test]
fn test_int32_pv_zero_value() {
    let test_value = 0i32;
    let test_pv_name = "test:loc:int32_zero";
    
    // Setup the server with an int32 PV with zero value
    let mut server = Server::from_env().expect("Failed to create server");
    let mut int_pv = server.create_pv_int32("test_int32_zero", test_value).expect("Failed to create int32 PV");
    server.add_pv(test_pv_name, &mut int_pv).expect("Failed to add PV to server");
    
    // Start the server
    server.start().expect("Failed to start server");
    
    // Setup the client context
    let mut ctx = Context::from_env().expect("Failed to create context");
    let timeout = 5.0; // 5 second timeout
    let value = ctx.get(test_pv_name, timeout).expect("Failed to get PV");
    
    // Verify the value is correct
    let actual_value = value.get_field_int32("value").expect("Failed to get value field");
    assert_eq!(actual_value, test_value);
    
    // Clean up
    server.stop().expect("Failed to stop server");
}

#[test]
fn test_int32_pv_max_value() {
    let test_value = i32::MAX;
    let test_pv_name = "test:loc:int32_max";
    
    // Setup the server with an int32 PV with maximum value
    let mut server = Server::from_env().expect("Failed to create server");
    let mut int_pv = server.create_pv_int32("test_int32_max", test_value).expect("Failed to create int32 PV");
    server.add_pv(test_pv_name, &mut int_pv).expect("Failed to add PV to server");
    
    // Start the server
    server.start().expect("Failed to start server");
    
    // Setup the client context
    let mut ctx = Context::from_env().expect("Failed to create context");
    let timeout = 5.0; // 5 second timeout
    let value = ctx.get(test_pv_name, timeout).expect("Failed to get PV");
    
    // Verify the value is correct
    let actual_value = value.get_field_int32("value").expect("Failed to get value field");
    assert_eq!(actual_value, test_value);
    
    // Clean up
    server.stop().expect("Failed to stop server");
}

#[test]
fn test_int32_pv_min_value() {
    let test_value = i32::MIN;
    let test_pv_name = "test:loc:int32_min";
    
    // Setup the server with an int32 PV with minimum value
    let mut server = Server::from_env().expect("Failed to create server");
    let mut int_pv = server.create_pv_int32("test_int32_min", test_value).expect("Failed to create int32 PV");
    server.add_pv(test_pv_name, &mut int_pv).expect("Failed to add PV to server");
    
    // Start the server
    server.start().expect("Failed to start server");
    
    // Setup the client context
    let mut ctx = Context::from_env().expect("Failed to create context");
    let timeout = 5.0; // 5 second timeout
    let value = ctx.get(test_pv_name, timeout).expect("Failed to get PV");
    
    // Verify the value is correct
    let actual_value = value.get_field_int32("value").expect("Failed to get value field");
    assert_eq!(actual_value, test_value);
    
    // Clean up
    server.stop().expect("Failed to stop server");
}

#[test]
fn test_int32_pv_alarm_fields() {
    let test_value = 123i32;
    let test_pv_name = "test:loc:int32_alarm";
    
    // Setup the server with an int32 PV
    let mut server = Server::from_env().expect("Failed to create server");
    let mut int_pv = server.create_pv_int32("test_int32_alarm", test_value).expect("Failed to create int32 PV");
    server.add_pv(test_pv_name, &mut int_pv).expect("Failed to add PV to server");
    
    // Start the server
    server.start().expect("Failed to start server");
    
    // Setup the client context
    let mut ctx = Context::from_env().expect("Failed to create context");
    let timeout = 5.0; // 5 second timeout
    let value = ctx.get(test_pv_name, timeout).expect("Failed to get PV");
    
    // Verify the main value is correct
    let actual_value = value.get_field_int32("value").expect("Failed to get value field");
    assert_eq!(actual_value, test_value);
    
    // Test alarm_t structure fields
    // Default alarm severity should be 0 (NO_ALARM)
    let alarm_severity = value.get_field_int32("alarm.severity").expect("Failed to get alarm severity");
    assert_eq!(alarm_severity, 0, "Default alarm severity should be 0 (NO_ALARM)");
    
    // Default alarm status should be 0 (NO_ALARM)
    let alarm_status = value.get_field_int32("alarm.status").expect("Failed to get alarm status");
    assert_eq!(alarm_status, 0, "Default alarm status should be 0 (NO_ALARM)");
    
    // Default alarm message should be empty
    let alarm_message = value.get_field_string("alarm.message").expect("Failed to get alarm message");
    assert_eq!(alarm_message, "", "Default alarm message should be empty");
    
    println!("Alarm severity: {}", alarm_severity);
    println!("Alarm status: {}", alarm_status);
    println!("Alarm message: '{}'", alarm_message);
    
    // Clean up
    server.stop().expect("Failed to stop server");
}

#[test]
fn test_int32_pv_set_alarm_fields() {
    let test_value = 456i32;
    let test_pv_name = "test:loc:int32_set_alarm";
    
    // Setup the server with an int32 PV
    let mut server = Server::from_env().expect("Failed to create server");
    let mut int_pv = server.create_pv_int32("test_int32_set_alarm", test_value).expect("Failed to create int32 PV");
    server.add_pv(test_pv_name, &mut int_pv).expect("Failed to add PV to server");
    
    // Start the server
    server.start().expect("Failed to start server");
    
    // Now post a new value with alarm information
    let new_value = 999i32;
    let alarm_severity = 2; // MAJOR alarm
    let alarm_status = 1;   // Some status code
    let alarm_message = "Test alarm condition";
    
    int_pv.post_int32_with_alarm(new_value, alarm_severity, alarm_status, alarm_message)
        .expect("Failed to post value with alarm");
    
    // Setup the client context and fetch the updated value
    let mut ctx = Context::from_env().expect("Failed to create context");
    let timeout = 5.0; // 5 second timeout
    let value = ctx.get(test_pv_name, timeout).expect("Failed to get PV");
    
    // Verify the main value was updated
    let actual_value = value.get_field_int32("value").expect("Failed to get value field");
    assert_eq!(actual_value, new_value);
    
    // Test that alarm fields were set correctly
    let retrieved_severity = value.get_field_int32("alarm.severity").expect("Failed to get alarm severity");
    assert_eq!(retrieved_severity, alarm_severity, "Alarm severity should match posted value");
    
    let retrieved_status = value.get_field_int32("alarm.status").expect("Failed to get alarm status");
    assert_eq!(retrieved_status, alarm_status, "Alarm status should match posted value");
    
    let retrieved_message = value.get_field_string("alarm.message").expect("Failed to get alarm message");
    assert_eq!(retrieved_message, alarm_message, "Alarm message should match posted value");
    
    println!("Successfully set and verified alarm fields:");
    println!("  Value: {}", actual_value);
    println!("  Severity: {} (MAJOR)", retrieved_severity);
    println!("  Status: {}", retrieved_status);
    println!("  Message: '{}'", retrieved_message);
    
    // Clean up
    server.stop().expect("Failed to stop server");
}

#[test]
fn test_double_pv_with_different_alarm_severities() {
    let test_pv_name = "test:loc:double_alarms";
    
    // Setup the server with a double PV
    let mut server = Server::from_env().expect("Failed to create server");
    let mut double_pv = server.create_pv_double("test_double_alarms", 0.0).expect("Failed to create double PV");
    server.add_pv(test_pv_name, &mut double_pv).expect("Failed to add PV to server");
    
    // Start the server
    server.start().expect("Failed to start server");
    
    // Test different alarm severity levels
    let test_cases = vec![
        (1.0, 0, 0, "NO_ALARM"),           // NO_ALARM
        (2.0, 1, 10, "MINOR alarm"),      // MINOR
        (3.0, 2, 20, "MAJOR alarm"),      // MAJOR  
        (4.0, 3, 30, "INVALID alarm"),    // INVALID
    ];
    
    let mut ctx = Context::from_env().expect("Failed to create context");
    let timeout = 5.0;
    
    for (value, severity, status, message) in test_cases {
        // Post value with alarm
        double_pv.post_double_with_alarm(value, severity, status, message)
            .expect("Failed to post double with alarm");
        
        // Fetch and verify
        let retrieved = ctx.get(test_pv_name, timeout).expect("Failed to get PV");
        
        let actual_value = retrieved.get_field_double("value").expect("Failed to get value");
        let actual_severity = retrieved.get_field_int32("alarm.severity").expect("Failed to get severity");
        let actual_status = retrieved.get_field_int32("alarm.status").expect("Failed to get status");
        let actual_message = retrieved.get_field_string("alarm.message").expect("Failed to get message");
        
        assert_eq!(actual_value, value);
        assert_eq!(actual_severity, severity);
        assert_eq!(actual_status, status);
        assert_eq!(actual_message, message);
        
        println!("✓ Verified alarm: value={}, severity={}, status={}, message='{}'", 
                 actual_value, actual_severity, actual_status, actual_message);
    }
    
    // Clean up
    server.stop().expect("Failed to stop server");
}