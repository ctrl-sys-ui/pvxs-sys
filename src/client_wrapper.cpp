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

    std::int16_t ValueWrapper::get_field_enum(const std::string& field_name) const {
        if (!value_.valid()) {
            throw PvxsError("Value is not valid");
        }
        
        try {
            auto field = value_[field_name];
            if (!field.valid()) {
                throw PvxsError("Field '" + field_name + "' not found");
            }
            return field.as<std::int16_t>();
        } catch (const std::exception& e) {
            throw PvxsError(std::string("Error getting field '") + field_name + "': " + e.what());
        }
    }

    rust::Vec<double> ValueWrapper::get_field_double_array(const std::string& field_name) const {
        if (!value_.valid()) {
            throw PvxsError("Value is not valid");
        }
        
        try {
            auto field = value_[field_name];
            if (!field.valid()) {
                throw PvxsError("Field '" + field_name + "' not found");
            }
            
            // Get the shared_array from PVXS
            auto arr = field.as<pvxs::shared_array<const double>>();
            
            // Convert to rust::Vec
            rust::Vec<double> result;
            for (size_t i = 0; i < arr.size(); ++i) {
                result.push_back(arr[i]);
            }
            return result;
        } catch (const std::exception& e) {
            throw PvxsError(std::string("Error getting array field '") + field_name + "': " + e.what());
        }
    }

    rust::Vec<int32_t> ValueWrapper::get_field_int32_array(const std::string& field_name) const {
        if (!value_.valid()) {
            throw PvxsError("Value is not valid");
        }
        
        try {
            auto field = value_[field_name];
            if (!field.valid()) {
                throw PvxsError("Field '" + field_name + "' not found");
            }
            
            // Get the shared_array from PVXS
            auto arr = field.as<pvxs::shared_array<const int32_t>>();
            
            // Convert to rust::Vec
            rust::Vec<int32_t> result;
            for (size_t i = 0; i < arr.size(); ++i) {
                result.push_back(arr[i]);
            }
            return result;
        } catch (const std::exception& e) {
            throw PvxsError(std::string("Error getting array field '") + field_name + "': " + e.what());
        }
    }

    rust::Vec<int16_t> ValueWrapper::get_field_enum_array(const std::string& field_name) const {
        if (!value_.valid()) {
            throw PvxsError("Value is not valid");
        }
        
        try {
            auto field = value_[field_name];
            if (!field.valid()) {
                throw PvxsError("Field '" + field_name + "' not found");
            }
            
            // Get the shared_array from PVXS
            auto arr = field.as<pvxs::shared_array<const int16_t>>();
            
            // Convert to rust::Vec
            rust::Vec<int16_t> result;
            for (size_t i = 0; i < arr.size(); ++i) {
                result.push_back(arr[i]);
            }
            return result;
        } catch (const std::exception& e) {
            throw PvxsError(std::string("Error getting array field '") + field_name + "': " + e.what());
        }
    }

    rust::Vec<rust::String> ValueWrapper::get_field_string_array(const std::string& field_name) const {
        if (!value_.valid()) {
            throw PvxsError("Value is not valid");
        }
        
        try {
            auto field = value_[field_name];
            if (!field.valid()) {
                throw PvxsError("Field '" + field_name + "' not found");
            }
            
            // Get the shared_array from PVXS
            auto arr = field.as<pvxs::shared_array<const std::string>>();
            
            // Convert to rust::Vec
            rust::Vec<rust::String> result;
            for (size_t i = 0; i < arr.size(); ++i) {
                result.push_back(rust::String(arr[i]));
            }
            return result;
        } catch (const std::exception& e) {
            throw PvxsError(std::string("Error getting array field '") + field_name + "': " + e.what());
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

    std::unique_ptr<ValueWrapper> ContextWrapper::get(
        const std::string& pv_name, 
        double timeout) {
        
        try {
            auto op = context_.get(pv_name).exec();
            auto result = op->wait(timeout);
            return std::make_unique<ValueWrapper>(std::move(result));
        } catch (const std::exception& e) {
            throw PvxsError(std::string("Error in get for '") + pv_name + "': " + e.what());
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

    std::unique_ptr<ValueWrapper> ContextWrapper::info(
        const std::string& pv_name,
        double timeout) {
        
        try {
            auto result = context_.info(pv_name).exec()->wait(timeout);
            return std::make_unique<ValueWrapper>(std::move(result));
        } catch (const std::exception& e) {
            throw PvxsError(std::string("Error in info for '") + pv_name + "': " + e.what());
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

    std::unique_ptr<ValueWrapper> context_get(
        ContextWrapper& ctx,
        rust::Str pv_name,
        double timeout) {
        return ctx.get(std::string(pv_name), timeout);
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
        return ctx.info(std::string(pv_name), timeout);
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

    int16_t value_get_field_enum(const ValueWrapper& val, rust::String field_name) {
        return val.get_field_enum(std::string(field_name));
    }

    rust::Vec<double> value_get_field_double_array(const ValueWrapper& val, rust::String field_name) {
        return val.get_field_double_array(std::string(field_name));
    }

    rust::Vec<int32_t> value_get_field_int32_array(const ValueWrapper& val, rust::String field_name) {
        return val.get_field_int32_array(std::string(field_name));
    }

    rust::Vec<int16_t> value_get_field_enum_array(const ValueWrapper& val, rust::String field_name) {
        return val.get_field_enum_array(std::string(field_name));
    }

    rust::Vec<rust::String> value_get_field_string_array(const ValueWrapper& val, rust::String field_name) {
        return val.get_field_string_array(std::string(field_name));
    }
} // namespace pvxs_wrapper