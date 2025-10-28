use epics_pvxs_sys::{Server, SharedPV, Context, PvxsError};

#[test]
fn test_pv_remote_int_get_put() {
    // This test creates a remote pv on the server and uses
    // a client context to get and put values.
    let timeout = 5.0;
    let initial_value = 50;
    let name = "remote:int";
    let mut srv = Server::from_env()
        .expect("Failed to create server from env");
    let mut srv_pv_int: SharedPV = srv.create_pv_int32(name, initial_value)
        .expect("Failed to create pv:int on server");

    // Add pv to server, making it accessible to clients
    srv.add_pv(name, &mut srv_pv_int)
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
            assert!(value.get_field_int32("value").unwrap() == initial_value);
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
    let new_value = 150;
    match ctx.put_int32(name, new_value, 5.0) {
        Ok(_) => (),
        Err(e) => panic!("Failed to put new value to remote pv: {:?}", e),
    }

    // Do a get again to verify the new value
    let second_get: Result<epics_pvxs_sys::Value, PvxsError> = ctx.get(name, timeout);
    match second_get {
        Ok(value) => {
            assert!(value.get_field_int32("value").unwrap() == new_value);
        },
        Err(e) => panic!("Failed to get value from remote pv: {:?}", e),
    }

    // Close the server after test
    srv.stop().expect("Failed to stop server");

}