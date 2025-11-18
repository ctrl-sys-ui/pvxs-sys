mod test_pvxs_remote_double_get_put {
    use epics_pvxs_sys::{Server, SharedPV, Context, PvxsError, NTScalarMetadataBuilder};

    #[test]
    fn test_pv_remote_double_get_put() {
        // This test creates a remote pv on the server and uses
        // a client context to get and put values.
        let timeout = 5.0;
        let initial_value = 3.14159;
        let name = "remote:double";
        let mut srv = Server::from_env().expect("Failed to create server from env");
        let mut srv_pv_double: SharedPV = srv.create_pv_double(name, initial_value, NTScalarMetadataBuilder::new()).expect("Failed to create pv:double on server");

        // Add pv to server, making it accessible to clients
        srv.add_pv(name, &mut srv_pv_double).expect("Failed to add pv to server");

        // start the server
        srv.start().expect("Failed to start server");

        // Create a client context to interact with the server
        let mut ctx = Context::from_env().expect("Failed to create client context from env");

        // Do a get to verify initial value
        let first_get: Result<epics_pvxs_sys::Value, PvxsError> = ctx.get(name, timeout);
        match first_get {
            Ok(value) => {
                assert!((value.get_field_double("value").unwrap() - initial_value).abs() < 1e-6);
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
                println!("Got expected error: {:?}", e);
                assert!(e.to_string().contains("Timeout") || e.to_string().contains("Error"));
            },
        }

        // Restart the server
        srv.start().expect("Failed to restart server");

        // Do a put to set a new value
        let new_value = 2.71828;
        match ctx.put_double(name, new_value, 5.0) {
            Ok(_) => (),
            Err(e) => panic!("Failed to put new value to remote pv: {:?}", e),
        }

        // Do a get again to verify the new value
        let second_get: Result<epics_pvxs_sys::Value, PvxsError> = ctx.get(name, timeout);
        match second_get {
            Ok(value) => {
                assert!((value.get_field_double("value").unwrap() - new_value).abs() < 1e-6);
            },
            Err(e) => panic!("Failed to get value from remote pv: {:?}", e),
        }

        // Close the server after test
        srv.stop().expect("Failed to stop server");
    }

    #[test]
    fn test_pv_remote_double_precision() {
        // Test that double precision is maintained across network
        let timeout = 5.0;
        let name = "remote:double:precision";
        let precision_value = 1.23456789012345; // High precision value
        
        let mut srv = Server::from_env()
            .expect("Failed to create server from env");
        let mut srv_pv_double: SharedPV = srv.create_pv_double(name, precision_value, NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:double on server");

        srv.add_pv(name, &mut srv_pv_double)
            .expect("Failed to add pv to server");
        srv.start().expect("Failed to start server");

        let mut ctx = Context::from_env()
            .expect("Failed to create client context from env");

        // Get the high precision value
        let value = ctx.get(name, timeout).expect("Failed to get precision value");
        let retrieved_value = value.get_field_double("value").unwrap();
        
        // Verify precision is maintained (within double precision limits)
        assert!((retrieved_value - precision_value).abs() < 1e-14);
        
        // Test very small numbers
        let small_value = 1e-15;
        ctx.put_double(name, small_value, timeout).expect("Failed to put small value");
        let value = ctx.get(name, timeout).expect("Failed to get small value");
        let retrieved_small = value.get_field_double("value").unwrap();
        assert!((retrieved_small - small_value).abs() < 1e-16);

        srv.stop().expect("Failed to stop server");
    }
}