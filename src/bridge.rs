// bridge.rs - CXX bridge definition for Rust/C++ FFI
// This defines the interface between Rust and C++

#[cxx::bridge(namespace = "pvxs_adapter")]
mod ffi {
    // Opaque C++ types - Rust sees these as opaque pointers
    unsafe extern "C++" {
        include!("adapter.h");
        
        // C++ types that Rust can hold but not inspect
        type ContextWrapper;
        type ValueWrapper;
        type OperationWrapper;
        
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
        
        // Value inspection
        fn value_is_valid(val: &ValueWrapper) -> bool;
        fn value_to_string(val: &ValueWrapper) -> String;
        fn value_get_field_double(val: &ValueWrapper, field_name: &str) -> Result<f64>;
        fn value_get_field_int32(val: &ValueWrapper, field_name: &str) -> Result<i32>;
        fn value_get_field_string(val: &ValueWrapper, field_name: &str) -> Result<String>;
    }
}

// Re-export the FFI types for use in the public API
pub use ffi::*;
