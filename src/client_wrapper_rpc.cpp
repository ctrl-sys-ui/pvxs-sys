#include "wrapper.h"

namespace pvxs_wrapper {
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
} // namespace pvxs_wrapper