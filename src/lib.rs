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
//! - **Array support**: Read/write arrays of double, int32, enum, and string values
//! - **Server support**: Create and manage PVAccess servers
//! - Thread-safe client context
//! 
//! ## Basic Example
//! 
//! ```no_run
//! use epics_pvxs_sys::{Context, PvxsError};
//! 
//! fn main() -> Result<(), PvxsError> {
//!     // Create a client context from environment variables
//!     let mut ctx = Context::from_env()?;
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
//! ## Monitor Example with MonitorBuilder
//! 
//! ```no_run
//! use epics_pvxs_sys::{Context, PvxsError};
//! 
//! fn main() -> Result<(), PvxsError> {
//!     let mut ctx = Context::from_env()?;
//!     
//!     // Create a monitor with advanced configuration
//!     let mut monitor = ctx.monitor_builder("my:pv:name")?
//!         .mask_connected(false)      // Don't include connection events
//!         .mask_disconnected(true)    // Include disconnection events
//!         .exec()?;
//!     
//!     monitor.start();
//!     
//!     // PVXS-style event processing
//!     loop {
//!         match monitor.pop() {
//!             Ok(Some(value)) => {
//!                 println!("Update: {}", value);
//!                 // Process the value...
//!             },
//!             Ok(None) => break, // Queue empty
//!             Err(e) => {
//!                 println!("Event: {}", e); // Connection events, errors
//!                 break;
//!             }
//!         }
//!     }
//!     
//!     monitor.stop();
//!     Ok(())
//! }
//! ```
//! 
//! ## Array Example
//! 
//! ```no_run
//! use epics_pvxs_sys::{Context, PvxsError};
//! 
//! fn main() -> Result<(), PvxsError> {
//!     let mut ctx = Context::from_env()?;
//!     
//!     // Read a waveform array
//!     let value = ctx.get("waveform:pv", 5.0)?;
//!     let array = value.get_field_double_array("value")?;
//!     println!("Waveform has {} points", array.len());
//!     
//!     // Read enum choices
//!     let enum_pv = ctx.get("enum:pv", 5.0)?;
//!     let choices = enum_pv.get_field_string_array("value.choices")?;
//!     let index = enum_pv.get_field_enum("value.index")?;
//!     println!("Current choice: '{}'", choices[index as usize]);
//!     
//!     // Write an array
//!     ctx.put_double_array("array:pv", vec![1.0, 2.0, 3.0], 5.0)?;
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

pub mod bridge;

use cxx::UniquePtr;
use std::fmt;

pub use bridge::{ContextWrapper, ValueWrapper, RpcWrapper, MonitorWrapper, MonitorBuilderWrapper, ServerWrapper, SharedPVWrapper, StaticSourceWrapper};

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
        let inner = bridge::context_get(self.inner.pin_mut(), pv_name, timeout)?;
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
    /// ctx.put_double("my:pv:double", 42.0, 5.0).expect("PUT failed");
    /// ```
    pub fn put_double(&mut self, pv_name: &str, value: f64, timeout: f64) -> Result<()> {
        bridge::context_put_double(self.inner.pin_mut(), pv_name, value, timeout)?;
        Ok(())
    }

    /// Perform a synchronous PUT operation with an int32 value
    /// 
    /// Sets the "value" field of a process variable to an int32.
    /// 
    /// # Arguments
    /// 
    /// * `pv_name` - The name of the process variable
    /// * `value` - The value to write
    /// * `timeout` - Maximum time to wait in seconds
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The PV doesn't exist or is read-only
    /// - The operation times out
    /// - The value type doesn't match
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// ctx.put_int32("my:pv:int", 42, 5.0).expect("PUT failed");
    /// ```
    pub fn put_int32(&mut self, pv_name: &str, value: i32, timeout: f64) -> Result<()> {
        bridge::context_put_int32(self.inner.pin_mut(), pv_name, value, timeout)?;
        Ok(())
    }

    /// Perform a synchronous PUT operation with a string value
    /// 
    /// Sets the "value" field of a process variable to a string.
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
    /// ctx.put_string("my:pv:string", "Hello, EPICS!", 5.0).expect("PUT failed");
    /// ```
    pub fn put_string(&mut self, pv_name: &str, value: &str, timeout: f64) -> Result<()> {
        bridge::context_put_string(self.inner.pin_mut(), pv_name, value.to_string(), timeout)?;
        Ok(())
    }

    /// Perform a synchronous PUT operation with an enum value
    /// 
    /// Sets the "value" field of a process variable to an enum (i16).
    /// 
    /// # Arguments
    /// 
    /// * `pv_name` - The name of the process variable
    /// * `value` - The enum value to write
    /// * `timeout` - Maximum time to wait in seconds
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The PV doesn't exist or is read-only
    /// - The operation times out
    /// - The value is not a valid enum choice
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// ctx.put_enum("my:pv:enum", 2, 5.0).expect("PUT failed");
    /// ```
    pub fn put_enum(&mut self, pv_name: &str, value: i16, timeout: f64) -> Result<()> {
        bridge::context_put_enum(self.inner.pin_mut(), pv_name, value, timeout)?;
        Ok(())
    }

    /// Perform a synchronous PUT operation with a double array
    /// 
    /// Sets the "value" field of a process variable to an array of doubles.
    /// 
    /// # Arguments
    /// 
    /// * `pv_name` - The name of the process variable
    /// * `value` - The array of values to write
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
    /// ctx.put_double_array("my:pv:array", vec![1.0, 2.0, 3.0], 5.0).expect("PUT failed");
    /// ```
    pub fn put_double_array(&mut self, pv_name: &str, value: Vec<f64>, timeout: f64) -> Result<()> {
        bridge::context_put_double_array(self.inner.pin_mut(), pv_name, value, timeout)?;
        Ok(())
    }

    /// Perform a synchronous PUT operation with an int32 array
    /// 
    /// Sets the "value" field of a process variable to an array of int32s.
    /// 
    /// # Arguments
    /// 
    /// * `pv_name` - The name of the process variable
    /// * `value` - The array of values to write
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
    /// ctx.put_int32_array("my:pv:array", vec![10, 20, 30], 5.0).expect("PUT failed");
    /// ```
    pub fn put_int32_array(&mut self, pv_name: &str, value: Vec<i32>, timeout: f64) -> Result<()> {
        bridge::context_put_int32_array(self.inner.pin_mut(), pv_name, value, timeout)?;
        Ok(())
    }

    /// Perform a synchronous PUT operation with an enum array
    /// 
    /// Sets the "value" field of a process variable to an array of enums (i16).
    /// 
    /// # Arguments
    /// 
    /// * `pv_name` - The name of the process variable
    /// * `value` - The array of enum values to write
    /// * `timeout` - Maximum time to wait in seconds
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The PV doesn't exist or is read-only
    /// - The operation times out
    /// - Any value is not a valid enum choice
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// ctx.put_enum_array("my:pv:array", vec![0, 1, 2], 5.0).expect("PUT failed");
    /// ```
    pub fn put_enum_array(&mut self, pv_name: &str, value: Vec<i16>, timeout: f64) -> Result<()> {
        bridge::context_put_enum_array(self.inner.pin_mut(), pv_name, value, timeout)?;
        Ok(())
    }

    /// Perform a synchronous PUT operation with a string array
    /// 
    /// Sets the "value" field of a process variable to an array of strings.
    /// 
    /// # Arguments
    /// 
    /// * `pv_name` - The name of the process variable
    /// * `value` - The array of string values to write
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
    /// ctx.put_string_array("my:pv:array", vec!["one".to_string(), "two".to_string()], 5.0).expect("PUT failed");
    /// ```
    pub fn put_string_array(&mut self, pv_name: &str, value: Vec<String>, timeout: f64) -> Result<()> {
        bridge::context_put_string_array(self.inner.pin_mut(), pv_name, value, timeout)?;
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
        let inner = bridge::context_info(self.inner.pin_mut(), pv_name, timeout)?;
        Ok(Value { inner })
    }
    
    /// Create an RPC (Remote Procedure Call) builder
    /// 
    /// Creates a builder for performing RPC operations on EPICS servers.
    /// RPC allows calling server-side functions with arguments.
    /// 
    /// # Arguments
    /// 
    /// * `pv_name` - The name of the RPC service/endpoint
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let mut rpc = ctx.rpc("my:service").expect("RPC creation failed");
    /// rpc.arg_string("command", "start");
    /// rpc.arg_double("value", 42.0);
    /// let result = rpc.execute(5.0).expect("RPC execution failed");
    /// ```
    pub fn rpc(&mut self, pv_name: &str) -> Result<Rpc> {
        let inner = bridge::context_rpc_create(self.inner.pin_mut(), pv_name.to_string())?;
        Ok(Rpc { inner })
    }

    /// Create a monitor for a process variable
    /// 
    /// Monitors allow you to subscribe to value changes and receive notifications
    /// when a PV updates, providing an efficient alternative to polling.
    /// 
    /// # Arguments
    /// 
    /// * `pv_name` - Name of the process variable to monitor
    /// 
    /// # Returns
    /// 
    /// A `Monitor` instance that can be used to receive value updates.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let mut monitor = ctx.monitor("TEST:PV_Double").expect("Monitor creation failed");
    /// 
    /// monitor.start();
    /// 
    /// // Check for updates
    /// if let Some(value) = monitor.try_get_update().expect("Monitor check failed") {
    ///     println!("PV updated: {}", value);
    /// }
    /// 
    /// monitor.stop();
    /// ```
    pub fn monitor(&mut self, pv_name: &str) -> Result<Monitor> {
        let inner = bridge::context_monitor_create(self.inner.pin_mut(), pv_name.to_string())?;
        Ok(Monitor { inner })
    }

    /// Create a MonitorBuilder for advanced monitor configuration
    /// 
    /// Returns a builder that allows configuring event masks and callbacks before
    /// creating the monitor subscription.
    /// 
    /// # Arguments
    /// 
    /// * `pv_name` - Name of the process variable to monitor
    /// 
    /// # Returns
    /// 
    /// A `MonitorBuilder` instance for configuring the monitor.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// use epics_pvxs_sys::Context;
    /// 
    /// let mut ctx = Context::from_env().expect("Context creation failed");
    /// let monitor = ctx.monitor_builder("TEST:PV_Double")
    ///     .mask_connected(false)
    ///     .mask_disconnected(true)
    ///     .exec()
    ///     .expect("Monitor creation failed");
    /// ```
    pub fn monitor_builder(&mut self, pv_name: &str) -> Result<MonitorBuilder> {
        let inner = bridge::context_monitor_builder_create(self.inner.pin_mut(), pv_name.to_string())?;
        Ok(MonitorBuilder { inner })
    }
}

// Context is safe to send between threads
unsafe impl Send for Context {}
unsafe impl Sync for Context {}

/// Async implementation for Context
#[cfg(feature = "async")]
impl Context {
    /// Asynchronously read a process variable value
    /// 
    /// This method uses PVXS RPC for non-blocking operations.
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
    /// # async fn example() -> Result<(), epics_pvxs_sys::PvxsError> {
    /// let mut ctx = Context::from_env()?;
    /// let value = ctx.get_async("my:pv:name", 5.0).await?;
    /// let val = value.get_field_double("value")?;
    /// println!("Value: {}", val);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_async(&mut self, pv_name: &str, timeout: f64) -> Result<Value> {
        let operation = bridge::context_get_async(self.inner.pin_mut(), pv_name, timeout)?;
        self.wait_for_operation(operation).await
    }
    
    /// Asynchronously write a double value to a process variable
    /// 
    /// # Arguments
    /// 
    /// * `pv_name` - The name of the process variable
    /// * `value` - The value to write
    /// * `timeout` - Maximum time to wait in seconds
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # async fn example() -> Result<(), epics_pvxs_sys::PvxsError> {
    /// let mut ctx = Context::from_env()?;
    /// ctx.put_double_async("my:pv:name", 42.0, 5.0).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn put_double_async(&mut self, pv_name: &str, value: f64, timeout: f64) -> Result<()> {
        let operation = bridge::context_put_double_async(self.inner.pin_mut(), pv_name, value, timeout)?;
        self.wait_for_operation(operation).await?;
        Ok(())
    }
    
    /// Asynchronously get type information about a process variable
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
    /// # async fn example() -> Result<(), epics_pvxs_sys::PvxsError> {
    /// let mut ctx = Context::from_env()?;
    /// let info = ctx.info_async("my:pv:name", 5.0).await?;
    /// println!("PV structure: {}", info);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn info_async(&mut self, pv_name: &str, timeout: f64) -> Result<Value> {
        let operation = bridge::context_info_async(self.inner.pin_mut(), pv_name, timeout)?;
        self.wait_for_operation(operation).await
    }
    
    /// Wait for an operation to complete using Tokio's async runtime
    async fn wait_for_operation(&self, mut operation: cxx::UniquePtr<bridge::OperationWrapper>) -> Result<Value> {
        use tokio::time::{sleep, Duration};
        
        loop {
            if bridge::operation_is_done(&operation) {
                let result = bridge::operation_get_result(operation.pin_mut())?;
                return Ok(Value { inner: result });
            }
            
            // Yield control to the async runtime
            sleep(Duration::from_millis(10)).await;
        }
    }
}

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
/// # let mut ctx = Context::from_env().unwrap();
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
        Ok(bridge::value_get_field_double(&self.inner, field_name.to_string())?)
    }
    
    /// Get a field value as an i32
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to an i32.
    pub fn get_field_int32(&self, field_name: &str) -> Result<i32> {
        Ok(bridge::value_get_field_int32(&self.inner, field_name.to_string())?)
    }
    
    /// Get a field value as a String
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to a string.
    pub fn get_field_string(&self, field_name: &str) -> Result<String> {
        Ok(bridge::value_get_field_string(&self.inner, field_name.to_string())?)
    }

    /// Get a field value as a enum
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to a enum.
    pub fn get_field_enum(&self, field_name: &str) -> Result<i16> {
        Ok(bridge::value_get_field_enum(&self.inner, field_name.to_string())?)
    }

    /// Get a field value as an array of doubles
    /// 
    /// Extracts a field containing an array of double-precision floating point values.
    /// Commonly used for waveform data, measurement arrays, or multi-point setpoints.
    /// 
    /// # Arguments
    /// 
    /// * `field_name` - The field path (e.g., "value", "waveform.data")
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to an array of doubles.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let value = ctx.get("waveform:double:pv", 5.0).unwrap();
    /// let array = value.get_field_double_array("value").unwrap();
    /// println!("Double array length: {}", array.len());
    /// for (i, val) in array.iter().enumerate().take(5) {
    ///     println!("  [{}] = {}", i, val);
    /// }
    /// ```
    pub fn get_field_double_array(&self, field_name: &str) -> Result<Vec<f64>> {
        Ok(bridge::value_get_field_double_array(&self.inner, field_name.to_string())?)
    }

    /// Get a field value as an array of int32
    /// 
    /// Extracts a field containing an array of 32-bit signed integers.
    /// Often used for status arrays, configuration parameters, or indexed data.
    /// 
    /// # Arguments
    /// 
    /// * `field_name` - The field path (e.g., "value", "status.codes")
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to an array of int32.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let value = ctx.get("array:int32:pv", 5.0).unwrap();
    /// let array = value.get_field_int32_array("value").unwrap();
    /// println!("Int32 array length: {}", array.len());
    /// for (i, val) in array.iter().enumerate().take(5) {
    ///     println!("  [{}] = {}", i, val);
    /// }
    /// ```
    pub fn get_field_int32_array(&self, field_name: &str) -> Result<Vec<i32>> {
        Ok(bridge::value_get_field_int32_array(&self.inner, field_name.to_string())?)
    }

    /// Get a field value as an array of enums (int16)
    /// 
    /// Extracts a field containing an array of enumerated values.
    /// Each enum is represented as a 16-bit signed integer index.
    /// Use in conjunction with choices arrays to map indices to string labels.
    /// 
    /// # Arguments
    /// 
    /// * `field_name` - The field path (e.g., "value", "states.index")
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to an array of enums.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let value = ctx.get("enum:array:pv", 5.0).unwrap();
    /// let indices = value.get_field_enum_array("value").unwrap();
    /// let choices = value.get_field_string_array("value.choices").unwrap();
    /// 
    /// for (i, &index) in indices.iter().enumerate().take(5) {
    ///     if (index as usize) < choices.len() {
    ///         println!("  [{}] = {} ('{}')", i, index, choices[index as usize]);
    ///     }
    /// }
    /// ```
    pub fn get_field_enum_array(&self, field_name: &str) -> Result<Vec<i16>> {
        Ok(bridge::value_get_field_enum_array(&self.inner, field_name.to_string())?)
    }

    /// Get a field value as an array of strings
    /// 
    /// Extracts a field containing an array of string values.
    /// Commonly used for enum choices, device names, status messages, or text lists.
    /// 
    /// # Arguments
    /// 
    /// * `field_name` - The field path (e.g., "value.choices", "devices.names")
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to an array of strings.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// // Get enum choices for an NTEnum PV
    /// let value = ctx.get("enum:pv", 5.0).unwrap();
    /// let choices = value.get_field_string_array("value.choices").unwrap();
    /// println!("Available choices:");
    /// for (i, choice) in choices.iter().enumerate() {
    ///     println!("  [{}] = '{}'", i, choice);
    /// }
    /// ```
    pub fn get_field_string_array(&self, field_name: &str) -> Result<Vec<String>> {
        Ok(bridge::value_get_field_string_array(&self.inner, field_name.to_string())?)
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

/// RPC (Remote Procedure Call) builder for EPICS servers
/// 
/// Provides a fluent interface for building and executing RPC calls.
/// RPC allows calling server-side functions with typed arguments.
/// 
/// # Example
/// 
/// ```no_run
/// # use epics_pvxs_sys::Context;
/// # let mut ctx = Context::from_env().unwrap();
/// let mut rpc = ctx.rpc("my:service").expect("RPC creation failed");
/// 
/// // Add arguments of different types
/// rpc.arg_string("command", "initialize");
/// rpc.arg_double("threshold", 3.14);
/// rpc.arg_int32("count", 100);
/// rpc.arg_bool("enabled", true);
/// 
/// // Execute synchronously
/// let result = rpc.execute(5.0).expect("RPC execution failed");
/// println!("RPC result: {}", result);
/// ```

/// Monitor represents a subscription to value changes for a process variable.
/// 
/// Monitors allow you to receive notifications when a PV's value changes,
/// providing an efficient way to track real-time updates without polling.
/// 
/// # Example
/// 
/// ```no_run
/// use epics_pvxs_sys::Context;
/// 
/// let mut ctx = Context::from_env()?;
/// let mut monitor = ctx.monitor("MY:PV")?;
/// 
/// monitor.start();
/// 
/// // Wait for updates
/// loop {
///     if let Some(value) = monitor.try_get_update()? {
///         println!("PV updated: {}", value);
///     }
///     std::thread::sleep(std::time::Duration::from_millis(100));
/// }
/// # Ok::<(), epics_pvxs_sys::PvxsError>(())
/// ```
pub struct Monitor {
    inner: UniquePtr<bridge::MonitorWrapper>,
}

impl Monitor {
    /// Start monitoring for value changes
    /// 
    /// This begins the subscription and the monitor will start receiving updates.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// # let mut monitor = ctx.monitor("MY:PV").unwrap();
    /// monitor.start();
    /// ```
    pub fn start(&mut self) {
        bridge::monitor_start(self.inner.pin_mut());
    }
    
    /// Stop monitoring for value changes
    /// 
    /// This ends the subscription and no more updates will be received.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// # let mut monitor = ctx.monitor("MY:PV").unwrap();
    /// # monitor.start();
    /// monitor.stop();
    /// ```
    pub fn stop(&mut self) {
        bridge::monitor_stop(self.inner.pin_mut());
    }
    
    /// Check if the monitor is currently running
    /// 
    /// # Returns
    /// 
    /// `true` if the monitor is active and receiving updates, `false` otherwise.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// # let mut monitor = ctx.monitor("MY:PV").unwrap();
    /// monitor.start();
    /// assert!(monitor.is_running());
    /// ```
    pub fn is_running(&self) -> bool {
        bridge::monitor_is_running(&self.inner)
    }
    
    /// Check if there are updates available without blocking
    /// 
    /// # Returns
    /// 
    /// `true` if updates are available, `false` otherwise.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// # let mut monitor = ctx.monitor("MY:PV").unwrap();
    /// # monitor.start();
    /// if monitor.has_update() {
    ///     let value = monitor.try_get_update()?;
    ///     println!("Update available: {:?}", value);
    /// }
    /// # Ok::<(), epics_pvxs_sys::PvxsError>(())
    /// ```
    pub fn has_update(&self) -> bool {
        bridge::monitor_has_update(&self.inner)
    }
    
    /// Get the next update, blocking with a timeout
    /// 
    /// This method will wait for an update to arrive, up to the specified timeout.
    /// 
    /// # Arguments
    /// 
    /// * `timeout` - Maximum time to wait in seconds
    /// 
    /// # Returns
    /// 
    /// A `Value` if an update was received within the timeout, or an error.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// # let mut monitor = ctx.monitor("MY:PV").unwrap();
    /// # monitor.start();
    /// match monitor.get_update(5.0) {
    ///     Ok(value) => println!("Update received: {}", value),
    ///     Err(e) => println!("No update within 5 seconds: {}", e),
    /// }
    /// # Ok::<(), epics_pvxs_sys::PvxsError>(())
    /// ```
    pub fn get_update(&mut self, timeout: f64) -> Result<Value> {
        let value_wrapper = bridge::monitor_get_update(self.inner.pin_mut(), timeout)?;
        Ok(Value { inner: value_wrapper })
    }
    
    /// Try to get the next update without blocking
    /// 
    /// This method returns immediately, either with an update if one is available,
    /// or `None` if no update is ready.
    /// 
    /// # Returns
    /// 
    /// `Some(Value)` if an update is available, `None` otherwise.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// # let mut monitor = ctx.monitor("MY:PV").unwrap();
    /// # monitor.start();
    /// if let Some(value) = monitor.try_get_update()? {
    ///     println!("Update: {}", value);
    /// } else {
    ///     println!("No update available");
    /// }
    /// # Ok::<(), epics_pvxs_sys::PvxsError>(())
    /// ```
    pub fn try_get_update(&mut self) -> Result<Option<Value>> {
        match bridge::monitor_try_get_update(self.inner.pin_mut()) {
            Ok(value_wrapper) => {
                if value_wrapper.is_null() {
                    Ok(None)
                } else {
                    Ok(Some(Value { inner: value_wrapper }))
                }
            },
            Err(_) => Ok(None), // No update available or error
        }
    }
    
    /// Pop the next update from the subscription queue (PVXS-style)
    /// 
    /// This follows the PVXS pattern where `pop()` returns a Value if available,
    /// or throws specific exceptions for connection events.
    /// 
    /// # Returns
    /// 
    /// A `Value` if an update is available, `None` if the queue is empty.
    /// 
    /// # Errors
    /// 
    /// May return errors for connection events (Connected, Disconnect, Finished)
    /// or other subscription-related issues.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// # let mut monitor = ctx.monitor("MY:PV").unwrap();
    /// # monitor.start();
    /// loop {
    ///     match monitor.pop() {
    ///         Ok(Some(value)) => println!("Update: {}", value),
    ///         Ok(None) => break, // Queue empty
    ///         Err(e) => {
    ///             println!("Event or error: {}", e);
    ///             break;
    ///         }
    ///     }
    /// }
    /// ```
    pub fn pop(&mut self) -> Result<Option<Value>> {
        match bridge::monitor_pop(self.inner.pin_mut()) {
            Ok(value_wrapper) => {
                if value_wrapper.is_null() {
                    Ok(None)
                } else {
                    Ok(Some(Value { inner: value_wrapper }))
                }
            },
            Err(e) => Err(e.into()),
        }
    }
    
    /// Check if the monitor is connected to the PV
    /// 
    /// # Returns
    /// 
    /// `true` if connected to the PV, `false` otherwise.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// # let mut monitor = ctx.monitor("MY:PV").unwrap();
    /// # monitor.start();
    /// if monitor.is_connected() {
    ///     println!("Connected to PV");
    /// } else {
    ///     println!("Not connected");
    /// }
    /// ```
    pub fn is_connected(&self) -> bool {
        bridge::monitor_is_connected(&self.inner)
    }
    
    /// Get the name of the PV being monitored
    /// 
    /// # Returns
    /// 
    /// The PV name as a string.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// # let monitor = ctx.monitor("MY:PV").unwrap();
    /// println!("Monitoring PV: {}", monitor.name());
    /// ```
    pub fn name(&self) -> String {
        bridge::monitor_get_name(&self.inner)
    }
}

/// MonitorBuilder provides a builder pattern for creating monitors with advanced configuration
/// 
/// This follows the PVXS MonitorBuilder pattern, allowing configuration of event masks
/// and callbacks before creating the subscription.
/// 
/// # Example
/// 
/// ```no_run
/// use epics_pvxs_sys::Context;
/// 
/// let mut ctx = Context::from_env()?;
/// let monitor = ctx.monitor_builder("MY:PV")
///     .mask_connected(false)
///     .mask_disconnected(true)
///     .exec()?;
/// # Ok::<(), epics_pvxs_sys::PvxsError>(())
/// ```
pub struct MonitorBuilder {
    inner: UniquePtr<bridge::MonitorBuilderWrapper>,
}

impl MonitorBuilder {
    /// Configure whether to include Connected events in the queue
    /// 
    /// # Arguments
    /// 
    /// * `mask` - true to include Connected events (default: true)
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let monitor = ctx.monitor_builder("MY:PV")
    ///     .mask_connected(false) // Don't include connection events
    ///     .exec()?;
    /// # Ok::<(), epics_pvxs_sys::PvxsError>(())
    /// ```
    pub fn mask_connected(mut self, mask: bool) -> Self {
        // Ignore errors for now - these should rarely fail
        let _ = bridge::monitor_builder_mask_connected(self.inner.pin_mut(), mask);
        self
    }
    
    /// Configure whether to include Disconnected events in the queue
    /// 
    /// # Arguments
    /// 
    /// * `mask` - true to include Disconnected events (default: false)
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let monitor = ctx.monitor_builder("MY:PV")
    ///     .mask_disconnected(true) // Include disconnection events
    ///     .exec()?;
    /// # Ok::<(), epics_pvxs_sys::PvxsError>(())
    /// ```
    pub fn mask_disconnected(mut self, mask: bool) -> Self {
        // Ignore errors for now - these should rarely fail
        let _ = bridge::monitor_builder_mask_disconnected(self.inner.pin_mut(), mask);
        self
    }
    
    /// Set an event callback function that will be invoked when the subscription queue becomes not-empty
    /// 
    /// This follows the PVXS pattern where the callback is invoked when events are available,
    /// not for each individual event. The callback should then use `pop()` to retrieve events.
    /// 
    /// # Arguments
    /// 
    /// * `callback` - Function to be called when events are available
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// 
    /// extern "C" fn my_callback() {
    ///     println!("Events available in subscription queue!");
    /// }
    /// 
    /// let monitor = ctx.monitor_builder("MY:PV")
    ///     .event(my_callback)
    ///     .exec()?;
    /// # Ok::<(), epics_pvxs_sys::PvxsError>(())
    /// ```
    pub fn event(mut self, callback: extern "C" fn()) -> Self {
        // Convert function pointer to usize for C++
        let callback_ptr = callback as usize;
        
        // Set the callback in C++
        let _ = bridge::monitor_builder_set_event_callback(self.inner.pin_mut(), callback_ptr);
        self
    }
    
    /// Execute and create the monitor subscription
    /// 
    /// Creates the actual monitor subscription with the configured settings.
    /// 
    /// # Returns
    /// 
    /// A `Monitor` instance ready for use.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let monitor = ctx.monitor_builder("MY:PV")
    ///     .mask_connected(false)
    ///     .exec()?;
    /// # Ok::<(), epics_pvxs_sys::PvxsError>(())
    /// ```
    pub fn exec(mut self) -> Result<Monitor> {
        let inner = bridge::monitor_builder_exec(self.inner.pin_mut())?;
        Ok(Monitor { inner })
    }
    
    /// Execute with an event callback (for future implementation)
    /// 
    /// This is a placeholder for future callback support. Currently behaves
    /// the same as `exec()`.
    /// 
    /// # Arguments
    /// 
    /// * `callback_id` - Identifier for the callback (currently unused)
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let monitor = ctx.monitor_builder("MY:PV")
    ///     .exec_with_callback(123)?;
    /// # Ok::<(), epics_pvxs_sys::PvxsError>(())
    /// ```
    pub fn exec_with_callback(mut self, callback_id: u64) -> Result<Monitor> {
        let inner = bridge::monitor_builder_exec_with_callback(self.inner.pin_mut(), callback_id)?;
        Ok(Monitor { inner })
    }
}

pub struct Rpc {
    inner: UniquePtr<bridge::RpcWrapper>,
}

impl Rpc {
    /// Add a string argument to the RPC call
    /// 
    /// # Arguments
    /// 
    /// * `name` - The argument name
    /// * `value` - The string value
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// # let mut rpc = ctx.rpc("my:service").unwrap();
    /// rpc.arg_string("filename", "/path/to/file.txt");
    /// ```
    pub fn arg_string(&mut self, name: &str, value: &str) -> Result<&mut Self> {
        bridge::rpc_arg_string(self.inner.pin_mut(), name.to_string(), value.to_string())?;
        Ok(self)
    }
    
    /// Add a double argument to the RPC call
    /// 
    /// # Arguments
    /// 
    /// * `name` - The argument name
    /// * `value` - The double value
    pub fn arg_double(&mut self, name: &str, value: f64) -> Result<&mut Self> {
        bridge::rpc_arg_double(self.inner.pin_mut(), name.to_string(), value)?;
        Ok(self)
    }
    
    /// Add an int32 argument to the RPC call
    /// 
    /// # Arguments
    /// 
    /// * `name` - The argument name
    /// * `value` - The int32 value
    pub fn arg_int32(&mut self, name: &str, value: i32) -> Result<&mut Self> {
        bridge::rpc_arg_int32(self.inner.pin_mut(), name.to_string(), value)?;
        Ok(self)
    }
    
    /// Add a boolean argument to the RPC call
    /// 
    /// # Arguments
    /// 
    /// * `name` - The argument name
    /// * `value` - The boolean value
    pub fn arg_bool(&mut self, name: &str, value: bool) -> Result<&mut Self> {
        bridge::rpc_arg_bool(self.inner.pin_mut(), name.to_string(), value)?;
        Ok(self)
    }
    
    /// Execute the RPC call synchronously
    /// 
    /// # Arguments
    /// 
    /// * `timeout` - Maximum time to wait in seconds
    /// 
    /// # Returns
    /// 
    /// Returns the result value from the server, or an error if the
    /// operation failed or timed out.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let mut rpc = ctx.rpc("calculator:add").unwrap();
    /// rpc.arg_double("a", 10.0);
    /// rpc.arg_double("b", 5.0);
    /// let result = rpc.execute(5.0).unwrap();
    /// let sum = result.get_field_double("result").unwrap();
    /// ```
    pub fn execute(mut self, timeout: f64) -> Result<Value> {
        let inner = bridge::rpc_execute_sync(self.inner.pin_mut(), timeout)?;
        Ok(Value { inner })
    }
}

/// Async implementation for RPC
#[cfg(feature = "async")]
impl Rpc {
    /// Execute the RPC call asynchronously
    /// 
    /// # Arguments
    /// 
    /// * `timeout` - Maximum time to wait in seconds
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # async fn example() -> Result<(), epics_pvxs_sys::PvxsError> {
    /// let mut ctx = Context::from_env()?;
    /// let mut rpc = ctx.rpc("my:service")?;
    /// rpc.arg_string("command", "process");
    /// let result = rpc.execute_async(5.0).await?;
    /// println!("Async RPC result: {}", result);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_async(mut self, timeout: f64) -> Result<Value> {
        use tokio::time::{sleep, Duration};
        
        let mut operation = bridge::rpc_execute_async(self.inner.pin_mut(), timeout)?;
        
        loop {
            if bridge::operation_is_done(&operation) {
                let result = bridge::operation_get_result(operation.pin_mut())?;
                return Ok(Value { inner: result });
            }
            
            // Yield control to the async runtime
            sleep(Duration::from_millis(10)).await;
        }
    }
}

/// A PVXS server for hosting process variables
/// 
/// The Server allows you to create and manage EPICS process variables,
/// making them available to clients over the network.
/// 
/// # Example
/// 
/// ```no_run
/// use epics_pvxs_sys::Server;
/// 
/// let mut server = Server::from_env()?; // Create server from environment
/// //let mut server = Server::create_isolated()?; // Create an isolated server
/// 
/// let mut pv = server.create_pv_double("test:pv", 42.0)?;
/// server.add_pv("test:pv", &mut pv)?;
/// 
/// server.start()?;
/// println!("Server running on port {}", server.tcp_port());
/// 
/// server.stop()?;
/// # Ok::<(), epics_pvxs_sys::PvxsError>(())
/// ```
pub struct Server {
    inner: UniquePtr<ServerWrapper>,
}

impl Server {
    /// Create a server from environment variables
    /// 
    /// Reads configuration from EPICS environment variables for network setup.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the server cannot be created or configured.
    pub fn from_env() -> Result<Self> {
        let inner = bridge::server_create_from_env()?;
        Ok(Self { inner })
    }
    
    /// Create an isolated server for testing
    /// 
    /// Creates a server that operates in isolation, using system-assigned ports
    /// and avoiding conflicts with other servers. Ideal for unit tests.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// use epics_pvxs_sys::Server;
    /// 
    /// let mut server = Server::create_isolated()?;
    /// server.start()?;
    /// println!("Isolated server started on TCP port {}", server.tcp_port());
    /// server.stop()?;
    /// # Ok::<(), epics_pvxs_sys::PvxsError>(())
    /// ```
    pub fn create_isolated() -> Result<Self> {
        let inner = bridge::server_create_isolated()?;
        Ok(Self { inner })
    }
    
    /// Start the server
    /// 
    /// Begins listening for client connections and serving PVs.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the server cannot be started (e.g., port conflicts).
    pub fn start(&mut self) -> Result<()> {
        bridge::server_start(self.inner.pin_mut())?;
        Ok(())
    }
    
    /// Stop the server
    /// 
    /// Stops listening for connections and shuts down the server.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the server cannot be stopped cleanly.
    pub fn stop(&mut self) -> Result<()> {
        bridge::server_stop(self.inner.pin_mut())?;
        Ok(())
    }
    
    /// Add a PV to the server
    /// 
    /// Makes a process variable available to clients under the given name.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The PV name that clients will use
    /// * `pv` - The SharedPV to add
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Server;
    /// # let mut server = Server::create_isolated().unwrap();
    /// let mut pv = server.create_pv_double("counter", 0.0)?;
    /// server.add_pv("test:counter", &mut pv)?;
    /// # Ok::<(), epics_pvxs_sys::PvxsError>(())
    /// ```
    pub fn add_pv(&mut self, name: &str, pv: &mut SharedPV) -> Result<()> {
        bridge::server_add_pv(self.inner.pin_mut(), name.to_string(), pv.inner.pin_mut())?;
        Ok(())
    }
    
    /// Remove a PV from the server
    /// 
    /// Removes the PV with the given name from the server.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the PV to remove
    pub fn remove_pv(&mut self, name: &str) -> Result<()> {
        bridge::server_remove_pv(self.inner.pin_mut(), name.to_string())?;
        Ok(())
    }
    
    /// Add a static source to the server
    /// 
    /// Static sources provide collections of PVs with a common configuration.
    /// 
    /// # Arguments
    /// 
    /// * `name` - Name for this source
    /// * `source` - The StaticSource to add
    /// * `order` - Priority order (lower numbers have higher priority)
    pub fn add_source(&mut self, name: &str, source: &mut StaticSource, order: i32) -> Result<()> {
        bridge::server_add_source(self.inner.pin_mut(), name.to_string(), source.inner.pin_mut(), order)?;
        Ok(())
    }
    
    /// Get the TCP port the server is listening on
    /// 
    /// Returns 0 if the server is not started.
    pub fn tcp_port(&self) -> u16 {
        bridge::server_get_tcp_port(&self.inner)
    }
    
    /// Get the UDP port the server is using
    /// 
    /// Returns 0 if the server is not started.
    pub fn udp_port(&self) -> u16 {
        bridge::server_get_udp_port(&self.inner)
    }
    
    /// Create a new mailbox SharedPV with a double value
    /// 
    /// Mailbox PVs allow both reading and writing by clients.
    /// 
    /// # Arguments
    /// 
    /// * `_name` - Name for debugging/logging (not the PV name)
    /// * `initial_value` - Initial value for the PV
    pub fn create_pv_double(&self, _name: &str, initial_value: f64) -> Result<SharedPV> {
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_double(initial_value)?;
        Ok(pv)
    }
    
    /// Create a new mailbox SharedPV with an int32 value
    /// 
    /// # Arguments
    /// 
    /// * `_name` - Name for debugging/logging (not the PV name)  
    /// * `initial_value` - Initial value for the PV
    pub fn create_pv_int32(&self, _name: &str, initial_value: i32) -> Result<SharedPV> {
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_int32(initial_value)?;
        Ok(pv)
    }
    
    /// Create a new mailbox SharedPV with a string value
    /// 
    /// # Arguments
    /// 
    /// * `_name` - Name for debugging/logging (not the PV name)
    /// * `initial_value` - Initial value for the PV
    pub fn create_pv_string(&self, _name: &str, initial_value: &str) -> Result<SharedPV> {
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_string(initial_value)?;
        Ok(pv)
    }
    
    /// Create a new readonly SharedPV with a double value
    /// 
    /// Readonly PVs only allow reading by clients.
    /// 
    /// # Arguments
    /// 
    /// * `_name` - Name for debugging/logging (not the PV name)
    /// * `initial_value` - Initial value for the PV
    pub fn create_readonly_pv_double(&self, _name: &str, initial_value: f64) -> Result<SharedPV> {
        let mut pv = SharedPV::create_readonly()?;
        pv.open_double(initial_value)?;
        Ok(pv)
    }
}

/// A shared process variable that can be hosted by a server
/// 
/// SharedPVs represent individual process variables with typed values
/// that can be accessed by EPICS clients.
/// 
/// # Example
/// 
/// ```no_run
/// use epics_pvxs_sys::SharedPV;
/// 
/// let mut pv = SharedPV::create_mailbox()?;
/// pv.open_double(42.5)?;
/// 
/// // Update the value
/// pv.post_double(99.9)?;
/// 
/// // Get current value
/// let value = pv.fetch()?;
/// println!("Current value: {}", value);
/// # Ok::<(), epics_pvxs_sys::PvxsError>(())
/// ```
pub struct SharedPV {
    inner: UniquePtr<SharedPVWrapper>,
}

impl SharedPV {
    /// Create a mailbox SharedPV
    /// 
    /// Mailbox PVs support both read and write operations by clients.
    pub fn create_mailbox() -> Result<Self> {
        let inner = bridge::shared_pv_create_mailbox()?;
        Ok(Self { inner })
    }
    
    /// Create a readonly SharedPV
    /// 
    /// Readonly PVs only support read operations by clients.
    pub fn create_readonly() -> Result<Self> {
        let inner = bridge::shared_pv_create_readonly()?;
        Ok(Self { inner })
    }
    
    /// Open the PV with a double value
    /// 
    /// # Arguments
    /// 
    /// * `initial_value` - The initial value for the PV
    pub fn open_double(&mut self, initial_value: f64) -> Result<()> {
        bridge::shared_pv_open_double(self.inner.pin_mut(), initial_value)?;
        Ok(())
    }
    
    /// Open the PV with an int32 value
    /// 
    /// # Arguments
    /// 
    /// * `initial_value` - The initial value for the PV
    pub fn open_int32(&mut self, initial_value: i32) -> Result<()> {
        bridge::shared_pv_open_int32(self.inner.pin_mut(), initial_value)?;
        Ok(())
    }
    
    /// Open the PV with a string value
    /// 
    /// # Arguments
    /// 
    /// * `initial_value` - The initial value for the PV
    pub fn open_string(&mut self, initial_value: &str) -> Result<()> {
        bridge::shared_pv_open_string(self.inner.pin_mut(), initial_value.to_string())?;
        Ok(())
    }
    
    /// Check if the PV is open
    pub fn is_open(&self) -> bool {
        bridge::shared_pv_is_open(&self.inner)
    }
    
    /// Close the PV
    pub fn close(&mut self) -> Result<()> {
        bridge::shared_pv_close(self.inner.pin_mut())?;
        Ok(())
    }
    
    /// Post a new double value to the PV
    /// 
    /// This updates the PV value and notifies connected clients.
    /// 
    /// # Arguments
    /// 
    /// * `value` - The new value to post
    pub fn post_double(&mut self, value: f64) -> Result<()> {
        bridge::shared_pv_post_double(self.inner.pin_mut(), value)?;
        Ok(())
    }
    
    /// Post a new int32 value to the PV
    /// 
    /// # Arguments
    /// 
    /// * `value` - The new value to post
    pub fn post_int32(&mut self, value: i32) -> Result<()> {
        bridge::shared_pv_post_int32(self.inner.pin_mut(), value)?;
        Ok(())
    }
    
    /// Post a new string value to the PV
    /// 
    /// # Arguments
    /// 
    /// * `value` - The new value to post
    pub fn post_string(&mut self, value: &str) -> Result<()> {
        bridge::shared_pv_post_string(self.inner.pin_mut(), value.to_string())?;
        Ok(())
    }
    
    /// Post a new double value to the PV with alarm information
    /// 
    /// This updates the PV value and alarm fields, then notifies connected clients.
    /// 
    /// # Arguments
    /// 
    /// * `value` - The new value to post
    /// * `severity` - Alarm severity (0=NO_ALARM, 1=MINOR, 2=MAJOR, 3=INVALID)
    /// * `status` - Alarm status code (0=NO_ALARM, various status codes)
    /// * `message` - Alarm message string
    pub fn post_double_with_alarm(&mut self, value: f64, severity: i32, status: i32, message: &str) -> Result<()> {
        bridge::shared_pv_post_double_with_alarm(self.inner.pin_mut(), value, severity, status, message.to_string())?;
        Ok(())
    }
    
    /// Post a new int32 value to the PV with alarm information
    /// 
    /// # Arguments
    /// 
    /// * `value` - The new value to post
    /// * `severity` - Alarm severity (0=NO_ALARM, 1=MINOR, 2=MAJOR, 3=INVALID)
    /// * `status` - Alarm status code (0=NO_ALARM, various status codes)
    /// * `message` - Alarm message string
    pub fn post_int32_with_alarm(&mut self, value: i32, severity: i32, status: i32, message: &str) -> Result<()> {
        bridge::shared_pv_post_int32_with_alarm(self.inner.pin_mut(), value, severity, status, message.to_string())?;
        Ok(())
    }
    
    /// Post a new string value to the PV with alarm information
    /// 
    /// # Arguments
    /// 
    /// * `value` - The new value to post
    /// * `severity` - Alarm severity (0=NO_ALARM, 1=MINOR, 2=MAJOR, 3=INVALID)
    /// * `status` - Alarm status code (0=NO_ALARM, various status codes)
    /// * `message` - Alarm message string
    pub fn post_string_with_alarm(&mut self, value: &str, severity: i32, status: i32, message: &str) -> Result<()> {
        bridge::shared_pv_post_string_with_alarm(self.inner.pin_mut(), value.to_string(), severity, status, message.to_string())?;
        Ok(())
    }
    
    /// Fetch the current value of the PV
    /// 
    /// Returns the current value as a Value that can be inspected.
    pub fn fetch(&self) -> Result<Value> {
        let inner = bridge::shared_pv_fetch(&self.inner)?;
        Ok(Value { inner })
    }
}

/// A static source for organizing collections of PVs
/// 
/// StaticSource allows grouping related PVs together with common
/// configuration and management.
/// 
/// # Example
/// 
/// ```no_run
/// use epics_pvxs_sys::{StaticSource, SharedPV};
/// 
/// let mut source = StaticSource::create()?;
/// 
/// let mut temp_pv = SharedPV::create_readonly()?;
/// temp_pv.open_double(23.5)?;
/// 
/// source.add_pv("temperature", &mut temp_pv)?;
/// 
/// // Add source to server with priority 0
/// // server.add_source("sensors", &mut source, 0)?;
/// # Ok::<(), epics_pvxs_sys::PvxsError>(())
/// ```
pub struct StaticSource {
    inner: UniquePtr<StaticSourceWrapper>,
}

impl StaticSource {
    /// Create a new StaticSource
    pub fn create() -> Result<Self> {
        let inner = bridge::static_source_create()?;
        Ok(Self { inner })
    }
    
    /// Add a PV to this source
    /// 
    /// # Arguments
    /// 
    /// * `name` - The PV name within this source
    /// * `pv` - The SharedPV to add
    pub fn add_pv(&mut self, name: &str, pv: &mut SharedPV) -> Result<()> {
        bridge::static_source_add_pv(self.inner.pin_mut(), name.to_string(), pv.inner.pin_mut())?;
        Ok(())
    }
    
    /// Remove a PV from this source
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the PV to remove
    pub fn remove_pv(&mut self, name: &str) -> Result<()> {
        bridge::static_source_remove_pv(self.inner.pin_mut(), name.to_string())?;
        Ok(())
    }
    
    /// Close all PVs in this source
    pub fn close_all(&mut self) -> Result<()> {
        bridge::static_source_close_all(self.inner.pin_mut())?;
        Ok(())
    }
}
