// client_wrapper.cpp - C++ client wrapper layer for PVXS

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
        return "(invalid)";
    }
    
    try {
        std::ostringstream ss;
        ss << value_;
        return ss.str();
    } catch (const std::exception& e) {
        return std::string("Error converting to string: ") + e.what();
    }
}

// ============================================================================
// OperationWrapper implementation
// ============================================================================

std::unique_ptr<ValueWrapper> OperationWrapper::wait(double timeout) const {
    if (!op_) {
        throw PvxsError("Operation is null");
    }
    
    try {
        auto result = (*op_).wait(timeout);
        return std::make_unique<ValueWrapper>(std::move(result));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error waiting for operation: ") + e.what());
    }
}

bool OperationWrapper::cancel() const {
    if (!op_) {
        return false;
    }
    (*op_).cancel();
    return true;
}

std::string OperationWrapper::name() const {
    if (!op_) {
        return "(null operation)";
    }
    return (*op_).name();
}

bool OperationWrapper::is_done() const {
    if (!op_) {
        return true; // null operation is "done" in a sense
    }
    
    try {
        // Check if operation is complete by trying a very short wait
        auto result = (*op_).wait(0.0);
        return result.valid(); // If we get a valid result, it's done
    } catch (const std::exception&) {
        return false; // If exception thrown, probably not done yet
    }
}

std::unique_ptr<ValueWrapper> OperationWrapper::get_result() {
    if (!op_) {
        throw PvxsError("Operation is null");
    }
    
    try {
        auto result = op_->wait(0.0); // Non-blocking wait
        if (!result.valid()) {
            throw PvxsError("Operation result not available yet");
        }
        return std::make_unique<ValueWrapper>(std::move(result));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error getting operation result: ") + e.what());
    }
}

bool OperationWrapper::wait_for_completion(uint64_t timeout_ms) {
    if (!op_) {
        return true; // null operation is already "complete"
    }
    
    try {
        // Convert milliseconds to seconds for PVXS
        double timeout_seconds = timeout_ms / 1000.0;
        auto result = op_->wait(timeout_seconds);
        return result.valid();
    } catch (const std::exception&) {
        return false;
    }
}

// ============================================================================
// ContextWrapper implementation
// ============================================================================

std::unique_ptr<ContextWrapper> ContextWrapper::from_env() {
    try {
        auto config = pvxs::client::Config::fromEnv();
        auto ctx = config.build();
        return std::make_unique<ContextWrapper>(std::move(ctx));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error creating context from environment: ") + e.what());
    }
}

std::unique_ptr<ValueWrapper> ContextWrapper::get_sync(
    const std::string& pv_name, 
    double timeout) {
    
    try {
        auto op = context_.get(pv_name).exec();
        auto result = op->wait(timeout);
        return std::make_unique<ValueWrapper>(std::move(result));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error in get_sync for '") + pv_name + "': " + e.what());
    }
}

std::unique_ptr<OperationWrapper> ContextWrapper::get_async(
    const std::string& pv_name,
    double timeout) {
    
    try {
        auto op = context_.get(pv_name).exec();
        return std::make_unique<OperationWrapper>(std::move(op));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error in get_async for '") + pv_name + "': " + e.what());
    }
}

std::unique_ptr<OperationWrapper> ContextWrapper::put_double_async(
    const std::string& pv_name,
    double value,
    double timeout) {
    
    try {
        auto op = context_.put(pv_name).build([value](pvxs::Value&& val) {
            val["value"] = value;
            return std::move(val);
        }).exec();
        return std::make_unique<OperationWrapper>(std::move(op));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error in put_double_async for '") + pv_name + "': " + e.what());
    }
}

std::unique_ptr<OperationWrapper> ContextWrapper::info_async(
    const std::string& pv_name,
    double timeout) {
    
    try {
        auto op = context_.info(pv_name).exec();
        return std::make_unique<OperationWrapper>(std::move(op));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error in info_async for '") + pv_name + "': " + e.what());
    }
}

void ContextWrapper::put_double(
    const std::string& pv_name,
    double value,
    double timeout) {
    
    try {
        context_.put(pv_name).build([value](pvxs::Value&& val) {
            val["value"] = value;
            return std::move(val);
        }).exec()->wait(timeout);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error in put_double for '") + pv_name + "': " + e.what());
    }
}

void ContextWrapper::put_int32(
    const std::string& pv_name,
    int32_t value,
    double timeout) {
    
    try {
        context_.put(pv_name).build([value](pvxs::Value&& val) {
            val["value"] = value;
            return std::move(val);
        }).exec()->wait(timeout);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error in put_int32 for '") + pv_name + "': " + e.what());
    }
}

std::unique_ptr<ValueWrapper> ContextWrapper::info_sync(
    const std::string& pv_name,
    double timeout) {
    
    try {
        auto result = context_.info(pv_name).exec()->wait(timeout);
        return std::make_unique<ValueWrapper>(std::move(result));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error in info_sync for '") + pv_name + "': " + e.what());
    }
}

std::unique_ptr<RpcWrapper> ContextWrapper::rpc_create(const std::string& pv_name) {
    try {
        return std::make_unique<RpcWrapper>(context_, pv_name);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error creating RPC for '") + pv_name + "': " + e.what());
    }
}

std::unique_ptr<MonitorWrapper> ContextWrapper::monitor(const std::string& pv_name) {
    try {
        return std::make_unique<MonitorWrapper>(context_, pv_name);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error creating monitor for '") + pv_name + "': " + e.what());
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
    double timeout) {
    return ctx.get_sync(std::string(pv_name), timeout);
}

void context_put_double(
    ContextWrapper& ctx,
    rust::Str pv_name,
    double value,
    double timeout) {
    ctx.put_double(std::string(pv_name), value, timeout);
}

std::unique_ptr<ValueWrapper> context_info_sync(
    ContextWrapper& ctx,
    rust::Str pv_name,
    double timeout) {
    return ctx.info_sync(std::string(pv_name), timeout);
}

// ============================================================================
// Value accessor functions for Rust FFI
// ============================================================================

bool value_is_valid(const ValueWrapper& val) {
    return val.valid();
}

rust::String value_to_string(const ValueWrapper& val) {
    return val.to_string();
}

double value_get_field_double(const ValueWrapper& val, rust::String field_name) {
    return val.get_field_double(std::string(field_name));
}

int32_t value_get_field_int32(const ValueWrapper& val, rust::String field_name) {
    return val.get_field_int32(std::string(field_name));
}

rust::String value_get_field_string(const ValueWrapper& val, rust::String field_name) {
    return val.get_field_string(std::string(field_name));
}

// ============================================================================
// Async operation functions for Rust FFI
// ============================================================================

std::unique_ptr<OperationWrapper> context_get_async(
    ContextWrapper& ctx,
    rust::Str pv_name,
    double timeout) {
    return ctx.get_async(std::string(pv_name), timeout);
}

std::unique_ptr<OperationWrapper> context_put_double_async(
    ContextWrapper& ctx,
    rust::Str pv_name,
    double value,
    double timeout) {
    return ctx.put_double_async(std::string(pv_name), value, timeout);
}

std::unique_ptr<OperationWrapper> context_info_async(
    ContextWrapper& ctx,
    rust::Str pv_name,
    double timeout) {
    return ctx.info_async(std::string(pv_name), timeout);
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
// RPC implementation
// ============================================================================

void RpcWrapper::arg_string(const std::string& name, const std::string& value) {
    if (!arguments_.valid()) {
        // Create a basic structure for arguments
        arguments_ = pvxs::TypeDef(pvxs::TypeCode::Struct, {}).create();
    }
    arguments_[name] = value;
}

void RpcWrapper::arg_double(const std::string& name, double value) {
    if (!arguments_.valid()) {
        // Create a basic structure for arguments  
        arguments_ = pvxs::TypeDef(pvxs::TypeCode::Struct, {}).create();
    }
    arguments_[name] = value;
}

void RpcWrapper::arg_int32(const std::string& name, int32_t value) {
    if (!arguments_.valid()) {
        // Create a basic structure for arguments
        arguments_ = pvxs::TypeDef(pvxs::TypeCode::Struct, {}).create();
    }
    arguments_[name] = value;
}

void RpcWrapper::arg_bool(const std::string& name, bool value) {
    if (!arguments_.valid()) {
        // Create a basic structure for arguments
        arguments_ = pvxs::TypeDef(pvxs::TypeCode::Struct, {}).create();
    }
    arguments_[name] = value;
}

std::unique_ptr<ValueWrapper> RpcWrapper::execute_sync(double timeout) {
    try {
        auto builder = context_.rpc(pv_name_);
        if (arguments_.valid()) {
            builder = builder.arg("argument", arguments_);
        }
        auto result = builder.exec()->wait(timeout);
        return std::make_unique<ValueWrapper>(std::move(result));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error in RPC execute_sync for '") + pv_name_ + "': " + e.what());
    }
}

std::unique_ptr<OperationWrapper> RpcWrapper::execute_async(double timeout) {
    try {
        auto builder = context_.rpc(pv_name_);
        if (arguments_.valid()) {
            builder = builder.arg("argument", arguments_);
        }
        auto op = builder.exec();
        return std::make_unique<OperationWrapper>(std::move(op));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error in RPC execute_async for '") + pv_name_ + "': " + e.what());
    }
}

// ============================================================================
// Bridge functions for RPC
// ============================================================================

std::unique_ptr<RpcWrapper> context_rpc_create(
    ContextWrapper& ctx,
    rust::String pv_name) {
    return ctx.rpc_create(std::string(pv_name));
}

void rpc_arg_string(RpcWrapper& rpc, rust::String name, rust::String value) {
    rpc.arg_string(std::string(name), std::string(value));
}

void rpc_arg_double(RpcWrapper& rpc, rust::String name, double value) {
    rpc.arg_double(std::string(name), value);
}

void rpc_arg_int32(RpcWrapper& rpc, rust::String name, int32_t value) {
    rpc.arg_int32(std::string(name), value);
}

void rpc_arg_bool(RpcWrapper& rpc, rust::String name, bool value) {
    rpc.arg_bool(std::string(name), value);
}

std::unique_ptr<ValueWrapper> rpc_execute_sync(RpcWrapper& rpc, double timeout) {
    return rpc.execute_sync(timeout);
}

std::unique_ptr<OperationWrapper> rpc_execute_async(RpcWrapper& rpc, double timeout) {
    return rpc.execute_async(timeout);
}

// ============================================================================
// MonitorWrapper implementation
// ============================================================================

void MonitorWrapper::start() {
    if (!monitor_) {
        try {
            auto sub = context_.monitor(pv_name_).maskConnected(true).maskDisconnected(true).exec();
            monitor_ = std::move(sub);
        } catch (const std::exception& e) {
            throw PvxsError(std::string("Error starting monitor for '") + pv_name_ + "': " + e.what());
        }
    }
}

void MonitorWrapper::stop() {
    if (monitor_) {
        monitor_.reset();
    }
}

bool MonitorWrapper::is_running() const {
    return monitor_ != nullptr;
}

bool MonitorWrapper::has_update() const {
    if (!monitor_) {
        return false;
    }
    
    try {
        // Check if there's an update by polling non-blocking
        return monitor_->pop().valid();
    } catch (const std::exception&) {
        return false;
    }
}

std::unique_ptr<ValueWrapper> MonitorWrapper::get_update(double timeout) {
    if (!monitor_) {
        throw PvxsError("Monitor not started for '" + pv_name_ + "'");
    }
    
    try {
        // Use pop() to get the next update - PVXS doesn't have wait with timeout on Subscription
        auto result = monitor_->pop();
        if (!result.valid()) {
            throw PvxsError("No update available for '" + pv_name_ + "'");
        }
        return std::make_unique<ValueWrapper>(std::move(result));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error getting monitor update for '") + pv_name_ + "': " + e.what());
    }
}

std::unique_ptr<ValueWrapper> MonitorWrapper::try_get_update() {
    if (!monitor_) {
        throw PvxsError("Monitor not started for '" + pv_name_ + "'");
    }
    
    try {
        // Try to get update non-blocking
        auto result = monitor_->pop();
        if (result.valid()) {
            return std::make_unique<ValueWrapper>(std::move(result));
        } else {
            return nullptr;
        }
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error trying to get monitor update for '") + pv_name_ + "': " + e.what());
    }
}

bool MonitorWrapper::is_connected() const {
    // This is a simplified implementation
    return monitor_ != nullptr;
}



// ============================================================================
// Monitor bridge functions for Rust
// ============================================================================

std::unique_ptr<MonitorWrapper> context_monitor_create(
    ContextWrapper& ctx,
    rust::String pv_name) {
    return ctx.monitor(std::string(pv_name));
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
    return monitor.name();
}

} // namespace pvxs_wrapper