#[cfg(test)]
mod test_server_alarm_manager_combined {
    use pvxs_sys::{ServerPvManager, Context, NTScalarMetadataBuilder, ControlMetadata, ValueAlarmMetadata};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_control_and_value_alarms_combined() {
        // Test that control limits take precedence over value alarms
        let manager = ServerPvManager::start_from_env()
            .expect("Failed to create ServerPvManager");

        let pv_name = "test:combined:control_value";
        let metadata = NTScalarMetadataBuilder::new()
            .control(ControlMetadata {
                limit_low: 0.0,
                limit_high: 100.0,
                min_step: 0.1,
            })
            .value_alarm(ValueAlarmMetadata {
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

        // Post a value outside control limits - should be rejected with Invalid alarm
        manager.post_double(pv_name, 150.0)
            .expect("Post should succeed");
        thread::sleep(Duration::from_millis(50));

        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let retrieved = value.get_field_double("value").expect("Failed to get value");
        let severity = value.get_field_int32("alarm.severity").expect("Failed to get severity");
        let status = value.get_field_int32("alarm.status").expect("Failed to get status");

        // Value should be unchanged (rejected)
        assert!((retrieved - 50.0).abs() < 1e-6, "Value should be rejected");
        // Control limit violation takes precedence
        assert_eq!(severity, 3, "Expected Invalid severity"); // AlarmSeverity::Invalid
        assert_eq!(status, 11, "Expected HwLimit status");    // AlarmStatus::HwLimit

        // Post a value within control limits but triggering high warning
        manager.post_double(pv_name, 85.0)
            .expect("Failed to post value");
        thread::sleep(Duration::from_millis(50));

        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let retrieved = value.get_field_double("value").expect("Failed to get value");
        let severity = value.get_field_int32("alarm.severity").expect("Failed to get severity");
        let status = value.get_field_int32("alarm.status").expect("Failed to get status");

        assert!((retrieved - 85.0).abs() < 1e-6, "Value should be accepted");
        assert_eq!(severity, 1, "Expected Minor severity for high warning");
        assert_eq!(status, 4, "Expected High status");

        manager.stop().expect("Failed to stop manager");
    }

    #[test]
    fn test_alarm_transitions() {
        // Test transitioning through different alarm states
        let manager = ServerPvManager::start_from_env()
            .expect("Failed to create ServerPvManager");

        let pv_name = "test:transitions";
        let metadata = NTScalarMetadataBuilder::new()
            .value_alarm(ValueAlarmMetadata {
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

        // Start with normal value
        manager.post_double(pv_name, 50.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert_eq!(value.get_field_int32("alarm.severity").unwrap(), 0, "Should be NoAlarm");

        // Transition to high warning
        manager.post_double(pv_name, 85.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert_eq!(value.get_field_int32("alarm.severity").unwrap(), 1, "Should be Minor");
        assert_eq!(value.get_field_int32("alarm.status").unwrap(), 4, "Should be High");

        // Transition to high alarm
        manager.post_double(pv_name, 95.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert_eq!(value.get_field_int32("alarm.severity").unwrap(), 2, "Should be Major");
        assert_eq!(value.get_field_int32("alarm.status").unwrap(), 3, "Should be HiHi");

        // Return to normal
        manager.post_double(pv_name, 50.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert_eq!(value.get_field_int32("alarm.severity").unwrap(), 0, "Should return to NoAlarm");

        // Transition to low warning
        manager.post_double(pv_name, 15.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert_eq!(value.get_field_int32("alarm.severity").unwrap(), 1, "Should be Minor");
        assert_eq!(value.get_field_int32("alarm.status").unwrap(), 6, "Should be Low");

        // Transition to low alarm
        manager.post_double(pv_name, 5.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert_eq!(value.get_field_int32("alarm.severity").unwrap(), 2, "Should be Major");
        assert_eq!(value.get_field_int32("alarm.status").unwrap(), 5, "Should be LoLo");

        manager.stop().expect("Failed to stop manager");
    }

    #[test]
    fn test_multiple_pvs_with_different_alarms() {
        let manager = ServerPvManager::start_from_env()
            .expect("Failed to create ServerPvManager");

        // Create multiple PVs with different alarm configurations
        let pv1 = "test:multi:pv1";
        let metadata1 = NTScalarMetadataBuilder::new()
            .value_alarm(ValueAlarmMetadata {
                active: true,
                low_alarm_limit: 0.0,
                low_warning_limit: 10.0,
                high_warning_limit: 90.0,
                high_alarm_limit: 100.0,
                low_alarm_severity: 2,
                low_warning_severity: 1,
                high_warning_severity: 1,
                high_alarm_severity: 2,
                hysteresis: 0,
            });

        let pv2 = "test:multi:pv2";
        let metadata2 = NTScalarMetadataBuilder::new()
            .control(ControlMetadata {
                limit_low: -100.0,
                limit_high: 100.0,
                min_step: 1.0,
            });

        manager.create_pv_double(pv1, 50.0, metadata1)
            .expect("Failed to create PV1");
        manager.create_pv_double(pv2, 0.0, metadata2)
            .expect("Failed to create PV2");

        thread::sleep(Duration::from_millis(100));

        let mut ctx = Context::from_env()
            .expect("Failed to create client");

        // Post alarm condition to PV1
        manager.post_double(pv1, 95.0).expect("Failed to post to PV1");
        thread::sleep(Duration::from_millis(50));

        // Post out-of-range to PV2
        manager.post_double(pv2, 150.0).expect("Failed to post to PV2");
        thread::sleep(Duration::from_millis(50));

        // Check PV1 has value alarm
        let value1 = ctx.get(pv1, 2.0).expect("Failed to get PV1");
        assert_eq!(value1.get_field_int32("alarm.severity").unwrap(), 2);
        assert!((value1.get_field_double("value").unwrap() - 95.0).abs() < 1e-6);

        // Check PV2 rejected the value
        let value2 = ctx.get(pv2, 2.0).expect("Failed to get PV2");
        assert_eq!(value2.get_field_int32("alarm.severity").unwrap(), 3);
        assert!((value2.get_field_double("value").unwrap() - 0.0).abs() < 1e-6);

        manager.stop().expect("Failed to stop manager");
    }

    #[test]
    fn test_boundary_alarm_conditions() {
        let manager = ServerPvManager::start_from_env()
            .expect("Failed to create ServerPvManager");

        let pv_name = "test:boundary:alarms";
        let metadata = NTScalarMetadataBuilder::new()
            .value_alarm(ValueAlarmMetadata {
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

        // Test exact alarm limit (should trigger alarm)
        manager.post_double(pv_name, 10.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert_eq!(value.get_field_int32("alarm.status").unwrap(), 5, "Exact low limit should trigger LoLo");

        // Test exact warning limit (should trigger warning)
        manager.post_double(pv_name, 20.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert_eq!(value.get_field_int32("alarm.status").unwrap(), 6, "Exact low warning should trigger Low");

        // Test between warning and alarm boundaries
        manager.post_double(pv_name, 15.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert_eq!(value.get_field_int32("alarm.status").unwrap(), 6, "Between limits should trigger Low");

        manager.stop().expect("Failed to stop manager");
    }
}
