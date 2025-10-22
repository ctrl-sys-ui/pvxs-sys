
use epics_pvxs_sys::{Server, Context};

#[test]
fn test_double_pv() {
    let test_value = 3.14f64;
    let test_pv_name = "test:loc:double";
    
    // Setup the server with a double PV
    let mut server = Server::from_env().expect("Failed to create isolated server");
    let mut double_pv = server.create_pv_double("test_double", test_value).expect("Failed to create double PV");
    server.add_pv(test_pv_name, &mut double_pv).expect("Failed to add PV to server");
    
    // Start the server
    server.start().expect("Failed to start server");
    
    // Setup the client context
    let mut ctx = Context::from_env().expect("Failed to create context");
    let timeout = 5.0; // 5 second timeout
    let value = ctx.get(test_pv_name, timeout).expect("Failed to get PV");
    
    // Verify the value is correct
    let actual_value = value.get_field_double("value").expect("Failed to get value field");
    assert_eq!(actual_value, test_value);
    
    // Clean up
    server.stop().expect("Failed to stop server");
}


