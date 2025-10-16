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

pub use bridge::{ContextWrapper, ValueWrapper, RpcWrapper, MonitorWrapper};

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
        let inner = bridge::context_rpc_create(self.inner.pin_mut(), pv_name)?;
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
        let inner = bridge::context_monitor_create(self.inner.pin_mut(), pv_name)?;
        Ok(Monitor { inner })
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
        bridge::rpc_arg_string(self.inner.pin_mut(), name, value)?;
        Ok(self)
    }
    
    /// Add a double argument to the RPC call
    /// 
    /// # Arguments
    /// 
    /// * `name` - The argument name
    /// * `value` - The double value
    pub fn arg_double(&mut self, name: &str, value: f64) -> Result<&mut Self> {
        bridge::rpc_arg_double(self.inner.pin_mut(), name, value)?;
        Ok(self)
    }
    
    /// Add an int32 argument to the RPC call
    /// 
    /// # Arguments
    /// 
    /// * `name` - The argument name
    /// * `value` - The int32 value
    pub fn arg_int32(&mut self, name: &str, value: i32) -> Result<&mut Self> {
        bridge::rpc_arg_int32(self.inner.pin_mut(), name, value)?;
        Ok(self)
    }
    
    /// Add a boolean argument to the RPC call
    /// 
    /// # Arguments
    /// 
    /// * `name` - The argument name
    /// * `value` - The boolean value
    pub fn arg_bool(&mut self, name: &str, value: bool) -> Result<&mut Self> {
        bridge::rpc_arg_bool(self.inner.pin_mut(), name, value)?;
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
