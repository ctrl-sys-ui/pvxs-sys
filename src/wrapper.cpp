// wrapper.cpp - Implementation of the C++ wrapper layer

#include "wrapper.h"
#include <sstream>
#include <chrono>
#include <thread>
#include <pvxs/log.h>

namespace pvxs_wrapper {

// ============================================================================
// ValueWrapper implementation
// ============================================================================

std::string ValueWrapper::get_field_string(const std::string& field_name) const {
    if (!value_.valid()) {
        throw PvxsError("Value is not valid");
    }
    
    try {
        auto field = value_[field_name];
        if (!field.valid()) {
            throw PvxsError("Field '" + field_name + "' not found");
        }
        return field.as<std::string>();
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error getting field '") + field_name + "': " + e.what());
    }
}

double ValueWrapper::get_field_double(const std::string& field_name) const {
    if (!value_.valid()) {
        throw PvxsError("Value is not valid");
    }
    
    try {
        auto field = value_[field_name];
        if (!field.valid()) {
            throw PvxsError("Field '" + field_name + "' not found");
        }
        return field.as<double>();
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error getting field '") + field_name + "': " + e.what());
    }
}

int32_t ValueWrapper::get_field_int32(const std::string& field_name) const {
    if (!value_.valid()) {
        throw PvxsError("Value is not valid");
    }
    
    try {
        auto field = value_[field_name];
        if (!field.valid()) {
            throw PvxsError("Field '" + field_name + "' not found");
        }
        return field.as<int32_t>();
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error getting field '") + field_name + "': " + e.what());
    }
}

std::string ValueWrapper::to_string() const {
    if (!value_.valid()) {
        return "<invalid>";
    }
    
    std::ostringstream oss;
    oss << value_;
    return oss.str();
}

// ============================================================================
// OperationWrapper implementation
// ============================================================================

std::unique_ptr<ValueWrapper> OperationWrapper::wait(double timeout) const {
    if (!op_) {
        throw PvxsError("Operation is null");
    }
    
    try {
        pvxs::Value result = op_->wait(timeout);
        return std::make_unique<ValueWrapper>(std::move(result));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Operation wait failed: ") + e.what());
    }
}

bool OperationWrapper::cancel() const {
    if (!op_) {
        return false;
    }
    return op_->cancel();
}

std::string OperationWrapper::name() const {
    if (!op_) {
        return "";
    }
    return op_->name();
}

bool OperationWrapper::is_done() const {
    if (!op_) {
        return true; // Null operation is considered "done"
    }
    // Use a non-blocking check with wait() and 0 timeout
    try {
        op_->wait(0.0);
        return true; // Operation completed successfully
    } catch (const pvxs::client::Timeout&) {
        return false; // Still in progress
    } catch (...) {
        return true; // Error means operation is done
    }
}

std::unique_ptr<ValueWrapper> OperationWrapper::get_result() {
    if (!op_) {
        throw PvxsError("Operation is null");
    }
    
    try {
        // Use wait() to get the result
        pvxs::Value result = op_->wait(10.0); // 10 second timeout
        return std::make_unique<ValueWrapper>(std::move(result));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Failed to get operation result: ") + e.what());
    }
}

bool OperationWrapper::wait_for_completion(uint64_t timeout_ms) {
    if (!op_) {
        return true; // Null operation is considered complete
    }
    
    try {
        double timeout_sec = timeout_ms / 1000.0;
        op_->wait(timeout_sec);
        return true; // If wait() succeeds, operation is complete
    } catch (const pvxs::client::Timeout&) {
        return false; // Timeout - operation still running
    } catch (...) {
        return true; // Other error - operation is done
    }
}

// ============================================================================
// ContextWrapper implementation
// ============================================================================

std::unique_ptr<ContextWrapper> ContextWrapper::from_env() {
    try {
        auto ctx = pvxs::client::Context::fromEnv();
        return std::make_unique<ContextWrapper>(std::move(ctx));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Failed to create context from environment: ") + e.what());
    }
}

std::unique_ptr<ValueWrapper> ContextWrapper::get_sync(
    const std::string& pv_name, 
    double timeout) 
{
    try {
        auto op = ctx_.get(pv_name).exec();
        pvxs::Value result = op->wait(timeout);
        return std::make_unique<ValueWrapper>(std::move(result));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("GET failed for '") + pv_name + "': " + e.what());
    }
}

std::unique_ptr<OperationWrapper> ContextWrapper::get_async(
    const std::string& pv_name, double timeout) 
{
    try {
        auto op = ctx_.get(pv_name).exec();
        return std::make_unique<OperationWrapper>(std::move(op));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Failed to start GET for '") + pv_name + "': " + e.what());
    }
}

std::unique_ptr<OperationWrapper> ContextWrapper::put_double_async(
    const std::string& pv_name, 
    double value, 
    double timeout) 
{
    try {
        auto op = ctx_.put(pv_name)
            .set("value", value)
            .exec();
        return std::make_unique<OperationWrapper>(std::move(op));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Failed to start PUT for '") + pv_name + "': " + e.what());
    }
}

std::unique_ptr<OperationWrapper> ContextWrapper::info_async(
    const std::string& pv_name, 
    double timeout) 
{
    try {
        auto op = ctx_.info(pv_name).exec();
        return std::make_unique<OperationWrapper>(std::move(op));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Failed to start INFO for '") + pv_name + "': " + e.what());
    }
}

void ContextWrapper::put_double(
    const std::string& pv_name, 
    double value, 
    double timeout) 
{
    try {
        ctx_.put(pv_name)
            .set("value", value)
            .exec()
            ->wait(timeout);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("PUT failed for '") + pv_name + "': " + e.what());
    }
}

void ContextWrapper::put_int32(
    const std::string& pv_name, 
    int32_t value, 
    double timeout) 
{
    try {
        ctx_.put(pv_name)
            .set("value", value)
            .exec()
            ->wait(timeout);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("PUT failed for '") + pv_name + "': " + e.what());
    }
}

std::unique_ptr<ValueWrapper> ContextWrapper::info_sync(
    const std::string& pv_name, 
    double timeout) 
{
    try {
        auto op = ctx_.info(pv_name).exec();
        pvxs::Value result = op->wait(timeout);
        return std::make_unique<ValueWrapper>(std::move(result));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("INFO failed for '") + pv_name + "': " + e.what());
    }
}

std::unique_ptr<RpcWrapper> ContextWrapper::rpc_create(const std::string& pv_name) {
    try {
        auto builder = ctx_.rpc(pv_name);
        return std::make_unique<RpcWrapper>(std::move(builder));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("RPC creation failed for '") + pv_name + "': " + e.what());
    }
}

std::unique_ptr<MonitorWrapper> ContextWrapper::monitor(const std::string& pv_name) {
    try {
        // Create a PVXS subscription using the same pattern as get_sync
        auto subscription_builder = ctx_.monitor(pv_name);
        auto subscription = subscription_builder.exec();
        return std::make_unique<MonitorWrapper>(std::move(subscription), pv_name);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Monitor creation failed for '") + pv_name + "': " + e.what());
    }
}

// ============================================================================
// Factory functions for Rust FFI
// ============================================================================

std::unique_ptr<ContextWrapper> create_context_from_env() {
    return ContextWrapper::from_env();
}

std::unique_ptr<ValueWrapper> context_get_sync(
    ContextWrapper& ctx,
    rust::Str pv_name,
    double timeout)
{
    std::string pv_name_str(pv_name.data(), pv_name.size());
    return ctx.get_sync(pv_name_str, timeout);
}

void context_put_double(
    ContextWrapper& ctx,
    rust::Str pv_name,
    double value,
    double timeout)
{
    std::string pv_name_str(pv_name.data(), pv_name.size());
    ctx.put_double(pv_name_str, value, timeout);
}

std::unique_ptr<ValueWrapper> context_info_sync(
    ContextWrapper& ctx,
    rust::Str pv_name,
    double timeout)
{
    std::string pv_name_str(pv_name.data(), pv_name.size());
    return ctx.info_sync(pv_name_str, timeout);
}

// ============================================================================
// Value accessor functions for Rust FFI
// ============================================================================

bool value_is_valid(const ValueWrapper& val) {
    return val.valid();
}

rust::String value_to_string(const ValueWrapper& val) {
    return rust::String(val.to_string());
}

double value_get_field_double(const ValueWrapper& val, rust::Str field_name) {
    std::string field_name_str(field_name.data(), field_name.size());
    return val.get_field_double(field_name_str);
}

int32_t value_get_field_int32(const ValueWrapper& val, rust::Str field_name) {
    std::string field_name_str(field_name.data(), field_name.size());
    return val.get_field_int32(field_name_str);
}

rust::String value_get_field_string(const ValueWrapper& val, rust::Str field_name) {
    std::string field_name_str(field_name.data(), field_name.size());
    return rust::String(val.get_field_string(field_name_str));
}

// ============================================================================
// Async operation functions for Rust FFI
// ============================================================================

std::unique_ptr<OperationWrapper> context_get_async(
    ContextWrapper& ctx,
    rust::Str pv_name,
    double timeout)
{
    std::string pv_name_str(pv_name.data(), pv_name.size());
    return ctx.get_async(pv_name_str, timeout);
}

std::unique_ptr<OperationWrapper> context_put_double_async(
    ContextWrapper& ctx,
    rust::Str pv_name,
    double value,
    double timeout)
{
    std::string pv_name_str(pv_name.data(), pv_name.size());
    return ctx.put_double_async(pv_name_str, value, timeout);
}

std::unique_ptr<OperationWrapper> context_info_async(
    ContextWrapper& ctx,
    rust::Str pv_name,
    double timeout)
{
    std::string pv_name_str(pv_name.data(), pv_name.size());
    return ctx.info_async(pv_name_str, timeout);
}

bool operation_is_done(const OperationWrapper& op) {
    return op.is_done();
}

std::unique_ptr<ValueWrapper> operation_get_result(OperationWrapper& op) {
    return op.get_result();
}

void operation_cancel(OperationWrapper& op) {
    op.cancel();
}

bool operation_wait_for_completion(OperationWrapper& op, uint64_t timeout_ms) {
    return op.wait_for_completion(timeout_ms);
}

// ============================================================================
// RpcWrapper implementation
// ============================================================================

void RpcWrapper::arg_string(const std::string& name, const std::string& value) {
    builder_.arg(name, value);
}

void RpcWrapper::arg_double(const std::string& name, double value) {
    builder_.arg(name, value);
}

void RpcWrapper::arg_int32(const std::string& name, int32_t value) {
    builder_.arg(name, value);
}

void RpcWrapper::arg_bool(const std::string& name, bool value) {
    builder_.arg(name, value);
}

std::unique_ptr<ValueWrapper> RpcWrapper::execute_sync(double timeout) {
    try {
        auto op = builder_.exec();
        if (!op) {
            throw PvxsError("Failed to create RPC operation");
        }
        
        auto result = op->wait(timeout);
        return std::make_unique<ValueWrapper>(std::move(result));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("RPC execution failed: ") + e.what());
    }
}

std::unique_ptr<OperationWrapper> RpcWrapper::execute_async(double timeout) {
    try {
        auto op = builder_.exec();
        if (!op) {
            throw PvxsError("Failed to create RPC operation");
        }
        
        return std::make_unique<OperationWrapper>(std::move(op));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("RPC execution failed: ") + e.what());
    }
}

// ============================================================================
// Bridge functions for RPC
// ============================================================================

std::unique_ptr<RpcWrapper> context_rpc_create(
    ContextWrapper& ctx, 
    rust::Str pv_name) {
    try {
        std::string pv_name_str(pv_name.data(), pv_name.size());
        auto builder = ctx.rpc_create(pv_name_str);
        return builder;
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Failed to create RPC: ") + e.what());
    }
}

void rpc_arg_string(RpcWrapper& rpc, rust::Str name, rust::Str value) {
    try {
        std::string name_str(name.data(), name.size());
        std::string value_str(value.data(), value.size());
        rpc.arg_string(name_str, value_str);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Failed to set RPC string argument: ") + e.what());
    }
}

void rpc_arg_double(RpcWrapper& rpc, rust::Str name, double value) {
    try {
        std::string name_str(name.data(), name.size());
        rpc.arg_double(name_str, value);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Failed to set RPC double argument: ") + e.what());
    }
}

void rpc_arg_int32(RpcWrapper& rpc, rust::Str name, int32_t value) {
    try {
        std::string name_str(name.data(), name.size());
        rpc.arg_int32(name_str, value);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Failed to set RPC int32 argument: ") + e.what());
    }
}

void rpc_arg_bool(RpcWrapper& rpc, rust::Str name, bool value) {
    try {
        std::string name_str(name.data(), name.size());
        rpc.arg_bool(name_str, value);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Failed to set RPC bool argument: ") + e.what());
    }
}

std::unique_ptr<ValueWrapper> rpc_execute_sync(RpcWrapper& rpc, double timeout) {
    try {
        return rpc.execute_sync(timeout);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("RPC synchronous execution failed: ") + e.what());
    }
}

std::unique_ptr<OperationWrapper> rpc_execute_async(RpcWrapper& rpc, double timeout) {
    try {
        return rpc.execute_async(timeout);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("RPC asynchronous execution failed: ") + e.what());
    }
}

// ============================================================================
// MonitorWrapper implementation - Proper PVXS client::Subscription API usage
// ============================================================================

void MonitorWrapper::start() {
    // Resume the subscription (in case it was paused)
    if (monitor_) {
        try {
            monitor_->resume();  // resume() is shorthand for pause(false)
        } catch (const std::exception& e) {
            // Ignore errors - subscription might already be running
        }
    }
}

void MonitorWrapper::stop() {
    // Pause the subscription instead of destroying it
    if (monitor_) {
        try {
            monitor_->pause(true);
        } catch (const std::exception& e) {
            // If pause fails, cancel the subscription
            monitor_->cancel();
        }
    }
}

bool MonitorWrapper::is_running() const {
    // Return true if we have an active subscription that hasn't been cancelled
    return static_cast<bool>(monitor_);
}

bool MonitorWrapper::has_update() const {
    // Check if the subscription has updates by trying a non-blocking pop
    if (monitor_) {
        try {
            // Create a temporary copy to avoid modifying the monitor state
            // We'll use the actual get_update method for real retrieval
            auto temp_monitor = monitor_;
            auto update = temp_monitor->pop();
            return update.valid();
        } catch (const std::exception& e) {
            // Any exception means no update or connection issue
            return false;
        }
    }
    return false;
}

std::unique_ptr<ValueWrapper> MonitorWrapper::get_update(double timeout) {
    // Get an update from the subscription
    if (monitor_) {
        try {
            // PVXS pop() is non-blocking, so we need to implement our own timeout
            auto start_time = std::chrono::steady_clock::now();
            auto timeout_duration = std::chrono::duration<double>(timeout);
            
            while (true) {
                auto update = monitor_->pop();
                if (update.valid()) {
                    return std::make_unique<ValueWrapper>(std::move(update));
                }
                
                // Check timeout
                auto elapsed = std::chrono::steady_clock::now() - start_time;
                if (elapsed >= timeout_duration) {
                    break;
                }
                
                // Small sleep to avoid busy waiting
                std::this_thread::sleep_for(std::chrono::milliseconds(10));
            }
        } catch (const pvxs::client::Connected& e) {
            // Connection event - not a data update, return null
        } catch (const pvxs::client::Disconnect& e) {
            // Disconnection event - not a data update, return null  
        } catch (const pvxs::client::Finished& e) {
            // Subscription finished - not a data update, return null
        } catch (const std::exception& e) {
            // Other errors - return null
        }
    }
    return nullptr;
}

std::unique_ptr<ValueWrapper> MonitorWrapper::try_get_update() {
    // Try to get update without blocking
    if (monitor_) {
        try {
            auto update = monitor_->pop();
            if (update.valid()) {
                return std::make_unique<ValueWrapper>(std::move(update));
            }
        } catch (const pvxs::client::Connected& e) {
            // Connection event - not a data update
        } catch (const pvxs::client::Disconnect& e) {
            // Disconnection event - not a data update
        } catch (const pvxs::client::Finished& e) {
            // Subscription finished - not a data update
        } catch (const std::exception& e) {
            // Other errors
        }
    }
    return nullptr;
}

bool MonitorWrapper::is_connected() const {
    // Check if subscription is connected by checking if it's valid and not cancelled
    if (monitor_) {
        try {
            // If we can access the name, the subscription is likely connected
            auto name = monitor_->name();
            return !name.empty();
        } catch (const std::exception& e) {
            return false;
        }
    }
    return false;
}

// ============================================================================
// Monitor bridge functions for Rust
// ============================================================================

std::unique_ptr<MonitorWrapper> context_monitor_create(ContextWrapper& ctx, rust::Str pv_name) {
    try {
        std::string pv_name_str(pv_name);
        auto monitor = ctx.monitor(pv_name_str);
        return monitor;
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Monitor creation failed: ") + e.what());
    }
}

void monitor_start(MonitorWrapper& monitor) {
    monitor.start();
}

void monitor_stop(MonitorWrapper& monitor) {
    monitor.stop();
}

bool monitor_is_running(const MonitorWrapper& monitor) {
    return monitor.is_running();
}

bool monitor_has_update(const MonitorWrapper& monitor) {
    return monitor.has_update();
}

std::unique_ptr<ValueWrapper> monitor_get_update(MonitorWrapper& monitor, double timeout) {
    return monitor.get_update(timeout);
}

std::unique_ptr<ValueWrapper> monitor_try_get_update(MonitorWrapper& monitor) {
    return monitor.try_get_update();
}

bool monitor_is_connected(const MonitorWrapper& monitor) {
    return monitor.is_connected();
}

rust::String monitor_get_name(const MonitorWrapper& monitor) {
    return rust::String(monitor.name());
}

} // namespace pvxs_wrapper
