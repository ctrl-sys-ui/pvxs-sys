#[cfg(test)]
mod test_server_alarm_manager_value_alarms {
    use pvxs_sys::{Server, Context, NTScalarMetadataBuilder, AlarmMetadata, AlarmSeverity, AlarmStatus};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_high_alarm_limit() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let pv_name = "test:alarm:high";
        let metadata = NTScalarMetadataBuilder::new()
            .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "")
            .alarm_metadata(AlarmMetadata {
                active: true,
                low_alarm_limit: 10.0,
                low_warning_limit: 20.0,
                high_warning_limit: 80.0,
                high_alarm_limit: 90.0,
                low_alarm_severity: AlarmSeverity::Major,    // Major
                low_warning_severity: AlarmSeverity::Minor,  // Minor
                high_warning_severity: AlarmSeverity::Minor, // Minor
                high_alarm_severity: AlarmSeverity::Major,   // Major
                hysteresis: 0,
            });

        manager.create_pv_double(pv_name, 50.0, metadata)
            .expect("Failed to create PV with value alarms");

        thread::sleep(Duration::from_millis(100));

        let mut ctx = Context::from_env()
            .expect("Failed to create client");

        // Post a value that triggers high alarm
        manager.post_double(pv_name, 95.0)
            .expect("Failed to post high alarm value");
        thread::sleep(Duration::from_millis(50));

        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let retrieved = value.get_field_double("value").expect("Failed to get value");
        assert!((retrieved - 95.0).abs() < 1e-6, "Value should be updated");

        let severity = value.get_field_int32("alarm.severity").expect("Failed to get severity");
        let status = value.get_field_int32("alarm.status").expect("Failed to get status");
        
        assert_eq!(severity, AlarmSeverity::Major as i32, "Expected Major severity (2), got {}", severity);
        assert_eq!(status, AlarmStatus::DeviceStatus as i32, "Expected Device status (1
        ), got {}", status);

        manager.stop_drop().expect("Failed to stop manager");
    }

    #[test]
    fn test_high_warning_limit() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let pv_name = "test:alarm:high_warn";
        let metadata = NTScalarMetadataBuilder::new()
            .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "Ok")
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

        // Post value that triggers high warning (between 80 and 90)
        manager.post_double(pv_name, 85.0)
            .expect("Failed to post high warning value");
        thread::sleep(Duration::from_millis(50));

        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let severity = value.get_field_int32("alarm.severity").expect("Failed to get severity");
        let status = value.get_field_int32("alarm.status").expect("Failed to get status");

        assert_eq!(severity, AlarmSeverity::Minor as i32, "Expected Minor severity (1), got {}", severity);
        assert_eq!(status, AlarmStatus::DeviceStatus as i32, "Expected Device status (1), got {}", status);

        manager.stop_drop().expect("Failed to stop manager");
    }

    #[test]
    fn test_low_alarm_limit() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let pv_name = "test:alarm:low";
        let metadata = NTScalarMetadataBuilder::new()
            .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "Ok")
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

        // Post value that triggers low alarm
        manager.post_double(pv_name, 5.0)
            .expect("Failed to post low alarm value");
        thread::sleep(Duration::from_millis(50));

        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let severity = value.get_field_int32("alarm.severity").expect("Failed to get severity");
        let status = value.get_field_int32("alarm.status").expect("Failed to get status");

        assert_eq!(severity, AlarmSeverity::Major as i32, "Expected Major severity (2), got {}", severity);
        assert_eq!(status, AlarmStatus::DeviceStatus as i32, "Expected Device status (1), got {}", status); // AlarmStatus::LoLo

        manager.stop_drop().expect("Failed to stop manager");
    }

    #[test]
    fn test_low_warning_limit() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let pv_name = "test:alarm:low_warn";
        let metadata = NTScalarMetadataBuilder::new()
            .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "Ok")
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

        // Post value that triggers low warning (between 10 and 20)
        manager.post_double(pv_name, 15.0)
            .expect("Failed to post low warning value");
        thread::sleep(Duration::from_millis(50));

        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let severity = value.get_field_int32("alarm.severity").expect("Failed to get severity");
        let status = value.get_field_int32("alarm.status").expect("Failed to get status");

        assert_eq!(severity, AlarmSeverity::Minor as i32, "Expected Minor severity (1), got {}", severity);
        assert_eq!(status, AlarmStatus::DeviceStatus as i32, "Expected Device status (1), got {}", status); // AlarmStatus::Low

        manager.stop_drop().expect("Failed to stop manager");
    }

    #[test]
    fn test_no_alarm_within_normal_range() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let pv_name = "test:alarm:normal";
        let metadata = NTScalarMetadataBuilder::new()
            .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "Ok")
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

        // Post value in normal range
        manager.post_double(pv_name, 50.0)
            .expect("Failed to post normal value");
        thread::sleep(Duration::from_millis(50));

        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let severity = value.get_field_int32("alarm.severity").expect("Failed to get severity");
        let status = value.get_field_int32("alarm.status").expect("Failed to get status");

        assert_eq!(severity, 0, "Expected NoAlarm severity (0), got {}", severity);
        assert_eq!(status, 0, "Expected NoAlarm status (0), got {}", status);

        manager.stop_drop().expect("Failed to stop manager");
    }

    #[test]
    fn test_inactive_value_alarm() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let pv_name = "test:alarm:inactive";
        let metadata = NTScalarMetadataBuilder::new()
            .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "Ok")
            .alarm_metadata(AlarmMetadata {
                active: false, // Alarms disabled
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

        // Post value that would trigger alarm if active
        manager.post_double(pv_name, 95.0)
            .expect("Failed to post value");
        thread::sleep(Duration::from_millis(50));

        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let severity = value.get_field_int32("alarm.severity").expect("Failed to get severity");

        // Should be no alarm because alarms are disabled
        assert_eq!(severity, 0, "Expected no alarm when inactive");

        manager.stop_drop().expect("Failed to stop manager");
    }

    #[test]
    fn test_alarm_severity_levels_int32() {
        let manager = Server::start_from_env()
            .expect("Failed to create Server");

        let pv_name = "test:alarm:int32:severity";
        let metadata = NTScalarMetadataBuilder::new()
            .alarm(AlarmSeverity::NoAlarm, AlarmStatus::NoAlarm, "Ok")
            .alarm_metadata(AlarmMetadata {
                active: true,
                low_alarm_limit: 5.0,
                low_warning_limit: 10.0,
                high_warning_limit: 90.0,
                high_alarm_limit: 95.0,
                low_alarm_severity: AlarmSeverity::Invalid,    // Invalid
                low_warning_severity: AlarmSeverity::Minor,  // Minor
                high_warning_severity: AlarmSeverity::Minor, // Minor
                high_alarm_severity: AlarmSeverity::Invalid,   // Invalid
                hysteresis: 0,
            });

        manager.create_pv_int32(pv_name, 50, metadata)
            .expect("Failed to create int32 PV");

        thread::sleep(Duration::from_millis(100));

        let mut ctx = Context::from_env()
            .expect("Failed to create client");

        // Test high alarm with Invalid severity
        manager.post_int32(pv_name, 100)
            .expect("Failed to post value");
        thread::sleep(Duration::from_millis(50));

        let value = ctx.get(pv_name, 2.0).expect("Failed to get value");
        let severity = value.get_field_int32("alarm.severity").expect("Failed to get severity");

        assert_eq!(severity, AlarmSeverity::Invalid as i32, "Expected Invalid severity (3) for high alarm");

        manager.stop_drop().expect("Failed to stop manager");
    }
}

