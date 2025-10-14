# Getting Started with EPICS PVXS Rust Bindings

This guide will help you set up and start using the EPICS PVXS Rust bindings.

## Step 1: Install EPICS Base

### On Linux

```bash
# Download EPICS Base
cd /opt
sudo mkdir epics
cd epics
sudo git clone --recursive https://github.com/epics-base/epics-base.git base
cd base

# Build EPICS Base
sudo make -j$(nproc)
```

### On Windows

1. Download EPICS Base from https://epics-controls.org/
2. Extract to `C:\epics\base`
3. Open Developer Command Prompt for VS
4. Navigate to base directory and run:
   ```cmd
   make
   ```

## Step 2: Install PVXS

### Prerequisites

PVXS requires CMake for building the libevent dependency:

```bash
# Ubuntu/Debian
sudo apt-get install cmake

# CentOS/RHEL
sudo yum install cmake

# macOS
brew install cmake
```

### Build PVXS

```bash
# Clone PVXS
cd /opt/epics
sudo git clone https://github.com/epics-base/pvxs.git
cd pvxs

# Configure (create a file CONFIG_SITE.local)
echo "EPICS_BASE=/opt/epics/base" | sudo tee CONFIG_SITE.local

# Build libevent first
sudo make -C bundle libevent.linux-x86_64  # or appropriate arch

# Build PVXS
sudo make -j$(nproc)
```

## Step 3: Set Environment Variables

### Linux (add to ~/.bashrc)

```bash
export EPICS_BASE=/opt/epics/base
export EPICS_HOST_ARCH=linux-x86_64
export PVXS_DIR=/opt/epics/pvxs

# Optional: Add EPICS binaries to PATH
export PATH="$EPICS_BASE/bin/$EPICS_HOST_ARCH:$PATH"
export LD_LIBRARY_PATH="$EPICS_BASE/lib/$EPICS_HOST_ARCH:$PVXS_DIR/lib/$EPICS_HOST_ARCH:$LD_LIBRARY_PATH"

# PVXS network configuration (optional)
export EPICS_PVA_ADDR_LIST=127.0.0.1
export EPICS_PVA_AUTO_ADDR_LIST=YES
```

### Windows (PowerShell profile)

```powershell
$env:EPICS_BASE = "C:\epics\base"
$env:EPICS_HOST_ARCH = "windows-x64"
$env:PVXS_DIR = "C:\epics\pvxs"

# Add to PATH
$env:PATH = "$env:EPICS_BASE\bin\$env:EPICS_HOST_ARCH;$env:PATH"
```

## Step 4: Create a Test Rust Project

```bash
# Create a new Rust project
cargo new --bin pvxs_test
cd pvxs_test

# Add dependency
cargo add epics-pvxs-sys --path /path/to/epics-pvxs-sys
```

Edit `src/main.rs`:

```rust
use epics_pvxs_sys::{Context, PvxsError};

fn main() -> Result<(), PvxsError> {
    println!("Creating PVXS context...");
    let ctx = Context::from_env()?;
    println!("Context created successfully!");
    
    // Try to get a PV value
    match ctx.get("test:pv", 5.0) {
        Ok(val) => println!("Value: {}", val),
        Err(e) => println!("Error (expected if no IOC running): {}", e),
    }
    
    Ok(())
}
```

## Step 5: Test with a Simple IOC

Create a test IOC to serve some PVs:

### Using pvxsCtx (PVXS command-line tool)

```bash
# Start a simple test server
pvxsCtx put test:pv=42.5

# In another terminal, test your Rust program
cd pvxs_test
cargo run
```

### Or create a minimal IOC

Create `test.db`:

```
record(ai, "test:pv") {
    field(DESC, "Test PV")
    field(VAL, 42.5)
    field(SCAN, "1 second")
}
```

## Step 6: Run Examples

```bash
cd /path/to/epics-pvxs-sys

# Make sure you have a running IOC with test:pv

# Get a PV value
cargo run --example simple_get -- test:pv

# Put a PV value
cargo run --example simple_put -- test:pv 123.456
```

## Common Issues

### "cannot find -lpvxs"

**Solution**: Check library paths
```bash
ls $PVXS_DIR/lib/$EPICS_HOST_ARCH/
# Should see libpvxs.so or similar
```

### "Failed to create context"

**Solution**: Check EPICS environment
```bash
# Test PVXS installation
pvxget -h  # Should show help

# Check network settings
echo $EPICS_PVA_ADDR_LIST
```

### Timeout errors

**Solution**: 
1. Verify IOC is running
2. Check firewall (UDP port 5076)
3. Check network connectivity

## Next Steps

- Read the [API documentation](https://docs.rs/epics-pvxs-sys)
- Explore the [examples](./examples/)
- Check out [PVXS documentation](https://epics-base.github.io/pvxs/)
- Join the [EPICS community](https://epics-controls.org/resources-and-support/community/)
