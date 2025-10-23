#include "wrapper.h"

namespace pvxs_wrapper {
    // ============================================================================
    // OperationWrapper implementation
    // ============================================================================
    #ifdef PVXS_ASYNC_ENABLED

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

    //std::unique_ptr<OperationWrapper> ContextWrapper::get_async(
    //    const std::string& pv_name,
    //    double timeout) {
    //    throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    //}

    //std::unique_ptr<OperationWrapper> ContextWrapper::put_double_async(
    //    const std::string& pv_name,
    //    double value,
    //    double timeout) {
    //    throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    //}

    //std::unique_ptr<OperationWrapper> ContextWrapper::info_async(
    //    const std::string& pv_name,
    //    double timeout) {
    //    throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    //}

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

    //std::unique_ptr<OperationWrapper> context_get_async(
    //    ContextWrapper& ctx,
    //    rust::Str pv_name,
    //    double timeout) {
    //    throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    //}

    //std::unique_ptr<OperationWrapper> context_put_double_async(
    //    ContextWrapper& ctx,
    //    rust::Str pv_name,
    //    double value,
    //    double timeout) {
   //     throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    //}

    //std::unique_ptr<OperationWrapper> context_info_async(
    //    ContextWrapper& ctx,
    //    rust::Str pv_name,
    //    double timeout) {
    //    throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    //}

    #endif // PVXS_ASYNC_ENABLED

    std::unique_ptr<OperationWrapper> rpc_execute_async(RpcWrapper& rpc, double timeout) {
    #ifdef PVXS_ASYNC_ENABLED
        return rpc.execute_async(timeout);
    #else
        throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    #endif
    }

    #ifdef PVXS_ASYNC_ENABLED
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
    #else // !PVXS_ASYNC_ENABLED

    std::unique_ptr<OperationWrapper> RpcWrapper::execute_async(double timeout) {
        throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    }

    // Stub implementations when async is disabled
    std::unique_ptr<ValueWrapper> OperationWrapper::wait(double timeout) const {
        throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    }

    bool OperationWrapper::cancel() const {
        throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    }

    std::string OperationWrapper::name() const {
        throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    }

    bool OperationWrapper::is_done() const {
        throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    }

    std::unique_ptr<ValueWrapper> OperationWrapper::get_result() {
        throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    }

    bool OperationWrapper::wait_for_completion(uint64_t timeout_ms) {
        throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    }

    #endif // PVXS_ASYNC_ENABLED

    bool operation_is_done(const OperationWrapper& op) {
    #ifdef PVXS_ASYNC_ENABLED
        return op.is_done();
    #else
        throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    #endif
    }

    std::unique_ptr<ValueWrapper> operation_get_result(OperationWrapper& op) {
    #ifdef PVXS_ASYNC_ENABLED
        return op.get_result();
    #else
        throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    #endif
    }

    void operation_cancel(OperationWrapper& op) {
    #ifdef PVXS_ASYNC_ENABLED
        op.cancel();
    #else
        throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    #endif
    }

    bool operation_wait_for_completion(OperationWrapper& op, uint64_t timeout_ms) {
    #ifdef PVXS_ASYNC_ENABLED
        return op.wait_for_completion(timeout_ms);
    #else
        throw PvxsError("Async operations are not enabled. Compile with --features async to use async functionality.");
    #endif
    }

    
} // namespace pvxs_wrapper
