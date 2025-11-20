mod test_pv_local_double_array_fetch_post {
    mod test_pv_local_double_array_fetch_post {
        use epics_pvxs_sys::{Server, SharedPV, NTEnumMetadataBuilder};

        #[test]
        fn test_pv_local_enum_fetch_post() {
            // This test creates a local pv (loc:enum) on a server and gets 
            // and sets the value on server side.
            let choices = vec!["OFF", "ON", "STANDBY"];
            let initial_index = 1; // "ON"
            
            let loc_srv = Server::create_isolated()
                .expect("Failed to create isolated server");

            let mut srv_pv_loc_enum: SharedPV = loc_srv.create_pv_enum("loc:enum", choices.clone(), initial_index, NTEnumMetadataBuilder::new())
                .expect("Failed to create pv:enum");

            // Do a server side fetch to verify initial value
            match srv_pv_loc_enum.fetch() {
                Ok(value) => {
                    let index = value.get_field_enum("value.index").unwrap();
                    assert_eq!(index, initial_index);
                    
                    // Verify choices array
                    let retrieved_choices = value.get_field_string_array("value.choices").unwrap();
                    assert_eq!(retrieved_choices.len(), choices.len());
                    for (i, choice) in choices.iter().enumerate() {
                        assert_eq!(&retrieved_choices[i], choice);
                    }
                },
                Err(e) => panic!("Failed to fetch value: {:?}", e),
            }

            // Post a different enum index
            let new_index = 2; // "STANDBY"
            match srv_pv_loc_enum.post_enum(new_index) {
                Ok(_) => (),
                Err(e) => panic!("Failed to post new enum index: {:?}", e),
            }

            // Fetch again to verify the new index
            match srv_pv_loc_enum.fetch() {
                Ok(value) => {
                    let index = value.get_field_enum("value.index").unwrap();
                    assert_eq!(index, new_index);
                },
                Err(e) => panic!("Failed to fetch value: {:?}", e),
            }

            // Test posting an invalid index (negative test)
            match srv_pv_loc_enum.post_enum(99) {
                Ok(_) => {
                    // Some implementations may allow out-of-range values
                    panic!("Server accepted out-of-range enum index");
                },
                Err(_) => assert!(true), // Expected error
            }
        }

        #[test]
        fn test_pv_local_enum_fetch_post_with_error_propagation() -> Result<(), Box<dyn std::error::Error>> {
            let choices = vec!["IDLE", "RUNNING", "ERROR", "STOPPED"];
            let initial_index = 0; // "IDLE"
            
            let loc_srv = Server::create_isolated()?;

            let mut srv_pv_loc_enum: SharedPV = loc_srv.create_pv_enum("loc:enum", choices.clone(), initial_index, NTEnumMetadataBuilder::new())?;

            // Verify initial state
            let fetched_value = srv_pv_loc_enum.fetch()?;
            let index = fetched_value.get_field_enum("value.index")?;
            assert_eq!(index, initial_index);

            // Change to different state
            let running_index = 1; // "RUNNING"
            srv_pv_loc_enum.post_enum(running_index)?;
            
            let fetched_value = srv_pv_loc_enum.fetch()?;
            let index = fetched_value.get_field_enum("value.index")?;
            assert_eq!(index, running_index);

            // Verify choices are intact
            let retrieved_choices = fetched_value.get_field_string_array("value.choices")?;
            assert_eq!(retrieved_choices.len(), choices.len());
            assert_eq!(retrieved_choices[running_index as usize], "RUNNING");

            Ok(())
        }

        #[test]
        fn test_pv_local_enum_all_states() {
            // Test cycling through all enum states
            let choices = vec!["STATE_0", "STATE_1", "STATE_2", "STATE_3", "STATE_4"];
            
            let loc_srv = Server::create_isolated()
                .expect("Failed to create isolated server");

            let mut srv_pv_loc_enum: SharedPV = loc_srv.create_pv_enum("loc:enum", choices.clone(), 0, NTEnumMetadataBuilder::new())
                .expect("Failed to create pv:enum");

            // Cycle through all states
            for (expected_index, expected_choice) in choices.iter().enumerate() {
                match srv_pv_loc_enum.post_enum(expected_index as i16) {
                    Ok(_) => {
                        let value = srv_pv_loc_enum.fetch().unwrap();
                        let index = value.get_field_enum("value.index").unwrap();
                        assert_eq!(index as usize, expected_index);
                        
                        let retrieved_choices = value.get_field_string_array("value.choices").unwrap();
                        assert_eq!(&retrieved_choices[index as usize], expected_choice);
                    },
                    Err(e) => panic!("Failed to set state {}: {:?}", expected_choice, e),
                }
            }
        }

        #[test]
        fn test_pv_local_enum_boundary_conditions() {
            // Test first and last choices
            let choices = vec!["FIRST", "MIDDLE", "LAST"];
            
            let loc_srv = Server::create_isolated()
                .expect("Failed to create isolated server");

            let mut srv_pv_loc_enum: SharedPV = loc_srv.create_pv_enum("loc:enum", choices.clone(), 0, NTEnumMetadataBuilder::new())
                .expect("Failed to create pv:enum");

            // Test first choice (index 0)
            srv_pv_loc_enum.post_enum(0).expect("Failed to set first choice");
            let value = srv_pv_loc_enum.fetch().unwrap();
            assert_eq!(value.get_field_enum("value.index").unwrap(), 0);

            // Test last choice (index 2)
            srv_pv_loc_enum.post_enum(2).expect("Failed to set last choice");
            let value = srv_pv_loc_enum.fetch().unwrap();
            assert_eq!(value.get_field_enum("value.index").unwrap(), 2);

            // Test negative index (should fail or be clamped)
            match srv_pv_loc_enum.post_enum(-1) {
                Ok(_) => panic!("Server accepted negative enum index"),
                Err(_) => assert!(true), // Expected behavior
            }
        }
    }
}
