// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
mod test_pvxs_remote_string_get_put {
    use pvxs_sys::{Context, NTScalarMetadataBuilder, PvxsError, Server};

    #[test]
    fn test_pv_remote_string_get_put() {
        // This test creates a remote pv on the server and uses
        // a client context to get and put values.
        let timeout = 5.0;
        let initial_value = "Remote string PV";
        let name = "remote:string";
        let srv = Server::start_from_env().expect("Failed to create server from env");
        srv.create_pv_string(name, initial_value, NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string on server");

        // Create a client context to interact with the server
        let mut ctx = Context::from_env().expect("Failed to create client context from env");

        // Do a get to verify initial value
        let first_get: Result<pvxs_sys::Value, PvxsError> = ctx.get(name, timeout);
        match first_get {
            Ok(value) => {
                assert_eq!(value.get_field_string("value").unwrap(), initial_value);
            }
            Err(e) => assert!(false, "Failed to get value from remote pv: {:?}", e),
        }

        // Stop the server to simulate a network error
        srv.stop_drop().expect("Failed to stop server");

        // Try to do a get which should fail due to server being down
        let failed_get: Result<pvxs_sys::Value, PvxsError> = ctx.get(name, timeout);
        match failed_get {
            Ok(_) => assert!(
                false,
                "Expected error when getting from stopped server, but got Ok"
            ),
            Err(e) => {
                // Just verify we got an error - could be timeout or connection error
                assert!(e.to_string().contains("Timeout") || e.to_string().contains("Error"));
            }
        }

        // Restart the server
        let srv = Server::start_from_env().expect("Failed to restart server");
        srv.create_pv_string(name, initial_value, NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string on server");

        // Do a put to set a new value
        let new_value = "Updated remote string";
        match ctx.put_string(name, new_value, 5.0) {
            Ok(_) => (),
            Err(e) => assert!(false, "Failed to put new value to remote pv: {:?}", e),
        }

        // Do a get again to verify the new value
        let second_get: Result<pvxs_sys::Value, PvxsError> = ctx.get(name, timeout);
        match second_get {
            Ok(value) => {
                assert_eq!(value.get_field_string("value").unwrap(), new_value);
            }
            Err(e) => assert!(false, "Failed to get value from remote pv: {:?}", e),
        }

        // Close the server after test
        srv.stop_drop().expect("Failed to stop server");
    }

    #[test]
    fn test_pv_remote_string_encoding() {
        // Test that string encoding is preserved across network
        let timeout = 5.0;
        let name = "remote:string:encoding";

        let srv = Server::start_from_env().expect("Failed to create server from env");
        srv.create_pv_string(name, "", NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string on server");

        let mut ctx = Context::from_env().expect("Failed to create client context from env");

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
                    assert_eq!(
                        retrieved, test_string,
                        "String encoding not preserved for: {}",
                        test_string
                    );
                }
                Err(e) => {
                    assert!(
                        false,
                        "Failed to put string '{}' to remote pv: {:?}",
                        test_string, e
                    );
                }
            }
        }

        srv.stop_drop().expect("Failed to stop server");
    }
}
