#include "wrapper.h"
#include <iostream>

namespace pvxs_wrapper {

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

    std::unique_ptr<ValueWrapper> MonitorWrapper::pop() {
        if (!monitor_) {
            throw PvxsError("Monitor not started for '" + pv_name_ + "'");
        }
        
        try {
            // PVXS-style pop() - returns update or throws
            auto result = monitor_->pop();
            if (result.valid()) {
                return std::make_unique<ValueWrapper>(std::move(result));
            } else {
                return nullptr; // Empty queue
            }
        } catch (const pvxs::client::Connected& e) {
            // Connection event - could be handled differently
            throw PvxsError("Connected: " + std::string(e.what()));
        } catch (const pvxs::client::Disconnect& e) {
            // Disconnection event
            throw PvxsError("Disconnected: " + std::string(e.what()));
        } catch (const pvxs::client::Finished& e) {
            // Finished event
            throw PvxsError("Finished: " + std::string(e.what()));
        } catch (const std::exception& e) {
            throw PvxsError(std::string("Error popping monitor update for '") + pv_name_ + "': " + e.what());
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

    std::unique_ptr<ValueWrapper> monitor_pop(MonitorWrapper& monitor) {
        return monitor.pop();
    }

    // ============================================================================
    // MonitorBuilderWrapper implementation
    // ============================================================================

    void MonitorBuilderWrapper::mask_connected(bool mask) {
        mask_connected_ = mask;
    }

    void MonitorBuilderWrapper::mask_disconnected(bool mask) {
        mask_disconnected_ = mask;
    }

    void MonitorBuilderWrapper::set_event_callback(void (*callback)()) {
        rust_callback_ = callback;
    }

    std::unique_ptr<MonitorWrapper> MonitorBuilderWrapper::exec() {
        try {
            auto builder = context_.monitor(pv_name_)
                .maskConnected(mask_connected_)
                .maskDisconnected(mask_disconnected_);
            
            // If we have a callback, set up the PVXS event handler and call exec in the chain
            if (rust_callback_) {
                // Capture the callback in a lambda for PVXS
                auto callback_ptr = rust_callback_;
                auto subscription = builder.event([callback_ptr](auto& subscription) {
                    // Call the Rust callback function (no parameters)
                    callback_ptr();
                }).exec();
                
                // Create wrapper with the subscription and callback
                return std::make_unique<MonitorWrapper>(
                    std::move(subscription), pv_name_, context_, rust_callback_);
            } else {
                // No callback, just exec directly
                auto subscription = builder.exec();
                
                // Create wrapper with the subscription
                return std::make_unique<MonitorWrapper>(
                    std::move(subscription), pv_name_, context_, nullptr);
            }
        } catch (const std::exception& e) {
            throw PvxsError(std::string("Error creating monitor for '") + pv_name_ + "': " + e.what());
        }
    }

    std::unique_ptr<MonitorWrapper> MonitorBuilderWrapper::exec_with_callback(uint64_t callback_id) {
        try {
            // Store callback ID for future use
            callback_id_ = callback_id;
            
            auto builder = context_.monitor(pv_name_)
                .maskConnected(mask_connected_)
                .maskDisconnected(mask_disconnected_);
                
            // For now, we'll store the callback ID and use it later
            // TODO: Implement proper callback mechanism
            auto subscription = builder.exec();
            
            // Create wrapper with the subscription
            return std::make_unique<MonitorWrapper>(
                std::move(subscription), pv_name_, context_);
        } catch (const std::exception& e) {
            throw PvxsError(std::string("Error creating monitor with callback for '") + pv_name_ + "': " + e.what());
        }
    }

    // ============================================================================
    // MonitorBuilder bridge functions for Rust
    // ============================================================================

    std::unique_ptr<MonitorBuilderWrapper> context_monitor_builder_create(
        ContextWrapper& ctx,
        rust::String pv_name) {
        return ctx.monitor_builder(std::string(pv_name));
    }

    void monitor_builder_mask_connected(MonitorBuilderWrapper& builder, bool mask) {
        builder.mask_connected(mask);
    }

    void monitor_builder_mask_disconnected(MonitorBuilderWrapper& builder, bool mask) {
        builder.mask_disconnected(mask);
    }

    void monitor_builder_set_event_callback(MonitorBuilderWrapper& builder, uintptr_t callback_ptr) {
        // Convert the uintptr_t back to an extern "C" function pointer with no parameters
        auto rust_fn = reinterpret_cast<void(*)()>(callback_ptr);
        
        // Directly set the callback without a wrapper since signatures match
        builder.set_event_callback(rust_fn);
    }

    std::unique_ptr<MonitorWrapper> monitor_builder_exec(MonitorBuilderWrapper& builder) {
        return builder.exec();
    }

    std::unique_ptr<MonitorWrapper> monitor_builder_exec_with_callback(
        MonitorBuilderWrapper& builder,
        uint64_t callback_id) {
        return builder.exec_with_callback(callback_id);
    }
}