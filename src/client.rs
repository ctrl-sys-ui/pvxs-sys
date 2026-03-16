// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use cxx::UniquePtr;
use std::fmt;

use crate::{bridge, Result, Value};

/// Monitor event types that can be returned by pop()
#[derive(Debug, Clone, PartialEq)]
pub enum MonitorEvent {
    /// Connection event (when maskConnected(true) is set)
    Connected(String),
    /// Disconnection event (when maskDisconnected(true) is set)
    Disconnected(String),
    /// Finished event (when maskDisconnected(true) is set).
    /// Subscription has completed normally and no more events will ever be received.
    Finished(String),
    /// Remote error event from server
    RemoteError(String),
    /// Standard client side error. Catchs std::exception for client side failures.
    ClientError(String),
}

impl fmt::Display for MonitorEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MonitorEvent::Connected(msg) => write!(f, "Monitor connected: {}", msg),
            MonitorEvent::Disconnected(msg) => write!(f, "Monitor disconnected: {}", msg),
            MonitorEvent::Finished(msg) => write!(f, "Monitor finished: {}", msg),
            MonitorEvent::RemoteError(msg) => write!(f, "Monitor remote error: {}", msg),
            MonitorEvent::ClientError(msg) => write!(f, "Monitor client error: {}", msg),
        }
    }
}

impl std::error::Error for MonitorEvent {}

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
    inner: UniquePtr<bridge::ContextWrapper>,
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
    /// use pvxs_sys::Context;
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
    /// # use pvxs_sys::Context;
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
    /// # use pvxs_sys::Context;
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
    /// # use pvxs_sys::Context;
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
    /// # use pvxs_sys::Context;
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
    /// # use pvxs_sys::Context;
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
    /// # use pvxs_sys::Context;
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
    /// # use pvxs_sys::Context;
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
    /// # use pvxs_sys::Context;
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
    /// # use pvxs_sys::Context;
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
    /// # use pvxs_sys::Context;
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
    /// # use pvxs_sys::Context;
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
    /// use pvxs_sys::Context;
    /// 
    /// let mut ctx = Context::from_env().expect("Context creation failed");
    /// let monitor = ctx.monitor_builder("TEST:PV_Double")?
    ///     .connect_exception(true)      // Throw connection exceptions
    ///     .disconnect_exception(true)   // Throw disconnection exceptions
    ///     .exec()
    ///     .expect("Monitor creation failed");
    /// # Ok::<(), pvxs_sys::PvxsError>(())
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
    /// # use pvxs_sys::Context;
    /// # async fn example() -> Result<(), pvxs_sys::PvxsError> {
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
    /// # use pvxs_sys::Context;
    /// # async fn example() -> Result<(), pvxs_sys::PvxsError> {
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
    /// # use pvxs_sys::Context;
    /// # async fn example() -> Result<(), pvxs_sys::PvxsError> {
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


/// RPC (Remote Procedure Call) builder for EPICS servers
/// 
/// Provides a fluent interface for building and executing RPC calls.
/// RPC allows calling server-side functions with typed arguments.
/// 
/// # Example
/// 
/// ```no_run
/// # use pvxs_sys::Context;
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
/// use pvxs_sys::Context;
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
/// # Ok::<(), pvxs_sys::PvxsError>(())
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
    /// # use pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// # let mut monitor = ctx.monitor("MY:PV").unwrap();
    /// monitor.start();
    /// ```
    pub fn start(&mut self) -> Result<()> {
        bridge::monitor_start(self.inner.pin_mut())?;
        Ok(())
    }
    
    /// Stop monitoring for value changes
    /// 
    /// This ends the subscription and no more updates will be received.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// # let mut monitor = ctx.monitor("MY:PV").unwrap();
    /// # monitor.start();
    /// monitor.stop()?;
    /// # Ok::<(), pvxs_sys::PvxsError>(())
    /// ```
    pub fn stop(&mut self) -> Result<()> {
        bridge::monitor_stop(self.inner.pin_mut())?;
        Ok(())
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
    /// # use pvxs_sys::Context;
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
    /// # use pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// # let mut monitor = ctx.monitor("MY:PV").unwrap();
    /// # monitor.start();
    /// if monitor.has_update() {
    ///     let value = monitor.try_get_update()?;
    ///     println!("Update available: {:?}", value);
    /// }
    /// # Ok::<(), pvxs_sys::PvxsError>(())
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
    /// # use pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// # let mut monitor = ctx.monitor("MY:PV").unwrap();
    /// # monitor.start();
    /// match monitor.get_update(5.0) {
    ///     Ok(value) => println!("Update received: {}", value),
    ///     Err(e) => println!("No update within 5 seconds: {}", e),
    /// }
    /// # Ok::<(), pvxs_sys::PvxsError>(())
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
    /// # use pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// # let mut monitor = ctx.monitor("MY:PV").unwrap();
    /// # monitor.start();
    /// if let Some(value) = monitor.try_get_update()? {
    ///     println!("Update: {}", value);
    /// } else {
    ///     println!("No update available");
    /// }
    /// # Ok::<(), pvxs_sys::PvxsError>(())
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
    /// or returns Err with MonitorEvent for connection/disconnection events.
    /// 
    /// # Returns
    /// 
    /// - `Ok(Some(Value))` if an update is available
    /// - `Ok(None)` if the queue is empty
    /// - `Err(MonitorEvent::Connected)` if connection exception (when connect_exception(true), i.e. maskConnected(false))
    /// - `Err(MonitorEvent::Disconnected)` if disconnection exception (when disconnect_exception(true), i.e. maskDisconnected(false))
    /// - `Err(MonitorEvent::Finished)` if finished exception (when disconnect_exception(true), i.e. maskDisconnected(false))
    /// 
    /// Note: The mask configuration controls whether exceptions are suppressed or thrown:
    /// - connect_exception(true) -> maskConnected(false) -> exceptions are thrown as MonitorEvent::Connected
    /// - connect_exception(false) -> maskConnected(true) -> exceptions are suppressed/masked out
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use pvxs_sys::{Context, MonitorEvent};
    /// # let mut ctx = Context::from_env().unwrap();
    /// # let mut monitor = ctx.monitor("MY:PV").unwrap();
    /// # monitor.start();
    /// loop {
    ///     match monitor.pop() {
    ///         Ok(Some(value)) => println!("Update: {}", value),
    ///         Ok(None) => break, // Queue empty
    ///         Err(e) if e.to_string().contains("connected") => {
    ///             println!("Connection event");
    ///             break;
    ///         }
    ///         Err(e) => {
    ///             println!("Other error: {}", e);
    ///             break;
    ///         }
    ///     }
    /// }
    /// ```
    pub fn pop(&mut self) -> std::result::Result<Option<Value>, MonitorEvent> {
        match bridge::monitor_pop(self.inner.pin_mut()) {
            Ok(value_wrapper) => {
                if value_wrapper.is_null() {
                    Ok(None)
                } else {
                    Ok(Some(Value { inner: value_wrapper }))
                }
            },
            Err(e) => {
                let err_msg = e.what();
                // Check if this is one of our monitor event exceptions
                if err_msg.contains("Monitor connected:") {
                    Err(MonitorEvent::Connected(err_msg.to_string()))
                } else if err_msg.contains("Monitor disconnected:") {
                    Err(MonitorEvent::Disconnected(err_msg.to_string()))
                } else if err_msg.contains("Monitor finished:") {
                    Err(MonitorEvent::Finished(err_msg.to_string()))
                } else if err_msg.contains("Monitor remote error:") {
                    Err(MonitorEvent::RemoteError(err_msg.to_string()))
                } else if err_msg.contains("Monitor client error:") {
                    Err(MonitorEvent::ClientError(err_msg.to_string()))
                } else {
                    // For other errors, panic or convert to a ClientError
                    Err(MonitorEvent::ClientError(err_msg.to_string()))
                }
            },
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
    /// # use pvxs_sys::Context;
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
    /// # use pvxs_sys::Context;
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
/// use pvxs_sys::Context;
/// 
/// let mut ctx = Context::from_env()?;
/// let monitor = ctx.monitor_builder("MY:PV")?
///     .connect_exception(true)
///     .disconnect_exception(true)
///     .exec()?;
/// # Ok::<(), pvxs_sys::PvxsError>(())
/// ```
pub struct MonitorBuilder {
    inner: UniquePtr<bridge::MonitorBuilderWrapper>,
}

impl MonitorBuilder {
    /// Enable or disable connection exceptions in the monitor queue
    /// 
    /// This is the user-friendly API - think in terms of what you want to enable.
    /// 
    /// # Arguments
    /// 
    /// * `enable` - true to throw connection exceptions, false to suppress them (default: false)
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let monitor = ctx.monitor_builder("MY:PV")?
    ///     .connect_exception(true) // Throw connection exceptions
    ///     .exec()?;
    /// # Ok::<(), pvxs_sys::PvxsError>(())
    /// ```
    pub fn connect_exception(mut self, enable: bool) -> Self {
        // PVXS maskConnected(false) = don't mask = throw events, maskConnected(true) = mask = suppress events
        // So enable=true means mask=false (don't suppress), enable=false means mask=true (suppress)
        let _ = bridge::monitor_builder_mask_connected(self.inner.pin_mut(), !enable);
        self
    }
    
    /// Enable or disable disconnection exceptions in the monitor queue
    /// 
    /// This is the user-friendly API - think in terms of what you want to enable.
    /// 
    /// # Arguments
    /// 
    /// * `enable` - true to throw disconnection exceptions, false to suppress them (default: true)
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let monitor = ctx.monitor_builder("MY:PV")?
    ///     .disconnect_exception(true) // Throw disconnection exceptions
    ///     .exec()?;
    /// # Ok::<(), pvxs_sys::PvxsError>(())
    /// ```
    pub fn disconnect_exception(mut self, enable: bool) -> Self {
        // PVXS maskDisconnected(false) = don't mask = throw events, maskDisconnected(true) = mask = suppress events
        // So enable=true means mask=false (don't suppress), enable=false means mask=true (suppress)
        let _ = bridge::monitor_builder_mask_disconnected(self.inner.pin_mut(), !enable);
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
    /// # use pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// 
    /// extern "C" fn my_callback() {
    ///     println!("Events available in subscription queue!");
    /// }
    /// 
    /// let monitor = ctx.monitor_builder("MY:PV")?
    ///     .event(my_callback)
    ///     .exec()?;
    /// # Ok::<(), pvxs_sys::PvxsError>(())
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
    /// # use pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let monitor = ctx.monitor_builder("MY:PV")?
    ///     .connect_exception(true)
    ///     .exec()?;
    /// # Ok::<(), pvxs_sys::PvxsError>(())
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
    /// # use pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let monitor = ctx.monitor_builder("MY:PV")?
    ///     .exec_with_callback(123)?;
    /// # Ok::<(), pvxs_sys::PvxsError>(())
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
    /// # use pvxs_sys::Context;
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
    /// # use pvxs_sys::Context;
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
    /// # use pvxs_sys::Context;
    /// # async fn example() -> Result<(), pvxs_sys::PvxsError> {
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
