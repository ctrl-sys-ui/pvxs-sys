// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
mod test_pv_local_double_array_fetch_post {
    mod test_pv_local_double_array_fetch_post {
        use pvxs_sys::{NTEnumMetadataBuilder, Server};

        #[test]
        fn test_pv_local_enum_fetch_post() {
            let name = "loc:enum";
            // This test creates a local pv (loc:enum) on a server and gets
            // and sets the value on server side.
            let choices = vec!["OFF", "ON", "STANDBY"];
            let initial_index = 1; // "ON"

            let loc_srv = Server::start_isolated().expect("Failed to create isolated server");

            loc_srv
                .create_pv_enum(
                    "loc:enum",
                    choices.clone(),
                    initial_index,
                    NTEnumMetadataBuilder::new(),
                )
                .expect("Failed to create pv:enum");

            // Do a server side fetch to verify initial value
            match loc_srv.fetch_enum(name) {
                Ok(fetched) => {
                    assert_eq!(fetched.value, initial_index);

                    // Verify choices array
                    assert_eq!(fetched.value_choices.len(), choices.len());
                    for (i, choice) in choices.iter().enumerate() {
                        assert_eq!(&fetched.value_choices[i], choice);
                    }
                }
                Err(e) => assert!(false, "Failed to fetch value: {:?}", e),
            }

            // Post a different enum index
            let new_index = 2; // "STANDBY"
            match loc_srv.post_enum(name, new_index) {
                Ok(_) => (),
                Err(e) => assert!(false, "Failed to post new enum index: {:?}", e),
            }

            // Fetch again to verify the new index
            match loc_srv.fetch_enum(name) {
                Ok(fetched) => {
                    assert_eq!(fetched.value, new_index);
                }
                Err(e) => assert!(false, "Failed to fetch value: {:?}", e),
            }

            // Test posting an invalid index (negative test)
            match loc_srv.post_enum(name, 99) {
                Ok(_) => {
                    // Some implementations may allow out-of-range values
                    assert!(false, "Server accepted out-of-range enum index");
                }
                Err(_) => assert!(true), // Expected error
            }
        }

        #[test]
        fn test_pv_local_enum_fetch_post_with_error_propagation(
        ) -> Result<(), Box<dyn std::error::Error>> {
            let name = "loc:enum";
            let baudrate = vec!["9600", "19200", "38400", "57600", "115200"];
            let initial_index = 0; // "9600"

            let loc_srv = Server::start_isolated()?;

            loc_srv.create_pv_enum(name, baudrate, initial_index, NTEnumMetadataBuilder::new())?;

            // Verify initial state
            match loc_srv.fetch_enum(name) {
                Ok(fetched) => {
                    assert_eq!(fetched.value, initial_index);
                }
                Err(e) => return Err(Box::new(e)),
            }

            // Change to different state
            let running_index = 1; // "19200"
            match loc_srv.post_enum(name, running_index) {
                Ok(_) => (),
                Err(e) => return Err(Box::new(e)),
            }

            match loc_srv.fetch_enum(name) {
                Ok(fetched) => {
                    assert_eq!(fetched.value, running_index);
                }
                Err(e) => return Err(Box::new(e)),
            }
            Ok(())
        }

        #[test]
        fn test_pv_local_enum_all_states() {
            let name = "loc:enum";
            // Test cycling through all enum states
            let choices = vec!["STATE_0", "STATE_1", "STATE_2", "STATE_3", "STATE_4"];

            let loc_srv = Server::start_isolated().expect("Failed to create isolated server");

            loc_srv
                .create_pv_enum(name, choices.clone(), 0, NTEnumMetadataBuilder::new())
                .expect("Failed to create pv:enum");

            // Cycle through all states
            for (expected_index, expected_choice) in choices.iter().enumerate() {
                match loc_srv.post_enum(name, expected_index as i16) {
                    Ok(_) => {
                        let fetched = loc_srv.fetch_enum(name).unwrap();
                        let index = fetched.value;
                        assert_eq!(index as usize, expected_index);

                        let retrieved_choices = fetched.value_choices;
                        assert_eq!(
                            retrieved_choices.len(),
                            choices.len(),
                            "Choices array length mismatch"
                        );
                        assert_eq!(&retrieved_choices[index as usize], expected_choice);
                    }
                    Err(e) => assert!(false, "Failed to set state {}: {:?}", expected_choice, e),
                }
            }
        }

        #[test]
        fn test_pv_local_enum_boundary_conditions() {
            // Test first and last choices
            let choices = vec!["FIRST", "MIDDLE", "LAST"];
            let name = "loc:enum";

            let loc_srv = Server::start_isolated().expect("Failed to create isolated server");

            loc_srv
                .create_pv_enum(name, choices.clone(), 0, NTEnumMetadataBuilder::new())
                .expect("Failed to create pv:enum");

            // Test first choice (index 0)
            loc_srv
                .post_enum(name, 0)
                .expect("Failed to set first choice");
            let value = loc_srv.fetch_enum(name).unwrap();
            assert_eq!(value.value, 0);

            // Test last choice (index 2)
            loc_srv
                .post_enum(name, 2)
                .expect("Failed to set last choice");
            let value = loc_srv.fetch_enum(name).unwrap();
            assert_eq!(value.value, 2);

            // Test negative index (should fail or be clamped)
            match loc_srv.post_enum(name, -1) {
                Ok(_) => assert!(false, "Server accepted negative enum index"),
                Err(_) => assert!(true), // Expected behavior
            }
        }
    }
}
