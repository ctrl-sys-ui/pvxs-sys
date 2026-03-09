use crate::{ControlMetadata, AlarmMetadata};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlarmSeverity {
    NoAlarm = 0,
    Minor = 1,
    Major = 2,
    Invalid = 3,
    UndefinedAlarm = 4,
 } 
impl Default for AlarmSeverity {
    fn default() -> Self {
        AlarmSeverity::NoAlarm
    }
}

impl From<i32> for AlarmSeverity {
    fn from(value: i32) -> Self {
        match value {
            0 => AlarmSeverity::NoAlarm,
            1 => AlarmSeverity::Minor,
            2 => AlarmSeverity::Major,
            3 => AlarmSeverity::Invalid,
            4 => AlarmSeverity::UndefinedAlarm,
            _ => AlarmSeverity::UndefinedAlarm,
        }
    }
}

 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlarmStatus  {
    NoAlarm = 0 ,
    DeviceStatus = 1,
    DriverStatus = 2,
    RecordStatus = 3,
    DbStatus = 4,
    ConfigStatus = 5,
    UndefinedStatus = 6,
    ClientStatus = 7,
}

impl Default for AlarmStatus {
    fn default() -> Self {
        AlarmStatus::NoAlarm
    }
}

impl From<i32> for AlarmStatus {
    fn from(value: i32) -> Self {
        match value {
            0 => AlarmStatus::NoAlarm,
            1 => AlarmStatus::DeviceStatus,
            2 => AlarmStatus::DriverStatus,
            3 => AlarmStatus::RecordStatus,
            4 => AlarmStatus::DbStatus,
            5 => AlarmStatus::ConfigStatus,
            6 => AlarmStatus::UndefinedStatus,
            7 => AlarmStatus::ClientStatus,
            _ => AlarmStatus::UndefinedStatus,
        }
    }
}



#[derive(Clone, Debug, Default)]
pub struct AlarmConfig {
    pub(crate) control: Option<ControlMetadata>,
    pub(crate) alarm_metadata: Option<AlarmMetadata>,
}

#[derive(Clone, Debug)]
pub struct AlarmResult {
    pub(crate) allow: bool,
    pub(crate) severity: AlarmSeverity,
    pub(crate) status: AlarmStatus,
    pub(crate) message: String,
}

pub fn compute_alarm_for_scalar(value: f64, config: &AlarmConfig) -> AlarmResult {
    // Control limits: if present, reject updates outside limits
    if let Some(control) = &config.control {
        let hysteresis = config.alarm_metadata.as_ref().map_or(0.0, |m| m.hysteresis as f64);
        if value < control.limit_low + hysteresis || value > control.limit_high - hysteresis {
            return AlarmResult {
                allow: false,
                severity: AlarmSeverity::Invalid,
                status: AlarmStatus::RecordStatus,
                message: "OUT_OF_CONTROL_LIMITS".to_string(),
            };
        }
    }

    if let Some(value_alarm) = &config.alarm_metadata {
        if value_alarm.active {
            if value <= value_alarm.low_alarm_limit + value_alarm.hysteresis as f64 {
                return AlarmResult {
                    allow: true,
                    severity: value_alarm.low_alarm_severity,
                    status: AlarmStatus::DeviceStatus,
                    message: "LOW_ALARM".to_string(),
                };
            }
            if value <= value_alarm.low_warning_limit + value_alarm.hysteresis as f64 {
                return AlarmResult {
                    allow: true,
                    severity: value_alarm.low_warning_severity,
                    status: AlarmStatus::DeviceStatus,
                    message: "LOW_WARNING".to_string(),
                };
            }
            if value >= value_alarm.high_alarm_limit - value_alarm.hysteresis as f64 {
                return AlarmResult {
                    allow: true,
                    severity: value_alarm.high_alarm_severity,
                    status: AlarmStatus::DeviceStatus,
                    message: "HIGH_ALARM".to_string(),
                };
            }
            if value >= value_alarm.high_warning_limit - value_alarm.hysteresis as f64 {
                return AlarmResult {
                    allow: true,
                    severity: value_alarm.high_warning_severity,
                    status: AlarmStatus::DeviceStatus,
                    message: "HIGH_WARNING".to_string(),
                };
            }
        }
    }

    AlarmResult {
        allow: true,
        severity: AlarmSeverity::NoAlarm,
        status: AlarmStatus::NoAlarm,
        message: "OK".to_string(),
    }
}
