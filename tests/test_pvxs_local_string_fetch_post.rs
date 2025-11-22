mod test_pvxs_local_string_fetch_post {
    use epics_pvxs_sys::{Server, NTScalarMetadataBuilder};

    #[test]
    fn test_pv_local_string_fetch_post() {
        // This test creates a local pv (loc:string) on a server and tests
        // server-side fetch() and post_string() operations.
        let initial_value = "Hello, EPICS!";
        let name = "loc:string";
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        let mut srv_pv = loc_srv.create_pv_string(name, initial_value, NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string");

        // Do a server-side fetch to verify initial value
        match srv_pv.fetch() {
            Ok(value) => assert_eq!(value.get_field_string("value").unwrap(), initial_value),
            Err(e) => assert!(false, "Failed to fetch value: {:?}", e),
        }
        
        // Now set a new string value using server-side post
        let new_value = "Updated string value";
        match srv_pv.post_string(new_value) {
            Ok(_) => (),
            Err(e) => assert!(false, "Failed to post new value: {:?}", e),
        } 
        
        // Fetch again to verify the new value
        match srv_pv.fetch() {
            Ok(value) => assert_eq!(value.get_field_string("value").unwrap(), new_value),
            Err(e) => assert!(false, "Failed to fetch value: {:?}", e),
        }
    }

    #[test]
    fn test_pv_local_string_fetch_post_with_error_propagation() -> Result<(), Box<dyn std::error::Error>> {
        let initial_value = "Initial string";
        let name = "loc:string:error";
        // This test verifies that server-side operations properly propagate errors.
        let mut loc_srv = Server::create_isolated()?;

        let mut srv_pv_loc_string = loc_srv.create_pv_string(name, initial_value, NTScalarMetadataBuilder::new())?;

        // Verify initial value using server-side fetch
        let fetched_value = srv_pv_loc_string.fetch()?;
        assert_eq!(fetched_value.get_field_string("value")?, initial_value);

        // Put a valid string value and verify using server-side operations
        let new_value = "New string value";
        srv_pv_loc_string.post_string(new_value)?;
        let fetched_value = srv_pv_loc_string.fetch()?;
        assert_eq!(fetched_value.get_field_string("value")?, new_value);

        Ok(())
    }

    #[test]
    fn test_pv_local_string_special_characters() {
        // Test handling of special characters in strings
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        let mut srv_pv_loc_string = loc_srv.create_pv_string("loc:string", "", NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string");

        // Test empty string
        srv_pv_loc_string.post_string("").expect("Failed to post empty string");
        let value = srv_pv_loc_string.fetch().unwrap();
        assert_eq!(value.get_field_string("value").unwrap(), "");

        // Test string with spaces and punctuation
        let special_string = "Hello, World! @#$%^&*()";
        srv_pv_loc_string.post_string(special_string).expect("Failed to post special characters");
        let value = srv_pv_loc_string.fetch().unwrap();
        assert_eq!(value.get_field_string("value").unwrap(), special_string);

        // Test string with newlines and tabs
        let whitespace_string = "Line 1\nLine 2\tTabbed";
        srv_pv_loc_string.post_string(whitespace_string).expect("Failed to post whitespace string");
        let value = srv_pv_loc_string.fetch().unwrap();
        assert_eq!(value.get_field_string("value").unwrap(), whitespace_string);

        // Test Unicode characters
        let unicode_string = "Unicode: αβγ δεζ 中文 🚀";
        srv_pv_loc_string.post_string(unicode_string).expect("Failed to post unicode string");
        let value = srv_pv_loc_string.fetch().unwrap();
        assert_eq!(value.get_field_string("value").unwrap(), unicode_string);

        // Test very long string
        let long_string = "A".repeat(1000);
        srv_pv_loc_string.post_string(&long_string).expect("Failed to post long string");
        let value = srv_pv_loc_string.fetch().unwrap();
        assert_eq!(value.get_field_string("value").unwrap(), long_string);
    }
}