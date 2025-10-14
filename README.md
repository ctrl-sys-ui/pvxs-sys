# epics-pvxs-sys

Safe Rust bindings for the [EPICS PVXS](https://github.com/epics-base/pvxs) (PVAccess) library.

This crate provides idiomatic Rust wrappers around the PVXS C++ library, which implements the PVAccess network protocol used in EPICS (Experimental Physics and Industrial Control System).

## Features

- ✅ **Safe Rust API** - Idiomatic Rust wrappers using the `cxx` crate
- ✅ **GET Operations** - Read process variable values
- ✅ **PUT Operations** - Write process variable values
- ✅ **INFO Operations** - Query PV type information
- ✅ **Thread-safe** - Context can be safely shared between threads
- 🚧 **Async Support** - Coming soon
- 🚧 **Monitor/Subscription** - Coming soon
- 🚧 **Server API** - Coming soon

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
- **Linux/macOS**: See [GETTING_STARTED.md](GETTING_STARTED.md)

### Environment Variables

Set the following environment variables:

- **`EPICS_BASE`** - Path to your EPICS base installation (required)
- **`EPICS_HOST_ARCH`** - Your host architecture (auto-detected if not set)
  - Examples: `linux-x86_64`, `windows-x64`, `darwin-x86`
- **`EPICS_PVXS`** - Path to PVXS installation (required)
  - Also accepts `PVXS_DIR` or `PVXS_BASE` as alternatives

Example setup:

```bash
# Linux
export EPICS_BASE=/opt/epics/base
export EPICS_HOST_ARCH=linux-x86_64
export EPICS_PVXS=/opt/epics/modules/pvxs
```

```powershell
# Windows (PowerShell)
$env:EPICS_BASE = "C:\epics\base"
$env:EPICS_HOST_ARCH = "windows-x64"
$env:EPICS_PVXS = "C:\epics\pvxs"
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
epics-pvxs-sys = "0.1"
```

## Quick Start

### Reading a PV Value (GET)

```rust
use epics_pvxs_sys::{Context, PvxsError};

fn main() -> Result<(), PvxsError> {
    // Create context from environment variables
    let mut ctx = Context::from_env()?;
    
    // Read a PV value with 5 second timeout
    let value = ctx.get("my:pv:name", 5.0)?;
    
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
    ctx.put_double("my:pv:name", 42.0, 5.0)?;
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
    let info = ctx.info("my:pv:name", 5.0)?;
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
cargo run --example simple_get -- TEST:PV1

# Run the simple_put example
cargo run --example simple_put -- TEST:PV1 42.5
```

```bash
# Linux/macOS - Build all examples
cargo build --examples

# Run examples
cargo run --example simple_get -- my:pv:name
cargo run --example simple_put -- my:pv:name 42.5
```

## Project Structure

```
epics-pvxs-sys/
├── Build.rs              # Build script (handles C++ compilation)
├── Cargo.toml            # Rust package manifest
├── include/
│   └── adapter.h        # C++ adapter header
├── src/
│   ├── lib.rs           # Main Rust API (safe, idiomatic)
│   ├── bridge.rs        # CXX bridge definitions
│   └── adapter.cpp      # C++ adapter implementation
├── examples/
│   ├── simple_get.rs    # GET operation example
│   └── simple_put.rs    # PUT operation example
└── README.md            # This file
```

## Architecture

The crate uses a three-layer architecture:

```
┌─────────────────────────────────────┐
│   Rust API (src/lib.rs)             │  ← Safe, idiomatic Rust
│   - Context, Value                  │
│   - Result<T>, PvxsError            │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│   CXX Bridge (src/bridge.rs)        │  ← Type-safe FFI boundary
│   - Opaque C++ types                │
│   - Function declarations           │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│   C++ Adapter (adapter.{h,cpp})     │  ← Simplifies C++ patterns
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

- [ ] Async/await support using Tokio
- [ ] Monitor/Subscription API for real-time updates
- [ ] Server API for serving PVs
- [ ] RPC (Remote Procedure Call) support
- [ ] Advanced value field navigation
- [ ] Custom type definitions
- [ ] Connection state callbacks
- [ ] Batch operations

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
