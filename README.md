# pvxs-sys

Safe Rust bindings for the [EPICS PVXS](https://github.com/epics-base/pvxs) (PVAccess) library.

`pvxs-sys` wraps the PVXS C++ API with idiomatic Rust types built on `cxx`. It covers the main client and server workflows needed for EPICS PVAccess applications: GET/PUT, monitors, server-side PV creation, metadata, and alarm handling.

## What This Crate Covers

- Client access through `Context`
- Real-time subscriptions through `MonitorBuilder`
- In-process test and application servers through `Server` and `ServerHandle`
- Scalar and array values for `double`, `int32`, `string`, and `enum`
- NT scalar and enum metadata, control limits, and value alarms

## Prerequisites

You need:

1. EPICS Base `>= 7.0.9`
2. PVXS `>= 1.4.1`
3. A C++17 compiler
4. CMake `>= 3.10`

Required environment variables:

| Variable | Description |
|---|---|
| `EPICS_BASE` | Path to the EPICS base installation |
| `EPICS_PVXS` | Path to the PVXS installation |
| `EPICS_HOST_ARCH` | Host architecture, if auto-detection is not enough |

Example setup:

```powershell
$env:EPICS_BASE = "C:\epics\base"
$env:EPICS_HOST_ARCH = "windows-x64"
$env:EPICS_PVXS = "C:\epics\pvxs"
```

```bash
export EPICS_BASE=/opt/epics/base
export EPICS_HOST_ARCH=linux-x86_64
export EPICS_PVXS=/opt/epics/pvxs
```

On Windows, the build script copies the required PVXS, EPICS, and libevent DLLs into `target/debug` and `target/release`.

## Install And Build

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
pvxs-sys = "0.1.1"
```

Then build and run tests:

```bash
cargo build
cargo test
```

## Start With The Tests

The unit tests in `tests/` are the best usage examples in this repository. They are small, focused, and cover the real APIs directly.

Recommended starting points:

- Local server-side fetch/post flows: `tests/test_local_double_fetch_post.rs`, `tests/test_local_int32_fetch_post.rs`, `tests/test_local_string_fetch_post.rs`, `tests/test_local_enum_fetch_post.rs`
- Local array flows: `tests/test_local_double_array_fetch_post.rs`, `tests/test_local_int32_array_fetch_post.rs`, `tests/test_local_string_array_fetch_post.rs`
- Remote client/server GET/PUT flows: `tests/test_remote_double_get_put.rs`, `tests/test_remote_int32_get_put.rs`, `tests/test_remote_string_get_put.rs`, `tests/test_remote_enum_get_put.rs`
- Remote array flows: `tests/test_remote_double_array_get_put.rs`, `tests/test_remote_int32_array_get_put.rs`, `tests/test_remote_string_array_get_put.rs`
- Monitor usage: `tests/test_monitor_builder.rs`, `tests/test_monitor_callbacks.rs`
- Alarm and metadata behavior: `tests/test_server_alarm_manager_lifecycle.rs`, `tests/test_server_alarm_manager_value_alarms.rs`, `tests/test_server_alarm_manager_control_limits.rs`, `tests/test_server_alarm_manager_combined.rs`

These tests show the crate in the form most users actually need: creating test servers, defining PVs, posting and fetching values, connecting client contexts, and verifying monitor and alarm behavior.

The files under `examples/` are still useful, but they are secondary to the tests when you want concrete API usage.

## Minimal Usage

Client GET:

```rust
use pvxs_sys::{Context, PvxsError};

fn main() -> Result<(), PvxsError> {
    let mut ctx = Context::from_env()?;
    let value = ctx.get("TEST:DOUBLE", 5.0)?;
    println!("{}", value.get_field_double("value")?);
    Ok(())
}
```

Server setup:

```rust
use pvxs_sys::{NTScalarMetadataBuilder, PvxsError, Server};

fn main() -> Result<(), PvxsError> {
    let server = Server::start_isolated()?;
    server.create_pv_double("sensor:temp", 23.5, NTScalarMetadataBuilder::new())?;
    server.post_double("sensor:temp", 25.0)?;
    let fetched = server.fetch_double("sensor:temp")?;
    println!("{}", fetched.value);
    server.stop_drop()?;
    Ok(())
}
```

## Main API Surface

- `Context` for GET, PUT, and monitor creation
- `MonitorBuilder` and monitor `pop()` for subscriptions
- `Server` and `ServerHandle` for hosted PVs
- `NTScalarMetadataBuilder` and `NTEnumMetadataBuilder` for metadata
- `AlarmSeverity`, `AlarmStatus`, `AlarmMetadata`, and `ControlMetadata` for alarm behavior
- `set_logger_level` for PVXS logging control

## Troubleshooting

`EPICS_BASE environment variable not set`

```powershell
$env:EPICS_BASE = "C:\epics\base"
```

`cannot find -lpvxs`

Ensure the PVXS libraries are present under the selected host architecture.

`pvxs/client.h: No such file or directory`

Ensure the PVXS headers are present in `include/pvxs` under your PVXS installation.

GET timeouts or failed discovery:

- verify the environment variables
- check any address-list settings for your EPICS environment
- make sure firewall rules are not blocking PVAccess traffic

## References

- [PVXS Documentation](https://epics-base.github.io/pvxs/)
- [PVXS GitHub](https://github.com/epics-base/pvxs)
- [CXX Crate](https://cxx.rs/)
- [LICENSE](LICENSE)

Safe Rust bindings for the [EPICS PVXS](https://github.com/epics-base/pvxs) (PVAccess) library.

This crate provides idiomatic Rust wrappers around the PVXS C++ library using the `cxx` crate. PVXS implements the PVAccess network protocol used in EPICS (Experimental Physics and Industrial Control System).

## Features

### Client
- **GET** — Read scalar and array PV values
- **PUT** — Write double, int32, string, and enum scalars and arrays
- **Monitor** — Real-time PV change subscriptions via `MonitorBuilder` + `pop()`

### Server
- **Start** — Network-enabled (`start_from_env`) or isolated for testing (`start_isolated`)
- **PV creation** — `create_pv_double/int32/string/enum` and `_array` variants
- **POST** — Publish new values with automatic alarm computation
- **Fetch** — Read server-side values with alarm state
- **Stop** — `stop_drop()` consumes the server and frees all resources
- **Handle** — `ServerHandle` for thread-safe access from multiple threads

Each server instance is backed by a dedicated worker thread and a thread-safe
[crossbeam](https://docs.rs/crossbeam-channel) channel. All `create_pv_*`,
`post_*`, and `fetch_*` calls are dispatched through this channel, so the
server can be driven safely from any number of threads simultaneously. The
worker thread also applies automatic alarm computation and control-limit
validation on every `post_*` call — bringing IOC-level alarm behaviour
(value alarms, control limit enforcement, severity/status propagation) into
pure Rust without any external IOC.

### Metadata & Alarms
- `NTScalarMetadataBuilder` / `NTEnumMetadataBuilder` — configure PV metadata at creation
- `ControlMetadata`, `AlarmMetadata` — control limits and value alarm thresholds
- `AlarmSeverity`, `AlarmStatus` — alarm state in fetched values and monitors

### Other
- `set_logger_level` — programmatically configure PVXS log levels
- Thread-safe `Context` (implements `Send + Sync`)

## Prerequisites

Before using this crate, you need:

1. **EPICS Base** (>= 7.0.9) — [epics-base](https://github.com/epics-base/epics-base)
2. **PVXS Library** (>= 1.4.1) — [pvxs](https://github.com/epics-base/pvxs)
3. **C++17 Compiler** — GCC >= 7, Clang >= 5, or MSVC >= 2017
4. **CMake** (>= 3.10) — Required for building the libevent dependency

### Environment Variables

| Variable | Required | Description |
|---|---|---|
| `EPICS_BASE` | Yes | Path to EPICS base installation |
| `EPICS_HOST_ARCH` | No | Host architecture (auto-detected if unset) |
| `EPICS_PVXS` | Yes | Path to PVXS installation (also accepts `PVXS_DIR` or `PVXS_BASE`) |
| `EPICS_PVXS_LIBEVENT` | No | Path to libevent (defaults to bundled libevent within PVXS) |

```powershell
# Windows (PowerShell)
$env:EPICS_BASE = "C:\epics\base"
$env:EPICS_HOST_ARCH = "windows-x64"
$env:EPICS_PVXS = "C:\epics\pvxs"
```

```bash
# Linux / macOS
export EPICS_BASE=/opt/epics/base
export EPICS_HOST_ARCH=linux-x86_64
export EPICS_PVXS=/opt/epics/modules/pvxs
```

### Runtime Requirements (Windows)

The build script automatically copies the following DLLs to `target/debug` and `target/release`:

- `pvxs.dll` from `{EPICS_PVXS}\bin\{EPICS_HOST_ARCH}`
- `Com.dll` from `{EPICS_BASE}\bin\{EPICS_HOST_ARCH}`
- `event_core.dll` from `{EPICS_PVXS}\bundle\usr\{EPICS_HOST_ARCH}\lib`

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
pvxs-sys = "0.1.1"
```

## Quick Start

### Reading a PV (GET)

```rust
use pvxs_sys::{Context, PvxsError};

fn main() -> Result<(), PvxsError> {
    let mut ctx = Context::from_env()?;

    let value = ctx.get("TEST:DOUBLE", 5.0)?;
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

    // Scalars
    ctx.put_double("TEST:DOUBLE", 42.0, 5.0)?;
    ctx.put_int32("TEST:INT", 123, 5.0)?;
    ctx.put_string("TEST:STRING", "hello", 5.0)?;
    ctx.put_enum("TEST:ENUM", 2, 5.0)?;

    // Arrays
    ctx.put_double_array("TEST:WAVEFORM", vec![1.0, 2.0, 3.0], 5.0)?;
    ctx.put_int32_array("TEST:SAMPLES", vec![10, 20, 30], 5.0)?;

    Ok(())
}
```

### Monitoring PV Changes

Use `monitor_builder` together with `pop()` for real-time subscriptions:

```rust
use pvxs_sys::{Context, MonitorEvent, PvxsError};

fn main() -> Result<(), PvxsError> {
    let mut ctx = Context::from_env()?;

    let mut monitor = ctx.monitor_builder("TEST:COUNTER")?
        .connect_exception(true)     // MonitorEvent::Connected on connect
        .disconnect_exception(true)  // MonitorEvent::Disconnected on disconnect
        .exec()?;

    monitor.start()?;

    loop {
        match monitor.pop() {
            Ok(Some(value)) => {
                let v = value.get_field_double("value")?;
                println!("Update: {}", v);
            }
            Ok(None) => {
                // Queue empty — sleep and retry
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(MonitorEvent::Connected(msg)) => println!("Connected: {}", msg),
            Err(MonitorEvent::Disconnected(msg)) => {
                println!("Disconnected: {}", msg);
                break;
            }
            Err(MonitorEvent::Finished(msg)) => {
                println!("Finished: {}", msg);
                break;
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    monitor.stop()?;
    Ok(())
}
```

### Creating an EPICS Server

```rust
use pvxs_sys::{Server, NTScalarMetadataBuilder, NTEnumMetadataBuilder,
               ControlMetadata, AlarmMetadata, AlarmSeverity, AlarmStatus, PvxsError};

fn main() -> Result<(), PvxsError> {
    let server = Server::start_from_env()?;
    println!("TCP port: {}", server.tcp_port());

    // Scalar PV with control limits — posts outside the range are rejected
    let metadata = NTScalarMetadataBuilder::new()
        .control(ControlMetadata {
            limit_low: 0.0,
            limit_high: 100.0,
            min_step: 0.1,
        });
    server.create_pv_double("sensor:temp", 23.5, metadata)?;

    // Scalar PV with value alarms
    let metadata = NTScalarMetadataBuilder::new()
        .alarm_metadata(AlarmMetadata {
            active: true,
            low_alarm_limit: 10.0,
            low_warning_limit: 20.0,
            high_warning_limit: 80.0,
            high_alarm_limit: 90.0,
            low_alarm_severity: AlarmSeverity::Major,
            low_warning_severity: AlarmSeverity::Minor,
            high_warning_severity: AlarmSeverity::Minor,
            high_alarm_severity: AlarmSeverity::Major,
            hysteresis: 0,
        });
    server.create_pv_double("sensor:pressure", 50.0, metadata)?;

    // Enum PV
    server.create_pv_enum("device:mode", vec!["Off", "On", "Auto"], 1,
                          NTEnumMetadataBuilder::new())?;

    // Array PV
    server.create_pv_double_array("sensor:waveform", vec![1.0, 2.0, 3.0],
                                  NTScalarMetadataBuilder::new())?;

    // Publish new values
    server.post_double("sensor:temp", 25.0)?;
    server.post_enum("device:mode", 2)?;

    // Read current server-side value (with alarm info)
    let fetched = server.fetch_double("sensor:temp")?;
    println!("temp={} severity={:?}", fetched.value, fetched.alarm_severity);

    // Use a cloneable handle for cross-thread access
    let handle = server.handle();
    std::thread::spawn(move || {
        handle.post_double("sensor:temp", 30.0).unwrap();
    });

    // Shut down and free all PVs
    server.stop_drop()?;
    Ok(())
}
```

## API Reference

### Client — `Context`

```rust
let mut ctx = Context::from_env()?;

// GET
let value = ctx.get("PV:NAME", timeout)?;

// PUT (scalars)
ctx.put_double("PV:NAME", 42.0, timeout)?;
ctx.put_int32("PV:NAME", 123, timeout)?;
ctx.put_string("PV:NAME", "text", timeout)?;
ctx.put_enum("PV:NAME", 2, timeout)?;

// PUT (arrays)
ctx.put_double_array("PV:NAME", vec![1.0, 2.0, 3.0], timeout)?;
ctx.put_int32_array("PV:NAME", vec![10, 20, 30], timeout)?;
ctx.put_string_array("PV:NAME", vec!["a".to_string()], timeout)?;
```

### Monitor

```rust
// MonitorBuilder — recommended
let mut monitor = ctx.monitor_builder("PV:NAME")?
    .connect_exception(true)     // true = raise MonitorEvent::Connected
    .disconnect_exception(true)  // true = raise MonitorEvent::Disconnected
    .event(my_callback)          // optional C callback: extern "C" fn()
    .exec()?;
monitor.start()?;

// pop() — non-blocking, call in a loop
match monitor.pop() {
    Ok(Some(value)) => { /* data update */ }
    Ok(None)        => { /* queue empty */ }
    Err(event)      => { /* MonitorEvent::Connected / Disconnected / Finished */ }
}

// Convenience queries
let connected = monitor.is_connected();
let pending   = monitor.has_update();
let name      = monitor.name();

monitor.stop()?;
```

### Server — `Server` / `ServerHandle`

```rust
// Start
let server = Server::start_from_env()?;   // network-enabled
let server = Server::start_isolated()?;   // test isolation

// Ports
server.tcp_port();
server.udp_port();

// Create PVs
server.create_pv_double("name", 0.0, NTScalarMetadataBuilder::new())?;
server.create_pv_int32("name", 0, NTScalarMetadataBuilder::new())?;
server.create_pv_string("name", "init", NTScalarMetadataBuilder::new())?;
server.create_pv_enum("name", vec!["Off", "On"], 0, NTEnumMetadataBuilder::new())?;
server.create_pv_double_array("name", vec![1.0], NTScalarMetadataBuilder::new())?;
server.create_pv_int32_array("name", vec![1], NTScalarMetadataBuilder::new())?;
server.create_pv_string_array("name", vec!["a".to_string()], NTScalarMetadataBuilder::new())?;

// Post values (with automatic alarm computation)
server.post_double("name", 42.0)?;
server.post_int32("name", 42)?;
server.post_string("name", "value")?;
server.post_enum("name", 1)?;
server.post_double_array("name", vec![1.0, 2.0])?;
server.post_int32_array("name", vec![1, 2])?;
server.post_string_array("name", vec!["a".to_string()])?;

// Fetch current value (server-side, with alarm info)
let f = server.fetch_double("name")?;   // FetchedDouble { value, alarm_severity, alarm_status, alarm_message, .. }
let f = server.fetch_int32("name")?;
let f = server.fetch_string("name")?;
let f = server.fetch_enum("name")?;
let f = server.fetch_double_array("name")?;
let f = server.fetch_int32_array("name")?;
let f = server.fetch_string_array("name")?;

// Remove a PV
server.remove_pv("name")?;

// Cloneable handle for multi-threaded access
let handle = server.handle();  // ServerHandle: Clone + Send

// Stop — consumes server, frees all resources
server.stop_drop()?;
```

### Metadata Builders

```rust
let metadata = NTScalarMetadataBuilder::new()
    .control(ControlMetadata {
        limit_low: 0.0,
        limit_high: 100.0,
        min_step: 0.1,
    })
    .alarm_metadata(AlarmMetadata {
        active: true,
        low_alarm_limit: 10.0,
        low_warning_limit: 20.0,
        high_warning_limit: 80.0,
        high_alarm_limit: 90.0,
        low_alarm_severity: AlarmSeverity::Major,
        low_warning_severity: AlarmSeverity::Minor,
        high_warning_severity: AlarmSeverity::Minor,
        high_alarm_severity: AlarmSeverity::Major,
        hysteresis: 0,
    });

// Enum metadata (no control/alarm fields)
let enum_meta = NTEnumMetadataBuilder::new();
```

### Value — reading fields

```rust
// Scalars
let d = value.get_field_double("value")?;
let i = value.get_field_int32("value")?;
let s = value.get_field_string("value")?;

// Arrays
let da = value.get_field_double_array("value")?;
let ia = value.get_field_int32_array("value")?;
let sa = value.get_field_string_array("value")?;

// Alarm fields
let sev = value.get_field_int32("alarm.severity")?;
let sta = value.get_field_int32("alarm.status")?;
let msg = value.get_field_string("alarm.message")?;

// Display value structure
println!("{}", value);
```

### Logging

```rust
// Suppress noisy TCP disconnect messages
pvxs_sys::set_logger_level("pvxs.tcp.io", "CRIT").ok();

// Available levels: CRIT < ERR < WARN < INFO < DEBUG
// Wildcard loggers: "pvxs.*"
```

## Building

```powershell
# Windows
$env:EPICS_BASE = "C:\epics\base"
$env:EPICS_HOST_ARCH = "windows-x64"
$env:EPICS_PVXS = "C:\epics\pvxs"
cargo build
cargo test
```

```bash
# Linux / macOS
export EPICS_BASE=/opt/epics/base
export EPICS_HOST_ARCH=linux-x86_64
export EPICS_PVXS=/opt/epics/pvxs
cargo build
cargo test
```

**Note**: Tests create isolated servers and do not require external IOCs.

## Troubleshooting

**`EPICS_BASE environment variable not set`**
```powershell
$env:EPICS_BASE = "C:\epics\base"
```

**`cannot find -lpvxs`**
Ensure PVXS is built and `$EPICS_PVXS/lib/$EPICS_HOST_ARCH` contains the PVXS library.

**`pvxs/client.h: No such file or directory`**
Ensure PVXS headers are in `$EPICS_PVXS/include/pvxs/`.

**`Failed to create context from environment` / GET timeout**
- Check `EPICS_PVA_ADDR_LIST` if targeting a remote IOC
- Ensure UDP port 5076 is not blocked by a firewall

## Platform Support

| Platform | Status | Notes |
|---|---|---|
| Windows x64 | ✅ Tested | Primary development platform |
| Linux x86_64 | 🔄 Should work | Build system compatible |
| macOS x86_64/ARM64 | 🔄 Should work | Build system compatible |

## License

MPL 2.0 — see [LICENSE](LICENSE)

## References

- [PVXS Documentation](https://epics-base.github.io/pvxs/)
- [PVXS GitHub](https://github.com/epics-base/pvxs)
- [CXX Crate](https://cxx.rs/)
