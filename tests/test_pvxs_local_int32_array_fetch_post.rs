mod test_pvxs_local_int32_array_fetch_post {
    use pvxs_sys::{Server, NTScalarMetadataBuilder};

    #[test]
    fn test_pv_local_int32_array_fetch_post() {
        // This test creates a local pv (loc:int32:array) and tests
        // server-side fetch() and post_int32_array() operations.
        let initial_array = vec![42, 43, 44];
        let name = "loc:int32:array";
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        // Create an int32 array PV and capture for server-side operations
        let srv_pv = loc_srv.create_pv_int32_array(name, initial_array.clone(), NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:int32:array");

        // Verify we can fetch the initial array value
        let value = srv_pv.fetch().expect("Failed to fetch initial value");
        let array = value.get_field_int32_array("value").expect("Failed to get array value");
        assert_eq!(array, initial_array, "Fetched array does not match initial array");
    }

    #[test]
    fn test_pv_local_int32_array_boundary_values() {
        // Test local handling of boundary int32 array values using server-side operations
        let name = "loc:int32:boundary";
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        // Test with boundary values
        let boundary_array = vec![i32::MIN, -1, 0, 1, i32::MAX];
        let srv_pv = loc_srv.create_pv_int32_array(name, boundary_array.clone(), NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:int32:boundary");

        let fetched = srv_pv.fetch().expect("Failed to fetch boundary array");
        let retrieved = fetched.get_field_int32_array("value").unwrap();
        assert_eq!(retrieved, boundary_array, "Boundary array values do not match");
    }

    #[test]
    fn test_pv_local_int32_array_type_conversions() {
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        // Test creating an empty array. This should fail.
        match loc_srv.create_pv_int32_array("loc:int32:convert", vec![], NTScalarMetadataBuilder::new()) {
            Ok(_) => assert!(false, "Expected error when creating empty int32 array PV, but got Ok"),
            Err(_) => assert!(true, "Empty array creation correctly failed"),
        };

        // Test large array
        let large_array: Vec<i32> = (0..1000).collect();
        assert!(loc_srv.create_pv_int32_array("loc:int32:large", large_array.clone(), NTScalarMetadataBuilder::new()).is_ok(),
            "Failed to create large int32 array PV");
    }

    #[test]
    fn test_pv_local_int32_posting_to_array() {
        let name = "loc:int32:post";
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        let mut initial_array = vec![10, 20, 30];
        let mut srv_pv = loc_srv.create_pv_int32_array(name, initial_array.clone(), NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:int32:post");

        // modify only element 0
        initial_array[0] = 99;

        srv_pv.post_int32_array(&initial_array).expect("Failed to post new int32 array");

        // Fetch and verify only element 0 has changed
        let fetched = srv_pv.fetch().expect("Failed to fetch after post");
        let retrieved = fetched.get_field_int32_array("value").unwrap();
        assert_eq!(retrieved, initial_array, "Array after post does not match expected values");
    }
}