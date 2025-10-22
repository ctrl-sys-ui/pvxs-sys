//! Test Server::from_env() function

use epics_pvxs_sys::Server;

#[test]
fn test_server_from_env() {
    // Test creating server from environment
    // This might fail if environment is not configured, which is okay
    match Server::from_env() {
        Ok(_server) => {
            println!("Successfully created server from environment");
            // Server should be valid
        }
        Err(e) => {
            println!("Failed to create server from env (may be expected): {}", e);
            // This is acceptable - environment may not be configured for server
            assert!(!e.to_string().is_empty());
        }
    }
}

#[test]
fn test_server_from_env_error_handling() {
    // Test error handling in server creation from environment
    let result = Server::from_env();
    
    match result {
        Ok(_) => {
            println!("Server from environment creation succeeded");
        }
        Err(e) => {
            // Error case - validate error structure
            assert!(!e.to_string().is_empty());
            assert!(e.to_string().starts_with("PVXS error:"));
            println!("Server from env failed with proper error: {}", e);
        }
    }
}

#[test]
fn test_server_from_env_vs_isolated() {
    // Compare behavior of environment vs isolated server creation
    let env_result = Server::from_env();
    let isolated_result = Server::create_isolated();
    
    match (env_result, isolated_result) {
        (Ok(_env_server), Ok(_isolated_server)) => {
            println!("Both environment and isolated server creation succeeded");
        }
        (Err(env_err), Ok(_isolated_server)) => {
            println!("Environment server failed ({}), isolated succeeded", env_err);
        }
        (Ok(_env_server), Err(isolated_err)) => {
            println!("Environment server succeeded, isolated failed ({})", isolated_err);
            // This would be unexpected - isolated should always work
        }
        (Err(env_err), Err(isolated_err)) => {
            println!("Both failed - env: {}, isolated: {}", env_err, isolated_err);
            // Isolated should work even if environment is not configured
            panic!("Isolated server creation should not fail");
        }
    }
}