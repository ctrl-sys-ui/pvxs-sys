mod test_pvxs_monitor_errors {

    use pvxs_sys::{Context, Server, NTScalarMetadataBuilder, MonitorEvent, PvxsError};
    use std::thread;
    use std::time::Duration;

    
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
                assert!(false, "Queue empty (monitor stopped)");
            },
            Ok(Some(_)) => {
                assert!(false, "Unexpectedly got data after stop");
            },
            Err(MonitorEvent::ClientError(msg)) => {
                assert!(true, "Expected ClientError after stopping monitor but got: {}", msg);
            }
            Err(e) => {
                assert!(false, "Got unexpected error type: {:?}", e);
            }
        }
        
        srv.stop()?;
        Ok(())
    }
}

