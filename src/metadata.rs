// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use crate::{AlarmSeverity};

/// Alarm metadata for NTScalar
#[derive(Clone, Debug, Default)]
pub struct AlarmMetadata {
    pub active: bool,
    pub low_alarm_limit: f64,
    pub low_warning_limit: f64,
    pub high_warning_limit: f64,
    pub high_alarm_limit: f64,
    pub low_alarm_severity: AlarmSeverity,
    pub low_warning_severity: AlarmSeverity,
    pub high_warning_severity: AlarmSeverity,
    pub high_alarm_severity: AlarmSeverity,
    pub hysteresis: u8,
}

/// Display metadata for NTScalar
#[derive(Clone, Debug, Default)]
pub struct DisplayMetadata {
    pub limit_low: i64,
    pub limit_high: i64,
    pub description: String,
    pub units: String,
    pub precision: i32,
}

/// Control metadata for NTScalar
#[derive(Clone, Debug, Default)]
pub struct ControlMetadata {
    pub limit_low: f64,
    pub limit_high: f64,
    pub min_step: f64,
}
