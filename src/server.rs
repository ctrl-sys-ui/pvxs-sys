// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use crossbeam_channel as channel;
use cxx::UniquePtr;
use std::collections::HashMap;
use std::thread;

use crate::{
    bridge, compute_alarm_for_scalar, AlarmConfig, AlarmMetadata, AlarmSeverity, AlarmStatus,
    ControlMetadata, DisplayMetadata, PvxsError, Result, Value,
};

/// Internal server implementation
///
/// Low-level server that manages PVs without automatic alarm handling.
/// Users should typically use the `Server` type instead, which provides
/// managed PVs with automatic alarm and validation logic.
pub(crate) struct ServerImpl {
    inner: UniquePtr<bridge::ServerWrapper>,
}

impl ServerImpl {
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
    /// use pvxs_sys::Server;
    ///
    /// let server = Server::start_isolated()?;
    /// println!("Isolated server started on TCP port {}", server.tcp_port());
    /// server.stop_drop()?;
    /// # Ok::<(), pvxs_sys::PvxsError>(())
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

    // TODO: TZ: Review later if needed
    /*
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
    }*/

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
    /// # use pvxs_sys::{Server, NTScalarMetadataBuilder};
    /// # let mut server = Server::start_isolated().unwrap();
    /// let pv = server.create_pv_double("test:double", 42.5, NTScalarMetadataBuilder::new())?;
    /// # Ok::<(), pvxs_sys::PvxsError>(())
    /// ```
    pub fn create_pv_double(
        &mut self,
        name: &str,
        initial_value: f64,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<SharedPV> {
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_double(initial_value, metadata)?;
        self.add_pv(name, &mut pv)?;
        Ok(pv)
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
    pub fn create_pv_double_array(
        &mut self,
        name: &str,
        initial_value: Vec<f64>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<SharedPV> {
        if initial_value.is_empty() {
            return Err(PvxsError::new("Initial double array cannot be empty"));
        }
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_double_array(initial_value, metadata)?;
        self.add_pv(name, &mut pv)?;
        Ok(pv)
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
    pub fn create_pv_int32(
        &mut self,
        name: &str,
        initial_value: i32,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<SharedPV> {
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_int32(initial_value, metadata)?;
        self.add_pv(name, &mut pv)?;
        Ok(pv)
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
    pub fn create_pv_int32_array(
        &mut self,
        name: &str,
        initial_value: Vec<i32>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<SharedPV> {
        if initial_value.is_empty() {
            return Err(PvxsError::new("Initial int32 array cannot be empty"));
        }
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_int32_array(initial_value, metadata)?;
        self.add_pv(name, &mut pv)?;
        Ok(pv)
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
    pub fn create_pv_string(
        &mut self,
        name: &str,
        initial_value: &str,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<SharedPV> {
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_string(initial_value, metadata)?;
        self.add_pv(name, &mut pv)?;
        Ok(pv)
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
    pub fn create_pv_string_array(
        &mut self,
        name: &str,
        initial_value: Vec<String>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<SharedPV> {
        if initial_value.is_empty() {
            return Err(PvxsError::new("Initial string array cannot be empty"));
        }
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_string_array(initial_value, metadata)?;
        self.add_pv(name, &mut pv)?;
        Ok(pv)
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
    pub fn create_pv_enum(
        &mut self,
        name: &str,
        choices: Vec<&str>,
        selected_index: i16,
        metadata: NTEnumMetadataBuilder,
    ) -> Result<SharedPV> {
        let mut pv = SharedPV::create_mailbox()?;
        pv.open_enum(choices, selected_index, metadata)?;
        self.add_pv(name, &mut pv)?;
        Ok(pv)
    }

    // TODO: TZ - template for readonly PVs if needed in the future
    /*
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
    pub fn create_readonly_pv_double(&mut self, name: &str, initial_value: f64, metadata: NTScalarMetadataBuilder) -> Result<SharedPV> {
        let mut pv = SharedPV::create_readonly()?;
        pv.open_double(initial_value, metadata)?;
        self.add_pv(name, &mut pv)?;
        Ok(pv)
    }*/
}

/// Fetched double value with alarm information
#[derive(Debug, Clone)]
pub struct FetchedDouble {
    pub value: f64,
    pub alarm_severity: AlarmSeverity,
    pub alarm_status: AlarmStatus,
    pub alarm_message: String,
    pub display_metadata: Option<DisplayMetadata>,
    pub control_metadata: Option<ControlMetadata>,
    pub alarm_metadata: Option<AlarmMetadata>,
}

/// Fetched int32 value with alarm information
#[derive(Debug, Clone)]
pub struct FetchedInt32 {
    pub value: i32,
    pub alarm_severity: AlarmSeverity,
    pub alarm_status: AlarmStatus,
    pub alarm_message: String,
    pub display_metadata: Option<DisplayMetadata>,
    pub control_metadata: Option<ControlMetadata>,
    pub alarm_metadata: Option<AlarmMetadata>,
}

/// Fetched string value with alarm information
#[derive(Debug, Clone)]
pub struct FetchedString {
    pub value: String,
    pub alarm_severity: AlarmSeverity,
    pub alarm_status: AlarmStatus,
    pub alarm_message: String,
}

/// Fetched double array value with alarm information
#[derive(Debug, Clone)]
pub struct FetchedDoubleArray {
    pub value: Vec<f64>,
    pub alarm_severity: AlarmSeverity,
    pub alarm_status: AlarmStatus,
    pub alarm_message: String,
    pub display_metadata: Option<DisplayMetadata>,
    pub control_metadata: Option<ControlMetadata>,
    pub alarm_metadata: Option<AlarmMetadata>,
}

/// Fetched int32 array value with alarm information
#[derive(Debug, Clone)]
pub struct FetchedInt32Array {
    pub value: Vec<i32>,
    pub alarm_severity: AlarmSeverity,
    pub alarm_status: AlarmStatus,
    pub alarm_message: String,
    pub display_metadata: Option<DisplayMetadata>,
    pub control_metadata: Option<ControlMetadata>,
    pub alarm_metadata: Option<AlarmMetadata>,
}

/// Fetched string array value with alarm information
#[derive(Debug, Clone)]
pub struct FetchedStringArray {
    pub value: Vec<String>,
    pub alarm_severity: AlarmSeverity,
    pub alarm_status: AlarmStatus,
    pub alarm_message: String,
}

/// Fetched enum value with alarm information
#[derive(Debug, Clone)]
pub struct FetchedEnum {
    pub value: i16,
    pub value_choices: Vec<String>,
    pub alarm_severity: AlarmSeverity,
    pub alarm_status: AlarmStatus,
    pub alarm_message: String,
}

enum ManagerCommand {
    CreateDouble {
        name: String,
        initial: f64,
        metadata: NTScalarMetadataBuilder,
        reply: channel::Sender<Result<()>>,
    },
    CreateDoubleArray {
        name: String,
        initial: Vec<f64>,
        metadata: NTScalarMetadataBuilder,
        reply: channel::Sender<Result<()>>,
    },
    CreateInt32 {
        name: String,
        initial: i32,
        metadata: NTScalarMetadataBuilder,
        reply: channel::Sender<Result<()>>,
    },
    CreateInt32Array {
        name: String,
        initial: Vec<i32>,
        metadata: NTScalarMetadataBuilder,
        reply: channel::Sender<Result<()>>,
    },
    CreateString {
        name: String,
        initial: String,
        metadata: NTScalarMetadataBuilder,
        reply: channel::Sender<Result<()>>,
    },
    CreateStringArray {
        name: String,
        initial: Vec<String>,
        metadata: NTScalarMetadataBuilder,
        reply: channel::Sender<Result<()>>,
    },
    CreateEnum {
        name: String,
        choices: Vec<String>,
        selected_index: i16,
        metadata: NTEnumMetadataBuilder,
        reply: channel::Sender<Result<()>>,
    },
    PostDouble {
        name: String,
        value: f64,
        reply: channel::Sender<Result<()>>,
    },
    PostDoubleArray {
        name: String,
        value: Vec<f64>,
        reply: channel::Sender<Result<()>>,
    },
    PostInt32 {
        name: String,
        value: i32,
        reply: channel::Sender<Result<()>>,
    },
    PostInt32Array {
        name: String,
        value: Vec<i32>,
        reply: channel::Sender<Result<()>>,
    },
    PostString {
        name: String,
        value: String,
        reply: channel::Sender<Result<()>>,
    },
    PostStringArray {
        name: String,
        value: Vec<String>,
        reply: channel::Sender<Result<()>>,
    },
    PostEnum {
        name: String,
        value: i16,
        reply: channel::Sender<Result<()>>,
    },
    Remove {
        name: String,
        reply: channel::Sender<Result<()>>,
    },
    FetchDouble {
        name: String,
        reply: channel::Sender<Result<FetchedDouble>>,
    },
    FetchInt32 {
        name: String,
        reply: channel::Sender<Result<FetchedInt32>>,
    },
    FetchString {
        name: String,
        reply: channel::Sender<Result<FetchedString>>,
    },
    FetchDoubleArray {
        name: String,
        reply: channel::Sender<Result<FetchedDoubleArray>>,
    },
    FetchInt32Array {
        name: String,
        reply: channel::Sender<Result<FetchedInt32Array>>,
    },
    FetchStringArray {
        name: String,
        reply: channel::Sender<Result<FetchedStringArray>>,
    },
    FetchEnum {
        name: String,
        reply: channel::Sender<Result<FetchedEnum>>,
    },
    Stop {
        reply: channel::Sender<Result<()>>,
    },
}

enum ManagedPv {
    Double {
        pv: SharedPV,
        alarm: AlarmConfig,
        last: f64,
    },
    DoubleArray(SharedPV),
    Int32 {
        pv: SharedPV,
        alarm: AlarmConfig,
        last: i32,
    },
    Int32Array(SharedPV),
    String(SharedPV),
    StringArray(SharedPV),
    PvEnum(SharedPV),
}

/// Handle to a running PVXS server
///
/// Provides methods to interact with a running server without blocking.
/// Can be cloned and shared across threads.
#[derive(Clone)]
pub struct ServerHandle {
    tx: channel::Sender<ManagerCommand>,
    tcp_port: u16,
    udp_port: u16,
}

impl ServerHandle {
    pub fn tcp_port(&self) -> u16 {
        self.tcp_port
    }

    pub fn udp_port(&self) -> u16 {
        self.udp_port
    }

    pub fn create_pv_double(
        &self,
        name: &str,
        initial: f64,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::CreateDouble {
                name: name.to_string(),
                initial,
                metadata,
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn create_pv_double_array(
        &self,
        name: &str,
        initial: Vec<f64>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::CreateDoubleArray {
                name: name.to_string(),
                initial,
                metadata,
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn create_pv_int32(
        &self,
        name: &str,
        initial: i32,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::CreateInt32 {
                name: name.to_string(),
                initial,
                metadata,
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn create_pv_int32_array(
        &self,
        name: &str,
        initial: Vec<i32>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::CreateInt32Array {
                name: name.to_string(),
                initial,
                metadata,
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn create_pv_string(
        &self,
        name: &str,
        initial: &str,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::CreateString {
                name: name.to_string(),
                initial: initial.to_string(),
                metadata,
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn create_pv_string_array(
        &self,
        name: &str,
        initial: Vec<String>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::CreateStringArray {
                name: name.to_string(),
                initial,
                metadata,
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn create_pv_enum(
        &self,
        name: &str,
        choices: Vec<&str>,
        selected_index: i16,
        metadata: NTEnumMetadataBuilder,
    ) -> Result<()> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::CreateEnum {
                name: name.to_string(),
                choices: choices.iter().map(|s| s.to_string()).collect(),
                selected_index,
                metadata,
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn post_double(&self, name: &str, value: f64) -> Result<()> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::PostDouble {
                name: name.to_string(),
                value,
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn post_double_array(&self, name: &str, value: Vec<f64>) -> Result<()> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::PostDoubleArray {
                name: name.to_string(),
                value,
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn post_int32(&self, name: &str, value: i32) -> Result<()> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::PostInt32 {
                name: name.to_string(),
                value,
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn post_int32_array(&self, name: &str, value: Vec<i32>) -> Result<()> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::PostInt32Array {
                name: name.to_string(),
                value,
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn post_string(&self, name: &str, value: &str) -> Result<()> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::PostString {
                name: name.to_string(),
                value: value.to_string(),
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn post_string_array(&self, name: &str, value: Vec<String>) -> Result<()> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::PostStringArray {
                name: name.to_string(),
                value,
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn post_enum(&self, name: &str, value: i16) -> Result<()> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::PostEnum {
                name: name.to_string(),
                value,
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn remove_pv(&self, name: &str) -> Result<()> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::Remove {
                name: name.to_string(),
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn fetch_double(&self, name: &str) -> Result<FetchedDouble> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::FetchDouble {
                name: name.to_string(),
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn fetch_int32(&self, name: &str) -> Result<FetchedInt32> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::FetchInt32 {
                name: name.to_string(),
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn fetch_string(&self, name: &str) -> Result<FetchedString> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::FetchString {
                name: name.to_string(),
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn fetch_double_array(&self, name: &str) -> Result<FetchedDoubleArray> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::FetchDoubleArray {
                name: name.to_string(),
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn fetch_int32_array(&self, name: &str) -> Result<FetchedInt32Array> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::FetchInt32Array {
                name: name.to_string(),
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn fetch_string_array(&self, name: &str) -> Result<FetchedStringArray> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::FetchStringArray {
                name: name.to_string(),
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }

    pub fn fetch_enum(&self, name: &str) -> Result<FetchedEnum> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.tx
            .send(ManagerCommand::FetchEnum {
                name: name.to_string(),
                reply: reply_tx,
            })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?
    }
}

/// A PVXS server for hosting process variables with automatic alarm management
///
/// The Server provides managed PVs with automatic alarm handling, control limit
/// validation, and value alarm checking. This is the recommended way to create
/// EPICS servers in Rust.
///
/// # Features
///
/// - Automatic alarm computation based on control limits and value alarms
/// - Thread-safe PV management with internal worker thread
/// - Support for multiple data types (double, int32, string, enum, arrays)
/// - Isolated or environment-configured operation
///
/// # Example
///
/// ```no_run
/// use pvxs_sys::{Server, NTScalarMetadataBuilder, ControlMetadata};
///
/// let server = Server::start_from_env()?;
///
/// // Create a PV with control limits
/// let metadata = NTScalarMetadataBuilder::new()
///     .control(ControlMetadata {
///         limit_low: 0.0,
///         limit_high: 100.0,
///         min_step: 0.1,
///     });
///
/// server.create_pv_double("test:pv", 50.0, metadata)?;
/// server.post_double("test:pv", 75.0)?;  // Validated and with alarms
///
/// server.stop_drop()?;
/// # Ok::<(), pvxs_sys::PvxsError>(())
/// ```
pub struct Server {
    handle: ServerHandle,
    join: Option<thread::JoinHandle<()>>,
}

impl Server {
    /// Start a server using environment configuration
    ///
    /// Creates and starts a server that reads EPICS network configuration
    /// from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if the server cannot be created or started.
    pub fn start_from_env() -> Result<Self> {
        Self::start_inner(false)
    }

    /// Start an isolated server for testing
    ///
    /// Creates and starts a server with system-assigned ports, ideal for
    /// unit tests and parallel test execution.
    ///
    /// # Errors
    ///
    /// Returns an error if the server cannot be created or started.
    pub fn start_isolated() -> Result<Self> {
        Self::start_inner(true)
    }

    /// Get a cloneable handle to this server
    ///
    /// The handle can be used from other threads to interact with the server.
    pub fn handle(&self) -> ServerHandle {
        self.handle.clone()
    }

    pub fn tcp_port(&self) -> u16 {
        self.handle.tcp_port()
    }

    pub fn udp_port(&self) -> u16 {
        self.handle.udp_port()
    }

    pub fn create_pv_double(
        &self,
        name: &str,
        initial: f64,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_double(name, initial, metadata)
    }

    pub fn create_pv_double_array(
        &self,
        name: &str,
        initial: Vec<f64>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_double_array(name, initial, metadata)
    }

    pub fn create_pv_int32(
        &self,
        name: &str,
        initial: i32,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_int32(name, initial, metadata)
    }

    pub fn create_pv_int32_array(
        &self,
        name: &str,
        initial: Vec<i32>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_int32_array(name, initial, metadata)
    }

    pub fn create_pv_string(
        &self,
        name: &str,
        initial: &str,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_string(name, initial, metadata)
    }

    pub fn create_pv_string_array(
        &self,
        name: &str,
        initial: Vec<String>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_string_array(name, initial, metadata)
    }

    pub fn create_pv_enum(
        &self,
        name: &str,
        choices: Vec<&str>,
        selected_index: i16,
        metadata: NTEnumMetadataBuilder,
    ) -> Result<()> {
        self.handle
            .create_pv_enum(name, choices, selected_index, metadata)
    }

    pub fn post_double(&self, name: &str, value: f64) -> Result<()> {
        self.handle.post_double(name, value)
    }

    pub fn post_double_array(&self, name: &str, value: Vec<f64>) -> Result<()> {
        self.handle.post_double_array(name, value)
    }

    pub fn post_int32(&self, name: &str, value: i32) -> Result<()> {
        self.handle.post_int32(name, value)
    }

    pub fn post_int32_array(&self, name: &str, value: Vec<i32>) -> Result<()> {
        self.handle.post_int32_array(name, value)
    }

    pub fn post_string(&self, name: &str, value: &str) -> Result<()> {
        self.handle.post_string(name, value)
    }

    pub fn post_string_array(&self, name: &str, value: Vec<String>) -> Result<()> {
        self.handle.post_string_array(name, value)
    }

    pub fn post_enum(&self, name: &str, value: i16) -> Result<()> {
        self.handle.post_enum(name, value)
    }

    pub fn remove_pv(&self, name: &str) -> Result<()> {
        self.handle.remove_pv(name)
    }

    pub fn fetch_double(&self, name: &str) -> Result<FetchedDouble> {
        self.handle.fetch_double(name)
    }

    pub fn fetch_int32(&self, name: &str) -> Result<FetchedInt32> {
        self.handle.fetch_int32(name)
    }

    pub fn fetch_string(&self, name: &str) -> Result<FetchedString> {
        self.handle.fetch_string(name)
    }

    pub fn fetch_double_array(&self, name: &str) -> Result<FetchedDoubleArray> {
        self.handle.fetch_double_array(name)
    }

    pub fn fetch_int32_array(&self, name: &str) -> Result<FetchedInt32Array> {
        self.handle.fetch_int32_array(name)
    }

    pub fn fetch_string_array(&self, name: &str) -> Result<FetchedStringArray> {
        self.handle.fetch_string_array(name)
    }

    pub fn fetch_enum(&self, name: &str) -> Result<FetchedEnum> {
        self.handle.fetch_enum(name)
    }

    /// Indirectly calls inner stop through the manager thread, ensuring proper shutdown and resource cleanup
    /// Once Stop command is sent the worker thread will exit, however it will destroy the ServerImpl and the
    /// entire pvs hashmap so the all underlying C++ objects are freed correctly.
    ///
    /// Call start_from_env or start_isolated again and create new PVs to start a new server instance if needed after stopping.
    pub fn stop_drop(mut self) -> Result<()> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.handle
            .tx
            .send(ManagerCommand::Stop { reply: reply_tx })
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        let result = reply_rx
            .recv()
            .map_err(|_| PvxsError::new("Server worker stopped"))?;
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        result
    }

    fn start_inner(isolated: bool) -> Result<Self> {
        let (tx, rx) = channel::unbounded::<ManagerCommand>();
        let (ready_tx, ready_rx) = channel::bounded::<Result<(u16, u16)>>(1);

        let join = thread::spawn(move || {
            let mut server = if isolated {
                match ServerImpl::create_isolated() {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                }
            } else {
                match ServerImpl::from_env() {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                }
            };

            if let Err(e) = server.start() {
                let _ = ready_tx.send(Err(e));
                return;
            }

            let _ = ready_tx.send(Ok((server.tcp_port(), server.udp_port())));

            let mut pvs: HashMap<String, ManagedPv> = HashMap::new();

            while let Ok(cmd) = rx.recv() {
                match cmd {
                    ManagerCommand::CreateDouble {
                        name,
                        initial,
                        metadata,
                        reply,
                    } => {
                        let result = if pvs.contains_key(&name) {
                            Err(PvxsError::new("PV already exists"))
                        } else {
                            let alarm = AlarmConfig {
                                control: metadata.control.clone(),
                                alarm_metadata: metadata.alarm_metadata.clone(),
                            };
                            // Compute alarm for initial value
                            let alarm_result = compute_alarm_for_scalar(initial, &alarm);
                            // Update metadata with computed alarm
                            let mut metadata_with_alarm = metadata;
                            metadata_with_alarm.alarm_severity = alarm_result.severity;
                            metadata_with_alarm.alarm_status = alarm_result.status;
                            metadata_with_alarm.alarm_message = alarm_result.message.clone();

                            match server.create_pv_double(&name, initial, metadata_with_alarm) {
                                Ok(pv) => {
                                    pvs.insert(
                                        name,
                                        ManagedPv::Double {
                                            pv,
                                            alarm,
                                            last: initial,
                                        },
                                    );
                                    Ok(())
                                }
                                Err(e) => Err(e),
                            }
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::CreateDoubleArray {
                        name,
                        initial,
                        metadata,
                        reply,
                    } => {
                        let result = if pvs.contains_key(&name) {
                            Err(PvxsError::new("PV already exists"))
                        } else {
                            match server.create_pv_double_array(&name, initial, metadata) {
                                Ok(pv) => {
                                    pvs.insert(name, ManagedPv::DoubleArray(pv));
                                    Ok(())
                                }
                                Err(e) => Err(e),
                            }
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::CreateInt32 {
                        name,
                        initial,
                        metadata,
                        reply,
                    } => {
                        let result = if pvs.contains_key(&name) {
                            Err(PvxsError::new("PV already exists"))
                        } else {
                            let alarm = AlarmConfig {
                                control: metadata.control.clone(),
                                alarm_metadata: metadata.alarm_metadata.clone(),
                            };
                            // Compute alarm for initial value
                            let alarm_result = compute_alarm_for_scalar(initial as f64, &alarm);
                            // Update metadata with computed alarm
                            let mut metadata_with_alarm = metadata;
                            metadata_with_alarm.alarm_severity = alarm_result.severity;
                            metadata_with_alarm.alarm_status = alarm_result.status;
                            metadata_with_alarm.alarm_message = alarm_result.message.clone();

                            match server.create_pv_int32(&name, initial, metadata_with_alarm) {
                                Ok(pv) => {
                                    pvs.insert(
                                        name,
                                        ManagedPv::Int32 {
                                            pv,
                                            alarm,
                                            last: initial,
                                        },
                                    );
                                    Ok(())
                                }
                                Err(e) => Err(e),
                            }
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::CreateInt32Array {
                        name,
                        initial,
                        metadata,
                        reply,
                    } => {
                        let result = if pvs.contains_key(&name) {
                            Err(PvxsError::new("PV already exists"))
                        } else {
                            match server.create_pv_int32_array(&name, initial, metadata) {
                                Ok(pv) => {
                                    pvs.insert(name, ManagedPv::Int32Array(pv));
                                    Ok(())
                                }
                                Err(e) => Err(e),
                            }
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::CreateString {
                        name,
                        initial,
                        metadata,
                        reply,
                    } => {
                        let result = if pvs.contains_key(&name) {
                            Err(PvxsError::new("PV already exists"))
                        } else {
                            match server.create_pv_string(&name, &initial, metadata) {
                                Ok(pv) => {
                                    pvs.insert(name, ManagedPv::String(pv));
                                    Ok(())
                                }
                                Err(e) => Err(e),
                            }
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::CreateStringArray {
                        name,
                        initial,
                        metadata,
                        reply,
                    } => {
                        let result = if pvs.contains_key(&name) {
                            Err(PvxsError::new("PV already exists"))
                        } else {
                            match server.create_pv_string_array(&name, initial, metadata) {
                                Ok(pv) => {
                                    pvs.insert(name, ManagedPv::StringArray(pv));
                                    Ok(())
                                }
                                Err(e) => Err(e),
                            }
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::CreateEnum {
                        name,
                        choices,
                        selected_index,
                        metadata,
                        reply,
                    } => {
                        let result = if pvs.contains_key(&name) {
                            Err(PvxsError::new("PV already exists"))
                        } else {
                            let choices_refs: Vec<&str> =
                                choices.iter().map(|s| s.as_str()).collect();
                            match server.create_pv_enum(
                                &name,
                                choices_refs,
                                selected_index,
                                metadata,
                            ) {
                                Ok(pv) => {
                                    pvs.insert(name, ManagedPv::PvEnum(pv));
                                    Ok(())
                                }
                                Err(e) => Err(e),
                            }
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::PostDouble { name, value, reply } => {
                        let result = match pvs.get_mut(&name) {
                            Some(ManagedPv::Double { pv, alarm, last }) => {
                                let alarm_result = compute_alarm_for_scalar(value, alarm);
                                // If not allowed, fall back to the live value currently in the PV
                                // (which may have been written by a client PUT since the last Rust post)
                                let post_value = if alarm_result.allow {
                                    value
                                } else {
                                    pv.fetch()
                                        .and_then(|v| v.get_field_double("value"))
                                        .unwrap_or(*last)
                                };
                                let result = pv.post_double_with_alarm(
                                    post_value,
                                    alarm_result.severity,
                                    alarm_result.status,
                                    alarm_result.message,
                                );
                                if result.is_ok() && alarm_result.allow {
                                    *last = post_value;
                                }
                                result
                            }
                            _ => Err(PvxsError::new("PV not found or type mismatch")),
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::PostDoubleArray { name, value, reply } => {
                        let result = match pvs.get_mut(&name) {
                            Some(ManagedPv::DoubleArray(pv)) => pv.post_double_array(&value),
                            _ => Err(PvxsError::new("PV not found or type mismatch")),
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::PostInt32 { name, value, reply } => {
                        let result = match pvs.get_mut(&name) {
                            Some(ManagedPv::Int32 { pv, alarm, last }) => {
                                let alarm_result = compute_alarm_for_scalar(value as f64, alarm);
                                // If not allowed, fall back to the live value currently in the PV
                                // (which may have been written by a client PUT since the last Rust post)
                                let post_value = if alarm_result.allow {
                                    value
                                } else {
                                    pv.fetch()
                                        .and_then(|v| v.get_field_int32("value"))
                                        .unwrap_or(*last)
                                };
                                let result = pv.post_int32_with_alarm(
                                    post_value,
                                    alarm_result.severity,
                                    alarm_result.status,
                                    alarm_result.message,
                                );
                                if result.is_ok() && alarm_result.allow {
                                    *last = post_value;
                                }
                                result
                            }
                            _ => Err(PvxsError::new("PV not found or type mismatch")),
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::PostInt32Array { name, value, reply } => {
                        let result = match pvs.get_mut(&name) {
                            Some(ManagedPv::Int32Array(pv)) => pv.post_int32_array(&value),
                            _ => Err(PvxsError::new("PV not found or type mismatch")),
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::PostString { name, value, reply } => {
                        let result = match pvs.get_mut(&name) {
                            Some(ManagedPv::String(pv)) => pv.post_string(&value),
                            _ => Err(PvxsError::new("PV not found or type mismatch")),
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::PostStringArray { name, value, reply } => {
                        let result = match pvs.get_mut(&name) {
                            Some(ManagedPv::StringArray(pv)) => pv.post_string_array(&value),
                            _ => Err(PvxsError::new("PV not found or type mismatch")),
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::PostEnum { name, value, reply } => {
                        let result = match pvs.get_mut(&name) {
                            Some(ManagedPv::PvEnum(pv)) => pv.post_enum(value),
                            _ => Err(PvxsError::new("PV not found or type mismatch")),
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::Remove { name, reply } => {
                        let result = if pvs.remove(&name).is_some() {
                            server.remove_pv(&name)
                        } else {
                            Err(PvxsError::new("PV not found"))
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::FetchDouble { name, reply } => {
                        let result = match pvs.get(&name) {
                            Some(ManagedPv::Double { pv, .. }) => {
                                pv.fetch().and_then(|v| {
                                    // Extract display metadata if present
                                    let display_metadata = (|| -> Option<DisplayMetadata> {
                                        Some(DisplayMetadata {
                                            limit_low: v.get_field_int32("display.limitLow").ok()?
                                                as i64,
                                            limit_high: v
                                                .get_field_int32("display.limitHigh")
                                                .ok()?
                                                as i64,
                                            description: v
                                                .get_field_string("display.description")
                                                .ok()?,
                                            units: v.get_field_string("display.units").ok()?,
                                            precision: v
                                                .get_field_int32("display.precision")
                                                .ok()?,
                                        })
                                    })();

                                    // Extract control metadata if present
                                    let control_metadata = (|| -> Option<ControlMetadata> {
                                        Some(ControlMetadata {
                                            limit_low: v
                                                .get_field_double("control.limitLow")
                                                .ok()?,
                                            limit_high: v
                                                .get_field_double("control.limitHigh")
                                                .ok()?,
                                            min_step: v.get_field_double("control.minStep").ok()?,
                                        })
                                    })();

                                    // Extract alarm metadata if present
                                    let alarm_metadata = (|| -> Option<AlarmMetadata> {
                                        Some(AlarmMetadata {
                                            active: v.get_field_int32("valueAlarm.active").ok()?
                                                != 0,
                                            low_alarm_limit: v
                                                .get_field_double("valueAlarm.lowAlarmLimit")
                                                .ok()?,
                                            low_warning_limit: v
                                                .get_field_double("valueAlarm.lowWarningLimit")
                                                .ok()?,
                                            high_warning_limit: v
                                                .get_field_double("valueAlarm.highWarningLimit")
                                                .ok()?,
                                            high_alarm_limit: v
                                                .get_field_double("valueAlarm.highAlarmLimit")
                                                .ok()?,
                                            low_alarm_severity: AlarmSeverity::from(
                                                v.get_field_int32("valueAlarm.lowAlarmSeverity")
                                                    .ok()?,
                                            ),
                                            low_warning_severity: AlarmSeverity::from(
                                                v.get_field_int32("valueAlarm.lowWarningSeverity")
                                                    .ok()?,
                                            ),
                                            high_warning_severity: AlarmSeverity::from(
                                                v.get_field_int32("valueAlarm.highWarningSeverity")
                                                    .ok()?,
                                            ),
                                            high_alarm_severity: AlarmSeverity::from(
                                                v.get_field_int32("valueAlarm.highAlarmSeverity")
                                                    .ok()?,
                                            ),
                                            hysteresis: v
                                                .get_field_int32("valueAlarm.hysteresis")
                                                .ok()?
                                                as u8,
                                        })
                                    })();

                                    Ok(FetchedDouble {
                                        value: v.get_field_double("value")?,
                                        alarm_severity: AlarmSeverity::from(
                                            v.get_field_int32("alarm.severity").unwrap_or(0),
                                        ),
                                        alarm_status: AlarmStatus::from(
                                            v.get_field_int32("alarm.status").unwrap_or(0),
                                        ),
                                        alarm_message: v
                                            .get_field_string("alarm.message")
                                            .unwrap_or_default(),
                                        display_metadata,
                                        control_metadata,
                                        alarm_metadata,
                                    })
                                })
                            }
                            _ => Err(PvxsError::new("PV not found or type mismatch")),
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::FetchInt32 { name, reply } => {
                        let result = match pvs.get(&name) {
                            Some(ManagedPv::Int32 { pv, .. }) => {
                                pv.fetch().and_then(|v| {
                                    // Extract display metadata if present
                                    let display_metadata = (|| -> Option<DisplayMetadata> {
                                        Some(DisplayMetadata {
                                            limit_low: v.get_field_int32("display.limitLow").ok()?
                                                as i64,
                                            limit_high: v
                                                .get_field_int32("display.limitHigh")
                                                .ok()?
                                                as i64,
                                            description: v
                                                .get_field_string("display.description")
                                                .ok()?,
                                            units: v.get_field_string("display.units").ok()?,
                                            precision: v
                                                .get_field_int32("display.precision")
                                                .ok()?,
                                        })
                                    })();

                                    // Extract control metadata if present
                                    let control_metadata = (|| -> Option<ControlMetadata> {
                                        Some(ControlMetadata {
                                            limit_low: v
                                                .get_field_double("control.limitLow")
                                                .ok()?,
                                            limit_high: v
                                                .get_field_double("control.limitHigh")
                                                .ok()?,
                                            min_step: v.get_field_double("control.minStep").ok()?,
                                        })
                                    })();

                                    // Extract alarm metadata if present
                                    let alarm_metadata = (|| -> Option<AlarmMetadata> {
                                        Some(AlarmMetadata {
                                            active: v.get_field_int32("valueAlarm.active").ok()?
                                                != 0,
                                            low_alarm_limit: v
                                                .get_field_double("valueAlarm.lowAlarmLimit")
                                                .ok()?,
                                            low_warning_limit: v
                                                .get_field_double("valueAlarm.lowWarningLimit")
                                                .ok()?,
                                            high_warning_limit: v
                                                .get_field_double("valueAlarm.highWarningLimit")
                                                .ok()?,
                                            high_alarm_limit: v
                                                .get_field_double("valueAlarm.highAlarmLimit")
                                                .ok()?,
                                            low_alarm_severity: AlarmSeverity::from(
                                                v.get_field_int32("valueAlarm.lowAlarmSeverity")
                                                    .ok()?,
                                            ),
                                            low_warning_severity: AlarmSeverity::from(
                                                v.get_field_int32("valueAlarm.lowWarningSeverity")
                                                    .ok()?,
                                            ),
                                            high_warning_severity: AlarmSeverity::from(
                                                v.get_field_int32("valueAlarm.highWarningSeverity")
                                                    .ok()?,
                                            ),
                                            high_alarm_severity: AlarmSeverity::from(
                                                v.get_field_int32("valueAlarm.highAlarmSeverity")
                                                    .ok()?,
                                            ),
                                            hysteresis: v
                                                .get_field_int32("valueAlarm.hysteresis")
                                                .ok()?
                                                as u8,
                                        })
                                    })();

                                    Ok(FetchedInt32 {
                                        value: v.get_field_int32("value")?,
                                        alarm_severity: AlarmSeverity::from(
                                            v.get_field_int32("alarm.severity").unwrap_or(0),
                                        ),
                                        alarm_status: AlarmStatus::from(
                                            v.get_field_int32("alarm.status").unwrap_or(0),
                                        ),
                                        alarm_message: v
                                            .get_field_string("alarm.message")
                                            .unwrap_or_default(),
                                        display_metadata,
                                        control_metadata,
                                        alarm_metadata,
                                    })
                                })
                            }
                            _ => Err(PvxsError::new("PV not found or type mismatch")),
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::FetchString { name, reply } => {
                        let result = match pvs.get(&name) {
                            Some(ManagedPv::String(pv)) => pv.fetch().and_then(|v| {
                                Ok(FetchedString {
                                    value: v.get_field_string("value")?,
                                    alarm_severity: AlarmSeverity::from(
                                        v.get_field_int32("alarm.severity").unwrap_or(0),
                                    ),
                                    alarm_status: AlarmStatus::from(
                                        v.get_field_int32("alarm.status").unwrap_or(0),
                                    ),
                                    alarm_message: v
                                        .get_field_string("alarm.message")
                                        .unwrap_or_default(),
                                })
                            }),
                            _ => Err(PvxsError::new("PV not found or type mismatch")),
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::FetchDoubleArray { name, reply } => {
                        let result = match pvs.get(&name) {
                            Some(ManagedPv::DoubleArray(pv)) => pv.fetch().and_then(|v| {
                                let display_metadata = (|| -> Option<DisplayMetadata> {
                                    Some(DisplayMetadata {
                                        limit_low: v.get_field_int32("display.limitLow").ok()?
                                            as i64,
                                        limit_high: v.get_field_int32("display.limitHigh").ok()?
                                            as i64,
                                        description: v
                                            .get_field_string("display.description")
                                            .ok()?,
                                        units: v.get_field_string("display.units").ok()?,
                                        precision: v.get_field_int32("display.precision").ok()?,
                                    })
                                })();
                                let control_metadata = (|| -> Option<ControlMetadata> {
                                    Some(ControlMetadata {
                                        limit_low: v.get_field_double("control.limitLow").ok()?,
                                        limit_high: v.get_field_double("control.limitHigh").ok()?,
                                        min_step: v.get_field_double("control.minStep").ok()?,
                                    })
                                })();
                                let alarm_metadata = (|| -> Option<AlarmMetadata> {
                                    Some(AlarmMetadata {
                                        active: v.get_field_int32("valueAlarm.active").ok()? != 0,
                                        low_alarm_limit: v
                                            .get_field_double("valueAlarm.lowAlarmLimit")
                                            .ok()?,
                                        low_warning_limit: v
                                            .get_field_double("valueAlarm.lowWarningLimit")
                                            .ok()?,
                                        high_warning_limit: v
                                            .get_field_double("valueAlarm.highWarningLimit")
                                            .ok()?,
                                        high_alarm_limit: v
                                            .get_field_double("valueAlarm.highAlarmLimit")
                                            .ok()?,
                                        low_alarm_severity: AlarmSeverity::from(
                                            v.get_field_int32("valueAlarm.lowAlarmSeverity")
                                                .ok()?,
                                        ),
                                        low_warning_severity: AlarmSeverity::from(
                                            v.get_field_int32("valueAlarm.lowWarningSeverity")
                                                .ok()?,
                                        ),
                                        high_warning_severity: AlarmSeverity::from(
                                            v.get_field_int32("valueAlarm.highWarningSeverity")
                                                .ok()?,
                                        ),
                                        high_alarm_severity: AlarmSeverity::from(
                                            v.get_field_int32("valueAlarm.highAlarmSeverity")
                                                .ok()?,
                                        ),
                                        hysteresis: v
                                            .get_field_int32("valueAlarm.hysteresis")
                                            .ok()?
                                            as u8,
                                    })
                                })();
                                Ok(FetchedDoubleArray {
                                    value: v.get_field_double_array("value")?,
                                    alarm_severity: AlarmSeverity::from(
                                        v.get_field_int32("alarm.severity").unwrap_or(0),
                                    ),
                                    alarm_status: AlarmStatus::from(
                                        v.get_field_int32("alarm.status").unwrap_or(0),
                                    ),
                                    alarm_message: v
                                        .get_field_string("alarm.message")
                                        .unwrap_or_default(),
                                    display_metadata,
                                    control_metadata,
                                    alarm_metadata,
                                })
                            }),
                            _ => Err(PvxsError::new("PV not found or type mismatch")),
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::FetchInt32Array { name, reply } => {
                        let result = match pvs.get(&name) {
                            Some(ManagedPv::Int32Array(pv)) => pv.fetch().and_then(|v| {
                                let display_metadata = (|| -> Option<DisplayMetadata> {
                                    Some(DisplayMetadata {
                                        limit_low: v.get_field_int32("display.limitLow").ok()?
                                            as i64,
                                        limit_high: v.get_field_int32("display.limitHigh").ok()?
                                            as i64,
                                        description: v
                                            .get_field_string("display.description")
                                            .ok()?,
                                        units: v.get_field_string("display.units").ok()?,
                                        precision: v.get_field_int32("display.precision").ok()?,
                                    })
                                })();
                                let control_metadata = (|| -> Option<ControlMetadata> {
                                    Some(ControlMetadata {
                                        limit_low: v.get_field_double("control.limitLow").ok()?,
                                        limit_high: v.get_field_double("control.limitHigh").ok()?,
                                        min_step: v.get_field_double("control.minStep").ok()?,
                                    })
                                })();
                                let alarm_metadata = (|| -> Option<AlarmMetadata> {
                                    Some(AlarmMetadata {
                                        active: v.get_field_int32("valueAlarm.active").ok()? != 0,
                                        low_alarm_limit: v
                                            .get_field_double("valueAlarm.lowAlarmLimit")
                                            .ok()?,
                                        low_warning_limit: v
                                            .get_field_double("valueAlarm.lowWarningLimit")
                                            .ok()?,
                                        high_warning_limit: v
                                            .get_field_double("valueAlarm.highWarningLimit")
                                            .ok()?,
                                        high_alarm_limit: v
                                            .get_field_double("valueAlarm.highAlarmLimit")
                                            .ok()?,
                                        low_alarm_severity: AlarmSeverity::from(
                                            v.get_field_int32("valueAlarm.lowAlarmSeverity")
                                                .ok()?,
                                        ),
                                        low_warning_severity: AlarmSeverity::from(
                                            v.get_field_int32("valueAlarm.lowWarningSeverity")
                                                .ok()?,
                                        ),
                                        high_warning_severity: AlarmSeverity::from(
                                            v.get_field_int32("valueAlarm.highWarningSeverity")
                                                .ok()?,
                                        ),
                                        high_alarm_severity: AlarmSeverity::from(
                                            v.get_field_int32("valueAlarm.highAlarmSeverity")
                                                .ok()?,
                                        ),
                                        hysteresis: v
                                            .get_field_int32("valueAlarm.hysteresis")
                                            .ok()?
                                            as u8,
                                    })
                                })();
                                Ok(FetchedInt32Array {
                                    value: v.get_field_int32_array("value")?,
                                    alarm_severity: AlarmSeverity::from(
                                        v.get_field_int32("alarm.severity").unwrap_or(0),
                                    ),
                                    alarm_status: AlarmStatus::from(
                                        v.get_field_int32("alarm.status").unwrap_or(0),
                                    ),
                                    alarm_message: v
                                        .get_field_string("alarm.message")
                                        .unwrap_or_default(),
                                    display_metadata,
                                    control_metadata,
                                    alarm_metadata,
                                })
                            }),
                            _ => Err(PvxsError::new("PV not found or type mismatch")),
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::FetchStringArray { name, reply } => {
                        let result = match pvs.get(&name) {
                            Some(ManagedPv::StringArray(pv)) => pv.fetch().and_then(|v| {
                                Ok(FetchedStringArray {
                                    value: v.get_field_string_array("value")?,
                                    alarm_severity: AlarmSeverity::from(
                                        v.get_field_int32("alarm.severity").unwrap_or(0),
                                    ),
                                    alarm_status: AlarmStatus::from(
                                        v.get_field_int32("alarm.status").unwrap_or(0),
                                    ),
                                    alarm_message: v
                                        .get_field_string("alarm.message")
                                        .unwrap_or_default(),
                                })
                            }),
                            _ => Err(PvxsError::new("PV not found or type mismatch")),
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::FetchEnum { name, reply } => {
                        let result = match pvs.get(&name) {
                            Some(ManagedPv::PvEnum(pv)) => pv.fetch().and_then(|v| {
                                Ok(FetchedEnum {
                                    value: v.get_field_enum("value.index")?,
                                    value_choices: v
                                        .get_field_string_array("value.choices")
                                        .unwrap_or_default(),
                                    alarm_severity: AlarmSeverity::from(
                                        v.get_field_int32("alarm.severity").unwrap_or(0),
                                    ),
                                    alarm_status: AlarmStatus::from(
                                        v.get_field_int32("alarm.status").unwrap_or(0),
                                    ),
                                    alarm_message: v
                                        .get_field_string("alarm.message")
                                        .unwrap_or_default(),
                                })
                            }),
                            _ => Err(PvxsError::new("PV not found or type mismatch")),
                        };
                        let _ = reply.send(result);
                    }
                    ManagerCommand::Stop { reply } => {
                        let result = server.stop();
                        let _ = reply.send(result);
                        break;
                    }
                }
            }
        });

        let (tcp_port, udp_port) = ready_rx
            .recv()
            .map_err(|_| PvxsError::new("Server failed to start"))??;

        Ok(Self {
            handle: ServerHandle {
                tx,
                tcp_port,
                udp_port,
            },
            join: Some(join),
        })
    }
}

/// A shared process variable that can be hosted by a server
///
/// SharedPVs represent individual process variables with typed values
/// that can be accessed by EPICS clients.
///
/// # Example
///
/// ```ignore
/// use pvxs_sys::SharedPV;
///
/// let mut pv = SharedPV::create_mailbox()?;
/// // Note: open_double is internal API, use Server::create_pv_* methods instead
///
/// // Update the value
/// pv.post_double(99.9)?;
///
/// // Get current value
/// let value = pv.fetch()?;
/// println!("Current value: {}", value);
/// # Ok::<(), pvxs_sys::PvxsError>(())
/// ```
pub struct SharedPV {
    inner: UniquePtr<bridge::SharedPVWrapper>,
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
    /// ```ignore
    /// # use pvxs_sys::{SharedPV, NTScalarMetadataBuilder, DisplayMetadata};
    /// // Note: open_double is internal API
    /// // Use Server::create_pv_double instead for public API
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
    ///
    /// pv.open_double(25.5, metadata)?;
    /// # Ok::<(), pvxs_sys::PvxsError>(())
    /// ```
    pub(crate) fn open_double(
        &mut self,
        initial_value: f64,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
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
    pub(crate) fn open_double_array(
        &mut self,
        initial_value: Vec<f64>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
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
    pub(crate) fn open_enum(
        &mut self,
        choices: Vec<&str>,
        selected_index: i16,
        metadata: NTEnumMetadataBuilder,
    ) -> Result<()> {
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
    pub(crate) fn open_int32(
        &mut self,
        initial_value: i32,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
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
    pub(crate) fn open_int32_array(
        &mut self,
        initial_value: Vec<i32>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
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
    pub(crate) fn open_string(
        &mut self,
        initial_value: &str,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
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
    pub(crate) fn open_string_array(
        &mut self,
        initial_value: Vec<String>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
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

    pub(crate) fn post_double_with_alarm(
        &mut self,
        value: f64,
        severity: AlarmSeverity,
        status: AlarmStatus,
        message: String,
    ) -> Result<()> {
        bridge::shared_pv_post_double_with_alarm(
            self.inner.pin_mut(),
            value,
            severity as i32,
            status as i32,
            message,
        )?;
        Ok(())
    }

    pub(crate) fn post_int32_with_alarm(
        &mut self,
        value: i32,
        severity: AlarmSeverity,
        status: AlarmStatus,
        message: String,
    ) -> Result<()> {
        bridge::shared_pv_post_int32_with_alarm(
            self.inner.pin_mut(),
            value,
            severity as i32,
            status as i32,
            message,
        )?;
        Ok(())
    }

    // TODO: TZ - Review later if needed
    /*pub(crate) fn post_enum_with_alarm(&mut self, value: i16, severity: AlarmSeverity, status: AlarmStatus, message: String) -> Result<()> {
        bridge::shared_pv_post_enum_with_alarm(self.inner.pin_mut(), value, severity as i32, status as i32, message)?;
        Ok(())
    }*/

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

/// A static source for organising collections of PVs
///
/// StaticSource allows grouping related PVs together with common
/// configuration and management.
///
/// # Example
///
/// ```ignore
/// use pvxs_sys::{StaticSource, SharedPV};
///
/// // Note: This example uses internal APIs
/// // Use Server::create_pv_* methods for public API
/// let mut source = StaticSource::create()?;
///
/// let mut temp_pv = SharedPV::create_readonly()?;
/// // temp_pv.open_double(23.5)?; // Internal API
///
/// source.add_pv("temperature", &mut temp_pv)?;
///
/// // Add source to server with priority 0
/// // server.add_source("sensors", &mut source, 0)?;
/// # Ok::<(), pvxs_sys::PvxsError>(())
/// ```
pub struct StaticSource {
    inner: UniquePtr<bridge::StaticSourceWrapper>,
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
#[derive(Debug, Clone)]
pub struct NTScalarMetadataBuilder {
    alarm_severity: AlarmSeverity,
    alarm_status: AlarmStatus,
    alarm_message: String,
    timestamp_seconds: i64,
    timestamp_nanos: i32,
    timestamp_user_tag: i32,
    display: Option<DisplayMetadata>,
    control: Option<ControlMetadata>,
    alarm_metadata: Option<AlarmMetadata>,
}

impl NTScalarMetadataBuilder {
    /// Create a new metadata builder with default values
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        Self {
            alarm_severity: AlarmSeverity::Invalid,
            alarm_status: AlarmStatus::UndefinedStatus,
            alarm_message: String::new(),
            timestamp_seconds: now.as_secs() as i64,
            timestamp_nanos: now.subsec_nanos() as i32,
            timestamp_user_tag: 0,
            display: None,
            control: None,
            alarm_metadata: None,
        }
    }

    /// Set alarm information
    pub fn alarm(
        mut self,
        severity: AlarmSeverity,
        status: AlarmStatus,
        message: impl Into<String>,
    ) -> Self {
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
    pub fn alarm_metadata(mut self, meta: AlarmMetadata) -> Self {
        self.alarm_metadata = Some(meta);
        self
    }

    /// Build the metadata using C++ builder functions with std::optional support
    fn build(self) -> Result<cxx::UniquePtr<bridge::NTScalarMetadata>> {
        // Create alarm and timestamp (always required)
        let alarm = bridge::create_alarm(
            self.alarm_severity as i32,
            self.alarm_status as i32,
            self.alarm_message,
        );
        let time_stamp = bridge::create_time(
            self.timestamp_seconds,
            self.timestamp_nanos,
            self.timestamp_user_tag,
        );

        let make_display = |d: &DisplayMetadata| {
            bridge::create_display(
                d.limit_low,
                d.limit_high,
                d.description.clone(),
                d.units.clone(),
                d.precision,
            )
        };

        // Build metadata based on which optional fields are present
        let metadata = match (&self.display, &self.control, &self.alarm_metadata) {
            (None, None, None) => bridge::create_metadata_no_optional(&alarm, &time_stamp),
            (Some(d), None, None) => {
                let display = make_display(d);
                bridge::create_metadata_with_display(&alarm, &time_stamp, &display)
            }
            (None, Some(c), None) => {
                let control = bridge::create_control(c.limit_low, c.limit_high, c.min_step);
                bridge::create_metadata_with_control(&alarm, &time_stamp, &control)
            }
            (None, None, Some(v)) => {
                let value_alarm = bridge::create_value_alarm(
                    v.active,
                    v.low_alarm_limit,
                    v.low_warning_limit,
                    v.high_warning_limit,
                    v.high_alarm_limit,
                    v.low_alarm_severity as i32,
                    v.low_warning_severity as i32,
                    v.high_warning_severity as i32,
                    v.high_alarm_severity as i32,
                    v.hysteresis,
                );
                bridge::create_metadata_with_value_alarm(&alarm, &time_stamp, &value_alarm)
            }
            (Some(d), Some(c), None) => {
                let display = make_display(d);
                let control = bridge::create_control(c.limit_low, c.limit_high, c.min_step);
                bridge::create_metadata_with_display_control(
                    &alarm,
                    &time_stamp,
                    &display,
                    &control,
                )
            }
            (Some(d), None, Some(v)) => {
                let display = make_display(d);
                let value_alarm = bridge::create_value_alarm(
                    v.active,
                    v.low_alarm_limit,
                    v.low_warning_limit,
                    v.high_warning_limit,
                    v.high_alarm_limit,
                    v.low_alarm_severity as i32,
                    v.low_warning_severity as i32,
                    v.high_warning_severity as i32,
                    v.high_alarm_severity as i32,
                    v.hysteresis,
                );
                bridge::create_metadata_with_display_value_alarm(
                    &alarm,
                    &time_stamp,
                    &display,
                    &value_alarm,
                )
            }
            (None, Some(c), Some(v)) => {
                let control = bridge::create_control(c.limit_low, c.limit_high, c.min_step);
                let value_alarm = bridge::create_value_alarm(
                    v.active,
                    v.low_alarm_limit,
                    v.low_warning_limit,
                    v.high_warning_limit,
                    v.high_alarm_limit,
                    v.low_alarm_severity as i32,
                    v.low_warning_severity as i32,
                    v.high_warning_severity as i32,
                    v.high_alarm_severity as i32,
                    v.hysteresis,
                );
                bridge::create_metadata_with_control_value_alarm(
                    &alarm,
                    &time_stamp,
                    &control,
                    &value_alarm,
                )
            }
            (Some(d), Some(c), Some(v)) => {
                let display = make_display(d);
                let control = bridge::create_control(c.limit_low, c.limit_high, c.min_step);
                let value_alarm = bridge::create_value_alarm(
                    v.active,
                    v.low_alarm_limit,
                    v.low_warning_limit,
                    v.high_warning_limit,
                    v.high_alarm_limit,
                    v.low_alarm_severity as i32,
                    v.low_warning_severity as i32,
                    v.high_warning_severity as i32,
                    v.high_alarm_severity as i32,
                    v.hysteresis,
                );
                bridge::create_metadata_full(&alarm, &time_stamp, &display, &control, &value_alarm)
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
        let alarm =
            bridge::create_alarm(self.alarm_severity, self.alarm_status, self.alarm_message);
        let time_stamp = bridge::create_time(
            self.timestamp_seconds,
            self.timestamp_nanos,
            self.timestamp_user_tag,
        );
        let metadata = bridge::create_enum_metadata(&alarm, &time_stamp);
        Ok(metadata)
    }
}

impl Default for NTEnumMetadataBuilder {
    fn default() -> Self {
        Self::new()
    }
}
