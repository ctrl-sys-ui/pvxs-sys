// wrapper.h - C++ wrapper layer to simplify PVXS for Rust FFI
// This layer handles the complex C++ patterns (callbacks, shared_ptr, etc.)

#pragma once

#include <memory>
#include <string>
#include <stdexcept>
#include <optional>
#include "rust/cxx.h" // For rust::String and rust::Str types
#include <pvxs/client.h>
#include <pvxs/server.h>
#include <pvxs/sharedpv.h>
#include <pvxs/nt.h>

namespace pvxs_wrapper
{

    // Forward declarations
    class ContextWrapper;
    class OperationWrapper;
    class ValueWrapper;
    class ServerWrapper;
    class SharedPVWrapper;
    class StaticSourceWrapper;
    class MonitorWrapper;
    class MonitorBuilderWrapper;

    /// Exception wrapper for Rust-friendly error handling
    class PvxsError : public std::runtime_error
    {
    public:
        explicit PvxsError(const std::string &msg) : std::runtime_error(msg) {}
    };

    /// Wraps pvxs::Value for safe Rust access
    class ValueWrapper
    {
    private:
        pvxs::Value value_;

    public:
        ValueWrapper() = default;
        explicit ValueWrapper(pvxs::Value &&val) : value_(std::move(val)) {}

        // Check if value is valid
        bool valid() const { return value_.valid(); }

        // Get field as string (simplified for now)
        std::string get_field_string(const std::string &field_name) const;

        // Get field as double
        double get_field_double(const std::string &field_name) const;

        // Get field as int32
        int32_t get_field_int32(const std::string &field_name) const;

        // Get field as enum (int16)
        std::int16_t get_field_enum(const std::string &field_name) const;

        // Get field as array of doubles
        rust::Vec<double> get_field_double_array(const std::string &field_name) const;

        // Get field as array of int32
        rust::Vec<int32_t> get_field_int32_array(const std::string &field_name) const;

        // Get field as array of enums (int16)
        rust::Vec<int16_t> get_field_enum_array(const std::string &field_name) const;

        // Get field as array of strings
        rust::Vec<rust::String> get_field_string_array(const std::string &field_name) const;

        // Convert entire value to string representation
        std::string to_string() const;

        // Get the underlying pvxs::Value (internal use)
        pvxs::Value &get() { return value_; }
        const pvxs::Value &get() const { return value_; }
    };

    /// Wraps pvxs::client::Operation for safe Rust access
    class OperationWrapper
    {
    private:
        std::shared_ptr<pvxs::client::Operation> op_;

    public:
        OperationWrapper() = default;
        explicit OperationWrapper(std::shared_ptr<pvxs::client::Operation> &&op)
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
    class MonitorWrapper
    {
    private:
        pvxs::client::Context &context_;
        std::shared_ptr<pvxs::client::Subscription> monitor_;
        std::shared_ptr<pvxs::client::Connect> connect_;  // Track connection state
        std::string pv_name_;
        void (*rust_callback_)() = nullptr;  // Function pointer to Rust callback (no parameters)

    public:
        MonitorWrapper() = delete; // Must have context and PV name
        MonitorWrapper(pvxs::client::Context &ctx, const std::string &pv_name)
            : context_(ctx), pv_name_(pv_name) {}
        explicit MonitorWrapper(std::shared_ptr<pvxs::client::Subscription> &&monitor, const std::string &pv_name, pvxs::client::Context &ctx)
            : context_(ctx), monitor_(std::move(monitor)), pv_name_(pv_name) {}
        explicit MonitorWrapper(std::shared_ptr<pvxs::client::Subscription> &&monitor, const std::string &pv_name, pvxs::client::Context &ctx, void (*callback)())
            : context_(ctx), monitor_(std::move(monitor)), pv_name_(pv_name), rust_callback_(callback) {}

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
        
        // Set the Connect object (used by MonitorBuilder)
        void set_connect(std::shared_ptr<pvxs::client::Connect> &&connect) {
            connect_ = std::move(connect);
        }
        
        // Pop next value from subscription queue (PVXS-style)
        std::unique_ptr<ValueWrapper> pop();
    };

    /// Builder pattern for creating monitors with callbacks (PVXS-style)
    class MonitorBuilderWrapper
    {
    private:
        pvxs::client::Context &context_;
        std::string pv_name_;
        bool mask_connected_ = true;
        bool mask_disconnected_ = false;
        uint64_t callback_id_ = 0;
        void (*rust_callback_)() = nullptr;  // Function pointer to Rust callback (no parameters)

    public:
        MonitorBuilderWrapper() = delete;
        MonitorBuilderWrapper(pvxs::client::Context &ctx, const std::string &pv_name)
            : context_(ctx), pv_name_(pv_name) {}

        // Configure whether to include Connected events in queue
        void mask_connected(bool mask);
        
        // Configure whether to include Disconnected events in queue  
        void mask_disconnected(bool mask);
        
        // Set event callback function pointer
        void set_event_callback(void (*callback)());
        
        // Execute and return a subscription without callback
        std::unique_ptr<MonitorWrapper> exec();
        
        // Execute and return a subscription with event callback
        std::unique_ptr<MonitorWrapper> exec_with_callback(uint64_t callback_id);
        
        // Get PV name
        std::string name() const { return pv_name_; }
    };

    /// Wraps pvxs::client::Context for safe Rust access
    class ContextWrapper
    {
    private:
        pvxs::client::Context context_;

    public:
        // Create context from environment variables
        static std::unique_ptr<ContextWrapper> from_env();

        // Create context with explicit configuration
        explicit ContextWrapper(pvxs::client::Context &&ctx)
            : context_(std::move(ctx)) {}

        // Perform a GET operation (synchronous version for simplicity)
        std::unique_ptr<ValueWrapper> get(const std::string &pv_name, double timeout);

        // Start an async GET operation
        std::unique_ptr<OperationWrapper> get_async(const std::string &pv_name, double timeout);

        // Start an async PUT operation
        std::unique_ptr<OperationWrapper> put_double_async(const std::string &pv_name, double value, double timeout);

        // Start an async INFO operation
        std::unique_ptr<OperationWrapper> info_async(const std::string &pv_name, double timeout);

        // Perform a PUT operation (simplified - just set a double value)
        void put(const std::string &pv_name, double value, double timeout);

        // Perform a PUT operation (simplified - just set an int32 value)
        void put(const std::string &pv_name, int32_t value, double timeout);

        // Perform a PUT operation (simplified - just set a string value)
        void put(const std::string &pv_name, const std::string &value, double timeout);

        // Perform a PUT operation (simplified - just set an enum value)
        void put(const std::string &pv_name, int16_t value, double timeout);

        // Perform a PUT operation (simplified - just set a double array)
        void put(const std::string &pv_name, const rust::Vec<double> &value, double timeout);

        // Perform a PUT operation (simplified - just set an int32 array)
        void put(const std::string &pv_name, const rust::Vec<int32_t> &value, double timeout);

        // Perform a PUT operation (simplified - just set an enum array)
        void put(const std::string &pv_name, const rust::Vec<int16_t> &value, double timeout);

        // Perform a PUT operation (simplified - just set a string array)
        void put(const std::string &pv_name, const rust::Vec<rust::String> &value, double timeout);

        // Get type information (INFO operation)
        std::unique_ptr<ValueWrapper> info(const std::string &pv_name, double timeout);

        // Create RPC builder
        std::unique_ptr<class RpcWrapper> rpc_create(const std::string &pv_name);

        // Create Monitor
        std::unique_ptr<MonitorWrapper> monitor(const std::string &pv_name);
        
        // Create MonitorBuilder (PVXS-style)
        std::unique_ptr<MonitorBuilderWrapper> monitor_builder(const std::string &pv_name);
    };

    /// Wraps RPC operations for safe Rust access
    class RpcWrapper
    {
    private:
        pvxs::client::Context &context_;
        std::string pv_name_;
        pvxs::Value arguments_;

    public:
        RpcWrapper(pvxs::client::Context &ctx, const std::string &pv_name)
            : context_(ctx), pv_name_(pv_name) {}

        // Add arguments to the RPC call
        void arg_string(const std::string &name, const std::string &value);
        void arg_double(const std::string &name, double value);
        void arg_int32(const std::string &name, int32_t value);
        void arg_bool(const std::string &name, bool value);

        // Execute RPC call synchronously
        std::unique_ptr<ValueWrapper> execute_sync(double timeout);

        // Execute RPC call asynchronously
        std::unique_ptr<OperationWrapper> execute_async(double timeout);
    };

    // Factory functions for Rust (these will be exposed via cxx bridge)
    std::unique_ptr<ContextWrapper> create_context_from_env();

    // RPC operations bridge functions
    std::unique_ptr<RpcWrapper> context_rpc_create(
        ContextWrapper &ctx,
        rust::String pv_name);

    void rpc_arg_string(RpcWrapper &rpc, rust::String name, rust::String value);
    void rpc_arg_double(RpcWrapper &rpc, rust::String name, double value);
    void rpc_arg_int32(RpcWrapper &rpc, rust::String name, int32_t value);
    void rpc_arg_bool(RpcWrapper &rpc, rust::String name, bool value);

    std::unique_ptr<ValueWrapper> rpc_execute_sync(RpcWrapper &rpc, double timeout);
    std::unique_ptr<OperationWrapper> rpc_execute_async(RpcWrapper &rpc, double timeout);
    std::unique_ptr<ValueWrapper> context_get(ContextWrapper &ctx, rust::Str pv_name, double timeout);
    void context_put_double(ContextWrapper &ctx, rust::Str pv_name, double value, double timeout);
    void context_put_int32(ContextWrapper &ctx, rust::Str pv_name, int32_t value, double timeout);
    void context_put_string(ContextWrapper &ctx, rust::Str pv_name, rust::String value, double timeout);
    void context_put_enum(ContextWrapper &ctx, rust::Str pv_name, int16_t value, double timeout);
    void context_put_double_array(ContextWrapper &ctx, rust::Str pv_name, rust::Vec<double> value, double timeout);
    void context_put_int32_array(ContextWrapper &ctx, rust::Str pv_name, rust::Vec<int32_t> value, double timeout);
    void context_put_string_array(ContextWrapper &ctx, rust::Str pv_name, rust::Vec<int16_t> value, double timeout);
    void context_put_string_array(ContextWrapper &ctx, rust::Str pv_name, rust::Vec<int16_t> value, double timeout);
    void context_put_string_array(ContextWrapper &ctx, rust::Str pv_name, rust::Vec<rust::String> value, double timeout);
    std::unique_ptr<ValueWrapper> context_info(ContextWrapper &ctx, rust::Str pv_name, double timeout);
    // ============================================================================
    // Async operations for Rust
    std::unique_ptr<OperationWrapper> context_get_async(ContextWrapper &ctx, rust::Str pv_name, double timeout);
    std::unique_ptr<OperationWrapper> context_put_double_async(ContextWrapper &ctx, rust::Str pv_name, double value, double timeout);
    std::unique_ptr<OperationWrapper> context_info_async(ContextWrapper &ctx, rust::Str pv_name, double timeout);

    // Operation management for Rust
    bool operation_is_done(const OperationWrapper &op);
    std::unique_ptr<ValueWrapper> operation_get_result(OperationWrapper &op);
    void operation_cancel(OperationWrapper &op);
    bool operation_wait_for_completion(OperationWrapper &op, uint64_t timeout_ms);

    // Value accessors for Rust
    bool value_is_valid(const ValueWrapper &val);
    rust::String value_to_string(const ValueWrapper &val);
    double value_get_field_double(const ValueWrapper &val, rust::String field_name);
    int32_t value_get_field_int32(const ValueWrapper &val, rust::String field_name);
    rust::String value_get_field_string(const ValueWrapper &val, rust::String field_name);
    int16_t value_get_field_enum(const ValueWrapper &val, rust::String field_name);
    rust::Vec<double> value_get_field_double_array(const ValueWrapper &val, rust::String field_name);
    rust::Vec<int32_t> value_get_field_int32_array(const ValueWrapper &val, rust::String field_name);
    rust::Vec<rust::String> value_get_field_string_array(const ValueWrapper &val, rust::String field_name);

    // Monitor operations for Rust
    std::unique_ptr<MonitorWrapper> context_monitor_create(ContextWrapper &ctx, rust::String pv_name);
    void monitor_start(MonitorWrapper &monitor);
    void monitor_stop(MonitorWrapper &monitor);
    bool monitor_is_running(const MonitorWrapper &monitor);
    bool monitor_has_update(const MonitorWrapper &monitor);
    std::unique_ptr<ValueWrapper> monitor_get_update(MonitorWrapper &monitor, double timeout);
    std::unique_ptr<ValueWrapper> monitor_try_get_update(MonitorWrapper &monitor);
    bool monitor_is_connected(const MonitorWrapper &monitor);
    rust::String monitor_get_name(const MonitorWrapper &monitor);
    std::unique_ptr<ValueWrapper> monitor_pop(MonitorWrapper &monitor);

    // MonitorBuilder operations for Rust
    std::unique_ptr<MonitorBuilderWrapper> context_monitor_builder_create(ContextWrapper &ctx, rust::String pv_name);
    void monitor_builder_mask_connected(MonitorBuilderWrapper &builder, bool mask);
    void monitor_builder_mask_disconnected(MonitorBuilderWrapper &builder, bool mask);
    void monitor_builder_set_event_callback(MonitorBuilderWrapper &builder, uintptr_t callback_ptr);
    std::unique_ptr<MonitorWrapper> monitor_builder_exec(MonitorBuilderWrapper &builder);
    std::unique_ptr<MonitorWrapper> monitor_builder_exec_with_callback(MonitorBuilderWrapper &builder, uint64_t callback_id);

    // ============================================================================
    // Server-side wrappers
    // ============================================================================

    /// Wraps pvxs::server::SharedPV for safe Rust access
    class SharedPVWrapper
    {
    private:
        pvxs::server::SharedPV pv_;
        pvxs::Value template_value_; // Store template for cloneEmpty()

    public:
        SharedPVWrapper() = default;
        explicit SharedPVWrapper(pvxs::server::SharedPV &&pv) : pv_(std::move(pv)) {}

        // Open the PV with initial value
        void open(const ValueWrapper &initial_value);

        // Check if PV is open
        bool is_open() const;

        // Close the PV
        void close();

        // Post a new value
        void post_value(const ValueWrapper &value);

        // Get current value
        std::unique_ptr<ValueWrapper> fetch_value() const;

        // Get the underlying SharedPV (internal use)
        pvxs::server::SharedPV &get() { return pv_; }
        const pvxs::server::SharedPV &get() const { return pv_; }

        // Get template value for creating compatible updates
        const pvxs::Value &get_template() const { return template_value_; }

        // Factory methods
        static std::unique_ptr<SharedPVWrapper> create_mailbox();
        static std::unique_ptr<SharedPVWrapper> create_readonly();
    };

    /// Wraps pvxs::server::StaticSource for safe Rust access
    class StaticSourceWrapper
    {
    private:
        pvxs::server::StaticSource source_;

    public:
        StaticSourceWrapper() = default;
        explicit StaticSourceWrapper(pvxs::server::StaticSource source) : source_(std::move(source)) {}

        // Add a SharedPV with a name
        void add_pv(const std::string &name, SharedPVWrapper &pv);

        // Remove a PV by name
        void remove_pv(const std::string &name);

        // Close all PVs
        void close_all();

        // Get the underlying source (internal use)
        pvxs::server::StaticSource &get() { return source_; }
        const pvxs::server::StaticSource &get() const { return source_; }

        // Factory method
        static std::unique_ptr<StaticSourceWrapper> create();
    };

    /// Wraps pvxs::server::Server for safe Rust access
    class ServerWrapper
    {
    private:
        pvxs::server::Server server_;

    public:
        ServerWrapper() = default;
        explicit ServerWrapper(pvxs::server::Server &&server) : server_(std::move(server)) {}

        // Start the server
        void start();

        // Stop the server
        void stop();

        // Add a SharedPV directly (uses built-in StaticSource)
        void add_pv(const std::string &name, SharedPVWrapper &pv);

        // Remove a PV by name
        void remove_pv(const std::string &name);

        // Add a source
        void add_source(const std::string &name, StaticSourceWrapper &source, int order);

        // Get server configuration info
        uint16_t get_tcp_port() const;
        uint16_t get_udp_port() const;

        // Factory methods
        static std::unique_ptr<ServerWrapper> from_env();
        static std::unique_ptr<ServerWrapper> isolated();
    };

    // ============================================================================
    // Server factory functions for Rust FFI
    // ============================================================================

    // Server creation
    std::unique_ptr<ServerWrapper> server_create_from_env();
    std::unique_ptr<ServerWrapper> server_create_isolated();

    // Server operations
    void server_start(ServerWrapper &server);
    void server_stop(ServerWrapper &server);
    void server_add_pv(ServerWrapper &server, rust::String name, SharedPVWrapper &pv);
    void server_remove_pv(ServerWrapper &server, rust::String name);
    void server_add_source(ServerWrapper &server, rust::String name, StaticSourceWrapper &source, int32_t order);
    uint16_t server_get_tcp_port(const ServerWrapper &server);
    uint16_t server_get_udp_port(const ServerWrapper &server);

    // SharedPV creation and operations
    std::unique_ptr<SharedPVWrapper> shared_pv_create_mailbox();
    std::unique_ptr<SharedPVWrapper> shared_pv_create_readonly();
    void shared_pv_open_double(SharedPVWrapper &pv, double initial_value);
    
    // NTScalar metadata structures with C++ optional for optional fields
    struct NTScalarAlarm {
        int32_t severity;
        int32_t status;
        rust::String message;
    };
    
    struct NTScalarTime {
        int64_t seconds_past_epoch;
        int32_t nanoseconds;
        int32_t user_tag;
    };
    
    struct NTScalarDisplay {
        int64_t limit_low;
        int64_t limit_high;
        rust::String description;
        rust::String units;
        int32_t precision;
    };
    
    struct NTScalarControl {
        double limit_low;
        double limit_high;
        double min_step;
    };
    
    struct NTScalarValueAlarm {
        bool active;
        double low_alarm_limit;
        double low_warning_limit;
        double high_warning_limit;
        double high_alarm_limit;
        int32_t low_alarm_severity;
        int32_t low_warning_severity;
        int32_t high_warning_severity;
        int32_t high_alarm_severity;
        uint8_t hysteresis;
    };
    
    struct NTScalarMetadata {
        NTScalarAlarm alarm;
        NTScalarTime time_stamp;
        std::optional<NTScalarDisplay> display;
        std::optional<NTScalarControl> control;
        std::optional<NTScalarValueAlarm> value_alarm;
        bool has_form;
    };
    
    // Builder functions for metadata construction from Rust
    std::unique_ptr<NTScalarAlarm> create_alarm(int32_t severity, int32_t status, rust::String message);
    std::unique_ptr<NTScalarTime> create_time(int64_t seconds_past_epoch, int32_t nanoseconds, int32_t user_tag);
    std::unique_ptr<NTScalarDisplay> create_display(int64_t limit_low, int64_t limit_high, rust::String description, rust::String units, int32_t precision);
    std::unique_ptr<NTScalarControl> create_control(double limit_low, double limit_high, double min_step);
    std::unique_ptr<NTScalarValueAlarm> create_value_alarm(bool active, double low_alarm_limit, double low_warning_limit, 
                                                            double high_warning_limit, double high_alarm_limit,
                                                            int32_t low_alarm_severity, int32_t low_warning_severity,
                                                            int32_t high_warning_severity, int32_t high_alarm_severity, uint8_t hysteresis);
    
    // Helper functions to build metadata with different combinations of optional fields
    std::unique_ptr<NTScalarMetadata> create_metadata_no_optional(const NTScalarAlarm& alarm, const NTScalarTime& time_stamp, bool has_form);
    std::unique_ptr<NTScalarMetadata> create_metadata_with_display(const NTScalarAlarm& alarm, const NTScalarTime& time_stamp, const NTScalarDisplay& display, bool has_form);
    std::unique_ptr<NTScalarMetadata> create_metadata_with_control(const NTScalarAlarm& alarm, const NTScalarTime& time_stamp, const NTScalarControl& control, bool has_form);
    std::unique_ptr<NTScalarMetadata> create_metadata_with_value_alarm(const NTScalarAlarm& alarm, const NTScalarTime& time_stamp, const NTScalarValueAlarm& value_alarm, bool has_form);
    std::unique_ptr<NTScalarMetadata> create_metadata_with_display_control(const NTScalarAlarm& alarm, const NTScalarTime& time_stamp, const NTScalarDisplay& display, const NTScalarControl& control, bool has_form);
    std::unique_ptr<NTScalarMetadata> create_metadata_with_display_value_alarm(const NTScalarAlarm& alarm, const NTScalarTime& time_stamp, const NTScalarDisplay& display, const NTScalarValueAlarm& value_alarm, bool has_form);
    std::unique_ptr<NTScalarMetadata> create_metadata_with_control_value_alarm(const NTScalarAlarm& alarm, const NTScalarTime& time_stamp, const NTScalarControl& control, const NTScalarValueAlarm& value_alarm, bool has_form);
    std::unique_ptr<NTScalarMetadata> create_metadata_full(const NTScalarAlarm& alarm, const NTScalarTime& time_stamp, const NTScalarDisplay& display, const NTScalarControl& control, const NTScalarValueAlarm& value_alarm, bool has_form);
    
    void shared_pv_open_double(SharedPVWrapper& pv, double initial_value, const NTScalarMetadata& metadata);
    void shared_pv_open_double_array(SharedPVWrapper& pv, rust::Vec<double> initial_value, const NTScalarMetadata& metadata);
    void shared_pv_open_int32(SharedPVWrapper &pv, int32_t initial_value);
    void shared_pv_open_string(SharedPVWrapper &pv, rust::String initial_value);
    void shared_pv_open_enum(SharedPVWrapper &pv, rust::Vec<rust::String> enum_choices, int16_t selected_choice);
    bool shared_pv_is_open(const SharedPVWrapper &pv);
    void shared_pv_close(SharedPVWrapper &pv);
    void shared_pv_post_double(SharedPVWrapper &pv, double value);
    void shared_pv_post_int32(SharedPVWrapper &pv, int32_t value);
    void shared_pv_post_string(SharedPVWrapper &pv, rust::String value);
    void shared_pv_post_enum(SharedPVWrapper &pv, int16_t value);
    std::unique_ptr<ValueWrapper> shared_pv_fetch(const SharedPVWrapper &pv);

    // StaticSource creation and operations
    std::unique_ptr<StaticSourceWrapper> static_source_create();
    void static_source_add_pv(StaticSourceWrapper &source, rust::String name, SharedPVWrapper &pv);
    void static_source_remove_pv(StaticSourceWrapper &source, rust::String name);
    void static_source_close_all(StaticSourceWrapper &source);

    // ============================================================================
    // Note: RPC Source implementation - to be added later when needed

} // namespace pvxs_wrapper
