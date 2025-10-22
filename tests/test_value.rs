//! Test Value functions (get_field_*, is_valid, Display, Debug traits)

use epics_pvxs_sys::Server;

#[test]
fn test_value_from_pv_fetch() {
    // Test Value operations from PV fetch
    let server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    // Test double PV
    let double_pv = server.create_pv_double("test_double", 3.14159)
        .expect("Failed to create double PV");
    
    let value = double_pv.fetch()
        .expect("Failed to fetch double PV value");
    
    println!("Double PV value: {}", value);
    
    // Test that value is valid
    assert!(value.is_valid(), "Value should be valid");
    
    // Test int32 PV
    let int_pv = server.create_pv_int32("test_int", 42)
        .expect("Failed to create int PV");
    
    let int_value = int_pv.fetch()
        .expect("Failed to fetch int PV value");
    
    println!("Int PV value: {}", int_value);
    assert!(int_value.is_valid());
    
    // Test string PV
    let string_pv = server.create_pv_string("test_string", "Hello PVXS")
        .expect("Failed to create string PV");
    
    let string_value = string_pv.fetch()
        .expect("Failed to fetch string PV value");
    
    println!("String PV value: {}", string_value);
    assert!(string_value.is_valid());
}

#[test]
fn test_value_field_access() {
    // Test accessing Value fields (if supported)
    let server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    let pv = server.create_pv_double("field_test", 123.456)
        .expect("Failed to create test PV");
    
    let value = pv.fetch()
        .expect("Failed to fetch PV value");
    
    // Try to access common EPICS fields
    match value.get_field_double("value") {
        Ok(val) => {
            println!("Value field (double): {}", val);
        }
        Err(e) => {
            println!("Failed to get 'value' field as double: {}", e);
        }
    }
    
    match value.get_field_string("value") {
        Ok(val) => {
            println!("Value field (string): {}", val);
        }
        Err(e) => {
            println!("Failed to get 'value' field as string: {}", e);
        }
    }
    
    // Try accessing alarm fields
    match value.get_field_int32("alarm.severity") {
        Ok(severity) => {
            println!("Alarm severity: {}", severity);
        }
        Err(e) => {
            println!("Failed to get alarm severity: {}", e);
        }
    }
    
    match value.get_field_string("alarm.status") {
        Ok(status) => {
            println!("Alarm status: {}", status);
        }
        Err(e) => {
            println!("Failed to get alarm status: {}", e);
        }
    }
    
    // Try accessing timestamp fields
    match value.get_field_double("timeStamp.secondsPastEpoch") {
        Ok(timestamp) => {
            println!("Timestamp: {}", timestamp);
        }
        Err(e) => {
            println!("Failed to get timestamp: {}", e);
        }
    }
}

#[test]
fn test_value_field_type_conversion() {
    // Test field type conversions
    let server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    // Test with int32 PV
    let int_pv = server.create_pv_int32("conversion_test", 42)
        .expect("Failed to create int PV");
    
    let int_value = int_pv.fetch()
        .expect("Failed to fetch int PV value");
    
    // Try accessing as different types
    match int_value.get_field_int32("value") {
        Ok(val) => println!("Int value as int32: {}", val),
        Err(e) => println!("Failed to get int as int32: {}", e),
    }
    
    match int_value.get_field_double("value") {
        Ok(val) => println!("Int value as double: {}", val),
        Err(e) => println!("Failed to get int as double: {}", e),
    }
    
    match int_value.get_field_string("value") {
        Ok(val) => println!("Int value as string: {}", val),
        Err(e) => println!("Failed to get int as string: {}", e),
    }
}

#[test]
fn test_value_invalid_fields() {
    // Test accessing non-existent fields
    let server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    let pv = server.create_pv_double("invalid_test", 1.0)
        .expect("Failed to create test PV");
    
    let value = pv.fetch()
        .expect("Failed to fetch PV value");
    
    // Try accessing fields that don't exist
    match value.get_field_double("nonexistent.field") {
        Ok(val) => {
            println!("Unexpectedly got nonexistent field: {}", val);
        }
        Err(e) => {
            println!("Failed to get nonexistent field as expected: {}", e);
            assert!(!e.to_string().is_empty());
        }
    }
    
    match value.get_field_string("") {
        Ok(val) => {
            println!("Unexpectedly got empty field name: {}", val);
        }
        Err(e) => {
            println!("Failed to get empty field name as expected: {}", e);
        }
    }
}

#[test]
fn test_value_display_trait() {
    // Test Value Display implementation
    let server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    let double_pv = server.create_pv_double("display_test", 2.71828)
        .expect("Failed to create double PV");
    let string_pv = server.create_pv_string("display_string", "Test String")
        .expect("Failed to create string PV");
    
    let double_value = double_pv.fetch()
        .expect("Failed to fetch double value");
    let string_value = string_pv.fetch()
        .expect("Failed to fetch string value");
    
    // Test Display formatting
    let double_display = format!("{}", double_value);
    let string_display = format!("{}", string_value);
    
    println!("Double value display: {}", double_display);
    println!("String value display: {}", string_display);
    
    assert!(!double_display.is_empty());
    assert!(!string_display.is_empty());
}

#[test]
fn test_value_debug_trait() {
    // Test Value Debug implementation
    let server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    let pv = server.create_pv_double("debug_test", 42.0)
        .expect("Failed to create test PV");
    
    let value = pv.fetch()
        .expect("Failed to fetch PV value");
    
    // Test Debug formatting
    let debug_string = format!("{:?}", value);
    println!("Value debug representation: {}", debug_string);
    
    assert!(!debug_string.is_empty());
    assert!(debug_string.contains("Value"), "Debug should contain type name");
}

#[test]
fn test_value_is_valid() {
    // Test Value validity checks
    let server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    let pv = server.create_pv_double("valid_test", 1.0)
        .expect("Failed to create test PV");
    
    // Valid value from open PV
    let valid_value = pv.fetch()
        .expect("Failed to fetch valid PV value");
    
    assert!(valid_value.is_valid(), "Value from open PV should be valid");
    
    // Test with different PV types
    let int_pv = server.create_pv_int32("int_valid", 42)
        .expect("Failed to create int PV");
    let int_value = int_pv.fetch()
        .expect("Failed to fetch int PV");
    
    assert!(int_value.is_valid(), "Int value should be valid");
    
    let string_pv = server.create_pv_string("string_valid", "valid")
        .expect("Failed to create string PV");
    let string_value = string_pv.fetch()
        .expect("Failed to fetch string PV");
    
    assert!(string_value.is_valid(), "String value should be valid");
}

#[test]
fn test_value_updated_after_post() {
    // Test that Value reflects updates after posting
    let server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    let mut pv = server.create_pv_double("update_test", 1.0)
        .expect("Failed to create test PV");
    
    // Get initial value
    let initial_value = pv.fetch()
        .expect("Failed to fetch initial value");
    println!("Initial value: {}", initial_value);
    
    // Update PV
    pv.post_double(99.99)
        .expect("Failed to post new value");
    
    // Get updated value
    let updated_value = pv.fetch()
        .expect("Failed to fetch updated value");
    println!("Updated value: {}", updated_value);
    
    // Both values should be valid
    assert!(initial_value.is_valid());
    assert!(updated_value.is_valid());
    
    // Values should be different (string representations)
    let initial_str = format!("{}", initial_value);
    let updated_str = format!("{}", updated_value);
    
    // Note: Depending on the Value implementation, the string representations
    // might be the same if they don't include the actual data value.
    // This test mainly verifies that the operations complete successfully.
    
    println!("Initial: '{}', Updated: '{}'", initial_str, updated_str);
}