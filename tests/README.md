# EPICS PVXS Comprehensive Test Suite

This directory contains comprehensive tests for all scalar types and their arrays in the EPICS PVXS Rust bindings.

## Test Structure

### Local Tests (Server-side operations)
These tests create an isolated server and perform server-side fetch/post operations without network communication.

#### Scalar Value Tests
- **`test_pvxs_local_int_fetch_post.rs`** - Original int32 local tests
- **`test_pvxs_local_double_fetch_post.rs`** - Double precision floating point tests
- **`test_pvxs_local_string_fetch_post.rs`** - String value tests

#### Array Tests  
- **`test_pvxs_local_double_array_fetch_post.rs`** - Double array local operations
- **`test_pvxs_local_int32_array_fetch_post.rs`** - Int32 array local operations
- **`test_pvxs_local_string_array_fetch_post.rs`** - String array local operations
- **`test_pvxs_local_enum_array_fetch_post.rs`** - Enum array local operations

### Remote Tests (Client-server operations) 
These tests create a server and use a separate client context to perform GET/PUT operations over the network.

#### Scalar Value Tests
- **`test_pvxs_remote_int_get_put.rs`** - Original int32 remote tests
- **`test_pvxs_remote_double_get_put.rs`** - Double precision remote tests
- **`test_pvxs_remote_string_get_put.rs`** - String remote tests

#### Array Tests
- **`test_pvxs_remote_double_array_get_put.rs`** - Double array operations
- **`test_pvxs_remote_int32_array_get_put.rs`** - Int32 array operations  
- **`test_pvxs_remote_string_array_get_put.rs`** - String array operations
- **`test_pvxs_remote_enum_array_get_put.rs`** - Enum array operations

## Test Coverage

### Scalar Types Covered
| Type | Local Tests | Remote Tests | Special Cases |
|------|-------------|--------------|---------------|
| **int32** | ✅ | ✅ | Boundary values, type conversion |
| **double** | ✅ | ✅ | Precision, special values (∞, NaN) |
| **string** | ✅ | ✅ | Unicode, special chars, encoding |

### Array Types Covered  
| Array Type | Local Tests | Remote Tests | Features Tested |
|------------|-------------|--------------|-----------------|
| **double[]** | ✅ | ✅ | Large arrays, special values, precision |
| **int32[]** | ✅ | ✅ | Boundary values, sequences, negatives |
| **string[]** | ✅ | ✅ | Unicode, empty strings, large strings |
| **enum[]** | ✅ | ✅ | Enum indices, choices, boundary values |

## Key Test Features

### Error Handling
- ✅ Invalid type conversions
- ✅ Network timeouts and failures
- ✅ Server restart scenarios
- ✅ Error propagation patterns

### Data Integrity
- ✅ Round-trip verification (PUT → GET → verify)
- ✅ Precision preservation for floating point
- ✅ Unicode and special character handling
- ✅ Boundary value testing

### Performance Considerations
- ✅ Large array handling (100+ elements)
- ✅ Empty array support
- ✅ Memory efficiency verification

### Real-world Scenarios
- ✅ Server stop/restart during operations
- ✅ Multiple data type conversions
- ✅ Enum with choices array handling
- ✅ Special floating point values (∞, -∞, NaN)

## Running the Tests

```bash
# Run all tests (requires EPICS environment)
cargo test

# Run specific test category
cargo test test_pvxs_local           # Local tests only (both scalar and array)
cargo test test_pvxs_remote          # Remote tests only  
cargo test array                     # Array tests only (both local and remote)
cargo test local.*array              # Local array tests only
cargo test remote.*array             # Remote array tests only

# Run individual test files
cargo test --test test_pvxs_local_double_array_fetch_post
cargo test --test test_pvxs_remote_string_array_get_put
cargo test --test test_pvxs_local_enum_array_fetch_post
```

## Test Requirements

- **EPICS Base** installed and configured
- **PVXS library** available
- **Environment variables** set:
  - `EPICS_BASE`
  - `EPICS_HOST_ARCH` 
  - `PVXS_DIR` (optional)

## Array Support Notes

Array tests include both **local** (server-side only) and **remote** (client-server) operations:

### Local Array Tests
- Test server-side array operations without network overhead
- Focus on data type handling, conversions, and boundary values
- More predictable behavior (no network timeouts)
- Useful for debugging server-side array processing

### Remote Array Tests  
- Test full client-server array communication
- Include network resilience and timeout handling
- Test real-world usage patterns
- May require specific server configurations depending on your EPICS setup

**Note:** Some servers may not support all array types. Array size limits may vary by server implementation. Enum arrays may need proper choices configuration.

Tests include fallback behavior when arrays aren't supported, printing informative messages rather than failing.

## Test Output

Tests provide detailed output including:
- Success/failure status for each operation
- Data verification results
- Performance information for large arrays
- Helpful error messages for unsupported features

Example output:
```
✓ String preserved: 'Unicode: αβγδ ελληνικά'
✓ Array values verified successfully
✓ Large array (100 elements) handled successfully
⚠ String not supported: 'Special chars' - Server limitation
```