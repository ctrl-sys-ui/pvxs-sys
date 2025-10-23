#include "wrapper.h"

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
}