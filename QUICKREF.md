# EPICS PVXS Rust Bindings - Quick Reference

## Installation

```toml
[dependencies]
epics-pvxs-sys = "0.1"
```

## Environment Setup

```bash
# Required environment variables
export EPICS_BASE=/path/to/epics/base
export EPICS_HOST_ARCH=linux-x86_64
export EPICS_PVXS=/path/to/pvxs  # Or PVXS_DIR

# Note: PVXS requires CMake for building libevent dependency
```

## Basic Usage

### Create Context

```rust
use epics_pvxs_sys::{Context, PvxsError};

let ctx = Context::from_env()?;
```

### GET Operation

```rust
// Simple get
let value = ctx.get("my:pv", 5.0)?;
println!("{}", value);

// Access specific field
let v = value.get_field_double("value")?;
let severity = value.get_field_int32("alarm.severity")?;
let msg = value.get_field_string("alarm.message")?;
```

### PUT Operation

```rust
// Write a double value
ctx.put_double("my:pv", 42.0, 5.0)?;
```

### INFO Operation

```rust
// Get type information
let info = ctx.info("my:pv", 5.0)?;
println!("PV structure: {}", info);
```

## Common Field Names

| Field | Type | Description |
|-------|------|-------------|
| `value` | varies | Main value |
| `alarm.severity` | int32 | 0=OK, 1=MINOR, 2=MAJOR, 3=INVALID |
| `alarm.status` | int32 | Status code |
| `alarm.message` | string | Alarm message |
| `timeStamp.secondsPastEpoch` | int32 | Unix timestamp |
| `timeStamp.nanoseconds` | int32 | Nanoseconds |

## Error Handling

```rust
match ctx.get("my:pv", 5.0) {
    Ok(value) => println!("Value: {}", value),
    Err(e) => eprintln!("Error: {}", e),
}

// Or use ? operator
let value = ctx.get("my:pv", 5.0)?;
```

## Common Patterns

### Read-Modify-Write

```rust
// Read current value
let current = ctx.get("my:pv", 5.0)?;
let v = current.get_field_double("value")?;

// Modify
let new_value = v + 1.0;

// Write back
ctx.put_double("my:pv", new_value, 5.0)?;
```

### Batch Operations

```rust
let pvs = vec!["pv1", "pv2", "pv3"];
for pv in pvs {
    match ctx.get(pv, 5.0) {
        Ok(v) => println!("{}: {}", pv, v),
        Err(e) => eprintln!("{}: Error - {}", pv, e),
    }
}
```

### Checking Alarm State

```rust
let value = ctx.get("my:pv", 5.0)?;
let severity = value.get_field_int32("alarm.severity")?;

match severity {
    0 => println!("OK"),
    1 => println!("MINOR alarm"),
    2 => println!("MAJOR alarm"),
    3 => println!("INVALID"),
    _ => println!("Unknown severity"),
}
```

## Command Line Examples

```bash
# Run simple_get example
cargo run --example simple_get -- my:pv:name

# Run simple_put example
cargo run --example simple_put -- my:pv:name 123.45

# Build release version
cargo build --release

# Run tests
cargo test
```

## Troubleshooting

### Timeout Errors
- Increase timeout value
- Check IOC is running: `pvget my:pv`
- Check network: `ping <ioc-host>`

### Build Errors
- Verify `$EPICS_BASE` is set and valid
- Check PVXS is installed
- Ensure C++ compiler is available

### Library Not Found
```bash
# Linux
export LD_LIBRARY_PATH=$EPICS_BASE/lib/$EPICS_HOST_ARCH:$LD_LIBRARY_PATH

# macOS
export DYLD_LIBRARY_PATH=$EPICS_BASE/lib/$EPICS_HOST_ARCH:$DYLD_LIBRARY_PATH
```

## More Information

- Full documentation: [README.md](README.md)
- Setup guide: [GETTING_STARTED.md](GETTING_STARTED.md)
- Design rationale: [DESIGN.md](DESIGN.md)
- PVXS docs: https://epics-base.github.io/pvxs/
