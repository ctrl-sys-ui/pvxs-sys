// Test file demonstrating how to trigger RemoteError and ClientError in monitors

use epics_pvxs_sys::{Context, Server, NTScalarMetadataBuilder, MonitorEvent, PvxsError};
use std::thread;
use std::time::Duration;

#[test]
fn test_monitor_client_error_nonexistent_pv() -> Result<(), PvxsError> {
    // This test demonstrates client-side errors when monitoring a non-existent PV
    // Note: This might not always trigger a ClientError - it depends on network timing
    
    let mut ctx = Context::from_env()?;
    
    // Create a monitor for a PV that doesn't exist
    let mut monitor = ctx.monitor_builder("definitely:does:not:exist:pv:12345")?
        .disconnection_events(true)  // Enable disconnection events
        .exec()?;
    
    monitor.start()?;
    
    // Try to pop - we might get ClientError or Disconnected
    let mut got_client_error = false;
    let mut got_disconnect = false;
    
    for _ in 0..20 {
        match monitor.pop() {
            Ok(None) => {}, // Queue empty
            Ok(Some(_)) => {
                println!("Unexpectedly got data");
            },
            Err(MonitorEvent::ClientError(msg)) => {
                println!("✓ Caught ClientError: {}", msg);
                got_client_error = true;
                break;
            }
            Err(MonitorEvent::Disconnected(msg)) => {
                println!("Got Disconnected: {}", msg);
                got_disconnect = true;
            }
            Err(e) => {
                println!("Got other error: {:?}", e);
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    
    monitor.stop()?;
    
    println!("Client error: {}, Disconnect: {}", got_client_error, got_disconnect);
    
    // Note: This test is for demonstration - exact behavior depends on PVXS internals
    Ok(())
}

#[test]
fn test_monitor_remote_error_closed_pv() -> Result<(), PvxsError> {
    // This test attempts to trigger RemoteError by closing the PV while monitoring
    // Note: PVXS might send Finished or Disconnected instead of RemoteError
    
    let mut srv = Server::from_env()?;
    let _pv = srv.create_pv_double("test:remote:error", 42.0, 
        NTScalarMetadataBuilder::new())?;
    srv.start()?;
    thread::sleep(Duration::from_millis(500));
    
    let mut ctx = Context::from_env()?;
    let mut monitor = ctx.monitor_builder("test:remote:error")?
        .disconnection_events(true)
        .exec()?;
    
    monitor.start()?;
    thread::sleep(Duration::from_millis(500));
    
    // Get initial data
    let mut got_data = false;
    for _ in 0..5 {
        match monitor.pop() {
            Ok(Some(_)) => {
                got_data = true;
                break;
            },
            Ok(None) => {},
            Err(e) => {
                println!("Unexpected error during initial pop: {:?}", e);
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    
    println!("Got initial data: {}", got_data);
    
    // Stop the server to simulate a server-side condition
    srv.stop()?;
    thread::sleep(Duration::from_millis(500));
    
    // Try to pop again - should get some kind of error
    let mut got_remote_error = false;
    let mut got_disconnect = false;
    let mut got_finished = false;
    
    for _ in 0..20 {
        match monitor.pop() {
            Ok(None) => {},
            Ok(Some(_)) => {
                println!("Still getting data after server stop");
            },
            Err(MonitorEvent::RemoteError(msg)) => {
                println!("✓ Caught RemoteError: {}", msg);
                got_remote_error = true;
                break;
            }
            Err(MonitorEvent::Disconnected(msg)) => {
                println!("Got Disconnected: {}", msg);
                got_disconnect = true;
                break;
            }
            Err(MonitorEvent::Finished(msg)) => {
                println!("Got Finished: {}", msg);
                got_finished = true;
                break;
            }
            Err(MonitorEvent::ClientError(msg)) => {
                println!("Got ClientError: {}", msg);
                break;
            }
            Err(e) => {
                println!("Got other error: {:?}", e);
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    
    monitor.stop()?;
    
    println!("Remote error: {}, Disconnect: {}, Finished: {}", 
             got_remote_error, got_disconnect, got_finished);
    
    // Note: Server stop typically triggers Disconnect or Finished, not RemoteError
    // RemoteError is for explicit server-signaled errors, not shutdown
    Ok(())
}

#[test]
fn test_monitor_error_after_stop() -> Result<(), PvxsError> {
    // This test demonstrates ClientError when trying to pop after stopping the monitor
    
    let mut srv = Server::from_env()?;
    let _pv = srv.create_pv_double("test:stop:error", 3.14, 
        NTScalarMetadataBuilder::new())?;
    srv.start()?;
    thread::sleep(Duration::from_millis(500));
    
    let mut ctx = Context::from_env()?;
    let mut monitor = ctx.monitor_builder("test:stop:error")?
        .exec()?;
    
    monitor.start()?;
    thread::sleep(Duration::from_millis(500));
    
    // Stop the monitor
    monitor.stop()?;
    
    // Try to pop after stopping - should get ClientError
    match monitor.pop() {
        Ok(None) => {
            println!("Queue empty (monitor stopped)");
        },
        Ok(Some(_)) => {
            println!("Unexpectedly got data after stop");
        },
        Err(MonitorEvent::ClientError(msg)) => {
            println!("✓ Got expected ClientError: {}", msg);
            assert!(msg.contains("doesn't have an active monitor"), 
                    "Expected 'doesn't have an active monitor' error");
        }
        Err(e) => {
            println!("Got unexpected error type: {:?}", e);
        }
    }
    
    srv.stop()?;
    Ok(())
}

// Additional ideas for triggering RemoteError:
// 
// 1. **Invalid field access**: Request a field that doesn't exist
//    - Requires modifying MonitorBuilder to add field selection
//    - Example: .field("nonexistent.subfield")
//
// 2. **Type mismatch**: Try to access data with wrong type
//    - This might be caught at build time rather than runtime
//
// 3. **Permission denied**: If PVXS supports ACLs/permissions
//    - Create a PV with restricted access
//    - Try to monitor without proper credentials
//
// 4. **Resource exhaustion**: Create too many monitors
//    - Server might reject with RemoteError
//
// 5. **Invalid pvRequest**: Malformed request structure
//    - Would need lower-level API access
//
// Note: The exact way to trigger RemoteError depends on PVXS server implementation
// and what error conditions it can signal vs. just disconnecting.
