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
        type OperationWrapper; // Re-enabled for async operations
        
        // Note: RpcSourceWrapper - to be implemented later
        
        // Context creation and operations
        fn create_context_from_env() -> Result<UniquePtr<ContextWrapper>>;
        
        fn context_get_sync(
            ctx: Pin<&mut ContextWrapper>,
            pv_name: &str,
            timeout: f64,
        ) -> Result<UniquePtr<ValueWrapper>>;
        
        fn context_put_double(
            ctx: Pin<&mut ContextWrapper>,
            pv_name: &str,
            value: f64,
            timeout: f64,
        ) -> Result<()>;
        
        fn context_info_sync(
            ctx: Pin<&mut ContextWrapper>,
            pv_name: &str,
            timeout: f64,
        ) -> Result<UniquePtr<ValueWrapper>>;
        
        // Async operations using PVXS RPC
        #[allow(dead_code)]
        fn context_get_async(
            ctx: Pin<&mut ContextWrapper>,
            pv_name: &str,
            timeout: f64,
        ) -> Result<UniquePtr<OperationWrapper>>;
        
        #[allow(dead_code)]
        fn context_put_double_async(
            ctx: Pin<&mut ContextWrapper>,
            pv_name: &str,
            value: f64,
            timeout: f64,
        ) -> Result<UniquePtr<OperationWrapper>>;
        
        #[allow(dead_code)]
        fn context_info_async(
            ctx: Pin<&mut ContextWrapper>,
            pv_name: &str,
            timeout: f64,
        ) -> Result<UniquePtr<OperationWrapper>>;
        
        // Operation polling and completion
        #[allow(dead_code)]
        fn operation_is_done(op: &OperationWrapper) -> bool;
        #[allow(dead_code)]
        fn operation_get_result(op: Pin<&mut OperationWrapper>) -> Result<UniquePtr<ValueWrapper>>;
        #[allow(dead_code)]
        fn operation_cancel(op: Pin<&mut OperationWrapper>);
        #[allow(dead_code)]
        fn operation_wait_for_completion(op: Pin<&mut OperationWrapper>, timeout_ms: u64) -> bool;
        
        // RPC operations
        type RpcWrapper;
        type MonitorWrapper;
        fn context_rpc_create(
            ctx: Pin<&mut ContextWrapper>,
            pv_name: String,
        ) -> Result<UniquePtr<RpcWrapper>>;
        
        fn rpc_arg_string(rpc: Pin<&mut RpcWrapper>, name: String, value: String) -> Result<()>;
        fn rpc_arg_double(rpc: Pin<&mut RpcWrapper>, name: String, value: f64) -> Result<()>;
        fn rpc_arg_int32(rpc: Pin<&mut RpcWrapper>, name: String, value: i32) -> Result<()>;
        fn rpc_arg_bool(rpc: Pin<&mut RpcWrapper>, name: String, value: bool) -> Result<()>;
        
        fn rpc_execute_sync(rpc: Pin<&mut RpcWrapper>, timeout: f64) -> Result<UniquePtr<ValueWrapper>>;
        #[allow(dead_code)]
        fn rpc_execute_async(rpc: Pin<&mut RpcWrapper>, timeout: f64) -> Result<UniquePtr<OperationWrapper>>;
        
        // Value inspection
        fn value_is_valid(val: &ValueWrapper) -> bool;
        fn value_to_string(val: &ValueWrapper) -> String;
        fn value_get_field_double(val: &ValueWrapper, field_name: String) -> Result<f64>;
        fn value_get_field_int32(val: &ValueWrapper, field_name: String) -> Result<i32>;
        fn value_get_field_string(val: &ValueWrapper, field_name: String) -> Result<String>;
        
        // Monitor operations
        fn context_monitor_create(
            ctx: Pin<&mut ContextWrapper>,
            pv_name: String,
        ) -> Result<UniquePtr<MonitorWrapper>>;
        fn monitor_start(monitor: Pin<&mut MonitorWrapper>);
        fn monitor_stop(monitor: Pin<&mut MonitorWrapper>);
        fn monitor_is_running(monitor: &MonitorWrapper) -> bool;
        fn monitor_has_update(monitor: &MonitorWrapper) -> bool;
        fn monitor_get_update(monitor: Pin<&mut MonitorWrapper>, timeout: f64) -> Result<UniquePtr<ValueWrapper>>;
        fn monitor_try_get_update(monitor: Pin<&mut MonitorWrapper>) -> Result<UniquePtr<ValueWrapper>>;
        fn monitor_is_connected(monitor: &MonitorWrapper) -> bool;
        fn monitor_get_name(monitor: &MonitorWrapper) -> String;
        
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
        fn shared_pv_open_double(pv: Pin<&mut SharedPVWrapper>, initial_value: f64) -> Result<()>;
        fn shared_pv_open_int32(pv: Pin<&mut SharedPVWrapper>, initial_value: i32) -> Result<()>;
        fn shared_pv_open_string(pv: Pin<&mut SharedPVWrapper>, initial_value: String) -> Result<()>;
        fn shared_pv_is_open(pv: &SharedPVWrapper) -> bool;
        fn shared_pv_close(pv: Pin<&mut SharedPVWrapper>) -> Result<()>;
        fn shared_pv_post_double(pv: Pin<&mut SharedPVWrapper>, value: f64) -> Result<()>;
        fn shared_pv_post_int32(pv: Pin<&mut SharedPVWrapper>, value: i32) -> Result<()>;
        fn shared_pv_post_string(pv: Pin<&mut SharedPVWrapper>, value: String) -> Result<()>;
        fn shared_pv_post_double_with_alarm(pv: Pin<&mut SharedPVWrapper>, value: f64, severity: i32, status: i32, message: String) -> Result<()>;
        fn shared_pv_post_int32_with_alarm(pv: Pin<&mut SharedPVWrapper>, value: i32, severity: i32, status: i32, message: String) -> Result<()>;
        fn shared_pv_post_string_with_alarm(pv: Pin<&mut SharedPVWrapper>, value: String, severity: i32, status: i32, message: String) -> Result<()>;
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
