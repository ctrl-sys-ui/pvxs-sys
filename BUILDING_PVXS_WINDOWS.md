# Building PVXS on Windows - Step by Step Guide

This guide will walk you through building EPICS Base and PVXS on your Windows machine.

## Prerequisites

### 1. Install Build Tools

You need:
- **Visual Studio 2015 or newer** (Community Edition is free)
  - Download from: https://visualstudio.microsoft.com/downloads/
  - Make sure to install "Desktop development with C++" workload
- **Git for Windows**
  - Download from: https://git-scm.com/download/win
- **Perl** (for EPICS build system)
  - Download Strawberry Perl: https://strawberryperl.com/
- **GNU Make** (usually included with Strawberry Perl)
- **CMake** (for building libevent dependency)
  - Download from: https://cmake.org/download/
  - Make sure to add CMake to PATH during installation

### 2. Verify Tools

Open PowerShell and verify:

```powershell
# Check Visual Studio compiler
cl.exe
# Should show Microsoft C/C++ Compiler version

# Check Perl
perl --version

# Check Make
make --version

# Check CMake
cmake --version

# Check Git
git --version
```

## Step 1: Build EPICS Base

### Create Directory Structure

```powershell
# Create EPICS directory
New-Item -ItemType Directory -Path C:\epics -Force
cd C:\epics

# Clone EPICS Base
git clone --recursive https://github.com/epics-base/epics-base.git base
cd base
```

### Configure EPICS Base

Check out a stable release (recommended):

```powershell
# List available tags
git tag

# Checkout latest 7.x release (or whatever is latest)
git checkout R7.0.8
```

### Build EPICS Base

Open **"x64 Native Tools Command Prompt for VS 2022"** (or your VS version):

```cmd
cd C:\epics\base
make
```

This will take 10-30 minutes depending on your machine.

### Set Environment Variables

Add to your PowerShell profile (or set permanently in System Properties):

```powershell
# Edit profile
notepad $PROFILE

# Add these lines:
$env:EPICS_BASE = "C:\epics\base"
$env:EPICS_HOST_ARCH = "windows-x64"
$env:PATH = "$env:EPICS_BASE\bin\$env:EPICS_HOST_ARCH;$env:PATH"

# Save and reload
. $PROFILE
```

Or set permanently:

```powershell
[Environment]::SetEnvironmentVariable("EPICS_BASE", "C:\epics\base", "User")
[Environment]::SetEnvironmentVariable("EPICS_HOST_ARCH", "windows-x64", "User")

# Add to PATH
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$newPath = "$currentPath;C:\epics\base\bin\windows-x64"
[Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
```

### Verify EPICS Base Installation

```powershell
# Test that softIoc works
softIoc.exe
# Should start (press Ctrl+C to exit)
```

## Step 2: Build PVXS

### Clone PVXS Repository

```powershell
cd C:\epics
git clone https://github.com/epics-base/pvxs.git
cd pvxs
```

### Check Out Stable Release

```powershell
# List available tags
git tag

# Checkout latest stable release
git checkout 1.4.1
```

### Configure PVXS

Create a configuration file:

```powershell
# Create CONFIG_SITE.local
@"
# EPICS Base location
EPICS_BASE = C:/epics/base

# Build shared libraries (DLLs on Windows)
SHARED_LIBRARIES = YES
STATIC_BUILD = NO
"@ | Out-File -FilePath CONFIG_SITE.local -Encoding ASCII
```

### Build libevent (PVXS Dependency)

PVXS requires libevent. Build it first:

```cmd
cd C:\epics\pvxs

:: Build libevent using the bundled source
make -C bundle libevent.windows-x64
```

This will download and build libevent using CMake. Takes 2-5 minutes.

### Build PVXS

In the **"x64 Native Tools Command Prompt for VS"**:

```cmd
cd C:\epics\pvxs

:: First build libevent
make -C bundle libevent.windows-x64

:: Then build PVXS
make
```

This should take 5-15 minutes total (libevent + PVXS).

### Set PVXS Environment Variable

```powershell
# Add to profile or set permanently
$env:PVXS_DIR = "C:\epics\pvxs"

# Or permanently:
[Environment]::SetEnvironmentVariable("PVXS_DIR", "C:\epics\pvxs", "User")

# Add PVXS DLLs to PATH
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$newPath = "$currentPath;C:\epics\pvxs\bin\windows-x64"
[Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
```

### Verify PVXS Installation

```powershell
# Test pvxget command
pvxget.exe -h
# Should show help message

# Check that libraries exist
dir C:\epics\pvxs\lib\windows-x64\
# Should see pvxs.lib and pvxs.dll

# Check headers
dir C:\epics\pvxs\include\pvxs\
# Should see client.h and other headers
```

## Step 3: Test Your Installation

### Create a Simple Test IOC

Create a test database file `test.db`:

```
record(ai, "test:pv") {
    field(DESC, "Test PV")
    field(VAL, 42.5)
    field(SCAN, "1 second")
}
```

Start a softIOC with PVXS:

```powershell
# In one PowerShell window
softIoc -d test.db
```

### Test with PVXS Tools

In another PowerShell window:

```powershell
# Get the value
pvxget test:pv

# Put a value
pvxput test:pv 123.456

# Monitor for changes
pvxmonitor test:pv
```

## Step 4: Build Your Rust Project

Now you can build your Rust wrapper:

```powershell
cd C:\Users\tinez\repos\epics-pvxs-sys

# Verify environment
echo $env:EPICS_BASE
echo $env:EPICS_HOST_ARCH
echo $env:PVXS_DIR

# Build
cargo build

# Run example (with IOC running)
cargo run --example simple_get -- test:pv
```

## Troubleshooting

### "make: command not found"

Install Strawberry Perl which includes GNU Make:
- Download: https://strawberryperl.com/
- Install and restart PowerShell

### "cmake: command not found"

Install CMake which is required for building libevent:
- Download: https://cmake.org/download/
- During installation, select "Add CMake to system PATH"
- Restart PowerShell after installation

### "cl.exe not found" or "MSVC not found"

You need to run commands in "x64 Native Tools Command Prompt for VS":
1. Press Windows key
2. Type "x64 native tools"
3. Run the VS command prompt

Or add VS to your path:

```powershell
# For VS 2022 (adjust version as needed)
$vcvarsall = "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat"
cmd /c "`"$vcvarsall`" x64 && set" | ForEach-Object {
    if ($_ -match "^(.*?)=(.*)$") {
        Set-Item -Path "Env:$($matches[1])" -Value $matches[2]
    }
}
```

### EPICS Base Build Fails

Common issues:
- **Perl not found**: Install Strawberry Perl
- **Make not found**: Install Strawberry Perl or GNU Make
- **Wrong compiler**: Use VS Native Tools Command Prompt

### PVXS Build Fails

Check:
```powershell
# Verify EPICS_BASE is set
echo $env:EPICS_BASE

# Verify EPICS Base is built
dir $env:EPICS_BASE\lib\$env:EPICS_HOST_ARCH\
# Should see Com.lib and other libraries

# Verify CMake is available
cmake --version
# Should show CMake 3.x or newer
```

If libevent build fails, ensure CMake is installed and in PATH.

### Rust Build Fails

Check environment variables:
```powershell
# All three should be set
echo $env:EPICS_BASE
echo $env:EPICS_HOST_ARCH
echo $env:PVXS_DIR

# Check libraries exist
dir $env:PVXS_DIR\lib\$env:EPICS_HOST_ARCH\
# Should see pvxs.lib

# Check headers
dir $env:PVXS_DIR\include\pvxs\
# Should see client.h
```

## Quick Commands Summary

```powershell
# === Build Everything ===

# 1. EPICS Base
cd C:\epics\base
# (In VS Native Tools Command Prompt)
make

# 2. PVXS (with libevent)
cd C:\epics\pvxs
# Create CONFIG_SITE.local with EPICS_BASE path
make -C bundle libevent.windows-x64  # Build libevent first
make                                   # Then build PVXS

# 3. Set Environment
$env:EPICS_BASE = "C:\epics\base"
$env:EPICS_HOST_ARCH = "windows-x64"
$env:PVXS_DIR = "C:\epics\pvxs"

# 4. Build Rust
cd C:\Users\tinez\repos\epics-pvxs-sys
cargo build

# 5. Test
# Start IOC in one window:
softIoc -d test.db

# Run example in another:
cargo run --example simple_get -- test:pv
```

## Alternative: Use Pre-built Binaries

If building is problematic, you can use pre-built EPICS binaries:

1. Download EPICS Base Windows binaries from: https://epics.anl.gov/download/base/
2. Extract to `C:\epics\base`
3. Build only PVXS following Step 2 above

## Next Steps

Once everything is built and working:

1. Read the main [README.md](README.md) for API documentation
2. Try the examples in `examples/`
3. Create your own EPICS applications!

## Getting Help

- **EPICS Tech-Talk**: https://epics.anl.gov/tech-talk/
- **PVXS GitHub Issues**: https://github.com/epics-base/pvxs/issues
- **EPICS Documentation**: https://docs.epics-controls.org/
