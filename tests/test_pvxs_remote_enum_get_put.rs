mod test_pvxs_remote_enum_get_put {
    use epics_pvxs_sys::{Server, SharedPV, Context, PvxsError, NTEnumMetadataBuilder};

    #[test]
    fn test_pv_remote_enum_get_put() {
        // This test creates a remote pv on the server and uses
        // a client context to get and put values.
        let timeout = 5.0;
        let choices = vec!["DISABLED", "ENABLED", "TESTING"];
        let initial_index = 0; // "DISABLED"
        let name = "remote:enum";
        
        let mut srv = Server::from_env()
            .expect("Failed to create server from env");
        let mut srv_pv_enum: SharedPV = srv.create_pv_enum(name, choices.clone(), initial_index, NTEnumMetadataBuilder::new())
            .expect("Failed to create pv:enum on server");

        // Add pv to server, making it accessible to clients
        srv.add_pv(name, &mut srv_pv_enum)
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
                let index = value.get_field_enum("value.index").unwrap();
                assert_eq!(index, initial_index);
                
                // Verify choices array
                let retrieved_choices = value.get_field_string_array("value.choices").unwrap();
                assert_eq!(retrieved_choices.len(), choices.len());
                assert_eq!(retrieved_choices[0], "DISABLED");
                assert_eq!(retrieved_choices[1], "ENABLED");
                assert_eq!(retrieved_choices[2], "TESTING");
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
                assert!(e.to_string().contains("Timeout") || e.to_string().contains("Error"));
            },
        }

        // Restart the server
        srv.start().expect("Failed to restart server");

        // Do a put to set a new value
        let new_index = 1; // "ENABLED"
        match ctx.put_enum(name, new_index, timeout) {
            Ok(_) => (),
            Err(e) => panic!("Failed to put new enum value to remote pv: {:?}", e),
        }

        // Do a get again to verify the new value
        let second_get: Result<epics_pvxs_sys::Value, PvxsError> = ctx.get(name, timeout);
        match second_get {
            Ok(value) => {
                let index = value.get_field_enum("value.index").unwrap();
                assert_eq!(index, new_index);
            },
            Err(e) => panic!("Failed to get value from remote pv: {:?}", e),
        }

        // Close the server after test
        srv.stop().expect("Failed to stop server");
    }

    #[test]
    fn test_pv_remote_enum_state_transitions() {
        // Test cycling through multiple enum states over the network
        let timeout = 5.0;
        let name = "remote:enum:states";
        let choices = vec!["INIT", "READY", "ACTIVE", "PAUSED", "STOPPED"];
        
        let mut srv = Server::from_env()
            .expect("Failed to create server from env");
        let mut srv_pv_enum: SharedPV = srv.create_pv_enum(name, choices.clone(), 0, NTEnumMetadataBuilder::new())
            .expect("Failed to create pv:enum on server");

        srv.add_pv(name, &mut srv_pv_enum)
            .expect("Failed to add pv to server");
        srv.start().expect("Failed to start server");

        let mut ctx = Context::from_env()
            .expect("Failed to create client context from env");

        // Test state transitions: INIT -> READY -> ACTIVE -> PAUSED -> STOPPED
        for (expected_index, expected_state) in choices.iter().enumerate() {
            // Put the new state
            ctx.put_enum(name, expected_index as i16, timeout)
                .expect(&format!("Failed to put state {}", expected_state));

            // Get and verify
            let value = ctx.get(name, timeout)
                .expect(&format!("Failed to get state {}", expected_state));
            
            let index = value.get_field_enum("value.index").unwrap();
            assert_eq!(index as usize, expected_index);
            
            let retrieved_choices = value.get_field_string_array("value.choices").unwrap();
            assert_eq!(&retrieved_choices[index as usize], expected_state);
        }

        srv.stop().expect("Failed to stop server");
    }

    #[test]
    fn test_pv_remote_enum_invalid_index() {
        // Test that invalid enum indices are handled properly
        let timeout = 5.0;
        let name = "remote:enum:invalid";
        let choices = vec!["OPTION_A", "OPTION_B", "OPTION_C"];
        
        let mut srv = Server::from_env()
            .expect("Failed to create server from env");
        let mut srv_pv_enum: SharedPV = srv.create_pv_enum(name, choices.clone(), 0, NTEnumMetadataBuilder::new())
            .expect("Failed to create pv:enum on server");

        srv.add_pv(name, &mut srv_pv_enum)
            .expect("Failed to add pv to server");
        srv.start().expect("Failed to start server");

        let mut ctx = Context::from_env()
            .expect("Failed to create client context from env");

        // Try to put an invalid (out of range) index
        match ctx.put_enum(name, 99, timeout) {
            Ok(_) => {
                panic!("Server accepted out-of-range enum index");
            },
            Err(_) => {
                assert!(true, "Server did not reject invalid index"); // Expected behavior
            },
        }

        // Try negative index
        match ctx.put_enum(name, -1, timeout) {
            Ok(_) => panic!("Server accepted negative enum index"),
            Err(_) => {
                assert!(true, "Server did not reject a negative index"); // Expected behavior
            },
        }

        srv.stop().expect("Failed to stop server");
    }

    #[test]
    fn test_pv_remote_enum_choices_immutable() {
        // Verify that the choices array remains constant across operations
        let timeout = 5.0;
        let name = "remote:enum:immutable";
        let choices = vec!["CHOICE_1", "CHOICE_2", "CHOICE_3", "CHOICE_4"];
        
        let mut srv = Server::from_env()
            .expect("Failed to create server from env");
        let mut srv_pv_enum: SharedPV = srv.create_pv_enum(name, choices.clone(), 0, NTEnumMetadataBuilder::new())
            .expect("Failed to create pv:enum on server");

        srv.add_pv(name, &mut srv_pv_enum)
            .expect("Failed to add pv to server");
        srv.start().expect("Failed to start server");

        let mut ctx = Context::from_env()
            .expect("Failed to create client context from env");

        // Get initial choices
        let initial_value = ctx.get(name, timeout).expect("Failed to get initial value");
        let initial_choices = initial_value.get_field_string_array("value.choices").unwrap();

        // Change the index multiple times
        for idx in 0..choices.len() {
            ctx.put_enum(name, idx as i16, timeout)
                .expect(&format!("Failed to put index {}", idx));
        }

        // Verify choices are still the same
        let final_value = ctx.get(name, timeout).expect("Failed to get final value");
        let final_choices = final_value.get_field_string_array("value.choices").unwrap();

        assert_eq!(initial_choices.len(), final_choices.len());
        for (i, (initial, final_choice)) in initial_choices.iter().zip(final_choices.iter()).enumerate() {
            assert_eq!(initial, final_choice, "Choice at index {} changed", i);
        }

        srv.stop().expect("Failed to stop server");
    }
}