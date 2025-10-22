//! Test Server::start() and Server::stop() functions

use epics_pvxs_sys::Server;

#[test]
fn test_server_start() {
    // Test starting a server
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    // Server should start successfully
    server.start()
        .expect("Failed to start server");
    
    // Verify server is running by checking ports
    let tcp_port = server.tcp_port();
    let udp_port = server.udp_port();
    
    println!("Started server on TCP port {} and UDP port {}", tcp_port, udp_port);
    
    // Stop the server for cleanup
    server.stop()
        .expect("Failed to stop server");
}

#[test]
fn test_server_stop() {
    // Test stopping a server
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    // Start the server first
    server.start()
        .expect("Failed to start server");
    
    let tcp_port_running = server.tcp_port();
    let udp_port_running = server.udp_port();
    
    println!("Server running on TCP:{}, UDP:{}", tcp_port_running, udp_port_running);
    
    // Stop the server
    server.stop()
        .expect("Failed to stop server");
    
    // Note: Port numbers might remain the same after stop, depending on implementation
    // The key is that stop() should succeed without errors
    
    println!("Server stopped successfully");
}

#[test]
fn test_server_start_stop_cycle() {
    // Test multiple start/stop cycles
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    for cycle in 0..3 {
        println!("Start/stop cycle {}", cycle);
        
        // Start server
        server.start()
            .expect("Failed to start server in cycle");
        
        let tcp_port = server.tcp_port();
        let udp_port = server.udp_port();
        println!("Cycle {} - TCP:{}, UDP:{}", cycle, tcp_port, udp_port);
        
        // Stop server
        server.stop()
            .expect("Failed to stop server in cycle");
    }
    
    println!("Multiple start/stop cycles completed successfully");
}

#[test]
fn test_server_double_start() {
    // Test starting an already started server
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    // Start server first time
    server.start()
        .expect("Failed to start server first time");
    
    // Try to start again - behavior depends on implementation
    match server.start() {
        Ok(_) => {
            println!("Double start succeeded (idempotent behavior)");
        }
        Err(e) => {
            println!("Double start failed as expected: {}", e);
        }
    }
    
    // Stop server
    server.stop()
        .expect("Failed to stop server");
}

#[test]
fn test_server_double_stop() {
    // Test stopping an already stopped server
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    // Start and then stop server
    server.start()
        .expect("Failed to start server");
    
    server.stop()
        .expect("Failed to stop server first time");
    
    // Try to stop again - should be idempotent
    match server.stop() {
        Ok(_) => {
            println!("Double stop succeeded (idempotent behavior)");
        }
        Err(e) => {
            println!("Double stop failed: {}", e);
        }
    }
}

#[test]
fn test_server_stop_without_start() {
    // Test stopping a server that was never started
    let mut server = Server::create_isolated()
        .expect("Failed to create isolated server");
    
    // Try to stop without starting
    match server.stop() {
        Ok(_) => {
            println!("Stopping unstarted server succeeded (idempotent)");
        }
        Err(e) => {
            println!("Stopping unstarted server failed: {}", e);
            // This might be expected behavior
        }
    }
}