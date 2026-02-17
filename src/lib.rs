//! # EPICS PVXS Rust Bindings
//! 
//! Safe Rust bindings for the EPICS PVXS (PVAccess) library.
//! 
//! ## Overview
//! 
//! This crate provides idiomatic Rust bindings to the EPICS PVXS C++ library,
//! which implements the PVAccess network protocol used in EPICS (Experimental
//! Physics and Industrial Control System).
//! 
//! ## Features
//! 
//! - **GET operations**: Read process variable values
//! - **PUT operations**: Write process variable values  
//! - **INFO operations**: Query PV type information
//! - **MONITOR operations**: Subscribe to value changes with callbacks
//! - **MonitorBuilder**: Advanced monitor configuration with PVXS-style API
//! - **Array support**: Read/write arrays of double, int32, and string values
//! - **Server support**: Create and manage PVAccess servers
//! - Thread-safe client context
//! 

pub mod bridge;

mod client;
mod server;
mod value;
mod alarms;
mod metadata;

use std::fmt;

pub use bridge::{
    ContextWrapper,
    ValueWrapper,
    RpcWrapper,
    MonitorWrapper,
    MonitorBuilderWrapper,
    ServerWrapper,
    SharedPVWrapper,
    StaticSourceWrapper,
};

/// Configure PVXS logging from environment variable `PVXS_LOG`
/// 
/// Reads the `PVXS_LOG` environment variable to configure logging levels.
/// Format: "logger_name=LEVEL,another=LEVEL"
/// 
/// Examples:
/// - `PVXS_LOG="*=DEBUG"` - all loggers at DEBUG level
/// - `PVXS_LOG="pvxs.*=INFO"` - all internal loggers at INFO
/// - `PVXS_LOG="pvxs.tcp.io=CRIT"` - suppress tcp.io errors below CRIT
/// 
/// Levels: CRIT < ERR < WARN < INFO < DEBUG
/// 
/// # Example
/// 
/// ```no_run
/// use pvxs_sys::configure_logging_from_env;
/// 
/// // Read from PVXS_LOG environment variable
/// configure_logging_from_env().ok();
/// ```
pub fn configure_logging_from_env() -> Result<()> {
    bridge::pvxs_logger_config_env().map_err(|e| e.into())
}

/// Set logging level for a specific PVXS logger
/// 
/// Programmatically set the log level for a named logger.
/// 
/// # Arguments
/// 
/// * `name` - Logger name (e.g., "pvxs.tcp.io", "pvxs.*" for wildcards)
/// * `level` - One of: "CRIT", "ERR", "WARN", "INFO", "DEBUG"
/// 
/// # Example
/// 
/// ```no_run
/// use pvxs_sys::set_logger_level;
/// 
/// // Suppress benign TCP disconnect errors (socket error 10054)
/// set_logger_level("pvxs.tcp.io", "CRIT").ok();
/// ```
pub fn set_logger_level(name: &str, level: &str) -> Result<()> {
    bridge::pvxs_logger_level_set(name.to_string(), level.to_string()).map_err(|e| e.into())
}

pub use metadata::{AlarmMetadata, DisplayMetadata, ControlMetadata};
pub use alarms::{AlarmSeverity, AlarmStatus, AlarmConfig, AlarmResult, compute_alarm_for_scalar};
pub use client::{Context, Monitor, MonitorBuilder, MonitorEvent, Rpc};
pub use value::Value;
pub use server::{
    Server,
    ServerHandle,
    SharedPV,
    StaticSource,
    NTScalarMetadataBuilder,
    NTEnumMetadataBuilder,
    FetchedDouble,
    FetchedDoubleArray,
    FetchedInt32,
    FetchedInt32Array,
    FetchedString,
    FetchedStringArray,
    FetchedEnum,
};

// Re-export for testing callbacks
pub use std::sync::atomic::{AtomicUsize, Ordering};

// Re-export for convenience
pub type Result<T> = std::result::Result<T, PvxsError>;

/// Error type for PVXS operations
#[derive(Debug, Clone)]
pub struct PvxsError {
    message: String,
}

impl PvxsError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PvxsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PVXS error: {}", self.message)
    }
}

impl std::error::Error for PvxsError {}

impl From<cxx::Exception> for PvxsError {
    fn from(e: cxx::Exception) -> Self {
        Self::new(e.what())
    }
}
