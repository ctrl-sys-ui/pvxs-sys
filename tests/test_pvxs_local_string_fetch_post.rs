mod test_pvxs_local_string_fetch_post {
    use epics_pvxs_sys::{Server, Context, NTScalarMetadataBuilder};

    #[test]
    fn test_pv_local_string_fetch_post() {
        // This test creates a local pv (loc:string) on a server and gets 
        // and sets the value using client operations.
        let initial_value = "Hello, EPICS!";
        let name = "loc:string";
        let timeout = 5.0;
        let mut loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        loc_srv.create_pv_string(name, initial_value, NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:string");

        loc_srv.start().expect("Failed to start server");

        // Create client context
        let mut ctx = Context::from_env().expect("Failed to create client context");

        // Do a client GET to verify initial value
        match ctx.get(name, timeout) {
            Ok(value) => assert_eq!(value.get_field_string("value").unwrap(), initial_value),
            Err(e) => assert!(false, "Failed to fetch value: {:?}", e),
        }
        
        // Now set a new string value using client PUT
        let new_value = "Updated string value";
        match ctx.put_string(name, new_value, timeout) {
            Ok(_) => (),
            Err(e) => assert!(false, "Failed to put new value: {:?}", e),
        } 
        
        // GET again to verify the new value
        match ctx.get(name, timeout) {
            Ok(value) => assert_eq!(value.get_field_string("value").unwrap(), new_value),
            Err(e) => assert!(false, "Failed to fetch value: {:?}", e),
        }

        loc_srv.stop().expect("Failed to stop server");
    }

    #[test]
    fn test_pv_local_string_fetch_post_with_error_propagation() -> Result<(), Box<dyn std::error::Error>> {
        let initial_value = "Initial string";
        let name = "loc:string:error";
        let timeout = 5.0;
        // This test verifies that errors in get/set operations are properly propagated.
        let mut loc_srv = Server::create_isolated()?;

        loc_srv.create_pv_string(name, initial_value, NTScalarMetadataBuilder::new())?;
        loc_srv.start()?;

        let mut ctx = Context::from_env()?;

        // Verify initial value using client GET
        let fetched_value = ctx.get(name, timeout)?;
        assert_eq!(fetched_value.get_field_string("value")?, initial_value);

        // Put a valid string value and verify using client operations
        let new_value = "New string value";
        srv_pv_loc_string.post_string(new_value)?;
        let fetched_value = srv_pv_loc_string.fetch()?;
        assert_eq!(fetched_value.get_field_string("value")?, new_value);

        Ok(())
    }

    #[test]
    fn test_pv_local_string_special_characters() {
        // Test handling of special characters in strings
        let loc_srv = Server::create_isolated()
            .expect("Failed to create isolated server");

        let mut srv_pv_loc_string: SharedPV = loc_srv.create_pv_string("loc:string", "", NTScalarMetadataBuilder::new())
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