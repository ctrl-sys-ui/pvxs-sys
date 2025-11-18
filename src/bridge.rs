// bridge.rs - CXX bridge definition for Rust/C++ FFI
// This defines the interface between Rust and C++

#[cxx::bridge(namespace = "pvxs_wrapper")]
mod ffi {
    
    // Opaque C++ types - Rust sees these as opaque pointers

    unsafe extern "C++" {
        include!("wrapper.h");
        
        // C++ types that Rust can hold but not inspect
        type ContextWrapper;
        type ValueWrapper;
        #[cfg(feature = "async")]
        type OperationWrapper; // Re-enabled for async operations
        
        // Metadata structs (defined in C++ with std::optional)
        type NTScalarAlarm;
        type NTScalarTime;
        type NTScalarDisplay;
        type NTScalarControl;
        type NTScalarValueAlarm;
        type NTScalarMetadata;
        type NTEnumMetadata;
        
        // Metadata builder functions - construct metadata from Rust
        fn create_alarm(severity: i32, status: i32, message: String) -> UniquePtr<NTScalarAlarm>;
        fn create_time(seconds_past_epoch: i64, nanoseconds: i32, user_tag: i32) -> UniquePtr<NTScalarTime>;
        fn create_display(limit_low: i64, limit_high: i64, description: String, units: String, precision: i32) -> UniquePtr<NTScalarDisplay>;
        fn create_control(limit_low: f64, limit_high: f64, min_step: f64) -> UniquePtr<NTScalarControl>;
        fn create_value_alarm(active: bool, low_alarm_limit: f64, low_warning_limit: f64, 
                             high_warning_limit: f64, high_alarm_limit: f64,
                             low_alarm_severity: i32, low_warning_severity: i32,
                             high_warning_severity: i32, high_alarm_severity: i32, hysteresis: u8) -> UniquePtr<NTScalarValueAlarm>;
        
        // Helper functions to build metadata with optional fields
        fn create_metadata_no_optional(alarm: &NTScalarAlarm, time_stamp: &NTScalarTime, has_form: bool) -> UniquePtr<NTScalarMetadata>;
        fn create_metadata_with_display(alarm: &NTScalarAlarm, time_stamp: &NTScalarTime, display: &NTScalarDisplay, has_form: bool) -> UniquePtr<NTScalarMetadata>;
        fn create_metadata_with_control(alarm: &NTScalarAlarm, time_stamp: &NTScalarTime, control: &NTScalarControl, has_form: bool) -> UniquePtr<NTScalarMetadata>;
        fn create_metadata_with_value_alarm(alarm: &NTScalarAlarm, time_stamp: &NTScalarTime, value_alarm: &NTScalarValueAlarm, has_form: bool) -> UniquePtr<NTScalarMetadata>;
        fn create_metadata_with_display_control(alarm: &NTScalarAlarm, time_stamp: &NTScalarTime, display: &NTScalarDisplay, control: &NTScalarControl, has_form: bool) -> UniquePtr<NTScalarMetadata>;
        fn create_metadata_with_display_value_alarm(alarm: &NTScalarAlarm, time_stamp: &NTScalarTime, display: &NTScalarDisplay, value_alarm: &NTScalarValueAlarm, has_form: bool) -> UniquePtr<NTScalarMetadata>;
        fn create_metadata_with_control_value_alarm(alarm: &NTScalarAlarm, time_stamp: &NTScalarTime, control: &NTScalarControl, value_alarm: &NTScalarValueAlarm, has_form: bool) -> UniquePtr<NTScalarMetadata>;
        fn create_metadata_full(alarm: &NTScalarAlarm, time_stamp: &NTScalarTime, display: &NTScalarDisplay, control: &NTScalarControl, value_alarm: &NTScalarValueAlarm, has_form: bool) -> UniquePtr<NTScalarMetadata>;

        fn create_enum_metadata(alarm: &NTScalarAlarm, time_stamp: &NTScalarTime, enum_choices: Vec<String>) -> UniquePtr<NTEnumMetadata>;
        
        // Note: RpcSourceWrapper - to be implemented later
        
        // Context creation and operations
        fn create_context_from_env() -> Result<UniquePtr<ContextWrapper>>;
        fn context_get(ctx: Pin<&mut ContextWrapper>, pv_name: &str, timeout: f64,) -> Result<UniquePtr<ValueWrapper>>;
        fn context_put_double(ctx: Pin<&mut ContextWrapper>, pv_name: &str, value: f64, timeout: f64,) -> Result<()>;
        fn context_put_int32(ctx: Pin<&mut ContextWrapper>, pv_name: &str, value: i32, timeout: f64,) -> Result<()>;
        fn context_put_string(ctx: Pin<&mut ContextWrapper>, pv_name: &str, value: String, timeout: f64,) -> Result<()>;
        fn context_put_enum(ctx: Pin<&mut ContextWrapper>, pv_name: &str, value: i16, timeout: f64,) -> Result<()>;
        fn context_put_double_array(ctx: Pin<&mut ContextWrapper>, pv_name: &str, value: Vec<f64>, timeout: f64,) -> Result<()>;
        fn context_put_int32_array(ctx: Pin<&mut ContextWrapper>, pv_name: &str, value: Vec<i32>, timeout: f64,) -> Result<()>;
        fn context_put_string_array(ctx: Pin<&mut ContextWrapper>, pv_name: &str, value: Vec<String>, timeout: f64,) -> Result<()>;
        fn context_info(ctx: Pin<&mut ContextWrapper>, pv_name: &str, timeout: f64,) -> Result<UniquePtr<ValueWrapper>>;

        // Value inspection
        fn value_is_valid(val: &ValueWrapper) -> bool;
        fn value_to_string(val: &ValueWrapper) -> String;
        fn value_get_field_double(val: &ValueWrapper, field_name: String) -> Result<f64>;
        fn value_get_field_int32(val: &ValueWrapper, field_name: String) -> Result<i32>;
        fn value_get_field_string(val: &ValueWrapper, field_name: String) -> Result<String>;
        fn value_get_field_enum(val: &ValueWrapper, field_name: String) -> Result<i16>;
        fn value_get_field_double_array(val: &ValueWrapper, field_name: String) -> Result<Vec<f64>>;
        fn value_get_field_int32_array(val: &ValueWrapper, field_name: String) -> Result<Vec<i32>>;
        fn value_get_field_string_array(val: &ValueWrapper, field_name: String) -> Result<Vec<String>>;
        
        // Monitor operations
        fn context_monitor_create(ctx: Pin<&mut ContextWrapper>, pv_name: String,) -> Result<UniquePtr<MonitorWrapper>>;
        fn monitor_start(monitor: Pin<&mut MonitorWrapper>);
        fn monitor_stop(monitor: Pin<&mut MonitorWrapper>);
        fn monitor_is_running(monitor: &MonitorWrapper) -> bool;
        fn monitor_has_update(monitor: &MonitorWrapper) -> bool;
        fn monitor_get_update(monitor: Pin<&mut MonitorWrapper>, timeout: f64) -> Result<UniquePtr<ValueWrapper>>;
        fn monitor_try_get_update(monitor: Pin<&mut MonitorWrapper>) -> Result<UniquePtr<ValueWrapper>>;
        fn monitor_is_connected(monitor: &MonitorWrapper) -> bool;
        fn monitor_get_name(monitor: &MonitorWrapper) -> String;
        fn monitor_pop(monitor: Pin<&mut MonitorWrapper>) -> Result<UniquePtr<ValueWrapper>>;
        
        // MonitorBuilder operations
        fn context_monitor_builder_create(ctx: Pin<&mut ContextWrapper>, pv_name: String) -> Result<UniquePtr<MonitorBuilderWrapper>>;
        fn monitor_builder_mask_connected(builder: Pin<&mut MonitorBuilderWrapper>, mask: bool) -> Result<()>;
        fn monitor_builder_mask_disconnected(builder: Pin<&mut MonitorBuilderWrapper>, mask: bool) -> Result<()>;
        fn monitor_builder_set_event_callback(builder: Pin<&mut MonitorBuilderWrapper>, callback_ptr: usize) -> Result<()>;
        fn monitor_builder_exec(builder: Pin<&mut MonitorBuilderWrapper>) -> Result<UniquePtr<MonitorWrapper>>;
        fn monitor_builder_exec_with_callback(builder: Pin<&mut MonitorBuilderWrapper>, callback_id: u64) -> Result<UniquePtr<MonitorWrapper>>;
        
        // Async operations using PVXS RPC (only available with async feature)
        #[cfg(feature = "async")]
        #[allow(dead_code)]
        fn context_get_async(ctx: Pin<&mut ContextWrapper>, pv_name: &str, timeout: f64,) -> Result<UniquePtr<OperationWrapper>>;
        
        #[cfg(feature = "async")]
        #[allow(dead_code)]
        fn context_put_double_async(ctx: Pin<&mut ContextWrapper>, pv_name: &str, value: f64, timeout: f64,) -> Result<UniquePtr<OperationWrapper>>;
        
        #[cfg(feature = "async")]
        #[allow(dead_code)]
        fn context_info_async(ctx: Pin<&mut ContextWrapper>, pv_name: &str, timeout: f64,) -> Result<UniquePtr<OperationWrapper>>;
        
        // Operation polling and completion (only available with async feature)
        #[cfg(feature = "async")]
        #[allow(dead_code)]
        fn operation_is_done(op: &OperationWrapper) -> bool;
        #[cfg(feature = "async")]
        #[allow(dead_code)]
        fn operation_get_result(op: Pin<&mut OperationWrapper>) -> Result<UniquePtr<ValueWrapper>>;
        #[cfg(feature = "async")]
        #[allow(dead_code)]
        fn operation_cancel(op: Pin<&mut OperationWrapper>);
        #[cfg(feature = "async")]
        #[allow(dead_code)]
        fn operation_wait_for_completion(op: Pin<&mut OperationWrapper>, timeout_ms: u64) -> bool;
        
        // RPC operations
        type RpcWrapper;
        type MonitorWrapper;
        type MonitorBuilderWrapper;
        
        fn context_rpc_create(
            ctx: Pin<&mut ContextWrapper>,
            pv_name: String,
        ) -> Result<UniquePtr<RpcWrapper>>;
        
        fn rpc_arg_string(rpc: Pin<&mut RpcWrapper>, name: String, value: String) -> Result<()>;
        fn rpc_arg_double(rpc: Pin<&mut RpcWrapper>, name: String, value: f64) -> Result<()>;
        fn rpc_arg_int32(rpc: Pin<&mut RpcWrapper>, name: String, value: i32) -> Result<()>;
        fn rpc_arg_bool(rpc: Pin<&mut RpcWrapper>, name: String, value: bool) -> Result<()>;
        
        fn rpc_execute_sync(rpc: Pin<&mut RpcWrapper>, timeout: f64) -> Result<UniquePtr<ValueWrapper>>;
        #[cfg(feature = "async")]
        #[allow(dead_code)]
        fn rpc_execute_async(rpc: Pin<&mut RpcWrapper>, timeout: f64) -> Result<UniquePtr<OperationWrapper>>;
        
        
        
        // ====================================================================
        // Server-side types and operations
        // ====================================================================
        
        // Server wrapper types
        type ServerWrapper;
        type SharedPVWrapper;
        type StaticSourceWrapper;
        
        // Server creation and management
        fn server_create_from_env() -> Result<UniquePtr<ServerWrapper>>;
        fn server_create_isolated() -> Result<UniquePtr<ServerWrapper>>;
        fn server_start(server: Pin<&mut ServerWrapper>) -> Result<()>;
        fn server_stop(server: Pin<&mut ServerWrapper>) -> Result<()>;
        fn server_add_pv(server: Pin<&mut ServerWrapper>, name: String, pv: Pin<&mut SharedPVWrapper>) -> Result<()>;
        fn server_remove_pv(server: Pin<&mut ServerWrapper>, name: String) -> Result<()>;
        fn server_add_source(server: Pin<&mut ServerWrapper>, name: String, source: Pin<&mut StaticSourceWrapper>, order: i32) -> Result<()>;
        // Note: server_add_rpc_source - to be implemented later
        fn server_get_tcp_port(server: &ServerWrapper) -> u16;
        fn server_get_udp_port(server: &ServerWrapper) -> u16;
        
        // SharedPV creation and operations
        fn shared_pv_create_mailbox() -> Result<UniquePtr<SharedPVWrapper>>;
        fn shared_pv_create_readonly() -> Result<UniquePtr<SharedPVWrapper>>;
        fn shared_pv_open_double(pv: Pin<&mut SharedPVWrapper>, initial_value: f64, metadata: &NTScalarMetadata) -> Result<()>;
        fn shared_pv_open_double_array(pv: Pin<&mut SharedPVWrapper>, initial_value: Vec<f64>, metadata: &NTScalarMetadata) -> Result<()>;
        fn shared_pv_open_int32(pv: Pin<&mut SharedPVWrapper>, initial_value: i32) -> Result<()>;
        fn shared_pv_open_string(pv: Pin<&mut SharedPVWrapper>, initial_value: String) -> Result<()>;
        fn shared_pv_open_enum(pv: Pin<&mut SharedPVWrapper>, choices: Vec<String>, selected_value: i16, metadata: &NTEnumMetadata) -> Result<()>;
        fn shared_pv_is_open(pv: &SharedPVWrapper) -> bool;
        fn shared_pv_close(pv: Pin<&mut SharedPVWrapper>) -> Result<()>;
        fn shared_pv_post_double(pv: Pin<&mut SharedPVWrapper>, value: f64) -> Result<()>;
        fn shared_pv_post_int32(pv: Pin<&mut SharedPVWrapper>, value: i32) -> Result<()>;
        fn shared_pv_post_string(pv: Pin<&mut SharedPVWrapper>, value: String) -> Result<()>;
        fn shared_pv_post_enum(pv: Pin<&mut SharedPVWrapper>, value: i16) -> Result<()>;
        fn shared_pv_fetch(pv: &SharedPVWrapper) -> Result<UniquePtr<ValueWrapper>>;
        
        // StaticSource creation and operations
        fn static_source_create() -> Result<UniquePtr<StaticSourceWrapper>>;
        fn static_source_add_pv(source: Pin<&mut StaticSourceWrapper>, name: String, pv: Pin<&mut SharedPVWrapper>) -> Result<()>;
        fn static_source_remove_pv(source: Pin<&mut StaticSourceWrapper>, name: String) -> Result<()>;
        fn static_source_close_all(source: Pin<&mut StaticSourceWrapper>) -> Result<()>;
        
        // Note: RpcSource creation operations - to be implemented later
    }
}

// Re-export the FFI types for use in the public API
pub use ffi::*;
