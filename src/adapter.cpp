// adapter.cpp - Implementation of the C++ adapter layer

#include "adapter.h"
#include <sstream>
#include <pvxs/log.h>

namespace pvxs_adapter {

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
    const std::string& pv_name) 
{
    try {
        auto op = ctx_.get(pv_name).exec();
        return std::make_unique<OperationWrapper>(std::move(op));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Failed to start GET for '") + pv_name + "': " + e.what());
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

} // namespace pvxs_adapter
