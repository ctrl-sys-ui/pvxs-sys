// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
#[cfg(test)]
mod test_server_alarm_manager_combined {
    use pvxs_sys::{Server, Context, NTScalarMetadataBuilder, ControlMetadata, AlarmMetadata, AlarmSeverity, AlarmStatus};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_control_and_value_alarms_combined() {
        // Test that control limits take precedence over value alarms
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let pv_name = "test:combined:control_value";
        let metadata = NTScalarMetadataBuilder::new()
            .control(ControlMetadata {
                limit_low: 0.0,
                limit_high: 100.0,
                min_step: 0.1,
            })
            .alarm (
                AlarmSeverity::NoAlarm,
                AlarmStatus::NoAlarm,
                "Ok",
            )
            .alarm_metadata(AlarmMetadata {
                active: true,
                low_alarm_limit: 10.0,
                low_warning_limit: 20.0,
                high_warning_limit: 80.0,
                high_alarm_limit: 90.0,
                low_alarm_severity: AlarmSeverity::Major,
                low_warning_severity: AlarmSeverity::Minor,
                high_warning_severity: AlarmSeverity::Minor,
                high_alarm_severity: AlarmSeverity::Major,
                hysteresis: 0,
            });
        
        let initial = 50.0;
        manager.create_pv_double(pv_name, initial, metadata)
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
        assert_eq!(retrieved, initial, "Value should be rejected");
        // Control limit violation takes precedence
        assert_eq!(severity, AlarmSeverity::Invalid as i32, "Expected Invalid severity");
        assert_eq!(status, AlarmStatus::RecordStatus as i32, "Expected Record status");

        // Post a value within control limits but triggering high warning
        manager.post_double(pv_name, 85.0)
            .expect("Failed to post value");
        thread::sleep(Duration::from_millis(50));

        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let retrieved = value.get_field_double("value").expect("Failed to get value");
        let severity = value.get_field_int32("alarm.severity").expect("Failed to get severity");
        let status = value.get_field_int32("alarm.status").expect("Failed to get status");

        assert_eq!(retrieved, 85.0, "Value should be accepted");
        assert_eq!(severity, AlarmSeverity::Minor as i32, "Expected Minor severity for high warning");
        assert_eq!(status, AlarmStatus::DeviceStatus as i32, "Expected Device status");

        manager.stop_drop().expect("Failed to stop manager");
    }

    #[test]
    fn test_alarm_transitions() {
        // Test transitioning through different alarm states
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let pv_name = "test:transitions";
        let metadata = NTScalarMetadataBuilder::new()
            .alarm(
                AlarmSeverity::NoAlarm, 
                AlarmStatus::NoAlarm, 
                "OK")
            .alarm_metadata(AlarmMetadata {
                active: true,
                low_alarm_limit: 10.0,
                low_warning_limit: 20.0,
                high_warning_limit: 80.0,
                high_alarm_limit: 90.0,
                low_alarm_severity: AlarmSeverity::Major,
                low_warning_severity: AlarmSeverity::Minor,
                high_warning_severity: AlarmSeverity::Minor,
                high_alarm_severity: AlarmSeverity::Major,
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
        assert_eq!(value.get_field_int32("alarm.severity").unwrap(), AlarmSeverity::Minor as i32, "Should be Minor");
        assert_eq!(value.get_field_int32("alarm.status").unwrap(), AlarmStatus::DeviceStatus as i32, "Should be DeviceStatus");

        // Transition to high alarm
        manager.post_double(pv_name, 95.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert_eq!(value.get_field_int32("alarm.severity").unwrap(), AlarmSeverity::Major as i32, "Should be Major");
        assert_eq!(value.get_field_int32("alarm.status").unwrap(), AlarmStatus::DeviceStatus as i32, "Should be DeviceStatus");

        // Return to normal
        manager.post_double(pv_name, 50.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert_eq!(value.get_field_int32("alarm.severity").unwrap(), AlarmSeverity::NoAlarm as i32, "Should return to NoAlarm");

        // Transition to low warning
        manager.post_double(pv_name, 15.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert_eq!(value.get_field_int32("alarm.severity").unwrap(), AlarmSeverity::Minor as i32, "Should be Minor");
        assert_eq!(value.get_field_int32("alarm.status").unwrap(), AlarmStatus::DeviceStatus as i32, "Should be Low");

        // Transition to low alarm
        manager.post_double(pv_name, 5.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert_eq!(value.get_field_int32("alarm.severity").unwrap(), AlarmSeverity::Major as i32, "Should be Major");
        assert_eq!(value.get_field_int32("alarm.status").unwrap(), AlarmStatus::DeviceStatus as i32, "Should be LoLo");

        manager.stop_drop().expect("Failed to stop manager");
    }

    #[test]
    fn test_multiple_pvs_with_different_alarms() {
        let srv = Server::start_from_env()
            .expect("Failed to create Server");

        // Create multiple PVs with different alarm configurations
        let pv1 = "test:multi:pv1";
        let metadata1 = NTScalarMetadataBuilder::new()
            .control(ControlMetadata {
                limit_low: 0.0,
                limit_high: 100.0,
                min_step: 0.1,
            })
            .alarm(AlarmSeverity::Minor, AlarmStatus::NoAlarm, "Ok")
            .alarm_metadata(AlarmMetadata {
                active: true,
                low_alarm_limit: 0.0,
                low_warning_limit: 10.0,
                high_warning_limit: 90.0,
                high_alarm_limit: 100.0,
                low_alarm_severity: AlarmSeverity::Major,
                low_warning_severity: AlarmSeverity::Minor,
                high_warning_severity: AlarmSeverity::Minor,
                high_alarm_severity: AlarmSeverity::Major,
                hysteresis: 0,
            });

        let pv2 = "test:multi:pv2";
        let metadata2 = NTScalarMetadataBuilder::new()
            .control(ControlMetadata {
                limit_low: -100.0,
                limit_high: 100.0,
                min_step: 1.0,
            })
            .alarm(AlarmSeverity::Major, AlarmStatus::NoAlarm, "Ok")
            .alarm_metadata(AlarmMetadata {
                active: true,
                low_alarm_limit: -50.0,
                low_warning_limit: -20.0,
                high_warning_limit: 20.0,
                high_alarm_limit: 50.0,
                low_alarm_severity: AlarmSeverity::Major,
                low_warning_severity: AlarmSeverity::Minor,
                high_warning_severity: AlarmSeverity::Minor,
                high_alarm_severity: AlarmSeverity::Major,
                hysteresis: 0,
            });
        let pv1_initial = 50.0;
        srv.create_pv_double(pv1, pv1_initial, metadata1)
            .expect("Failed to create PV1");
        let pv2_initial = 1.0;
        srv.create_pv_double(pv2, pv2_initial, metadata2)
            .expect("Failed to create PV2");

        thread::sleep(Duration::from_millis(100));

        let mut ctx = Context::from_env()
            .expect("Failed to create client");

        let pv1_value = 95.0; // Should trigger high alarm on PV1
        // Post alarm condition to PV1
        srv.post_double(pv1, pv1_value).expect("Failed to post to PV1");
        thread::sleep(Duration::from_millis(50));

        // Check PV1 has value alarm
        let value1 = ctx.get(pv1, 2.0).expect("Failed to get PV1");
        // Check that the value is still the same as pv1_value, the server should have rejected the out-of-range value.
        match value1.get_field_double("value") {
            Ok(v) => assert_eq!(v, pv1_value, "PV1 value should be {}", pv1_value),
            Err(e) => assert!(false, "Failed to get PV1 value: {:?}", e),
        }
        // Check that since pv1_value is above 90 but less than 100, it should trigger a Minor alarm
        match value1.get_field_int32("alarm.severity") {
            Ok(s) => assert_eq!(s, AlarmSeverity::Minor as i32, "Expected Minor severity for PV1"),
            Err(e) => assert!(false, "Failed to get PV1 alarm severity: {:?}", e),
        }

        // Post out-of-range to PV2
        let pv2_value = 150.0; // Should be rejected by PV2
        srv.post_double(pv2, pv2_value).expect("Failed to post to PV2");
        thread::sleep(Duration::from_millis(50));

        // Check PV2 rejected the value
        let value2 = ctx.get(pv2, 2.0).expect("Failed to get PV2");
        assert_eq!(value2.get_field_int32("alarm.severity").unwrap(), AlarmSeverity::Invalid as i32);
        assert_eq!(value2.get_field_double("value").unwrap(), pv2_initial, "PV2 value should remain unchanged at {}", pv2_initial);

        srv.stop_drop().expect("Failed to stop manager");
    }

    #[test]
    fn test_boundary_alarm_conditions() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let pv_name = "test:boundary:alarms";
        let metadata = NTScalarMetadataBuilder::new()
            .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "OK")
            .alarm_metadata(AlarmMetadata {
                active: true,
                low_alarm_limit: 10.0,
                low_warning_limit: 20.0,
                high_warning_limit: 80.0,
                high_alarm_limit: 90.0,
                low_alarm_severity: AlarmSeverity::Major,
                low_warning_severity: AlarmSeverity::Minor,
                high_warning_severity: AlarmSeverity::Minor,
                high_alarm_severity: AlarmSeverity::Major,
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
        assert_eq!(value.get_field_int32("alarm.status").unwrap(), AlarmStatus::DeviceStatus as i32, "Exact low limit should trigger LoLo");

        // Test exact warning limit (should trigger warning)
        manager.post_double(pv_name, 20.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert_eq!(value.get_field_int32("alarm.status").unwrap(), AlarmStatus::DeviceStatus as i32, "Exact low warning should trigger Low");

        // Test between warning and alarm boundaries
        manager.post_double(pv_name, 15.0).expect("Failed to post");
        thread::sleep(Duration::from_millis(50));
        let value = ctx.get(pv_name, 2.0).expect("Failed to get");
        assert_eq!(value.get_field_int32("alarm.status").unwrap(), AlarmStatus::DeviceStatus as i32, "Between limits should trigger Low");

        manager.stop_drop().expect("Failed to stop manager");
    }
}

