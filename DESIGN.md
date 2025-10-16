# Design Decisions and Architecture

This document explains the key design decisions made in creating these Rust bindings for EPICS PVXS.

## Why CXX Instead of Bindgen?

After analyzing the PVXS C++ API, we chose the `cxx` crate over `bindgen` for several important reasons:

### PVXS Uses Modern C++11 Features

PVXS heavily relies on:
- `std::shared_ptr<>` for memory management
- `std::function<>` for callbacks
- Template classes and methods
- RAII patterns
- Method chaining (Builder pattern)

**Bindgen limitations**: Cannot properly handle these patterns. Would generate raw FFI that's unsafe and difficult to use.

**CXX advantages**: Understands C++ ownership semantics, provides safe shared_ptr wrappers, and enforces lifetime rules.

### Callback Complexity

PVXS operations use callbacks extensively:
```cpp
auto op = ctx.get("pv")
             .result([](Result&& result){ /* callback */ })
             .exec();
```

**CXX approach**: We hide callbacks in the C++ adapter layer, exposing only synchronous operations to Rust for simplicity. Future async support can use Rust futures/channels.

### Type Safety at FFI Boundary

PVXS has a complex type system with dynamic structures (`Value` containers with runtime type information).

**CXX approach**: Opaque types + accessor functions maintain type safety while allowing flexible field access.

## Three-Layer Architecture

### Layer 1: C++ Adapter (adapter.h/cpp)

**Purpose**: Simplify PVXS patterns for FFI consumption

**Responsibilities**:
- Convert callbacks to synchronous operations
- Manage C++ object lifetimes
- Handle std::exception → error codes
- Provide simplified wrappers (ContextWrapper, ValueWrapper)

**Example**:
```cpp
// Simplifies complex builder pattern + callback
std::unique_ptr<ValueWrapper> get_sync(
    const std::string& pv_name, 
    double timeout)
{
    auto op = ctx_.get(pv_name).exec();
    return std::make_unique<ValueWrapper>(op->wait(timeout));
}
```

### Layer 2: CXX Bridge (bridge.rs)

**Purpose**: Define safe FFI boundary

**Responsibilities**:
- Declare opaque C++ types
- Define function signatures
- Handle automatic type conversions (rust::String ↔ std::string)
- Provide Result<T> for error handling

**Example**:
```rust
#[cxx::bridge(namespace = "pvxs_adapter")]
mod ffi {
    unsafe extern "C++" {
        type ContextWrapper;
        fn context_get_sync(...) -> Result<UniquePtr<ValueWrapper>>;
    }
}
```

### Layer 3: Safe Rust API (lib.rs)

**Purpose**: Provide idiomatic Rust interface

**Responsibilities**:
- Wrap opaque C++ types in Rust structs
- Implement Display, Debug traits
- Provide comprehensive documentation
- Convert cxx::Exception to PvxsError
- Add convenience methods

**Example**:
```rust
pub struct Context {
    inner: UniquePtr<ContextWrapper>,
}

impl Context {
    pub fn get(&self, pv_name: &str, timeout: f64) -> Result<Value> {
        let inner = bridge::context_get_sync(&self.inner, pv_name, timeout)?;
        Ok(Value { inner })
    }
}
```

## Key Design Patterns

### Opaque Wrapper Pattern

C++ objects are never exposed directly to Rust:
```rust
// Rust code never sees the C++ internals
pub struct Value {
    inner: UniquePtr<ValueWrapper>,  // Opaque C++ type
}
```

Benefits:
- C++ can use any internal representation
- No C++ headers needed in Rust code
- Clear ownership semantics

### Synchronous-First Approach

Initial implementation focuses on blocking operations:

**Rationale**:
1. Simpler to implement and test
2. Easier for users to understand
3. Covers most use cases
4. Async can be added later without breaking changes

**Future async approach**:
```rust
// Future API (non-breaking addition)
impl Context {
    pub async fn get_async(&self, pv_name: &str) -> Result<Value> {
        // Use Rust channels/futures to bridge C++ callbacks
    }
}
```

### Error Handling Strategy

All C++ exceptions are caught in the adapter layer:

```cpp
try {
    // PVXS operation
} catch (const std::exception& e) {
    throw PvxsError(std::string("Operation failed: ") + e.what());
}
```

CXX automatically converts to `Result<T>`:
```rust
let value = ctx.get("pv", 5.0)?;  // ? operator for error propagation
```

### Value Field Access

Dynamic field access using string names:
```rust
let v = value.get_field_double("value")?;
let severity = value.get_field_int32("alarm.severity")?;
```

**Rationale**: 
- PV structures are dynamic (not known at compile time)
- Matches PVXS's own field access pattern
- Type conversion happens at runtime with proper error handling

## Build System Design

### Environment Variable Strategy

Required: `EPICS_BASE`, `PVXS_DIR`  
Optional: `EPICS_HOST_ARCH` (auto-detected)

**Rationale**: 
- Standard EPICS convention
- Allows flexible installation paths
- build.rs can find includes and libraries

### Cross-Platform Considerations

build.rs handles platform differences:
- Library extensions (.so vs .dll vs .dylib)
- Compiler flags (-std=c++11 vs /std:c++11)
- System libraries (pthread on Linux, ws2_32 on Windows)

### Cargo Integration

Uses `links = "pvxs"` to:
- Prevent multiple versions linking to same C library
- Allow dependent crates to find include paths
- Follow Rust FFI best practices

## Trade-offs and Limitations

### Current Limitations

1. **No async support** - Only blocking operations
2. **No monitors** - Real-time subscriptions not yet implemented
3. **Limited value types** - Only double, int32, string accessors
4. **No server API** - Client operations only
5. **Simplified PUT** - Only writes to "value" field

### Why These Limitations?

**MVP approach**: Get basic functionality working first, then iterate.

**Each can be added without breaking changes**:
- Async: Add new `*_async()` methods
- Monitors: New `Subscription` type
- Value types: Add more `get_field_*` methods
- Server: New `Server` type in separate module
- Complex PUT: Add builder pattern for constructing Values

## Future Enhancements

### Phase 2: Async Support

```rust
pub struct Context {
    // Add async methods alongside sync ones
    pub async fn get_async(&self, pv_name: &str) -> Result<Value> {
        // Use tokio channels to bridge C++ callbacks
    }
}
```

### Phase 3: Monitors

```rust
pub struct Subscription {
    // Stream of value updates
    pub async fn next(&mut self) -> Option<Result<Value>> { ... }
}

impl Context {
    pub fn monitor(&self, pv_name: &str) -> Result<Subscription> { ... }
}
```

### Phase 4: Server API

```rust
pub struct Server {
    // Serve PVs from Rust
}

impl Server {
    pub fn add_pv(&mut self, name: &str, initial: Value) { ... }
}
```

### Phase 5: Advanced Value Construction

```rust
impl Value {
    pub fn builder() -> ValueBuilder { ... }
}

// Create complex structures
let value = Value::builder()
    .field("value", 42.0)
    .field("alarm.severity", 0)
    .build()?;
```

## Performance Considerations

### Minimal Overhead

- **Zero-copy** where possible (UniquePtr transfers ownership)
- **Direct calls** through CXX (no serialization)
- **Stack allocation** for simple types

### Allocation Strategy

- C++ objects stay in C++ heap (managed by unique_ptr)
- String conversions allocate but unavoidable for FFI
- Future optimization: Pass string views for read-only access

### Threading

- Context is Send + Sync (safe to share)
- PVXS manages its own thread pool
- No additional Rust threads needed for sync operations

## Testing Strategy

### Unit Tests

Test at each layer:
1. **C++ layer**: Test adapter functions independently
2. **Bridge layer**: Test FFI boundary
3. **Rust layer**: Test public API

### Integration Tests

Require running EPICS environment:
- Test against real IOCs
- Test timeout behavior
- Test error conditions

### CI/CD Considerations

Challenge: Need EPICS environment in CI

Options:
1. Docker containers with EPICS pre-installed
2. GitHub Actions with setup scripts
3. Mock C++ implementation for basic tests

## Documentation Philosophy

### Three Levels of Documentation

1. **README.md**: Quick start, examples, troubleshooting
2. **GETTING_STARTED.md**: Step-by-step setup guide
3. **API docs**: Comprehensive rustdoc comments

### Code Comments

- **Why** not **what**: Code should be self-documenting
- **Rationale**: Explain design decisions
- **Examples**: Show usage patterns

## Conclusion

This design prioritizes:
1. **Safety**: Type-safe FFI, no manual unsafe blocks
2. **Ergonomics**: Idiomatic Rust API
3. **Maintainability**: Clear layering, well-documented
4. **Extensibility**: Easy to add features without breaking changes

The result is a solid foundation that can grow into a complete PVXS binding while remaining safe and pleasant to use.
