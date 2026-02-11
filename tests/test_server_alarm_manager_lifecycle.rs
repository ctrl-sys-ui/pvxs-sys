#[cfg(test)]
mod test_server_alarm_manager_lifecycle {
    use pvxs_sys::{Server, Context, NTScalarMetadataBuilder, ControlMetadata, AlarmMetadata};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_create_multiple_pvs_with_alarms() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        // Create multiple PVs with alarm metadata
        for i in 0..5 {
            let pv_name = format!("test:lifecycle:pv{}", i);
            let metadata = NTScalarMetadataBuilder::new()
                .alarm(AlarmMetadata {
                    active: true,
                    low_alarm_limit: (i * 10) as f64,
                    low_warning_limit: (i * 10 + 10) as f64,
                    high_warning_limit: (i * 10 + 80) as f64,
                    high_alarm_limit: (i * 10 + 90) as f64,
                    low_alarm_severity: 2,
                    low_warning_severity: 1,
                    high_warning_severity: 1,
                    high_alarm_severity: 2,
                    hysteresis: 0,
                });

            manager.create_pv_double(&pv_name, 50.0, metadata)
                .expect(&format!("Failed to create PV {}", i));
        }

        thread::sleep(Duration::from_millis(100));

        let mut ctx = Context::from_env()
            .expect("Failed to create client");

        // Verify all PVs are accessible
        for i in 0..5 {
            let pv_name = format!("test:lifecycle:pv{}", i);
            let value = ctx.get(&pv_name, 2.0)
                .expect(&format!("Failed to get PV {}", i));
            assert!((value.get_field_double("value").unwrap() - 50.0).abs() < 1e-6);
        }

        manager.stop().expect("Failed to stop manager");
    }

    #[test]
    fn test_remove_pv_with_alarms() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let pv_name = "test:lifecycle:remove";
        let metadata = NTScalarMetadataBuilder::new()
            .alarm(AlarmMetadata {
                active: true,
                low_alarm_limit: 10.0,
                low_warning_limit: 20.0,
                high_warning_limit: 80.0,
                high_alarm_limit: 90.0,
                low_alarm_severity: 2,
                low_warning_severity: 1,
                high_warning_severity: 1,
                high_alarm_severity: 2,
                hysteresis: 0,
            });

        manager.create_pv_double(pv_name, 50.0, metadata)
            .expect("Failed to create PV");

        thread::sleep(Duration::from_millis(100));

        let mut ctx = Context::from_env()
            .expect("Failed to create client");

        // Verify PV exists
        let value = ctx.get(pv_name, 2.0).expect("Failed to get PV");
        assert!((value.get_field_double("value").unwrap() - 50.0).abs() < 1e-6);

        // Remove the PV
        manager.remove_pv(pv_name)
            .expect("Failed to remove PV");

        thread::sleep(Duration::from_millis(100));

        // Verify PV is no longer accessible
        let result = ctx.get(pv_name, 2.0);
        assert!(result.is_err(), "PV should not be accessible after removal");

        manager.stop().expect("Failed to stop manager");
    }

    #[test]
    fn test_duplicate_pv_creation() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let pv_name = "test:lifecycle:duplicate";
        let metadata = NTScalarMetadataBuilder::new()
            .alarm(AlarmMetadata {
                active: true,
                low_alarm_limit: 10.0,
                low_warning_limit: 20.0,
                high_warning_limit: 80.0,
                high_alarm_limit: 90.0,
                low_alarm_severity: 2,
                low_warning_severity: 1,
                high_warning_severity: 1,
                high_alarm_severity: 2,
                hysteresis: 0,
            });

        // Create first PV
        manager.create_pv_double(pv_name, 50.0, metadata.clone())
            .expect("Failed to create PV");

        // Try to create duplicate - should fail
        let result = manager.create_pv_double(pv_name, 75.0, metadata);
        assert!(result.is_err(), "Should not be able to create duplicate PV");
        assert!(result.unwrap_err().to_string().contains("already exists"));

        manager.stop().expect("Failed to stop manager");
    }

    #[test]
    fn test_post_to_nonexistent_pv() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let result = manager.post_double("test:lifecycle:nonexistent", 42.0);
        assert!(result.is_err(), "Should not be able to post to non-existent PV");
        assert!(result.unwrap_err().to_string().contains("not found"));

        manager.stop().expect("Failed to stop manager");
    }

    #[test]
    fn test_alarm_persistence_across_posts() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let pv_name = "test:lifecycle:persistence";
        let metadata = NTScalarMetadataBuilder::new()
            .control(ControlMetadata {
                limit_low: 0.0,
                limit_high: 100.0,
                min_step: 0.1,
            });

        manager.create_pv_double(pv_name, 50.0, metadata)
            .expect("Failed to create PV");

        thread::sleep(Duration::from_millis(100));

        let mut ctx = Context::from_env()
            .expect("Failed to create client");

        // Post valid value
        manager.post_double(pv_name, 75.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert!((value.get_field_double("value").unwrap() - 75.0).abs() < 1e-6);

        // Post invalid value (out of control range)
        manager.post_double(pv_name, 150.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        // Value should remain at last valid value
        assert!((value.get_field_double("value").unwrap() - 75.0).abs() < 1e-6);

        // Post another valid value to verify system still works
        manager.post_double(pv_name, 25.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert!((value.get_field_double("value").unwrap() - 25.0).abs() < 1e-6);

        manager.stop().expect("Failed to stop manager");
    }

    #[test]
    fn test_manager_handle_after_stop() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let handle = manager.handle();
        let pv_name = "test:lifecycle:handle";
        let metadata = NTScalarMetadataBuilder::new();

        handle.create_pv_double(pv_name, 42.0, metadata)
            .expect("Failed to create PV via handle");

        manager.stop().expect("Failed to stop manager");

        // Try to use handle after manager stopped
        let result = handle.post_double(pv_name, 100.0);
        assert!(result.is_err(), "Handle should not work after manager stopped");
    }

    #[test]
    fn test_mixed_pv_types_with_alarms() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let metadata_double = NTScalarMetadataBuilder::new()
            .alarm(AlarmMetadata {
                active: true,
                low_alarm_limit: 10.0,
                low_warning_limit: 20.0,
                high_warning_limit: 80.0,
                high_alarm_limit: 90.0,
                low_alarm_severity: 2,
                low_warning_severity: 1,
                high_warning_severity: 1,
                high_alarm_severity: 2,
                hysteresis: 0,
            });

        let metadata_int = NTScalarMetadataBuilder::new()
            .control(ControlMetadata {
                limit_low: 0.0,
                limit_high: 255.0,
                min_step: 1.0,
            });

        manager.create_pv_double("test:lifecycle:mixed:double", 50.0, metadata_double)
            .expect("Failed to create double PV");
        manager.create_pv_int32("test:lifecycle:mixed:int32", 128, metadata_int)
            .expect("Failed to create int32 PV");
        manager.create_pv_string("test:lifecycle:mixed:string", "test", NTScalarMetadataBuilder::new())
            .expect("Failed to create string PV");

        thread::sleep(Duration::from_millis(100));

        let mut ctx = Context::from_env()
            .expect("Failed to create client");

        // Verify all types are accessible
        let val_double = ctx.get("test:lifecycle:mixed:double", 2.0).expect("Failed to get double");
        assert!((val_double.get_field_double("value").unwrap() - 50.0).abs() < 1e-6);

        let val_int = ctx.get("test:lifecycle:mixed:int32", 2.0).expect("Failed to get int32");
        assert_eq!(val_int.get_field_int32("value").unwrap(), 128);

        let val_string = ctx.get("test:lifecycle:mixed:string", 2.0).expect("Failed to get string");
        assert_eq!(val_string.get_field_string("value").unwrap(), "test");

        manager.stop().expect("Failed to stop manager");
    }
}

