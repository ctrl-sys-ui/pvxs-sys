mod test_pvxs_local_string_fetch_post {
    use pvxs_sys::{Server, NTScalarMetadataBuilder};

    #[test]
    fn test_pv_local_string_fetch_post() {
        // This test creates a local pv (loc:string) on a server and tests
        // server-side fetch() and post_string() operations.
        let initial_value = "Hello, EPICS!";
        let name = "loc:string";
        let loc_srv = Server::start_isolated()
            .expect("Failed to create isolated server");

        loc_srv.create_pv_string(name, initial_value, NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string");

        // Do a server-side fetch to verify initial value
        match loc_srv.fetch_string(name) {
            Ok(value) => assert_eq!(value.value, initial_value),
            Err(e) => assert!(false, "Failed to fetch value: {:?}", e),
        }
        
        // Now set a new string value using server-side post
        let new_value = "Updated string value";
        match loc_srv.post_string(name, new_value) {
            Ok(_) => (),
            Err(e) => assert!(false, "Failed to post new value: {:?}", e),
        } 
        
        // Fetch again to verify the new value
        match loc_srv.fetch_string(name) {
            Ok(value) => assert_eq!(value.value, new_value),
            Err(e) => assert!(false, "Failed to fetch value: {:?}", e),
        }
    }

    #[test]
    fn test_pv_local_string_fetch_post_with_error_propagation() -> Result<(), Box<dyn std::error::Error>> {
        let initial_value = "Initial string";
        let name = "loc:string:error";
        // This test verifies that server-side operations properly propagate errors.
        let loc_srv = Server::start_isolated()?;

        loc_srv.create_pv_string(name, initial_value, NTScalarMetadataBuilder::new())?;

        // Verify initial value using server-side fetch
        let fetched_value = loc_srv.fetch_string(name)?;
        assert_eq!(fetched_value.value, initial_value);

        // Put a valid string value and verify using server-side operations
        let new_value = "New string value";
        loc_srv.post_string(name, new_value)?;
        let fetched_value = loc_srv.fetch_string(name)?;
        assert_eq!(fetched_value.value, new_value);

        Ok(())
    }

    #[test]
    fn test_pv_local_string_special_characters() {
        // Test handling of special characters in strings
        let loc_srv = Server::start_isolated()
            .expect("Failed to create isolated server");

        loc_srv.create_pv_string("loc:string", "", NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string");

        // Test empty string
        loc_srv.post_string("loc:string", "").expect("Failed to post empty string");
        let value = loc_srv.fetch_string("loc:string").unwrap();
        assert_eq!(value.value, "");

        // Test string with spaces and punctuation
        let special_string = "Hello, World! @#$%^&*()";
        loc_srv.post_string("loc:string", special_string).expect("Failed to post special characters");
        let value = loc_srv.fetch_string("loc:string").unwrap();
        assert_eq!(value.value, special_string);

        // Test string with newlines and tabs
        let whitespace_string = "Line 1\nLine 2\tTabbed";
        loc_srv.post_string("loc:string", whitespace_string).expect("Failed to post whitespace string");
        let value = loc_srv.fetch_string("loc:string").unwrap();
        assert_eq!(value.value, whitespace_string);

        // Test Unicode characters
        let unicode_string = "Unicode: αβγ δεζ 中文 🚀";
        loc_srv.post_string("loc:string", unicode_string).expect("Failed to post unicode string");
        let value = loc_srv.fetch_string("loc:string").unwrap();
        assert_eq!(value.value, unicode_string);

        // Test very long string
        let long_string = "A".repeat(1000);
        loc_srv.post_string("loc:string", &long_string).expect("Failed to post long string");
        let value = loc_srv.fetch_string("loc:string").unwrap();
        assert_eq!(value.value, long_string);
    }
}
