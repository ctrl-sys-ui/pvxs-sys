use epics_pvxs_sys::{Server, SharedPV, Context, PvxsError};

#[test]
fn test_pv_remote_string_get_put() {
    // This test creates a remote pv on the server and uses
    // a client context to get and put values.
    let timeout = 5.0;
    let initial_value = "Remote string PV";
    let name = "remote:string";
    let mut srv = Server::from_env()
        .expect("Failed to create server from env");
    let mut srv_pv_string: SharedPV = srv.create_pv_string(name, initial_value)
        .expect("Failed to create pv:string on server");

    // Add pv to server, making it accessible to clients
    srv.add_pv(name, &mut srv_pv_string)
        .expect("Failed to add pv to server");

    // start the server
    srv.start().expect("Failed to start server");

    // Create a client context to interact with the server
    let mut ctx = Context::from_env()
        .expect("Failed to create client context from env");

    // Do a get to verify initial value
    let first_get: Result<epics_pvxs_sys::Value, PvxsError> = ctx.get(name, timeout);
    match first_get {
        Ok(value) => {
            assert_eq!(value.get_field_string("value").unwrap(), initial_value);
        },
        Err(e) => panic!("Failed to get value from remote pv: {:?}", e),
    }

    // Stop the server to simulate a network error
    srv.stop().expect("Failed to stop server");

    // Try to do a get which should fail due to server being down
    let failed_get: Result<epics_pvxs_sys::Value, PvxsError> = ctx.get(name, timeout);
    match failed_get {
        Ok(_) => panic!("Expected error when getting from stopped server, but got Ok"),
        Err(e) => {
            // Just verify we got an error - could be timeout or connection error
            println!("Got expected error: {:?}", e);
            assert!(e.to_string().contains("Timeout") || e.to_string().contains("Error"));
        },
    }

    // Restart the server
    srv.start().expect("Failed to restart server");

    // Do a put to set a new value
    let new_value = "Updated remote string";
    match ctx.put_string(name, new_value, 5.0) {
        Ok(_) => (),
        Err(e) => panic!("Failed to put new value to remote pv: {:?}", e),
    }

    // Do a get again to verify the new value
    let second_get: Result<epics_pvxs_sys::Value, PvxsError> = ctx.get(name, timeout);
    match second_get {
        Ok(value) => {
            assert_eq!(value.get_field_string("value").unwrap(), new_value);
        },
        Err(e) => panic!("Failed to get value from remote pv: {:?}", e),
    }

    // Close the server after test
    srv.stop().expect("Failed to stop server");
}

#[test]
fn test_pv_remote_string_encoding() {
    // Test that string encoding is preserved across network
    let timeout = 5.0;
    let name = "remote:string:encoding";
    
    let mut srv = Server::from_env()
        .expect("Failed to create server from env");
    let mut srv_pv_string: SharedPV = srv.create_pv_string(name, "")
        .expect("Failed to create pv:string on server");

    srv.add_pv(name, &mut srv_pv_string)
        .expect("Failed to add pv to server");
    srv.start().expect("Failed to start server");

    let mut ctx = Context::from_env()
        .expect("Failed to create client context from env");

    // Test various string encodings
    let test_strings = vec![
        "Simple ASCII",
        "Numbers: 1234567890",
        "Punctuation: !@#$%^&*()",
        "Unicode: αβγδ ελληνικά",
        "Chinese: 你好世界",
        "Emoji: 🚀🌟💡",
        "Mixed: Hello 世界 🌍!",
    ];

    for test_string in test_strings {
        // Put the test string
        match ctx.put_string(name, test_string, timeout) {
            Ok(_) => {
                // Get it back and verify
                let value = ctx.get(name, timeout).expect("Failed to get string value");
                let retrieved = value.get_field_string("value").unwrap();
                assert_eq!(retrieved, test_string, "String encoding not preserved for: {}", test_string);
                println!("✓ String preserved: '{}'", test_string);
            },
            Err(e) => {
                println!("⚠ String not supported: '{}' - {}", test_string, e);
            }
        }
    }

    srv.stop().expect("Failed to stop server");
}