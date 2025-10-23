# Array Field Accessors Documentation

## Overview

The EPICS PVXS Rust bindings now support array field accessors for all primitive types:
- `f64` (double)
- `i32` (int32)
- `i16` (enum)
- `String`

## API Methods

### On the `Value` type:

```rust
impl Value {
    /// Get a field as an array of doubles
    pub fn get_field_double_array(&self, field_name: &str) -> Result<Vec<f64>>;
    
    /// Get a field as an array of int32 values
    pub fn get_field_int32_array(&self, field_name: &str) -> Result<Vec<i32>>;
    
    /// Get a field as an array of enums (int16 values)
    pub fn get_field_enum_array(&self, field_name: &str) -> Result<Vec<i16>>;
    
    /// Get a field as an array of strings
    pub fn get_field_string_array(&self, field_name: &str) -> Result<Vec<String>>;
}
```

## Usage Examples

### Reading a Double Array PV

```rust
use epics_pvxs_sys::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Context::from_env()?;
    
    // Get a PV that contains a double array
    let value = ctx.get("my:waveform:pv", 5.0)?;
    
    // Access the array field
    let data = value.get_field_double_array("value")?;
    
    println!("Array length: {}", data.len());
    for (i, val) in data.iter().enumerate() {
        println!("  [{}] = {:.3}", i, val);
    }
    
    Ok(())
}
```

### Reading Int32 Array

```rust
let value = ctx.get("my:int:array", 5.0)?;
let data = value.get_field_int32_array("value")?;

println!("Integer array: {:?}", data);
```

### Working with NTEnum (Enums with Choices)

EPICS NTEnum structures contain both an index and a choices array:

```rust
let value = ctx.get("my:enum:pv", 5.0)?;

// Get the current enum index
let index = value.get_field_enum("value.index")?;

// Get all available choices as strings
let choices = value.get_field_string_array("value.choices")?;

println!("Current selection: '{}' (index {})", choices[index as usize], index);
println!("Available choices:");
for (i, choice) in choices.iter().enumerate() {
    println!("  [{}] = '{}'", i, choice);
}
```

### Reading String Arrays

String arrays are useful for enum choices, labels, and text data:

```rust
let value = ctx.get("my:string:array", 5.0)?;
let strings = value.get_field_string_array("value")?;

for (i, s) in strings.iter().enumerate() {
    println!("String[{}]: '{}'", i, s);
}
```

## PVXS Internal Implementation

### How Arrays Work

PVXS stores arrays using `pvxs::shared_array<T>`, which is a reference-counted array type. The conversion process:

1. **PVXS Storage**: `pvxs::shared_array<const T>` (C++)
2. **Field Access**: `value_[field_name].as<pvxs::shared_array<const T>>()`
3. **Conversion**: Loop through and copy to `rust::Vec<T>`
4. **Return**: Zero-copy move to Rust as `Vec<T>`

### Example C++ Implementation

```cpp
rust::Vec<double> ValueWrapper::get_field_double_array(const std::string& field_name) const {
    auto field = value_[field_name];
    
    // Get the shared_array from PVXS
    auto arr = field.as<pvxs::shared_array<const double>>();
    
    // Convert to rust::Vec
    rust::Vec<double> result;
    for (size_t i = 0; i < arr.size(); ++i) {
        result.push_back(arr[i]);
    }
    return result;
}
```

## Common Use Cases

### 1. Waveform Records (Double Arrays)
```rust
// ADC waveform data
let waveform = ctx.get("scope:ch1:waveform", 1.0)?;
let data = waveform.get_field_double_array("value")?;
let avg: f64 = data.iter().sum::<f64>() / data.len() as f64;
println!("Average: {:.3}", avg);
```

### 2. Multi-State Enums
```rust
// Enum with multiple states
let status = ctx.get("device:status", 1.0)?;
let current_state = status.get_field_enum("value.index")?;
let states = status.get_field_string_array("value.choices")?;
println!("Status: {}", states[current_state as usize]);
```

### 3. Integer Arrays (Configuration Data)
```rust
// Configuration array
let config = ctx.get("device:config:array", 1.0)?;
let settings = config.get_field_int32_array("value")?;
for (i, setting) in settings.iter().enumerate() {
    println!("Setting {}: {}", i, setting);
}
```

### 4. String Arrays (Labels/Descriptions)
```rust
// Channel labels
let labels = ctx.get("device:channel:labels", 1.0)?;
let names = labels.get_field_string_array("value")?;
for (i, name) in names.iter().enumerate() {
    println!("Channel {}: {}", i, name);
}
```

## Error Handling

All array accessors return `Result<Vec<T>, PvxsError>`:

```rust
match value.get_field_double_array("value") {
    Ok(data) => {
        // Process data
        println!("Got {} values", data.len());
    }
    Err(e) => {
        eprintln!("Error reading array: {}", e);
        // Common errors:
        // - Field not found
        // - Field is not an array
        // - Type mismatch
    }
}
```

## Performance Considerations

1. **Data is Copied**: Arrays are copied from PVXS's shared_array to Rust's Vec
2. **Memory**: Large arrays will allocate Rust heap memory
3. **Efficiency**: For processing, consider iterating once rather than multiple field accesses
4. **Zero-Cost Abstraction**: The conversion uses standard iteration, no allocations until the Vec

## Type Mapping

| PVXS Type | Rust Type | Notes |
|-----------|-----------|-------|
| `pvxs::shared_array<const double>` | `Vec<f64>` | IEEE 754 double precision |
| `pvxs::shared_array<const int32_t>` | `Vec<i32>` | 32-bit signed integer |
| `pvxs::shared_array<const int16_t>` | `Vec<i16>` | 16-bit signed (enum indices) |
| `pvxs::shared_array<const std::string>` | `Vec<String>` | UTF-8 strings |

## Testing

See `tests/test_value_arrays.rs` for comprehensive examples and test cases.

Run tests (requires EPICS server):
```bash
cargo test --test test_value_arrays -- --ignored
```
