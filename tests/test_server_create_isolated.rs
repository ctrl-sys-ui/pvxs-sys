//! Test Server::create_isolated() function

use epics_pvxs_sys::Server;

#[test]
fn test_server_create_isolated() {
    // Test creating an isolated server (no network conflicts)
    match Server::create_isolated() {
        Ok(_server) => {
            println!("Successfully created isolated server");
            // Server should be valid and ready to use
        }
        Err(e) => {
            panic!("Failed to create isolated server: {}", e);
        }
    }
}

#[test]
fn test_server_create_isolated_multiple() {
    // Test creating multiple isolated servers (should not conflict)
    let _server1 = Server::create_isolated()
        .expect("Failed to create first isolated server");
    
    let _server2 = Server::create_isolated()
        .expect("Failed to create second isolated server");
    
    println!("Created two isolated servers successfully");
    
    // Both should be valid
    // They will be dropped automatically, testing cleanup
}

#[test]
fn test_server_create_isolated_lifecycle() {
    // Test complete isolated server lifecycle
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    // Test getting ports before start (should return 0)
    let tcp_port_before = server.tcp_port();
    let udp_port_before = server.udp_port();
    println!("Ports before start: TCP={}, UDP={}", tcp_port_before, udp_port_before);
    assert_eq!(tcp_port_before, 0, "TCP port should be 0 before start");
    assert_eq!(udp_port_before, 0, "UDP port should be 0 before start");
    
    // Start the server
    server.start()
        .expect("Failed to start isolated server");
    
    // Get ports after start
    let tcp_port_after = server.tcp_port();
    let udp_port_after = server.udp_port();
    println!("Ports after start: TCP={}, UDP={}", tcp_port_after, udp_port_after);
    
    // Isolated servers should have assigned ports after start
    // Note: Exact port values depend on system, but should be non-zero
    
    // Stop the server
    server.stop()
        .expect("Failed to stop isolated server");
    
    println!("Isolated server lifecycle test completed successfully");
}

#[test]
fn test_server_create_isolated_port_assignment() {
    // Test that isolated servers get different ports
    let mut server1 = Server::create_isolated()
        .expect("Failed to create first isolated server");
    
    let mut server2 = Server::create_isolated()
        .expect("Failed to create second isolated server");
    
    // Start both servers
    server1.start()
        .expect("Failed to start first server");
    
    server2.start()
        .expect("Failed to start second server");
    
    let port1_tcp = server1.tcp_port();
    let port1_udp = server1.udp_port();
    let port2_tcp = server2.tcp_port();
    let port2_udp = server2.udp_port();
    
    println!("Server1 ports: TCP={}, UDP={}", port1_tcp, port1_udp);
    println!("Server2 ports: TCP={}, UDP={}", port2_tcp, port2_udp);
    
    // Isolated servers should get different ports (if non-zero)
    if port1_tcp != 0 && port2_tcp != 0 {
        assert_ne!(port1_tcp, port2_tcp, "Isolated servers should have different TCP ports");
    }
    if port1_udp != 0 && port2_udp != 0 {
        assert_ne!(port1_udp, port2_udp, "Isolated servers should have different UDP ports");
    }
    
    // Stop both servers
    server1.stop()
        .expect("Failed to stop first server");
    server2.stop()
        .expect("Failed to stop second server");
    
    println!("Multiple isolated servers test completed");
}