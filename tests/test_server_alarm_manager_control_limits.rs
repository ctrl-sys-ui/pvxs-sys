#[cfg(test)]
mod test_server_alarm_manager_control_limits {
    use pvxs_sys::{Server, Context, NTScalarMetadataBuilder, ControlMetadata};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_control_limits_reject_out_of_bounds() {
        // Create a Server with control limits
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let pv_name = "test:control:reject";
        let metadata = NTScalarMetadataBuilder::new()
            .control(ControlMetadata {
                limit_low: 0.0,
                limit_high: 100.0,
                min_step: 0.1,
            });

        manager.create_pv_double(pv_name, 50.0, metadata)
            .expect("Failed to create PV with control limits");

        thread::sleep(Duration::from_millis(100));

        // Create a client to read the values
        let mut ctx = Context::from_env()
            .expect("Failed to create client context");

        // Post a value within limits - should succeed
        manager.post_double(pv_name, 75.0)
            .expect("Failed to post value within limits");
        
        thread::sleep(Duration::from_millis(50));
        
        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let retrieved = value.get_field_double("value").expect("Failed to get double value");
        assert!((retrieved - 75.0).abs() < 1e-6, "Expected 75.0, got {}", retrieved);

        // Post a value above upper limit - should be rejected
        manager.post_double(pv_name, 150.0)
            .expect("Post should succeed but value should be rejected");
        
        thread::sleep(Duration::from_millis(50));
        
        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let retrieved = value.get_field_double("value").expect("Failed to get double value");
        // Value should still be 75.0 (previous valid value)
        assert!((retrieved - 75.0).abs() < 1e-6, "Expected 75.0 (rejected update), got {}", retrieved);

        // Check that alarm status indicates out of bounds
        let severity = value.get_field_int32("alarm.severity").expect("Failed to get severity");
        let status = value.get_field_int32("alarm.status").expect("Failed to get status");
        assert_eq!(severity, 3, "Expected Invalid severity (3), got {}", severity); // AlarmSeverity::Invalid
        assert_eq!(status, 11, "Expected HwLimit status (11), got {}", status); // AlarmStatus::HwLimit

        // Post a value below lower limit - should be rejected
        manager.post_double(pv_name, -10.0)
            .expect("Post should succeed but value should be rejected");
        
        thread::sleep(Duration::from_millis(50));
        
        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let retrieved = value.get_field_double("value").expect("Failed to get double value");
        // Value should still be 75.0
        assert!((retrieved - 75.0).abs() < 1e-6, "Expected 75.0 (rejected update), got {}", retrieved);

        let severity = value.get_field_int32("alarm.severity").expect("Failed to get severity");
        assert_eq!(severity, 3, "Expected Invalid severity for out-of-bounds");

        manager.stop_drop().expect("Failed to stop manager");
    }

    #[test]
    fn test_control_limits_boundary_values() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let pv_name = "test:control:boundary";
        let metadata = NTScalarMetadataBuilder::new()
            .control(ControlMetadata {
                limit_low: -50.0,
                limit_high: 50.0,
                min_step: 1.0,
            });

        manager.create_pv_double(pv_name, 0.0, metadata)
            .expect("Failed to create PV");

        thread::sleep(Duration::from_millis(100));

        let mut ctx = Context::from_env()
            .expect("Failed to create client");

        // Test exact lower boundary
        manager.post_double(pv_name, -50.0)
            .expect("Failed to post lower boundary");
        thread::sleep(Duration::from_millis(50));
        
        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let retrieved = value.get_field_double("value").expect("Failed to get value");
        assert!((retrieved - (-50.0)).abs() < 1e-6, "Lower boundary should be accepted");

        // Test exact upper boundary
        manager.post_double(pv_name, 50.0)
            .expect("Failed to post upper boundary");
        thread::sleep(Duration::from_millis(50));
        
        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let retrieved = value.get_field_double("value").expect("Failed to get value");
        assert!((retrieved - 50.0).abs() < 1e-6, "Upper boundary should be accepted");

        manager.stop_drop().expect("Failed to stop manager");
    }

    #[test]
    fn test_control_limits_int32() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let pv_name = "test:control:int32";
        let metadata = NTScalarMetadataBuilder::new()
            .control(ControlMetadata {
                limit_low: 0.0,
                limit_high: 255.0,
                min_step: 1.0,
            });

        manager.create_pv_int32(pv_name, 128, metadata)
            .expect("Failed to create int32 PV");

        thread::sleep(Duration::from_millis(100));

        let mut ctx = Context::from_env()
            .expect("Failed to create client");

        // Post valid value
        manager.post_int32(pv_name, 200)
            .expect("Failed to post valid int32");
        thread::sleep(Duration::from_millis(50));
        
        let valid_value = 200;
        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let retrieved = value.get_field_int32("value").expect("Failed to get int32");
        assert_eq!(retrieved, valid_value, "Expected {}", valid_value);

        // Post out of range value (above)
        manager.post_int32(pv_name, 300)
            .expect("Post should succeed");
        thread::sleep(Duration::from_millis(50));
        
        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let retrieved = value.get_field_int32("value").expect("Failed to get int32");
        assert_eq!(retrieved, valid_value, "Value should be unchanged due to control limit");

        let severity = value.get_field_int32("alarm.severity").expect("Failed to get severity");
        assert_eq!(severity, 3, "Expected Invalid severity");

        manager.stop_drop().expect("Failed to stop manager");
    }
}

