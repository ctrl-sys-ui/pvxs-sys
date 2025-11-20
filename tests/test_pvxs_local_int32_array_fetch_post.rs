mod test_pvxs_local_int32_array_fetch_post {
    use epics_pvxs_sys::{Server, SharedPV, NTScalarMetadataBuilder};

    #[test]
    fn test_pv_local_int32_array_fetch_post() {
        // This test creates a local pv (loc:int32:array) on a server and gets the array value
        let initial_array = vec![42, 43, 44];
        let loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        // Create an int32 array PV
        let srv_pv_loc_array: SharedPV = loc_srv.create_pv_int32_array("loc:int32:array", initial_array.clone(), NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:int32:array");

        // Verify we can fetch the initial array value
        match srv_pv_loc_array.fetch() {
            Ok(value) => {
                match value.get_field_int32_array("value") {
                    Ok(array) => {
                        println!("Successfully got int32 array with {} elements", array.len());
                        assert_eq!(array, initial_array);
                    },
                    Err(e) => panic!("Failed to get array value: {:?}", e),
                }
            },
            Err(e) => panic!("Failed to fetch initial value: {:?}", e),
        }

        println!("✓ Int32 array PV created and fetched successfully");
    }

    #[test]
    fn test_pv_local_int32_array_boundary_values() {
        // Test local handling of boundary int32 array values
        let loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        // Test with boundary values
        let boundary_array = vec![i32::MIN, -1, 0, 1, i32::MAX];
        let srv_pv_loc_array: SharedPV = loc_srv.create_pv_int32_array("loc:int32:boundary", boundary_array.clone(), NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:int32:boundary");

        let fetched = srv_pv_loc_array.fetch().expect("Failed to fetch boundary array");
        let retrieved = fetched.get_field_int32_array("value").unwrap();
        assert_eq!(retrieved, boundary_array);
        
        println!("✓ Boundary values handled successfully: {:?}", retrieved);
    }

    #[test]
    fn test_pv_local_int32_array_type_conversions() {
        // Test various operations with int32 arrays
        let loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        // Test creating an empty array. This should fail.
        match loc_srv.create_pv_int32_array("loc:int32:convert", vec![], NTScalarMetadataBuilder::new()) {
            Ok(_) => assert!(false, "Expected error when creating empty int32 array PV, but got Ok"),
            Err(_) => assert!(true, "Empty array creation correctly failed"),
        };

        // Test large array
        let large_array: Vec<i32> = (0..1000).collect();
        srv_pv_loc_array
        match srv_pv_loc_array.post_int32(large_array.clone()) {
            Ok(_) => {
                let fetched = srv_pv_loc_array.fetch().unwrap();
                let retrieved = fetched.get_field_int32_array("value").unwrap();
                assert_eq!(retrieved.len(), large_array.len());
                println!("✓ Large array with {} elements posted successfully", large_array.len());
            },
            Err(e) => println!("⚠ Large array not supported: {}", e),
        }*/
    }

    #[test]
    fn test_pv_local_int32_array_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        // Test error handling for int32 arrays with proper error propagation
        let loc_srv = Server::create_isolated()?;
        let mut srv_pv_loc_array: SharedPV = loc_srv.create_pv_int32_array("loc:int32:errors", vec![123], NTScalarMetadataBuilder::new())?;

        // Verify initial state
        let initial_fetch = srv_pv_loc_array.fetch()?;
        let initial_array = initial_fetch.get_field_int32_array("value")?;
        assert_eq!(initial_array, vec![123]);

        // Test that valid operations work
        /*srv_pv_loc_array.post_int32_array(vec![987, 654, 321])?;
        let updated_fetch = srv_pv_loc_array.fetch()?;
        let updated_array = updated_fetch.get_field_int32_array("value")?;
        assert_eq!(updated_array, vec![987, 654, 321]);

        // Test posting single element array
        srv_pv_loc_array.post_int32_array(vec![42])?;
        let single_fetch = srv_pv_loc_array.fetch()?;
        let single_array = single_fetch.get_field_int32_array("value")?;
        assert_eq!(single_array, vec![42]);

        // Verify PV still works with various sizes
        srv_pv_loc_array.post_int32_array(vec![1, 2, 3, 4, 5])?;
        let final_fetch = srv_pv_loc_array.fetch()?;
        let final_array = final_fetch.get_field_int32_array("value")?;
        assert_eq!(final_array, vec![1, 2, 3, 4, 5]);

        */
        Ok(())
    }
}