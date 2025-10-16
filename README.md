# epics-pvxs-sys

Low-level FFI bindings for the [EPICS PVXS](https://github.com/epics-base/pvxs) (PVAccess) library.

> **Note**: This is a `-sys` crate providing raw FFI bindings. For a high-level, idiomatic Rust API, use the `epics-pvxs` crate (coming soon).

This crate provides safe Rust bindings around the PVXS C++ library using the `cxx` crate. PVXS implements the PVAccess network protocol used in EPICS (Experimental Physics and Industrial Control System).

## Features

- ✅ **Safe FFI Bindings** - Memory-safe wrappers using the `cxx` crate
- ✅ **GET Operations** - Read process variable values
- ✅ **PUT Operations** - Write process variable values  
- ✅ **INFO Operations** - Query PV type information
- ✅ **Async Support** - Async/await support using Tokio
- ✅ **Monitor/Subscription** - Real-time PV monitoring
- ✅ **Thread-safe Examples** - Multiple concurrency patterns demonstrated
- 🚧 **Server API** - Coming soon
- 🚧 **RPC Support** - Remote procedure calls (in development)

## Crate Structure

This is a `-sys` crate following Rust conventions:

- **`epics-pvxs-sys`** (this crate) - Low-level FFI bindings
- **`epics-pvxs`** (planned) - High-level, idiomatic Rust API

## Prerequisites

Before using this crate, you need:

1. **EPICS Base** (>=3.15.1) - [Download here](https://epics-controls.org/resources-and-support/base/)
2. **PVXS Library** (>=1.0.0) - [Download here](https://github.com/epics-base/pvxs)
3. **C++11 Compiler** - GCC >= 4.8, Clang, or MSVC >= 2015
4. **CMake** (>=3.10) - Required for building libevent dependency - [Download here](https://cmake.org/download/)

### Building PVXS from Source

If you don't have PVXS installed, see our detailed guides:

- **Windows**: [BUILDING_PVXS_WINDOWS.md](BUILDING_PVXS_WINDOWS.md) - Step-by-step guide
  - Or use the automated script: `.\build-pvxs-only.ps1`
  - **Note**: Requires CMake for building libevent dependency
  - **Tip**: For environments with group policy restrictions, use: `.\build-pvxs-only.ps1 -TempDir "C:\Projects\Temp"`
- **Linux/macOS**: See [GETTING_STARTED.md](GETTING_STARTED.md)

#### Automated Windows Build Script

The `build-pvxs-only.ps1` script supports several options for different environments:

```powershell
# Basic usage (uses system defaults)
.\build-pvxs-only.ps1

# With custom temp directory (helpful for group policy restrictions)
.\build-pvxs-only.ps1 -TempDir "C:\Projects\Temp"

# With custom architecture
.\build-pvxs-only.ps1 -HostArch "windows-x64"

# With custom PVXS version and install location
.\build-pvxs-only.ps1 -PvxsVersion "1.4.1" -InstallDir "C:\Custom\Path"

# All options combined
.\build-pvxs-only.ps1 -PvxsVersion "1.4.1" -InstallDir "C:\epics\pvxs" -TempDir "C:\Projects\Temp" -HostArch "windows-x64"
```

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
epics-pvxs-sys = "0.1"

# For async support
epics-pvxs-sys = { version = "0.1", features = ["async"] }
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
use epics_pvxs_sys::{Context, PvxsError};

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

### Writing a PV Value (PUT)

```rust
use epics_pvxs_sys::{Context, PvxsError};

fn main() -> Result<(), PvxsError> {
    let mut ctx = Context::from_env()?;
    
    // Write a double value with 5 second timeout
    ctx.put_double("TEST:DOUBLE", 42.0, 5.0)?;
    println!("Value written successfully!");
    
    Ok(())
}
```

### Querying PV Type Information (INFO)

```rust
use epics_pvxs_sys::{Context, PvxsError};

fn main() -> Result<(), PvxsError> {
    let mut ctx = Context::from_env()?;
    
    // Get type information without reading data
    let info = ctx.info("TEST:DOUBLE", 5.0)?;
    println!("PV structure:\n{}", info);
    
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
# Windows - Build all examples
cargo build --examples

# Run the simple_get example (requires running IOC with test PV)
cargo run --example simple_get -- TEST:DOUBLE

# Run the simple_put example
cargo run --example simple_put -- TEST:DOUBLE 42.5

# Run the simple_info example (query PV type information)
cargo run --example simple_info -- TEST:DOUBLE

# Run the async example (requires async feature)
cargo run --features async --example simple_async -- TEST:COUNTER

# Run the RPC example (demonstrates remote procedure calls)
cargo run --example rpc_example -- service:function arg1=value1 arg2=42.0
```

## Testing

### Setting up Test IOC

This repository includes a comprehensive test database (`test.db`) with various PV types for testing. To use it:

```bash
# Start the soft IOC with the test database
softIocPVA test.db

# In another terminal, list available PVs
pvlist

# Test individual PVs
pvget TEST:DOUBLE
pvput TEST:DOUBLE 456.789
pvmonitor TEST:COUNTER
```

**Note**: The test database creates auto-updating PVs (like `TEST:COUNTER` and `TEST:SINEWAVE`) that change automatically, making them ideal for monitoring examples.

### Available Test PVs

The `test.db` database provides these PVs for testing:

- **Basic Data Types**: `TEST:DOUBLE`, `TEST:INTEGER`, `TEST:STRING`, `TEST:ENUM`
- **Auto-updating PVs**: `TEST:COUNTER`, `TEST:RANDOM`, `TEST:SINEWAVE`, `TEST:TEMPERATURE`
- **Setpoints**: `TEST:TEMP_SETPOINT`, `TEST:PRESSURE_SETPOINT`
- **Status/Control**: `TEST:STATUS`, `TEST:ENABLE`
- **Arrays**: `TEST:WAVEFORM`, `TEST:SUBARRAY`
- **Binary/Bits**: `TEST:BITS_IN`, `TEST:BITS_OUT`
- **Motor Simulation**: `TEST:MOTOR_POS`, `TEST:MOTOR_VEL`
- **Alarm Testing**: `TEST:ALARM_CYCLE`, `TEST:INIT_ALARM`
- **Special Cases**: `TEST:LONG_STRING`, `TEST:TIMESTAMP`
- **Calculations**: `TEST:CALC1`, `TEST:CALC2`

### Available Examples

This repository includes several examples demonstrating different functionality:

- **`simple_get.rs`** - Basic PV value retrieval
- **`simple_put.rs`** - PV value setting  
- **`simple_info.rs`** - PV metadata inspection
- **`simple_monitor.rs`** - Basic PV monitoring
- **`simple_async.rs`** - Asynchronous operations (requires `async` feature)
- **`rpc_example.rs`** - Remote procedure call demonstration

### Running Examples

```bash
# Test basic GET operation
cargo run --example simple_get -- TEST:DOUBLE

# Test PUT operation  
cargo run --example simple_put -- TEST:DOUBLE 123.456

# Test structure discovery
cargo run --example simple_info -- TEST:TEMPERATURE

# Test monitoring
cargo run --example simple_monitor -- TEST:COUNTER

# Test async operations (requires async feature)
cargo run --features async --example simple_async -- TEST:COUNTER

# Run the RPC example (demonstrates remote procedure calls)
cargo run --example rpc_example -- service:function arg1=value1 arg2=42.0
```

### Linux/macOS Examples

```bash
# Build all examples
cargo build --examples

# Run examples
cargo run --example simple_get -- TEST:DOUBLE
cargo run --example simple_put -- TEST:DOUBLE 42.5
cargo run --example simple_info -- TEST:TEMPERATURE
cargo run --example simple_monitor -- TEST:COUNTER
cargo run --features async --example simple_async -- TEST:COUNTER
```

## Project Structure

```text
epics-pvxs-sys/
├── build.rs                    # Build script (handles C++ compilation)
├── Cargo.toml                  # Rust package manifest
├── build-pvxs-only.ps1         # Automated PVXS build script for Windows
├── BUILDING_PVXS_WINDOWS.md    # Detailed Windows build guide
├── include/
│   └── wrapper.h               # C++ wrapper header
├── src/
│   ├── lib.rs                  # Main Rust API (safe, idiomatic)
│   ├── bridge.rs               # CXX bridge definitions
│   └── wrapper.cpp             # C++ wrapper implementation
├── examples/
│   ├── simple_get.rs           # GET operation example
│   ├── simple_put.rs           # PUT operation example
│   ├── simple_info.rs          # INFO operation example (query PV structure)
│   ├── simple_monitor.rs       # Basic PV monitoring
│   ├── simple_async.rs         # Async/await demonstration (requires 'async' feature)
│   └── rpc_example.rs          # RPC demonstration
└── README.md                   # This file
```

## Architecture

The crate uses a three-layer architecture:

```text
┌─────────────────────────────────────┐
│   Rust API (src/lib.rs)             │  ← Safe, idiomatic Rust
│   - Context, Value                  │
│   - Result<T, E>, PvxsError         │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│   CXX Bridge (src/bridge.rs)        │  ← Type-safe FFI boundary
│   - Opaque C++ types                │
│   - Function declarations           │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│   C++ Adapter (wrapper.{h,cpp})     │  ← Simplifies C++ patterns
│   - ContextWrapper                  │
│   - ValueWrapper                    │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│   PVXS C++ Library                  │  ← Original EPICS PVXS
│   - pvxs::client::Context           │
│   - pvxs::Value                     │
└─────────────────────────────────────┘
```

### Why This Architecture?

1. **CXX Bridge**: Provides type-safe FFI without manual `unsafe` blocks
2. **C++ Adapter**: Handles complex C++ patterns (callbacks, shared_ptr, templates)
3. **Rust API**: Provides idiomatic Rust interface with proper error handling

## Common PV Field Names

When accessing fields in a `Value`, common field names include:

- **`value`** - The primary data value
- **`alarm.severity`** - Alarm severity (0=NO_ALARM, 1=MINOR, 2=MAJOR, 3=INVALID)
- **`alarm.status`** - Alarm status code
- **`alarm.message`** - Alarm message string
- **`timeStamp.secondsPastEpoch`** - Timestamp seconds since POSIX epoch
- **`timeStamp.nanoseconds`** - Nanoseconds component of timestamp

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

| Platform | Status | Notes |
|----------|--------|-------|
| Windows x64 | ✅ Tested | Primary development platform, requires MSVC 2015+ and CMake |
| Linux x86_64 | 🚧 Should work | Build system supports it, not tested |
| macOS x86_64 | 🚧 Should work | Build system supports it, not tested |
| macOS ARM64 | 🚧 Untested | Should work with Apple Silicon |

## Future Enhancements

- ✅ Async/await support using Tokio
- ✅ Monitor/Subscription API for real-time updates
- [ ] Server API for serving PVs
- [ ] RPC (Remote Procedure Call) support
- [ ] Advanced value field navigation
- [ ] Custom type definitions
- [ ] Connection state callbacks
- [ ] Batch operations
- [ ] Enhanced error handling with detailed error contexts
- [ ] Performance optimizations for high-frequency monitoring

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

## License

This project is licensed under MIT License ([LICENSE-MIT](LICENSE-MIT))

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
