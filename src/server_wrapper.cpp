// server_wrapper.cpp - C++ server wrapper layer for PVXS

#include "wrapper.h"
#include <sstream>
#include <chrono>
#include <thread>
#include <pvxs/log.h>

namespace pvxs_wrapper {

// ============================================================================
// SharedPVWrapper implementation
// ============================================================================

void SharedPVWrapper::open(const ValueWrapper& initial_value) {
    try {
        // Store the template value for future cloneEmpty() operations
        template_value_ = initial_value.get();
        pv_.open(initial_value.get());
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error opening SharedPV: ") + e.what());
    }
}

bool SharedPVWrapper::is_open() const {
    return pv_.isOpen();
}

void SharedPVWrapper::close() {
    try {
        pv_.close();
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error closing SharedPV: ") + e.what());
    }
}

void SharedPVWrapper::post_value(const ValueWrapper& value) {
    try {
        pv_.post(value.get());
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error posting value to SharedPV: ") + e.what());
    }
}

std::unique_ptr<ValueWrapper> SharedPVWrapper::fetch_value() const {
    try {
        auto value = pv_.fetch();
        return std::make_unique<ValueWrapper>(std::move(value));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error fetching value from SharedPV: ") + e.what());
    }
}

std::unique_ptr<SharedPVWrapper> SharedPVWrapper::create_mailbox() {
    try {
        auto pv = pvxs::server::SharedPV::buildMailbox();
        return std::make_unique<SharedPVWrapper>(std::move(pv));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error creating mailbox SharedPV: ") + e.what());
    }
}

std::unique_ptr<SharedPVWrapper> SharedPVWrapper::create_readonly() {
    try {
        auto pv = pvxs::server::SharedPV::buildReadonly();
        return std::make_unique<SharedPVWrapper>(std::move(pv));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error creating readonly SharedPV: ") + e.what());
    }
}

// ============================================================================
// StaticSourceWrapper implementation
// ============================================================================

void StaticSourceWrapper::add_pv(const std::string& name, SharedPVWrapper& pv) {
    try {
        source_.add(name, pv.get());
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error adding PV '") + name + "' to StaticSource: " + e.what());
    }
}

void StaticSourceWrapper::remove_pv(const std::string& name) {
    try {
        source_.remove(name);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error removing PV '") + name + "' from StaticSource: " + e.what());
    }
}

void StaticSourceWrapper::close_all() {
    try {
        source_.close();
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error closing all PVs in StaticSource: ") + e.what());
    }
}

std::unique_ptr<StaticSourceWrapper> StaticSourceWrapper::create() {
    try {
        auto source = pvxs::server::StaticSource::build();
        return std::make_unique<StaticSourceWrapper>(std::move(source));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error creating StaticSource: ") + e.what());
    }
}

// ============================================================================
// ServerWrapper implementation
// ============================================================================

void ServerWrapper::start() {
    try {
        server_.start();
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error starting server: ") + e.what());
    }
}

void ServerWrapper::stop() {
    try {
        server_.stop();
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error stopping server: ") + e.what());
    }
}

void ServerWrapper::add_pv(const std::string& name, SharedPVWrapper& pv) {
    try {
        server_.addPV(name, pv.get());
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error adding PV '") + name + "' to server: " + e.what());
    }
}

void ServerWrapper::remove_pv(const std::string& name) {
    try {
        server_.removePV(name);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error removing PV '") + name + "' from server: " + e.what());
    }
}

void ServerWrapper::add_source(const std::string& name, StaticSourceWrapper& source, int order) {
    try {
        server_.addSource(name, source.get().source(), order);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error adding source '") + name + "' to server: " + e.what());
    }
}

uint16_t ServerWrapper::get_tcp_port() const {
    try {
        return server_.config().tcp_port;
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error getting TCP port: ") + e.what());
    }
}

uint16_t ServerWrapper::get_udp_port() const {
    try {
        return server_.config().udp_port;
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error getting UDP port: ") + e.what());
    }
}

std::unique_ptr<ServerWrapper> ServerWrapper::from_env() {
    try {
        auto server = pvxs::server::Server::fromEnv();
        return std::make_unique<ServerWrapper>(std::move(server));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error creating server from environment: ") + e.what());
    }
}

std::unique_ptr<ServerWrapper> ServerWrapper::isolated() {
    try {
        auto config = pvxs::server::Config::isolated();
        auto server = config.build();
        return std::make_unique<ServerWrapper>(std::move(server));
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error creating isolated server: ") + e.what());
    }
}

// ============================================================================
// Server factory functions for Rust FFI
// ============================================================================

std::unique_ptr<ServerWrapper> server_create_from_env() {
    return ServerWrapper::from_env();
}

std::unique_ptr<ServerWrapper> server_create_isolated() {
    return ServerWrapper::isolated();
}

void server_start(ServerWrapper& server) {
    server.start();
}

void server_stop(ServerWrapper& server) {
    server.stop();
}

void server_add_pv(ServerWrapper& server, rust::String name, SharedPVWrapper& pv) {
    server.add_pv(std::string(name), pv);
}

void server_remove_pv(ServerWrapper& server, rust::String name) {
    server.remove_pv(std::string(name));
}

void server_add_source(ServerWrapper& server, rust::String name, StaticSourceWrapper& source, int32_t order) {
    server.add_source(std::string(name), source, order);
}

uint16_t server_get_tcp_port(const ServerWrapper& server) {
    return server.get_tcp_port();
}

uint16_t server_get_udp_port(const ServerWrapper& server) {
    return server.get_udp_port();
}

// ============================================================================
// SharedPV factory and operation functions for Rust FFI
// ============================================================================

std::unique_ptr<SharedPVWrapper> shared_pv_create_mailbox() {
    return SharedPVWrapper::create_mailbox();
}

std::unique_ptr<SharedPVWrapper> shared_pv_create_readonly() {
    return SharedPVWrapper::create_readonly();
}

void shared_pv_open_double(SharedPVWrapper& pv, double initial_value) {
    try {
        // Create an NTScalar with double value
        auto initial = pvxs::nt::NTScalar{pvxs::TypeCode::Float64}.create();
        initial["value"] = initial_value;
        
        ValueWrapper wrapper(std::move(initial));
        pv.open(wrapper);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error opening SharedPV with double value: ") + e.what());
    }
}

void shared_pv_open_int32(SharedPVWrapper& pv, int32_t initial_value) {
    try {
        // Create an NTScalar with int32 value
        auto initial = pvxs::nt::NTScalar{pvxs::TypeCode::Int32}.create();
        initial["value"] = initial_value;
        
        ValueWrapper wrapper(std::move(initial));
        pv.open(wrapper);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error opening SharedPV with int32 value: ") + e.what());
    }
}

void shared_pv_open_string(SharedPVWrapper& pv, rust::String initial_value) {
    try {
        // Create an NTScalar with string value
        auto initial = pvxs::nt::NTScalar{pvxs::TypeCode::String}.create();
        initial["value"] = std::string(initial_value);
        
        ValueWrapper wrapper(std::move(initial));
        pv.open(wrapper);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error opening SharedPV with string value: ") + e.what());
    }
}

void shared_pv_open_enum(SharedPVWrapper& pv, rust::Vec<rust::String> enum_choices, int16_t selected_choice) {
    try {
        auto enums = pvxs::nt::NTEnum{}.create();

        // Set the selected index
        enums["value.index"] = selected_choice;

        // Build a shared_array for the choices
        pvxs::shared_array<const std::string> choices_array;
        {
            // Create a temporary vector and convert to shared_array
            std::vector<std::string> temp_vec;
            temp_vec.reserve(enum_choices.size());
            for (const auto& choice : enum_choices) {
                temp_vec.emplace_back(std::string(choice));
            }
            choices_array = pvxs::shared_array<const std::string>(temp_vec.begin(), temp_vec.end());
        }
        
        // Try to assign the shared_array
        enums["value.choices"].from(choices_array);

        // Add an onPut handler to validate enum indices
        auto onPut = [choices_array](pvxs::server::SharedPV& spv, std::unique_ptr<pvxs::server::ExecOp>&& op, pvxs::Value&& value) {
            try {
                // Check if value.index is being set
                auto new_index = value["value.index"].as<int16_t>();
                
                // Validate the index
                if (new_index < 0) {
                    op->error("Enum index cannot be negative");
                    return;
                }
                if (static_cast<size_t>(new_index) >= choices_array.size()) {
                    op->error("Enum index " + std::to_string(new_index) + " is out of range (max: " + std::to_string(choices_array.size() - 1) + ")");
                    return;
                }
                
                // If validation passes, apply the update
                spv.post(std::move(value));
                op->reply();
            } catch (const std::exception& e) {
                op->error(std::string("Error validating enum PUT: ") + e.what());
            }
        };

        ValueWrapper wrapper(std::move(enums));
        pv.open(wrapper);
        pv.get().onPut(onPut);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error opening SharedPV with enum value: ") + e.what());
    }
}

bool shared_pv_is_open(const SharedPVWrapper& pv) {
    return pv.is_open();
}

void shared_pv_close(SharedPVWrapper& pv) {
    pv.close();
}

void shared_pv_post_double(SharedPVWrapper& pv, double value) {
    try {
        // Use cloneEmpty() to get correct structure, then set the value
        auto update = pv.get_template().cloneEmpty();
        update["value"] = value;
        
        ValueWrapper wrapper(std::move(update));
        pv.post_value(wrapper);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error posting double value to SharedPV: ") + e.what());
    }
}

void shared_pv_post_int32(SharedPVWrapper& pv, int32_t value) {
    try {
        // Use cloneEmpty() to get correct structure, then set the value
        auto update = pv.get_template().cloneEmpty();
        update["value"] = value;
        
        ValueWrapper wrapper(std::move(update));
        pv.post_value(wrapper);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error posting int32 value to SharedPV: ") + e.what());
    }
}

void shared_pv_post_string(SharedPVWrapper& pv, rust::String value) {
    try {
        // Use cloneEmpty() to get correct structure, then set the value
        auto update = pv.get_template().cloneEmpty();
        update["value"] = std::string(value);
        
        ValueWrapper wrapper(std::move(update));
        pv.post_value(wrapper);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error posting string value to SharedPV: ") + e.what());
    }
}

void shared_pv_post_enum(SharedPVWrapper& pv, int16_t value) {
    try {
        // Get the current template to validate against choices
        auto current = pv.get_template();
        
        // Validate the enum index is within valid range
        if (value < 0) {
            throw PvxsError("Enum index cannot be negative");
        }
        
        // Get the choices array to validate the index
        auto choices = current["value.choices"].as<pvxs::shared_array<const std::string>>();
        if (static_cast<size_t>(value) >= choices.size()) {
            throw PvxsError("Enum index " + std::to_string(value) + " is out of range (max: " + std::to_string(choices.size() - 1) + ")");
        }
        
        // Use cloneEmpty() to get correct structure, then set the enum index
        auto update = current.cloneEmpty();
        update["value.index"] = value;
        
        ValueWrapper wrapper(std::move(update));
        pv.post_value(wrapper);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error posting enum value to SharedPV: ") + e.what());
    }
}

void shared_pv_post_double_with_alarm(SharedPVWrapper& pv, double value, int32_t severity, int32_t status, rust::String message) {
    try {
        // Use cloneEmpty() to get correct structure, then set the value and alarm fields
        auto update = pv.get_template().cloneEmpty();
        update["value"] = value;
        update["alarm.severity"] = severity;
        update["alarm.status"] = status;
        update["alarm.message"] = std::string(message);
        
        ValueWrapper wrapper(std::move(update));
        pv.post_value(wrapper);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error posting double value with alarm to SharedPV: ") + e.what());
    }
}

void shared_pv_post_int32_with_alarm(SharedPVWrapper& pv, int32_t value, int32_t severity, int32_t status, rust::String message) {
    try {
        // Use cloneEmpty() to get correct structure, then set the value and alarm fields
        auto update = pv.get_template().cloneEmpty();
        update["value"] = value;
        update["alarm.severity"] = severity;
        update["alarm.status"] = status;
        update["alarm.message"] = std::string(message);
        
        ValueWrapper wrapper(std::move(update));
        pv.post_value(wrapper);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error posting int32 value with alarm to SharedPV: ") + e.what());
    }
}

void shared_pv_post_string_with_alarm(SharedPVWrapper& pv, rust::String value, int32_t severity, int32_t status, rust::String message) {
    try {
        // Use cloneEmpty() to get correct structure, then set the value and alarm fields
        auto update = pv.get_template().cloneEmpty();
        update["value"] = std::string(value);
        update["alarm.severity"] = severity;
        update["alarm.status"] = status;
        update["alarm.message"] = std::string(message);
        
        ValueWrapper wrapper(std::move(update));
        pv.post_value(wrapper);
    } catch (const std::exception& e) {
        throw PvxsError(std::string("Error posting string value with alarm to SharedPV: ") + e.what());
    }
}

std::unique_ptr<ValueWrapper> shared_pv_fetch(const SharedPVWrapper& pv) {
    return pv.fetch_value();
}

// ============================================================================
// StaticSource factory and operation functions for Rust FFI
// ============================================================================

std::unique_ptr<StaticSourceWrapper> static_source_create() {
    return StaticSourceWrapper::create();
}

void static_source_add_pv(StaticSourceWrapper& source, rust::String name, SharedPVWrapper& pv) {
    source.add_pv(std::string(name), pv);
}

void static_source_remove_pv(StaticSourceWrapper& source, rust::String name) {
    source.remove_pv(std::string(name));
}

void static_source_close_all(StaticSourceWrapper& source) {
    source.close_all();
}

} // namespace pvxs_wrapper