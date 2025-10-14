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
//! - Thread-safe client context
//! 
//! ## Example
//! 
//! ```no_run
//! use epics_pvxs_sys::{Context, PvxsError};
//! 
//! fn main() -> Result<(), PvxsError> {
//!     // Create a client context from environment variables
//!     let ctx = Context::from_env()?;
//!     
//!     // Read a PV value (timeout after 5 seconds)
//!     let value = ctx.get("my:pv:name", 5.0)?;
//!     
//!     // Access the value field as a double
//!     let val = value.get_field_double("value")?;
//!     println!("Value: {}", val);
//!     
//!     // Write a new value
//!     ctx.put_double("my:pv:name", 42.0, 5.0)?;
//!     
//!     Ok(())
//! }
//! ```
//! 
//! ## Requirements
//! 
//! - EPICS Base (set `EPICS_BASE` environment variable)
//! - PVXS library (set `PVXS_DIR` or built within EPICS)
//! - `EPICS_HOST_ARCH` environment variable (auto-detected if not set)

mod bridge;

use cxx::UniquePtr;
use std::fmt;

pub use bridge::{ContextWrapper, ValueWrapper};

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

/// A PVXS client context for performing PVAccess operations
/// 
/// The Context is the main entry point for interacting with PVAccess.
/// It manages network connections and provides methods for GET, PUT,
/// and other PV operations.
/// 
/// # Thread Safety
/// 
/// Context is Send and Sync, and can be safely shared between threads.
pub struct Context {
    inner: UniquePtr<ContextWrapper>,
}

impl Context {
    /// Create a new Context configured from environment variables
    /// 
    /// Reads configuration from `EPICS_PVA_*` environment variables:
    /// - `EPICS_PVA_ADDR_LIST`: List of server addresses
    /// - `EPICS_PVA_AUTO_ADDR_LIST`: Auto-discover servers (default: YES)
    /// - `EPICS_PVA_BROADCAST_PORT`: UDP broadcast port (default: 5076)
    /// 
    /// # Errors
    /// 
    /// Returns an error if the context cannot be created.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// use epics_pvxs_sys::Context;
    /// 
    /// let ctx = Context::from_env().expect("Failed to create context");
    /// ```
    pub fn from_env() -> Result<Self> {
        let inner = bridge::create_context_from_env()?;
        Ok(Self { inner })
    }
    
    /// Perform a synchronous GET operation
    /// 
    /// Retrieves the current value of a process variable.
    /// 
    /// # Arguments
    /// 
    /// * `pv_name` - The name of the process variable
    /// * `timeout` - Maximum time to wait in seconds
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The PV doesn't exist
    /// - The operation times out
    /// - A network error occurs
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let value = ctx.get("my:pv:name", 5.0).expect("GET failed");
    /// println!("Value: {}", value);
    /// ```
    pub fn get(&mut self, pv_name: &str, timeout: f64) -> Result<Value> {
        let inner = bridge::context_get_sync(self.inner.pin_mut(), pv_name, timeout)?;
        Ok(Value { inner })
    }
    
    /// Perform a synchronous PUT operation with a double value
    /// 
    /// Sets the "value" field of a process variable to a double.
    /// 
    /// # Arguments
    /// 
    /// * `pv_name` - The name of the process variable
    /// * `value` - The value to write
    /// * `timeout` - Maximum time to wait in seconds
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The PV doesn't exist or is read-only
    /// - The operation times out
    /// - The value type doesn't match
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// ctx.put_double("my:pv:name", 42.0, 5.0).expect("PUT failed");
    /// ```
    pub fn put_double(&mut self, pv_name: &str, value: f64, timeout: f64) -> Result<()> {
        bridge::context_put_double(self.inner.pin_mut(), pv_name, value, timeout)?;
        Ok(())
    }
    
    /// Get type information about a process variable
    /// 
    /// Retrieves the structure definition without fetching data.
    /// Useful for discovering the schema of a PV.
    /// 
    /// # Arguments
    /// 
    /// * `pv_name` - The name of the process variable
    /// * `timeout` - Maximum time to wait in seconds
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let info = ctx.info("my:pv:name", 5.0).expect("INFO failed");
    /// println!("PV structure: {}", info);
    /// ```
    pub fn info(&mut self, pv_name: &str, timeout: f64) -> Result<Value> {
        let inner = bridge::context_info_sync(self.inner.pin_mut(), pv_name, timeout)?;
        Ok(Value { inner })
    }
}

// Context is safe to send between threads
unsafe impl Send for Context {}
unsafe impl Sync for Context {}

/// A PVAccess value container
/// 
/// Represents a structured data value returned from PVXS operations.
/// Values have a hierarchical structure with named fields.
/// 
/// # Field Access
/// 
/// Values are accessed by field name. Common fields include:
/// - `"value"`: The primary data value
/// - `"alarm.severity"`: Alarm severity level
/// - `"alarm.status"`: Alarm status code
/// - `"timeStamp.secondsPastEpoch"`: Timestamp seconds
/// 
/// # Example
/// 
/// ```no_run
/// # use epics_pvxs_sys::{Context, Value};
/// # let ctx = Context::from_env().unwrap();
/// let value: Value = ctx.get("my:pv:name", 5.0).unwrap();
/// 
/// // Access different field types
/// let v = value.get_field_double("value").unwrap();
/// let severity = value.get_field_int32("alarm.severity").unwrap();
/// ```
pub struct Value {
    inner: UniquePtr<ValueWrapper>,
}

impl Value {
    /// Check if this value is valid
    /// 
    /// Returns `false` if the value is empty or uninitialized.
    pub fn is_valid(&self) -> bool {
        bridge::value_is_valid(&self.inner)
    }
    
    /// Get a field value as a double
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to a double.
    pub fn get_field_double(&self, field_name: &str) -> Result<f64> {
        Ok(bridge::value_get_field_double(&self.inner, field_name)?)
    }
    
    /// Get a field value as an i32
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to an i32.
    pub fn get_field_int32(&self, field_name: &str) -> Result<i32> {
        Ok(bridge::value_get_field_int32(&self.inner, field_name)?)
    }
    
    /// Get a field value as a String
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to a string.
    pub fn get_field_string(&self, field_name: &str) -> Result<String> {
        Ok(bridge::value_get_field_string(&self.inner, field_name)?)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", bridge::value_to_string(&self.inner))
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Value")
            .field("data", &bridge::value_to_string(&self.inner))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        // This test requires EPICS environment to be set up
        // In a real environment, this should succeed
        let result = Context::from_env();
        
        // We can't assert success without a running EPICS environment
        // but we can check that the function doesn't panic
        match result {
            Ok(_) => println!("Context created successfully"),
            Err(e) => println!("Expected error without EPICS environment: {}", e),
        }
    }
}
