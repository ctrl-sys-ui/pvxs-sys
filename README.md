# pvxs-sys

Complete low-level FFI bindings for the [EPICS PVXS](https://github.com/epics-base/pvxs) (PVAccess) library.

> **Note**: This is a `-sys` crate providing raw FFI bindings. For a high-level, idiomatic Rust API, use the `epics-pvxs` crate (coming soon).

This crate provides safe Rust bindings around the PVXS C++ library using the `cxx` crate. PVXS implements the PVAccess network protocol used in EPICS (Experimental Physics and Industrial Control System).

**🎉 Production-ready EPICS server and client implementation!** Create EPICS servers and clients with full network discovery, rich metadata support, array operations, and real-time monitoring.

## Features

### Client Features
- ✅ **Safe FFI Bindings** - Memory-safe wrappers using the `cxx` crate
- ✅ **GET Operations** - Read PV values (scalars and arrays)
- ✅ **PUT Operations** - Write PV values (double, int32, string, enum, and arrays)
- ✅ **INFO Operations** - Query PV type information and structure
- ✅ **Async Support** - Async/await support using Tokio (optional feature)
- ✅ **Monitor/Subscription** - Real-time PV monitoring with customizable callbacks
- ✅ **Array Support** - Full support for double[], int32[], and string[] arrays
- ✅ **RPC Support** - Remote procedure calls (client and server)

### Server Features
- ✅ **Complete Server API** - Full PVXS server implementation with network discovery
- ✅ **Rich Metadata** - NTScalar metadata including display limits, control ranges, and alarms
- ✅ **Multiple Data Types** - double, int32, string, enum, and array variants
- ✅ **SharedPV** - Process variables with mailbox (read/write) and readonly modes
- ✅ **StaticSource** - Organize PVs into logical device groups and hierarchies
- ✅ **Network Discovery** - Full EPICS beacon and search response functionality
- ✅ **Thread-safe** - Safe concurrent access to PVs from multiple threads

## Crate Structure

This is a `-sys` crate following Rust conventions:

- **`pvxs-sys`** (this crate) - Low-level FFI bindings
- **`epics-pvxs`** (planned) - High-level, idiomatic Rust API

## Architecture

This crate provides a complete EPICS PVXS implementation with separate client and server capabilities:

### Client Architecture
- **Safe FFI Wrappers** - Memory-safe C++ bindings using `cxx` crate
- **Context Management** - Thread-safe client contexts with connection pooling
- **Operation Types** - Synchronous and asynchronous GET/PUT/INFO operations  
- **Monitoring** - Real-time PV subscription and change notifications
- **RPC Support** - Remote procedure call client implementation

### Server Architecture  
- **ServerWrapper** - Complete PVXS server with network discovery and broadcasting
- **SharedPV** - Individual process variables with mailbox (read/write) and readonly modes
- **StaticSource** - Logical grouping of PVs into device/system hierarchies
- **Value Management** - Proper PVXS value structure handling with `cloneEmpty()` for updates
- **Network Discovery** - Full EPICS beacon and search response functionality

### Build System
- **Modular C++ Sources** - Separate `client_wrapper.cpp` and `server_wrapper.cpp` implementations
- **Shared Header** - Common `wrapper.h` for both client and server functionality
- **FFI Bridge** - Complete Rust-C++ type mapping with `cxx-bridge`
- **Cross-platform** - Windows (MSVC), Linux (GCC), and macOS (Clang) support

## Prerequisites

Before using this crate, you need:

1. **EPICS Base** (>= 7.0.9 recommended) - [Download here](https://github.com/epics-base/epics-base)
2. **PVXS Library** (>= 1.4.1 recommended) - [Download here](https://github.com/epics-base/pvxs)
3. **C++17 Compiler** - GCC >= 7, Clang >= 5, or MSVC >= 2017
4. **CMake** (>= 3.10) - Required for building libevent dependency - [Download here](https://cmake.org/download/)

### Environment Variables

Set the following environment variables:

- **`EPICS_BASE`** - Path to your EPICS base installation (required)
- **`EPICS_HOST_ARCH`** - Your host architecture (auto-detected if not set)
  - Examples: `linux-x86_64`, `windows-x64`, `darwin-x86`
- **`EPICS_PVXS`** - Path to PVXS installation (required)
  - Also accepts `PVXS_DIR` or `PVXS_BASE` as alternatives
- **`EPICS_PVXS_LIBEVENT`** - Path to libevent installation (optional)
  - Defaults to bundled libevent within PVXS: `{PVXS}/bundle/usr/{ARCH}`
  - Required DLLs: `event.dll`, `event_core.dll`, `event_extra.dll`

Example setup:

```bash
# Linux
export EPICS_BASE=/opt/epics/base
export EPICS_HOST_ARCH=linux-x86_64
export EPICS_PVXS=/opt/epics/modules/pvxs
# Optional: export EPICS_PVXS_LIBEVENT=/opt/epics/modules/pvxs/bundle/usr/linux-x86_64
```

```powershell
# Windows (PowerShell)
$env:EPICS_BASE = "C:\epics\base"
$env:EPICS_HOST_ARCH = "windows-x64"
$env:EPICS_PVXS = "C:\epics\pvxs"
# Optional: $env:EPICS_PVXS_LIBEVENT = "C:\epics\pvxs\bundle\usr\windows-x64"
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
pvxs-sys = "0.1"

# For async support
pvxs-sys = { version = "0.1", features = ["async"] }
```

### Optional Features

- **`async`** - Enables async/await support using Tokio
  - Adds `get_async()`, `put_double_async()`, and `info_async()` methods
  - Requires Tokio runtime
  - Example: `cargo run --features async --example async_operations`

### Runtime Requirements (Windows)

For Windows users, the EPICS and PVXS DLLs must be in your system PATH for the examples to run:

1. **EPICS Base DLLs**: `{EPICS_BASE}\bin\{EPICS_HOST_ARCH}`
2. **PVXS DLLs**: `{EPICS_PVXS}\bin\{EPICS_HOST_ARCH}`  
3. **libevent DLLs**: `{EPICS_PVXS}\bundle\usr\{EPICS_HOST_ARCH}\lib`

Example PowerShell commands to add to PATH for current session:
```powershell
$env:PATH = "C:\epics\base\bin\windows-x64;C:\epics\pvxs\bin\windows-x64;C:\epics\pvxs\bundle\usr\windows-x64\lib;" + $env:PATH
```

## Quick Start

### Reading a PV Value (GET)

```rust
use pvxs_sys::{Context, PvxsError};

fn main() -> Result<(), PvxsError> {
    // Create context from environment variables
    let mut ctx = Context::from_env()?;
    
    // Read a PV value with 5 second timeout
    let value = ctx.get("TEST:DOUBLE", 5.0)?;
    
    // Access the main value field
    let v = value.get_field_double("value")?;
    println!("Value: {}", v);
    
    Ok(())
}
```

### Writing PV Values (PUT)

```rust
use pvxs_sys::{Context, PvxsError};

fn main() -> Result<(), PvxsError> {
    let mut ctx = Context::from_env()?;
    
    // Write scalar values
    ctx.put_double("TEST:DOUBLE", 42.0, 5.0)?;
    ctx.put_int32("TEST:INT", 123, 5.0)?;
    ctx.put_string("TEST:STRING", "Hello", 5.0)?;
    
    // Write array values
    ctx.put_double_array("TEST:ARRAY", vec![1.0, 2.0, 3.0], 5.0)?;
    ctx.put_int32_array("TEST:INT_ARRAY", vec![10, 20, 30], 5.0)?;
    
    println!("Values written successfully!");
    Ok(())
}
```

### Monitoring PV Changes

```rust
use pvxs_sys::{Context, PvxsError};

fn main() -> Result<(), PvxsError> {
    let mut ctx = Context::from_env()?;
    
    // Create and start a monitor
    let mut monitor = ctx.monitor("TEST:COUNTER")?;
    monitor.start()?;
    
    // Poll for updates
    for _ in 0..10 {
        match monitor.get_update(5.0) {
            Ok(value) => {
                let v = value.get_field_double("value")?;
                println!("New value: {}", v);
            }
            Err(e) => eprintln!("Monitor error: {}", e),
        }
    }
    
    monitor.stop()?;
    Ok(())
}
```

### Creating an EPICS Server with Metadata

```rust
use pvxs_sys::{Server, NTScalarMetadataBuilder, DisplayMetadata, PvxsError};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), PvxsError> {
    // Create server from environment (enables network discovery)
    let mut server = Server::from_env()?;
    
    // Create PV with rich metadata
    let metadata = NTScalarMetadataBuilder::new()
        .alarm(0, 0, "OK")
        .display(DisplayMetadata {
            limit_low: 0,
            limit_high: 100,
            description: "Temperature sensor".to_string(),
            units: "DegC".to_string(),
            precision: 2,
        });
    
    let mut temp_pv = server.create_pv_double("temp:sensor1", 23.5, metadata)?;
    
    // Start server
    server.start()?;
    println!("Server running - PV available at: temp:sensor1");
    
    // Update values periodically
    for i in 0..100 {
        let new_temp = 23.5 + (i as f64 * 0.1);
        temp_pv.post_double(new_temp)?;
        thread::sleep(Duration::from_secs(1));
    }
    
    Ok(())
}
```

## Building

### Standard Build

```powershell
# Windows - Make sure environment variables are set
$env:EPICS_BASE = "C:\epics\base"
$env:EPICS_HOST_ARCH = "windows-x64"
$env:EPICS_PVXS = "C:\epics\pvxs"

# Build the library
cargo build

# Run tests (requires EPICS environment)
cargo test
```

```bash
# Linux/macOS - Make sure environment variables are set
export EPICS_BASE=/path/to/epics/base
export EPICS_HOST_ARCH=linux-x86_64
export EPICS_PVXS=/path/to/pvxs

# Build the library
cargo build
```

### Build Examples

```powershell
# Windows - Run the metadata server example
cargo run --example metadata_server

# Test from another terminal using EPICS pvget/pvinfo
pvget temperature:sensor1
pvinfo temperature:sensor1
```

### Available Examples

This repository includes comprehensive examples demonstrating all major features:

#### Server Examples
- **`metadata_server.rs`** - Complete EPICS server with rich NTScalar metadata (display, control, alarms)

Run the metadata server example:
```bash
cargo run --example metadata_server

# In another terminal, test the PV:
pvget temperature:sensor1
pvinfo temperature:sensor1  # See full metadata structure
```

### Running Tests

The crate includes an extensive test suite covering all functionality:

```bash
# Run all tests
cargo test

# Run specific test categories
cargo test test_client         # Client operations
cargo test test_server         # Server operations
cargo test test_monitor        # Monitor functionality
cargo test test_value          # Value operations
cargo test test_arrays         # Array operations
```

**Note**: Tests create isolated servers and do not require external IOCs.

### Available Examples

This repository includes comprehensive examples demonstrating all major features:

#### Client Examples
- **`metadata_server.rs`** - Complete EPICS server with rich NTScalar metadata (display, control, alarms)

Run the metadata server example:
```bash
cargo run --example metadata_server

# In another terminal, test the PV:
pvget temperature:sensor1
pvinfo temperature:sensor1  # See full metadata structure
```

### Running Tests

The crate includes an extensive test suite covering all functionality:

```bash
# Run all tests
cargo test

# Run specific test categories
cargo test test_client         # Client operations
cargo test test_server         # Server operations
cargo test test_monitor        # Monitor functionality
cargo test test_value          # Value operations
cargo test test_arrays         # Array operations
```

**Note**: Tests create isolated servers and do not require external IOCs.

## Project Structure

```text
pvxs-sys/
├── build.rs                           # Build script (C++ compilation, C++17)
├── Cargo.toml                         # Rust package manifest
├── build-pvxs-only.ps1                # Automated PVXS build script for Windows
├── BUILDING_PVXS_WINDOWS.md           # Detailed Windows build guide
├── include/
│   └── wrapper.h                      # C++ wrapper header (shared by client & server)
├── src/
│   ├── lib.rs                         # Main Rust API (safe, idiomatic)
│   ├── bridge.rs                      # CXX bridge definitions
│   ├── client_wrapper.cpp             # C++ client wrapper (GET/PUT/INFO)
│   ├── client_wrapper_async.cpp       # C++ async operations wrapper
│   ├── client_wrapper_monitor.cpp     # C++ monitor/subscription wrapper
│   ├── client_wrapper_rpc.cpp         # C++ RPC wrapper
│   └── server_wrapper.cpp             # C++ server wrapper (Server/SharedPV/StaticSource)
├── examples/
│   └── metadata_server.rs             # Server with full NTScalar metadata
├── tests/                             # Comprehensive test suite
│   ├── test_client_context_*.rs       # Client operation tests
│   ├── test_server_*.rs               # Server tests
│   ├── test_monitor_*.rs              # Monitor tests
│   ├── test_value*.rs                 # Value and array tests
│   └── test_integration_*.rs          # Integration tests
└── README.md                          # This file
```

## API Overview

### Client API

```rust
// Context - Main client entry point
let mut ctx = Context::from_env()?;

// GET operations
let value = ctx.get("PV:NAME", timeout)?;
let v = value.get_field_double("value")?;

// PUT operations (scalars)
ctx.put_double("PV:NAME", 42.0, timeout)?;
ctx.put_int32("PV:NAME", 123, timeout)?;
ctx.put_string("PV:NAME", "text", timeout)?;
ctx.put_enum("PV:NAME", 2, timeout)?;

// PUT operations (arrays)
ctx.put_double_array("PV:NAME", vec![1.0, 2.0, 3.0], timeout)?;
ctx.put_int32_array("PV:NAME", vec![10, 20, 30], timeout)?;
ctx.put_string_array("PV:NAME", vec!["a".to_string(), "b".to_string()], timeout)?;

// INFO operations
let info = ctx.info("PV:NAME", timeout)?;

// Monitor operations - Basic usage with get_update()
let mut monitor = ctx.monitor("PV:NAME")?;
monitor.start()?;
let update = monitor.get_update(timeout)?;  // Blocking, waits for data
monitor.stop()?;

// Monitor operations - Advanced with MonitorBuilder
let mut monitor = ctx.monitor_builder("PV:NAME")?
    .connect_exception(true)      // Throw exception on connection events
    .disconnect_exception(true)   // Throw exception on disconnection events
    .exec()?;
monitor.start()?;

// Using pop() - Non-blocking, returns immediately
use pvxs_sys::MonitorEvent;
loop {
    match monitor.pop() {
        Ok(Some(value)) => {
            // Got data update
            println!("Value: {}", value.get_field_double("value")?);
        }
        Ok(None) => {
            // Queue empty, no data available
            break;
        }
        Err(MonitorEvent::Connected(msg)) => {
            // Connection event (when connect_exception(true))
            println!("Connected: {}", msg);
        }
        Err(MonitorEvent::Disconnected(msg)) => {
            // Disconnection event (when disconnect_exception(true))
            println!("Disconnected: {}", msg);
        }
        Err(MonitorEvent::Finished(msg)) => {
            // Monitor finished/closed
            println!("Finished: {}", msg);
            break;
        }
    }
}

// Monitor with C-style callback
extern "C" fn my_callback() {
    println!("Monitor event occurred!");
}

let mut monitor = ctx.monitor_builder("PV:NAME")?
    .connect_exception(false)     // Queue connection events as data
    .disconnect_exception(false)  // Queue disconnection events as data
    .event(my_callback)           // Set callback function
    .exec()?;
monitor.start()?;
// Callback is invoked automatically when events occur
```

### Server API

```rust
// Create server
let mut server = Server::from_env()?;          // Network-enabled
let mut server = Server::create_isolated()?;   // Local-only

// Create PVs with metadata
let metadata = NTScalarMetadataBuilder::new()
    .alarm(severity, status, "message")
    .display(DisplayMetadata { ... })
    .control(ControlMetadata { ... })
    .value_alarm(ValueAlarmMetadata { ... });

// Scalar PVs
let mut pv1 = server.create_pv_double("name", 42.0, metadata)?;
let mut pv2 = server.create_pv_int32("name", 123, metadata)?;
let mut pv3 = server.create_pv_string("name", "text", metadata)?;

// Array PVs
let mut pv4 = server.create_pv_double_array("name", vec![1.0, 2.0], metadata)?;
let mut pv5 = server.create_pv_int32_array("name", vec![10, 20], metadata)?;
let mut pv6 = server.create_pv_string_array("name", vec!["a".to_string()], metadata)?;

// StaticSource - organize PVs into groups
let mut source = StaticSource::create()?;
source.add_pv("device:pv1", &mut pv1)?;
server.add_source("static", &mut source, priority)?;

// Server lifecycle
server.start()?;
let port = server.tcp_port();
server.stop()?;

// Update PV values
pv1.post_double(99.9)?;
pv2.post_int32(456)?;
pv3.post_string("updated")?;
```

### Value API

```rust
// Access scalar fields
let d = value.get_field_double("value")?;
let i = value.get_field_int32("value")?;
let s = value.get_field_string("value")?;
let e = value.get_field_enum("value")?;

// Access array fields
let da = value.get_field_double_array("value")?;
let ia = value.get_field_int32_array("value")?;
let sa = value.get_field_string_array("value")?;

// Access alarm information
let severity = value.get_field_int32("alarm.severity")?;
let status = value.get_field_int32("alarm.status")?;
let message = value.get_field_string("alarm.message")?;

// Display value structure
println!("{}", value);  // Pretty-print entire structure
```

### Monitor API

The Monitor API provides real-time PV change notifications with flexible event handling:

```rust
use pvxs_sys::{Context, MonitorEvent};

// 3. Event-driven with callbacks
extern "C" fn on_monitor_event() {
    println!("Monitor event detected!");
}

// 1. Simple monitoring with get_update() - Blocking
let mut monitor = ctx.monitor("PV:NAME")?;
monitor.start()?;
let value = monitor.get_update(5.0)?;  // Wait up to 5 seconds
monitor.stop()?;

// 2. Non-blocking with pop() - Returns immediately
let mut monitor = ctx.monitor_builder("PV:NAME")?
    .connect_exception(true)      // Enable connection exceptions
    .disconnect_exception(true)   // Enable disconnection exceptions
    .event(on_monitor_event)      // Register callback
    .exec()?;

// 3. Exception masking behavior
// connect_exception(true)  -> Connection events throw MonitorEvent::Connected
// connect_exception(false) -> Connection events queued as normal data
// disconnect_exception(true)  -> Disconnection events throw MonitorEvent::Disconnected
// disconnect_exception(false) -> Disconnection events queued as normal data

// 4. Registered callback
// Callback invoked automatically when data arrives

monitor.start()?;
loop {
    match monitor.pop() {
        Ok(Some(value)) => {
            // New data available
            println!("Got update: {}", value.get_field_double("value")?);
        }
        Ok(None) => {
            // Queue is empty, no data right now
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }
        Err(MonitorEvent::Connected(msg)) => {
            println!("PV Connected: {}", msg);
        }
        Err(MonitorEvent::Disconnected(msg)) => {
            println!("PV Disconnected: {}", msg);
        }
        Err(MonitorEvent::Finished(msg)) => {
            println!("Monitor finished: {}", msg);
            break;
        }
    }
}
monitor.stop()?;
```

**Monitor Methods:**
- `start()` - Begin monitoring (enables event flow)
- `stop()` - Stop monitoring (disables event flow)
- `get_update(timeout)` - Blocking wait for next update (convenience method)
- `pop()` - Non-blocking check for updates (returns `Result<Option<Value>, MonitorEvent>`)

**MonitorEvent Exceptions:**
- `Connected(String)` - Connection established (when `connect_exception(true)`)
- `Disconnected(String)` - Connection lost (when `disconnect_exception(true)`)
- `Finished(String)` - Monitor closed/finished

**Callback Signature:**
```rust
extern "C" fn callback() {
    // Called from PVXS worker thread
    // Keep processing minimal - no blocking operations
}
```

## Architecture

The crate uses a four-layer architecture with modular client/server separation optimized for C++17:

```text
┌─────────────────────────────────────┐
│   Rust API (src/lib.rs)             │  ← Safe, idiomatic Rust
│   - Context, Server, Value          │    High-level abstractions
│   - Result<T, E>, PvxsError         │    Ergonomic error handling
│   - NTScalarMetadataBuilder         │    Builder patterns
└─────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────┐
│   CXX Bridge (src/bridge.rs)        │  ← Type-safe FFI boundary
│   - Opaque C++ types                │    Zero-cost abstractions
│   - Shared structs (metadata)       │    Shared data structures
│   - Function declarations           │    C++17 features exposed
└─────────────────────────────────────┘
                    ↓
┌──────────────────┬──────────────────┐
│   Client Layer   │   Server Layer   │  ← Parallel C++ adapters
│ (4 cpp files)    │ (server_wrapper) │    Modular design
├──────────────────┼──────────────────┤
│ • GET/PUT/INFO   │ • Server/SharedPV│
│ • Async ops      │ • StaticSource   │
│ • Monitoring     │ • NTScalar types │
│ • RPC client     │ • Metadata       │
└──────────────────┴──────────────────┘
              ↓              ↓
┌─────────────────────────────────────┐
│   PVXS C++ Library (v1.4.1+)        │  ← EPICS PVXS
│   - pvxs::client::Context           │    C++17 based
│   - pvxs::server::Server            │
│   - pvxs::Value, pvxs::SharedPV     │
│   - pvxs::nt::NTScalar              │
└─────────────────────────────────────┘
```

### Why This Architecture?

1. **CXX Bridge**: Type-safe FFI without manual `unsafe` blocks, leveraging C++17 features
2. **Modular C++ Adapters**: Separate client modules (wrapper, async, monitor, RPC) and server for maintainability
3. **Client Layer**: Four specialized C++ files handle different client patterns (sync, async, monitoring, RPC)
4. **Server Layer**: Complete server implementation with metadata builders and NTScalar support
5. **Rust API**: Idiomatic Rust interface with builder patterns, error handling, and safe abstractions

## Common PV Field Names

When accessing fields in a `Value`, these field names are commonly used:

### NTScalar Structure
- **`value`** - The primary data value (double, int32, string, enum, or array)
- **`alarm.severity`** - Alarm severity (0=NO_ALARM, 1=MINOR, 2=MAJOR, 3=INVALID)
- **`alarm.status`** - Alarm status code
- **`alarm.message`** - Alarm message string
- **`timeStamp.secondsPastEpoch`** - Seconds since POSIX epoch
- **`timeStamp.nanoseconds`** - Nanoseconds component

### Metadata Fields (when present)
- **`display.limitLow`** - Display lower limit
- **`display.limitHigh`** - Display upper limit
- **`display.description`** - Human-readable description
- **`display.units`** - Engineering units (e.g., "DegC", "m/s")
- **`display.precision`** - Decimal precision for display
- **`control.limitLow`** - Control lower limit
- **`control.limitHigh`** - Control upper limit
- **`control.minStep`** - Minimum increment
- **`valueAlarm.lowAlarmLimit`** - Low alarm threshold
- **`valueAlarm.highAlarmLimit`** - High alarm threshold

## Troubleshooting

### Build Errors

**Error: "EPICS_BASE environment variable not set"**
```powershell
# Windows
$env:EPICS_BASE = "C:\epics\base"

# Linux/macOS
export EPICS_BASE=/path/to/epics/base
```

**Error: "cannot find -lpvxs"**
- Ensure PVXS is built and installed
- Check that `$EPICS_PVXS/lib/$EPICS_HOST_ARCH` contains `pvxs.lib` and `pvxs.dll` (Windows) or `libpvxs.so` (Linux) or `libpvxs.dylib` (macOS)

**Error: "pvxs/client.h: No such file or directory"**
- Ensure PVXS headers are installed in `$EPICS_PVXS/include/pvxs/`

### Runtime Errors

**Error: "Failed to create context from environment"**
- Check that EPICS network configuration is correct
- Verify `EPICS_PVA_ADDR_LIST` if needed
- Ensure no firewall is blocking UDP port 5076

**Error: "GET failed: timeout"**
- Increase the timeout value
- Check that the PV exists and IOC is running
- Verify network connectivity to IOC

## Platform Support

| Platform | Status | Compiler Requirements | Notes |
|----------|--------|----------------------|-------|
| Windows x64 | ✅ Fully Tested | MSVC 2017+ (C++17) | Primary development platform, requires CMake |
| Linux x86_64 | 🔄 Supported implicitlty but not tested | GCC 7+ or Clang 5+ (C++17) | Build system tested |
| macOS x86_64 | 🔄 Supported implicitlty but not tested | Clang 5+ (C++17) | Build system tested |
| macOS ARM64 | 🔄 Should work | Clang (C++17) | Apple Silicon compatibility expected |

## Implementation Status

### ✅ Fully Implemented
- **Client Operations**: GET, PUT (all types), INFO, async variants
- **Server Operations**: Full server with SharedPV, StaticSource, NTScalar metadata
- **Data Types**: double, int32, string, enum, and array variants
- **Monitoring**: MonitorBuilder with callbacks, event masking, exception handling
- **Arrays**: Complete support for double[], int32[], string[] in both client and server
- **Metadata**: NTScalar with display, control, valueAlarm, and enum choices
- **Network**: Full EPICS discovery, broadcasting, TCP/UDP communication
- **Async**: Tokio-based async/await for client operations (optional feature)

### 🚧 Planned Enhancements
- [ ] RPC (Remote Procedure Call) - Framework exists, needs comprehensive examples
- [ ] Custom normative types beyond NTScalar
- [ ] Advanced value field navigation utilities
- [ ] Connection state callbacks and event handlers
- [ ] Batch operations for improved performance
- [ ] Higher-level idiomatic `epics-pvxs` crate (non-sys)

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

## License

This project is licensed under MPL 2.0 ([LICENSE](LICENSE))

## References

- [EPICS Website](https://epics-controls.org/)
- [PVXS Documentation](https://epics-base.github.io/pvxs/)
- [PVXS GitHub Repository](https://github.com/epics-base/pvxs)
- [CXX Crate Documentation](https://cxx.rs/)

## Acknowledgments

This project builds upon:

- **PVXS** - The EPICS PVXS library by Michael Davidsaver and contributors
- **EPICS Base** - The Experimental Physics and Industrial Control System
- **CXX** - Safe C++/Rust interop by David Tolnay


