# EPICS PVXS Async Feature Documentation

## Overview

The EPICS PVXS Rust bindings now support optional async functionality to reduce dependencies for users who don't need asynchronous operations. This allows users to choose between a minimal synchronous-only build or a full-featured async-enabled build.

## Feature Configuration

### Default Configuration (Async Enabled)

```toml
[dependencies]
epics-pvxs-sys = "0.1.0"
```

By default, the `async` feature is enabled, providing access to both synchronous and asynchronous APIs along with tokio and futures dependencies.

### Minimal Configuration (Async Disabled)

```toml
[dependencies]
epics-pvxs-sys = { version = "0.1.0", default-features = false }
```

This configuration excludes async functionality and the associated tokio/futures dependencies, resulting in a smaller dependency tree.

## API Availability

### Synchronous API (Always Available)

The following methods are available in both configurations:

- `Context::from_env()` - Create client context
- `Context::get()` - Synchronous GET operations  
- `Context::put_double()` - Synchronous PUT operations
- `Context::info()` - Synchronous INFO operations
- All `Value` accessor methods
- Server and RPC functionality

### Asynchronous API (Async Feature Only)

The following methods are only available when the `async` feature is enabled:

- `Context::get_async()` - Asynchronous GET operations
- `Context::put_double_async()` - Asynchronous PUT operations  
- `Context::info_async()` - Asynchronous INFO operations
- `Operation` wrapper for managing async operations
- `wait_for_operation()` helper for async completion

## Build Examples

### Building with async support (default):
```bash
cargo build
# or explicitly:
cargo build --features async
```

### Building without async support:
```bash
cargo build --no-default-features
```

### Testing different configurations:
```bash
# Test without async
cargo test --no-default-features

# Test with async  
cargo test --features async
```

## Error Handling

When async functionality is disabled, attempting to call async methods will result in helpful error messages:

```
"Async operations are not enabled. Compile with --features async to use async functionality."
```

This ensures that users get clear guidance on how to enable async features if needed.

## Benefits

### Minimal Build (--no-default-features):
- Smaller dependency tree (no tokio, futures)
- Faster compilation
- Reduced binary size
- Simpler deployment for sync-only applications

### Full Build (default):
- Complete async support
- Integration with tokio ecosystem
- Non-blocking operations
- Concurrent PV operations

## Implementation Details

The optional async feature is implemented through:

1. **Rust Feature Flags**: `async` feature controls tokio/futures dependencies
2. **Conditional Compilation**: `#[cfg(feature = "async")]` guards async code
3. **C++ Preprocessor**: `PVXS_ASYNC_ENABLED` macro controls C++ async implementations
4. **Build System Integration**: Feature detection in build.rs passes flags to C++ compiler

This design ensures that:
- The C++ bridge interface remains consistent
- Async methods exist but throw helpful errors when disabled
- No runtime overhead for unused async functionality
- Clean separation between sync and async code paths

## Usage Recommendations

- **Use minimal build** for embedded systems, CLI tools, or simple applications
- **Use full build** for servers, concurrent applications, or tokio-based projects
- **Feature detection** at runtime is not necessary - compile-time selection is sufficient