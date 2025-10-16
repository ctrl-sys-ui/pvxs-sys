// wrapper.h - C++ wrapper layer to simplify PVXS for Rust FFI
// This layer handles the complex C++ patterns (callbacks, shared_ptr, etc.)

#pragma once

#include <memory>
#include <string>
#include <stdexcept>
#include "rust/cxx.h"  // For rust::String and rust::Str types
#include <pvxs/client.h>
#include <pvxs/nt.h>

namespace pvxs_wrapper {

// Forward declarations
class ContextWrapper;
class OperationWrapper;
class ValueWrapper;

/// Exception wrapper for Rust-friendly error handling
class PvxsError : public std::runtime_error {
public:
    explicit PvxsError(const std::string& msg) : std::runtime_error(msg) {}
};

/// Wraps pvxs::Value for safe Rust access
class ValueWrapper {
private:
    pvxs::Value value_;

public:
    ValueWrapper() = default;
    explicit ValueWrapper(pvxs::Value&& val) : value_(std::move(val)) {}
    
    // Check if value is valid
    bool valid() const { return value_.valid(); }
    
    // Get field as string (simplified for now)
    std::string get_field_string(const std::string& field_name) const;
    
    // Get field as double
    double get_field_double(const std::string& field_name) const;
    
    // Get field as int32
    int32_t get_field_int32(const std::string& field_name) const;
    
    // Convert entire value to string representation
    std::string to_string() const;
    
    // Get the underlying pvxs::Value (internal use)
    pvxs::Value& get() { return value_; }
    const pvxs::Value& get() const { return value_; }
};

/// Wraps pvxs::client::Operation for safe Rust access
class OperationWrapper {
private:
    std::shared_ptr<pvxs::client::Operation> op_;
    
public:
    OperationWrapper() = default;
    explicit OperationWrapper(std::shared_ptr<pvxs::client::Operation>&& op) 
        : op_(std::move(op)) {}
    
    // Wait for operation to complete (blocking)
    std::unique_ptr<ValueWrapper> wait(double timeout) const;
    
    // Cancel the operation
    bool cancel() const;
    
    // Check if operation is complete
    bool is_done() const;
    
    // Get result (non-blocking, throws if not ready)
    std::unique_ptr<ValueWrapper> get_result();
    
    // Wait for completion with timeout (returns true if completed)
    bool wait_for_completion(uint64_t timeout_ms);
    
    // Get PV name
    std::string name() const;
};

/// Wraps pvxs::client::Subscription for safe Rust access
class MonitorWrapper {
private:
    std::shared_ptr<pvxs::client::Subscription> monitor_;
    std::string pv_name_;
    
public:
    MonitorWrapper() = default;
    explicit MonitorWrapper(std::shared_ptr<pvxs::client::Subscription>&& monitor, const std::string& pv_name) 
        : monitor_(std::move(monitor)), pv_name_(pv_name) {}
    
    // Start monitoring
    void start();
    
    // Stop monitoring
    void stop();
    
    // Check if monitor is running
    bool is_running() const;
    
    // Check if there are updates available (non-blocking)
    bool has_update() const;
    
    // Get the next update (blocking with timeout)
    std::unique_ptr<ValueWrapper> get_update(double timeout);
    
    // Get the next update (non-blocking, returns nullptr if no update)
    std::unique_ptr<ValueWrapper> try_get_update();
    
    // Get PV name
    std::string name() const { return pv_name_; }
    
    // Get connection status
    bool is_connected() const;
};

/// Wraps pvxs::client::Context for safe Rust access
class ContextWrapper {
private:
    pvxs::client::Context ctx_;
    
public:
    // Create context from environment variables
    static std::unique_ptr<ContextWrapper> from_env();
    
    // Create context with explicit configuration
    explicit ContextWrapper(pvxs::client::Context&& ctx) 
        : ctx_(std::move(ctx)) {}
    
    // Perform a GET operation (synchronous version for simplicity)
    std::unique_ptr<ValueWrapper> get_sync(const std::string& pv_name, double timeout);
    
    // Start an async GET operation
    std::unique_ptr<OperationWrapper> get_async(const std::string& pv_name, double timeout);
    
    // Start an async PUT operation
    std::unique_ptr<OperationWrapper> put_double_async(const std::string& pv_name, double value, double timeout);
    
    // Start an async INFO operation
    std::unique_ptr<OperationWrapper> info_async(const std::string& pv_name, double timeout);
    
    // Perform a PUT operation (simplified - just set a double value)
    void put_double(const std::string& pv_name, double value, double timeout);
    
    // Perform a PUT operation (simplified - just set an int32 value)
    void put_int32(const std::string& pv_name, int32_t value, double timeout);
    
    // Get type information (INFO operation)
    std::unique_ptr<ValueWrapper> info_sync(const std::string& pv_name, double timeout);
    
    // Create RPC builder
    std::unique_ptr<class RpcWrapper> rpc_create(const std::string& pv_name);
    
    // Create Monitor
    std::unique_ptr<MonitorWrapper> monitor(const std::string& pv_name);
};

/// Wraps pvxs::client::RPCBuilder for safe RPC operations
class RpcWrapper {
private:
    pvxs::client::RPCBuilder builder_;
    
public:
    explicit RpcWrapper(pvxs::client::RPCBuilder&& builder) 
        : builder_(std::move(builder)) {}
    
    // Add arguments to the RPC call
    void arg_string(const std::string& name, const std::string& value);
    void arg_double(const std::string& name, double value);
    void arg_int32(const std::string& name, int32_t value);
    void arg_bool(const std::string& name, bool value);
    
    // Execute RPC call synchronously
    std::unique_ptr<ValueWrapper> execute_sync(double timeout);
    
    // Execute RPC call asynchronously
    std::unique_ptr<OperationWrapper> execute_async(double timeout);
};

// Factory functions for Rust (these will be exposed via cxx bridge)
std::unique_ptr<ContextWrapper> create_context_from_env();

// RPC operations bridge functions
std::unique_ptr<RpcWrapper> context_rpc_create(
    ContextWrapper& ctx,
    rust::Str pv_name);

void rpc_arg_string(RpcWrapper& rpc, rust::Str name, rust::Str value);
void rpc_arg_double(RpcWrapper& rpc, rust::Str name, double value);
void rpc_arg_int32(RpcWrapper& rpc, rust::Str name, int32_t value);
void rpc_arg_bool(RpcWrapper& rpc, rust::Str name, bool value);

std::unique_ptr<ValueWrapper> rpc_execute_sync(RpcWrapper& rpc, double timeout);
std::unique_ptr<OperationWrapper> rpc_execute_async(RpcWrapper& rpc, double timeout);
std::unique_ptr<ValueWrapper> context_get_sync(
    ContextWrapper& ctx, 
    rust::Str pv_name, 
    double timeout);
void context_put_double(
    ContextWrapper& ctx,
    rust::Str pv_name,
    double value,
    double timeout);
std::unique_ptr<ValueWrapper> context_info_sync(
    ContextWrapper& ctx,
    rust::Str pv_name,
    double timeout);

// Async operations for Rust
std::unique_ptr<OperationWrapper> context_get_async(
    ContextWrapper& ctx,
    rust::Str pv_name,
    double timeout);
std::unique_ptr<OperationWrapper> context_put_double_async(
    ContextWrapper& ctx,
    rust::Str pv_name,
    double value,
    double timeout);
std::unique_ptr<OperationWrapper> context_info_async(
    ContextWrapper& ctx,
    rust::Str pv_name,
    double timeout);

// Operation management for Rust
bool operation_is_done(const OperationWrapper& op);
std::unique_ptr<ValueWrapper> operation_get_result(OperationWrapper& op);
void operation_cancel(OperationWrapper& op);
bool operation_wait_for_completion(OperationWrapper& op, uint64_t timeout_ms);

// Value accessors for Rust
bool value_is_valid(const ValueWrapper& val);
rust::String value_to_string(const ValueWrapper& val);
double value_get_field_double(const ValueWrapper& val, rust::Str field_name);
int32_t value_get_field_int32(const ValueWrapper& val, rust::Str field_name);
rust::String value_get_field_string(const ValueWrapper& val, rust::Str field_name);

// Monitor operations for Rust
std::unique_ptr<MonitorWrapper> context_monitor_create(
    ContextWrapper& ctx,
    rust::Str pv_name);
void monitor_start(MonitorWrapper& monitor);
void monitor_stop(MonitorWrapper& monitor);
bool monitor_is_running(const MonitorWrapper& monitor);
bool monitor_has_update(const MonitorWrapper& monitor);
std::unique_ptr<ValueWrapper> monitor_get_update(MonitorWrapper& monitor, double timeout);
std::unique_ptr<ValueWrapper> monitor_try_get_update(MonitorWrapper& monitor);
bool monitor_is_connected(const MonitorWrapper& monitor);
rust::String monitor_get_name(const MonitorWrapper& monitor);

} // namespace pvxs_wrapper
