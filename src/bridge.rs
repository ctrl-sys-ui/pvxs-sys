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
            pv_name: &str,
        ) -> Result<UniquePtr<RpcWrapper>>;
        
        fn rpc_arg_string(rpc: Pin<&mut RpcWrapper>, name: &str, value: &str) -> Result<()>;
        fn rpc_arg_double(rpc: Pin<&mut RpcWrapper>, name: &str, value: f64) -> Result<()>;
        fn rpc_arg_int32(rpc: Pin<&mut RpcWrapper>, name: &str, value: i32) -> Result<()>;
        fn rpc_arg_bool(rpc: Pin<&mut RpcWrapper>, name: &str, value: bool) -> Result<()>;
        
        fn rpc_execute_sync(rpc: Pin<&mut RpcWrapper>, timeout: f64) -> Result<UniquePtr<ValueWrapper>>;
        #[allow(dead_code)]
        fn rpc_execute_async(rpc: Pin<&mut RpcWrapper>, timeout: f64) -> Result<UniquePtr<OperationWrapper>>;
        
        // Value inspection
        fn value_is_valid(val: &ValueWrapper) -> bool;
        fn value_to_string(val: &ValueWrapper) -> String;
        fn value_get_field_double(val: &ValueWrapper, field_name: &str) -> Result<f64>;
        fn value_get_field_int32(val: &ValueWrapper, field_name: &str) -> Result<i32>;
        fn value_get_field_string(val: &ValueWrapper, field_name: &str) -> Result<String>;
        
        // Monitor operations
        fn context_monitor_create(
            ctx: Pin<&mut ContextWrapper>,
            pv_name: &str,
        ) -> Result<UniquePtr<MonitorWrapper>>;
        fn monitor_start(monitor: Pin<&mut MonitorWrapper>);
        fn monitor_stop(monitor: Pin<&mut MonitorWrapper>);
        fn monitor_is_running(monitor: &MonitorWrapper) -> bool;
        fn monitor_has_update(monitor: &MonitorWrapper) -> bool;
        fn monitor_get_update(monitor: Pin<&mut MonitorWrapper>, timeout: f64) -> Result<UniquePtr<ValueWrapper>>;
        fn monitor_try_get_update(monitor: Pin<&mut MonitorWrapper>) -> Result<UniquePtr<ValueWrapper>>;
        fn monitor_is_connected(monitor: &MonitorWrapper) -> bool;
        fn monitor_get_name(monitor: &MonitorWrapper) -> String;
    }
}

// Re-export the FFI types for use in the public API
pub use ffi::*;
