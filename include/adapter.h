// adapter.h - C++ adapter layer to simplify PVXS for Rust FFI
// This layer handles the complex C++ patterns (callbacks, shared_ptr, etc.)

#pragma once

#include <memory>
#include <string>
#include <stdexcept>
#include "rust/cxx.h"  // For rust::String and rust::Str types
#include <pvxs/client.h>
#include <pvxs/nt.h>

namespace pvxs_adapter {

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
    
    // Get PV name
    std::string name() const;
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
    std::unique_ptr<OperationWrapper> get_async(const std::string& pv_name);
    
    // Perform a PUT operation (simplified - just set a double value)
    void put_double(const std::string& pv_name, double value, double timeout);
    
    // Perform a PUT operation (simplified - just set an int32 value)
    void put_int32(const std::string& pv_name, int32_t value, double timeout);
    
    // Get type information (INFO operation)
    std::unique_ptr<ValueWrapper> info_sync(const std::string& pv_name, double timeout);
};

// Factory functions for Rust (these will be exposed via cxx bridge)
std::unique_ptr<ContextWrapper> create_context_from_env();
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

// Value accessors for Rust
bool value_is_valid(const ValueWrapper& val);
rust::String value_to_string(const ValueWrapper& val);
double value_get_field_double(const ValueWrapper& val, rust::Str field_name);
int32_t value_get_field_int32(const ValueWrapper& val, rust::Str field_name);
rust::String value_get_field_string(const ValueWrapper& val, rust::Str field_name);

} // namespace pvxs_adapter
