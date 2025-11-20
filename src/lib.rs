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
    ///     .connection_events(true)      // Include connection events
    ///     .disconnection_events(true)   // Include disconnection events
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
    /// Enable or disable connection events in the monitor queue
    /// 
    /// This is the user-friendly API - think in terms of what you want to enable.
    /// 
    /// # Arguments
    /// 
    /// * `enable` - true to include connection events, false to exclude them (default: true)
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let monitor = ctx.monitor_builder("MY:PV")
    ///     .connection_events(true) // Include connection events
    ///     .exec()?;
    /// # Ok::<(), epics_pvxs_sys::PvxsError>(())
    /// ```
    pub fn connection_events(mut self, enable: bool) -> Self {
        // Invert the logic: enable=true means mask=false (don't mask out)
        let _ = bridge::monitor_builder_mask_connected(self.inner.pin_mut(), !enable);
        self
    }
    
    /// Enable or disable disconnection events in the monitor queue
    /// 
    /// This is the user-friendly API - think in terms of what you want to enable.
    /// 
    /// # Arguments
    /// 
    /// * `enable` - true to include disconnection events, false to exclude them (default: false)
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let monitor = ctx.monitor_builder("MY:PV")
    ///     .disconnection_events(true) // Include disconnection events
    ///     .exec()?;
    /// # Ok::<(), epics_pvxs_sys::PvxsError>(())
    /// ```
    pub fn disconnection_events(mut self, enable: bool) -> Self {
        // Invert the logic: enable=true means mask=false (don't mask out)
        let _ = bridge::monitor_builder_mask_disconnected(self.inner.pin_mut(), !enable);
        self
    }
    
    /// Configure whether to mask Connected events in the queue (low-level API)
    /// 
    /// **Note:** This is the low-level API that directly exposes PVXS semantics.
    /// Consider using `connection_events()` instead for more intuitive API.
    /// 
    /// # Arguments
    /// 
    /// * `mask` - true to mask out (exclude) connection events, false to include them
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let monitor = ctx.monitor_builder("MY:PV")
    ///     .mask_connected(false) // false = don't mask = include events
    ///     .exec()?;
    /// # Ok::<(), epics_pvxs_sys::PvxsError>(())
    /// ```
    pub fn mask_connected(mut self, mask: bool) -> Self {
        let _ = bridge::monitor_builder_mask_connected(self.inner.pin_mut(), mask);
        self
    }
    
    /// Configure whether to mask Disconnected events in the queue (low-level API)
    /// 
    /// **Note:** This is the low-level API that directly exposes PVXS semantics.
    /// Consider using `disconnection_events()` instead for more intuitive API.
    /// 
    /// # Arguments
    /// 
    /// * `mask` - true to mask out (exclude) disconnection events, false to include them
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let monitor = ctx.monitor_builder("MY:PV")
    ///     .mask_disconnected(false) // false = don't mask = include events
    ///     .exec()?;
    /// # Ok::<(), epics_pvxs_sys::PvxsError>(())
    /// ```
    pub fn mask_disconnected(mut self, mask: bool) -> Self {
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
/// use epics_pvxs_sys::{Server, NTScalarMetadataBuilder};
/// 
/// let mut server = Server::from_env()?; // Create server from environment
/// //let mut server = Server::create_isolated()?; // Create an isolated server
/// 
/// // Create and add PV in one step
/// server.create_pv_double("test:pv", 42.0, NTScalarMetadataBuilder::new())?;
/// 
/// server.start()?;
/// println!(\"Server running on port {}\", server.tcp_port());
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
    
    /// Add a PV to the server (internal use only)
    /// 
    /// Makes a process variable available to clients under the given name.
    /// This is now internal - use create_pv_* methods instead.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The PV name that clients will use
    /// * `pv` - The SharedPV to add
    pub(crate) fn add_pv(&mut self, name: &str, pv: &mut SharedPV) -> Result<()> {
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
    
    /// Create and add a new mailbox SharedPV with a double value and metadata
    /// 
    /// Mailbox PVs allow both reading and writing by clients.
    /// The PV is automatically added to the server with the given name.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The PV name that clients will use
    /// * `initial_value` - Initial value for the PV
    /// * `metadata` - Metadata for the scalar PV
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::{Server, NTScalarMetadataBuilder};
    /// # let mut server = Server::create_isolated().unwrap();
    /// server.create_pv_double("test:double", 42.5, NTScalarMetadataBuilder::new())?;
    /// # Ok::<(), epics_pvxs_sys::PvxsError>(())
    /// ```
    pub fn create_pv_double(&mut self, name: &str, initial_value: f64, metadata: NTScalarMetadataBuilder) -> Result<()> {
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_double(initial_value, metadata)?;
        self.add_pv(name, &mut pv)?;
        Ok(())
    }

    /// Create and add a new mailbox SharedPV with a double array value and metadata
    /// 
    /// Create should fail if array is empty.
    /// The PV is automatically added to the server with the given name.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The PV name that clients will use
    /// * `initial_value` - Initial array value for the PV
    /// * `metadata` - Metadata for the scalar array PV
    pub fn create_pv_double_array(&mut self, name: &str, initial_value: Vec<f64>, metadata: NTScalarMetadataBuilder) -> Result<()> {
        if initial_value.is_empty() {
            return Err(PvxsError::new("Initial double array cannot be empty"));
        }
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_double_array(initial_value, metadata)?;
        self.add_pv(name, &mut pv)?;
        Ok(())
    }
    
    /// Create and add a new mailbox SharedPV with an int32 value and metadata
    /// 
    /// The PV is automatically added to the server with the given name.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The PV name that clients will use
    /// * `initial_value` - Initial value for the PV
    /// * `metadata` - Metadata for the scalar PV
    pub fn create_pv_int32(&mut self, name: &str, initial_value: i32, metadata: NTScalarMetadataBuilder) -> Result<()> {
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_int32(initial_value, metadata)?;
        self.add_pv(name, &mut pv)?;
        Ok(())
    }
    
    /// Create and add a new mailbox SharedPV with an int32 array value and metadata
    /// 
    /// Create should fail if array is empty.
    /// The PV is automatically added to the server with the given name.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The PV name that clients will use
    /// * `initial_value` - Initial array value for the PV
    /// * `metadata` - Metadata for the array PV
    pub fn create_pv_int32_array(&mut self, name: &str, initial_value: Vec<i32>, metadata: NTScalarMetadataBuilder) -> Result<()> {
        if initial_value.is_empty() {
            return Err(PvxsError::new("Initial int32 array cannot be empty"));
        }
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_int32_array(initial_value, metadata)?;
        self.add_pv(name, &mut pv)?;
        Ok(())
    }
    
    /// Create and add a new mailbox SharedPV with a string value and metadata
    /// 
    /// The PV is automatically added to the server with the given name.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The PV name that clients will use
    /// * `initial_value` - Initial value for the PV
    /// * `metadata` - Metadata for the string PV
    pub fn create_pv_string(&mut self, name: &str, initial_value: &str, metadata: NTScalarMetadataBuilder) -> Result<()> {
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_string(initial_value, metadata)?;
        self.add_pv(name, &mut pv)?;
        Ok(())
    }
    
    /// Create and add a new mailbox SharedPV with a string array value and metadata
    /// 
    /// Create should fail if array is empty.
    /// The PV is automatically added to the server with the given name.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The PV name that clients will use
    /// * `initial_value` - Initial array value for the PV
    /// * `metadata` - Metadata for the string array PV
    pub fn create_pv_string_array(&mut self, name: &str, initial_value: Vec<String>, metadata: NTScalarMetadataBuilder) -> Result<()> {
        if initial_value.is_empty() {
            return Err(PvxsError::new("Initial string array cannot be empty"));
        }
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_string_array(initial_value, metadata)?;
        self.add_pv(name, &mut pv)?;
        Ok(())
    }

    /// Create and add a new mailbox SharedPV with an enum value and metadata
    /// 
    /// The PV is automatically added to the server with the given name.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The PV name that clients will use
    /// * `choices` - List of string choices for the enum
    /// * `selected_index` - Initial selected index (0-based)
    /// * `metadata` - Metadata for the enum PV
    pub fn create_pv_enum(&mut self, name: &str, choices: Vec<&str>, selected_index: i16, metadata: NTEnumMetadataBuilder) -> Result<()> {
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_enum(choices, selected_index, metadata)?;
        self.add_pv(name, &mut pv)?;
        Ok(())
    }
    
    /// Create and add a new readonly SharedPV with a double value and metadata
    /// 
    /// Readonly PVs only allow reading by clients.
    /// The PV is automatically added to the server with the given name.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The PV name that clients will use
    /// * `initial_value` - Initial value for the PV
    /// * `metadata` - Metadata for the scalar PV
    pub fn create_readonly_pv_double(&mut self, name: &str, initial_value: f64, metadata: NTScalarMetadataBuilder) -> Result<()> {
        let mut pv = SharedPV::create_readonly()?;
        pv.open_double(initial_value, metadata)?;
        self.add_pv(name, &mut pv)?;
        Ok(())
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
    
    /// Open the PV with a double value and metadata
    /// 
    /// # Arguments
    /// 
    /// * `initial_value` - The initial value for the PV
    /// * `metadata` - Metadata builder for the scalar PV
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use epics_pvxs_sys::{SharedPV, NTScalarMetadataBuilder, DisplayMetadata};
    /// let mut pv = SharedPV::create_mailbox()?;
    /// 
    /// let metadata = NTScalarMetadataBuilder::new()
    ///     .alarm(0, 0, "OK")
    ///     .display(DisplayMetadata {
    ///         limit_low: 0,
    ///         limit_high: 100,
    ///         description: "Temperature".to_string(),
    ///         units: "°C".to_string(),
    ///         precision: 2,
    ///     })
    ///     .with_form(true);
    /// 
    /// pv.open_double(25.5, metadata)?;
    /// # Ok::<(), epics_pvxs_sys::PvxsError>(())
    /// ```
    pub(crate) fn open_double(&mut self, initial_value: f64, metadata: NTScalarMetadataBuilder) -> Result<()> {
        let meta = metadata.build()?;
        bridge::shared_pv_open_double(self.inner.pin_mut(), initial_value, &meta)?;
        Ok(())
    }

    /// Open the PV with a double array value and metadata
    /// 
    /// # Arguments
    /// 
    /// * `initial_value` - The initial array value for the PV
    /// * `metadata` - Metadata builder for the scalar array PV
    pub(crate) fn open_double_array(&mut self, initial_value: Vec<f64>, metadata: NTScalarMetadataBuilder) -> Result<()> {
        let meta = metadata.build()?;
        bridge::shared_pv_open_double_array(self.inner.pin_mut(), initial_value, &meta)?;
        Ok(())
    }

    /// Open the PV with an enum value and metadata
    /// 
    /// # Arguments
    /// 
    /// * `choices` - List of string choices for the enum
    /// * `selected_index` - Initial selected index (0-based)
    /// * `metadata` - Metadata builder for the enum PV
    pub(crate) fn open_enum(&mut self, choices: Vec<&str>, selected_index: i16, metadata: NTEnumMetadataBuilder) -> Result<()> {
        let meta = metadata.build()?;
        let choices_vec: Vec<String> = choices.iter().map(|s| s.to_string()).collect();
        bridge::shared_pv_open_enum(self.inner.pin_mut(), choices_vec, selected_index, &meta)?;
        Ok(())
    }
    
    /// Open the PV with an int32 value and metadata
    /// 
    /// # Arguments
    /// 
    /// * `initial_value` - The initial value for the PV
    /// * `metadata` - Metadata builder for the int32 PV
    pub(crate) fn open_int32(&mut self, initial_value: i32, metadata: NTScalarMetadataBuilder) -> Result<()> {
        let meta = metadata.build()?;
        bridge::shared_pv_open_int32(self.inner.pin_mut(), initial_value, &meta)?;
        Ok(())
    }
    
    /// Open the PV with an int32 array value and metadata
    /// 
    /// # Arguments
    /// 
    /// * `initial_value` - The initial array value for the PV
    /// * `metadata` - Metadata builder for the int32 array PV
    pub(crate) fn open_int32_array(&mut self, initial_value: Vec<i32>, metadata: NTScalarMetadataBuilder) -> Result<()> {
        let meta = metadata.build()?;
        bridge::shared_pv_open_int32_array(self.inner.pin_mut(), initial_value, &meta)?;
        Ok(())
    }
    
    /// Open the PV with a string value and metadata
    /// 
    /// # Arguments
    /// 
    /// * `initial_value` - The initial value for the PV
    /// * `metadata` - Metadata builder for the string PV
    pub(crate) fn open_string(&mut self, initial_value: &str, metadata: NTScalarMetadataBuilder) -> Result<()> {
        let meta = metadata.build()?;
        bridge::shared_pv_open_string(self.inner.pin_mut(), initial_value.to_string(), &meta)?;
        Ok(())
    }
    
    /// Open the PV with a string array value and metadata
    /// 
    /// # Arguments
    /// 
    /// * `initial_value` - The initial array value for the PV
    /// * `metadata` - Metadata builder for the string array PV
    pub(crate) fn open_string_array(&mut self, initial_value: Vec<String>, metadata: NTScalarMetadataBuilder) -> Result<()> {
        let meta = metadata.build()?;
        bridge::shared_pv_open_string_array(self.inner.pin_mut(), initial_value, &meta)?;
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
    /// If the PV is a double array, this will just replace the value at position 0.
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
    /// This updates the PV value and notifies connected clients.
    /// If the PV is an int32 array, this will just replace the value at position 0.
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
    
    /// Post a new enum value to the PV
    /// 
    /// Updates the enum index (value.index field) and notifies connected clients.
    /// 
    /// # Arguments
    /// 
    /// * `value` - The enum index to post (should be valid for the choices array)
    pub fn post_enum(&mut self, value: i16) -> Result<()> {
        bridge::shared_pv_post_enum(self.inner.pin_mut(), value)?;
        Ok(())
    }
    
    /// Post a new double array to the PV
    /// 
    /// Updates the array value and notifies connected clients.
    /// 
    /// # Arguments
    /// 
    /// * `value` - The new array to post
    pub fn post_double_array(&mut self, value: &[f64]) -> Result<()> {
        if value.is_empty() {
            return Err(PvxsError::new("Cannot post empty double array"));
        }
        bridge::shared_pv_post_double_array(self.inner.pin_mut(), value.to_vec())?;
        Ok(())
    }
    
    /// Post a new int32 array to the PV
    /// 
    /// Updates the array value and notifies connected clients.
    /// 
    /// # Arguments
    /// 
    /// * `value` - The new array to post
    pub fn post_int32_array(&mut self, value: &[i32]) -> Result<()> {
        if value.is_empty() {
            return Err(PvxsError::new("Cannot post empty int32 array"));
        }
        bridge::shared_pv_post_int32_array(self.inner.pin_mut(), value.to_vec())?;
        Ok(())
    }
    
    /// Post a new string array to the PV
    /// 
    /// Updates the array value and notifies connected clients.
    /// 
    /// # Arguments
    /// 
    /// * `value` - The new array to post
    pub fn post_string_array(&mut self, value: &[String]) -> Result<()> {
        if value.is_empty() {
            return Err(PvxsError::new("Cannot post empty string array"));
        }
        bridge::shared_pv_post_string_array(self.inner.pin_mut(), value.to_vec())?;
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

// ============================================================================
// NTScalar Metadata Support with C++ std::optional
// ============================================================================

/// Builder for creating NTScalar metadata with optional fields
/// 
/// This provides a clean, type-safe API for configuring PV metadata.
/// The metadata is constructed using C++ builder functions that support std::optional.
/// 
/// ```text
/// epics:nt/NTScalar:1.0
/// double value
/// alarm_t alarm
///     int severity
///     int status
///     string message
/// structure timeStamp
///     long secondsPastEpoch
///     int nanoseconds
///     int userTag
/// structure display
///     double limitLow
///     double limitHigh
///     string description
///     string units
///     int precision
///     enum_t form
///         int index
///         string[] choices
/// control_t control
///     double limitLow
///     double limitHigh
///     double minStep
/// valueAlarm_t valueAlarm
///     boolean active
///     double lowAlarmLimit
///     double lowWarningLimit
///     double highWarningLimit
///     double highAlarmLimit
///     int lowAlarmSeverity
///     int lowWarningSeverity
///     int highWarningSeverity
///     int highAlarmSeverity
///     byte hysteresis
/// ```
pub struct NTScalarMetadataBuilder {
    alarm_severity: i32,
    alarm_status: i32,
    alarm_message: String,
    timestamp_seconds: i64,
    timestamp_nanos: i32,
    timestamp_user_tag: i32,
    display: Option<DisplayMetadata>,
    control: Option<ControlMetadata>,
    value_alarm: Option<ValueAlarmMetadata>,
    with_form: bool,
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

/// Value alarm metadata for NTScalar
#[derive(Clone, Debug, Default)]
pub struct ValueAlarmMetadata {
    pub active: bool,
    pub low_alarm_limit: f64,
    pub low_warning_limit: f64,
    pub high_warning_limit: f64,
    pub high_alarm_limit: f64,
    pub low_alarm_severity: i32,
    pub low_warning_severity: i32,
    pub high_warning_severity: i32,
    pub high_alarm_severity: i32,
    pub hysteresis: u8,
}

impl NTScalarMetadataBuilder {
    /// Create a new metadata builder with default values
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        
        Self {
            alarm_severity: 0,
            alarm_status: 0,
            alarm_message: String::new(),
            timestamp_seconds: now.as_secs() as i64,
            timestamp_nanos: now.subsec_nanos() as i32,
            timestamp_user_tag: 0,
            display: None,
            control: None,
            value_alarm: None,
            with_form: false,
        }
    }
    
    /// Set alarm information
    pub fn alarm(mut self, severity: i32, status: i32, message: impl Into<String>) -> Self {
        self.alarm_severity = severity;
        self.alarm_status = status;
        self.alarm_message = message.into();
        self
    }
    
    /// Set timestamp (defaults to current time)
    pub fn timestamp(mut self, seconds: i64, nanos: i32, user_tag: i32) -> Self {
        self.timestamp_seconds = seconds;
        self.timestamp_nanos = nanos;
        self.timestamp_user_tag = user_tag;
        self
    }
    
    /// Add display metadata
    pub fn display(mut self, meta: DisplayMetadata) -> Self {
        self.display = Some(meta);
        self
    }
    
    /// Add control metadata
    pub fn control(mut self, meta: ControlMetadata) -> Self {
        self.control = Some(meta);
        self
    }
    
    /// Add value alarm metadata
    pub fn value_alarm(mut self, meta: ValueAlarmMetadata) -> Self {
        self.value_alarm = Some(meta);
        self
    }
    
    /// Enable form field (precision for numeric displays)
    pub fn with_form(mut self, enable: bool) -> Self {
        self.with_form = enable;
        self
    }
    
    /// Build the metadata using C++ builder functions with std::optional support
    fn build(self) -> Result<cxx::UniquePtr<bridge::NTScalarMetadata>> {
        // Create alarm and timestamp (always required)
        let alarm = bridge::create_alarm(self.alarm_severity, self.alarm_status, self.alarm_message);
        let time_stamp = bridge::create_time(self.timestamp_seconds, self.timestamp_nanos, self.timestamp_user_tag);
        
        // Build metadata based on which optional fields are present
        let metadata = match (&self.display, &self.control, &self.value_alarm) {
            (None, None, None) => {
                bridge::create_metadata_no_optional(&alarm, &time_stamp, self.with_form)
            }
            (Some(d), None, None) => {
                let display = bridge::create_display(d.limit_low, d.limit_high, d.description.clone(), d.units.clone(), d.precision);
                bridge::create_metadata_with_display(&alarm, &time_stamp, &display, self.with_form)
            }
            (None, Some(c), None) => {
                let control = bridge::create_control(c.limit_low, c.limit_high, c.min_step);
                bridge::create_metadata_with_control(&alarm, &time_stamp, &control, self.with_form)
            }
            (None, None, Some(v)) => {
                let value_alarm = bridge::create_value_alarm(
                    v.active, v.low_alarm_limit, v.low_warning_limit,
                    v.high_warning_limit, v.high_alarm_limit,
                    v.low_alarm_severity, v.low_warning_severity,
                    v.high_warning_severity, v.high_alarm_severity, v.hysteresis
                );
                bridge::create_metadata_with_value_alarm(&alarm, &time_stamp, &value_alarm, self.with_form)
            }
            (Some(d), Some(c), None) => {
                let display = bridge::create_display(d.limit_low, d.limit_high, d.description.clone(), d.units.clone(), d.precision);
                let control = bridge::create_control(c.limit_low, c.limit_high, c.min_step);
                bridge::create_metadata_with_display_control(&alarm, &time_stamp, &display, &control, self.with_form)
            }
            (Some(d), None, Some(v)) => {
                let display = bridge::create_display(d.limit_low, d.limit_high, d.description.clone(), d.units.clone(), d.precision);
                let value_alarm = bridge::create_value_alarm(
                    v.active, v.low_alarm_limit, v.low_warning_limit,
                    v.high_warning_limit, v.high_alarm_limit,
                    v.low_alarm_severity, v.low_warning_severity,
                    v.high_warning_severity, v.high_alarm_severity, v.hysteresis
                );
                bridge::create_metadata_with_display_value_alarm(&alarm, &time_stamp, &display, &value_alarm, self.with_form)
            }
            (None, Some(c), Some(v)) => {
                let control = bridge::create_control(c.limit_low, c.limit_high, c.min_step);
                let value_alarm = bridge::create_value_alarm(
                    v.active, v.low_alarm_limit, v.low_warning_limit,
                    v.high_warning_limit, v.high_alarm_limit,
                    v.low_alarm_severity, v.low_warning_severity,
                    v.high_warning_severity, v.high_alarm_severity, v.hysteresis
                );
                bridge::create_metadata_with_control_value_alarm(&alarm, &time_stamp, &control, &value_alarm, self.with_form)
            }
            (Some(d), Some(c), Some(v)) => {
                let display = bridge::create_display(d.limit_low, d.limit_high, d.description.clone(), d.units.clone(), d.precision);
                let control = bridge::create_control(c.limit_low, c.limit_high, c.min_step);
                let value_alarm = bridge::create_value_alarm(
                    v.active, v.low_alarm_limit, v.low_warning_limit,
                    v.high_warning_limit, v.high_alarm_limit,
                    v.low_alarm_severity, v.low_warning_severity,
                    v.high_warning_severity, v.high_alarm_severity, v.hysteresis
                );
                bridge::create_metadata_full(&alarm, &time_stamp, &display, &control, &value_alarm, self.with_form)
            }
        };
        
        Ok(metadata)
    }
}

impl Default for NTScalarMetadataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// NTEnum Metadata support
// ============================================================================
/// Builder for creating NTEnum metadata
/// 
/// This provides a clean, type-safe API for configuring enum PV metadata.
/// The metadata is constructed using C++ builder functions.
/// 
/// ```text
/// epics:nt/NTEnum:1.0
/// enum_t value
///     int index
///     string[] choices
/// alarm_t alarm
///     int severity
///     int status
///     string message
/// structure timeStamp
///     long secondsPastEpoch
///     int nanoseconds
///     int userTag
/// ```
pub struct NTEnumMetadataBuilder {
    alarm_severity: i32,
    alarm_status: i32,
    alarm_message: String,
    timestamp_seconds: i64,
    timestamp_nanos: i32,
    timestamp_user_tag: i32,
}

impl NTEnumMetadataBuilder {
    /// Create a new metadata builder with default values
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        
        Self {
            alarm_severity: 0,
            alarm_status: 0,
            alarm_message: String::new(),
            timestamp_seconds: now.as_secs() as i64,
            timestamp_nanos: now.subsec_nanos() as i32,
            timestamp_user_tag: 0,
        }
    }
    
    /// Set alarm information
    pub fn alarm(mut self, severity: i32, status: i32, message: impl Into<String>) -> Self {
        self.alarm_severity = severity;
        self.alarm_status = status;
        self.alarm_message = message.into();
        self
    }
    
    /// Set timestamp (defaults to current time)
    pub fn timestamp(mut self, seconds: i64, nanos: i32, user_tag: i32) -> Self {
        self.timestamp_seconds = seconds;
        self.timestamp_nanos = nanos;
        self.timestamp_user_tag = user_tag;
        self
    }

    fn build(self) -> Result<cxx::UniquePtr<bridge::NTEnumMetadata>> {
        let alarm = bridge::create_alarm(self.alarm_severity, self.alarm_status, self.alarm_message);
        let time_stamp = bridge::create_time(self.timestamp_seconds, self.timestamp_nanos, self.timestamp_user_tag);
        let metadata = bridge::create_enum_metadata(&alarm, &time_stamp);
        Ok(metadata)
    }
}

impl Default for NTEnumMetadataBuilder {
    fn default() -> Self {
        Self::new()
    }
}
