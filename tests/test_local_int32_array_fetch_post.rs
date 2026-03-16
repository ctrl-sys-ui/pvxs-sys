// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
mod test_pvxs_local_int32_array_fetch_post {
    use pvxs_sys::{NTScalarMetadataBuilder, Server};

    #[test]
    fn test_pv_local_int32_array_fetch_post() {
        // This test creates a local pv (loc:int32:array) and tests
        // server-side fetch() and post_int32_array() operations.
        let initial_array = vec![42, 43, 44];
        let name = "loc:int32:array";
        let loc_srv = Server::start_isolated().expect("Failed to create isolated server");

        // Create an int32 array PV and capture for server-side operations
        loc_srv
            .create_pv_int32_array(name, initial_array.clone(), NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:int32:array");

        // Verify we can fetch the initial array value
        match loc_srv.fetch_int32_array(name) {
            Ok(fetched) => {
                assert_eq!(
                    fetched.value, initial_array,
                    "Fetched array does not match initial array"
                );
            }
            Err(e) => assert!(false, "Failed to fetch int32 array: {:?}", e),
        }
    }

    #[test]
    fn test_pv_local_int32_array_boundary_values() {
        // Test local handling of boundary int32 array values using server-side operations
        let name = "loc:int32:boundary";
        let loc_srv = Server::start_isolated().expect("Failed to create isolated server");

        let boundary_array = vec![i32::MIN, -1, 0, 1, i32::MAX];
        loc_srv
            .create_pv_int32_array(name, boundary_array.clone(), NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:int32:boundary");
        // Test with boundary values

        match loc_srv.fetch_int32_array(name) {
            Ok(fetched) => {
                assert_eq!(
                    fetched.value, boundary_array,
                    "Fetched arrar does not match boundary array"
                );
            }
            Err(e) => assert!(false, "Failed to fetch boundary int32 array: {:?}", e),
        }
    }

    #[test]
    fn test_pv_local_int32_array_empty_array_failes() {
        let loc_srv = Server::start_isolated().expect("Failed to create isolated server");

        // Test creating an empty array. This should fail.
        match loc_srv.create_pv_int32_array(
            "loc:int32:convert",
            vec![],
            NTScalarMetadataBuilder::new(),
        ) {
            Ok(_) => assert!(
                false,
                "Expected error when creating empty int32 array PV, but got Ok"
            ),
            Err(_) => assert!(true, "Empty array creation correctly failed"),
        };
    }

    #[test]
    fn test_pv_local_int32_array_large_array() {
        let loc_srv = Server::start_isolated().expect("Failed to create isolated server");
        // Test large array
        let large_array: Vec<i32> = (0..1000).collect();
        assert!(
            loc_srv
                .create_pv_int32_array(
                    "loc:int32:large",
                    large_array.clone(),
                    NTScalarMetadataBuilder::new()
                )
                .is_ok(),
            "Failed to create large int32 array PV"
        );
    }

    #[test]
    fn test_pv_local_int32_posting_to_array() {
        let name = "loc:int32:post";
        let loc_srv = Server::start_isolated().expect("Failed to create isolated server");

        let mut initial_array = vec![10, 20, 30];
        loc_srv
            .create_pv_int32_array(name, initial_array.clone(), NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:int32:post");

        // modify only element 0
        initial_array[0] = 99;

        loc_srv
            .post_int32_array(name, initial_array.clone())
            .expect("Failed to post new int32 array");

        // Fetch and verify only element 0 has changed
        let fetched = loc_srv
            .fetch_int32_array(name)
            .expect("Failed to fetch after post");
        assert_eq!(
            fetched.value, initial_array,
            "Array after post does not match expected values"
        );
    }
}
