# Quick Start: Building PVXS on Windows

## TL;DR - Automated Build

```powershell
# Run this script (as Administrator recommended)
.\build-pvxs.ps1
```

That's it! The script will:
1. Check prerequisites
2. Clone and build EPICS Base
3. Build libevent (using CMake)
4. Clone and build PVXS
5. Set environment variables
6. Verify installation

Then:
```powershell
# Restart PowerShell, then:
cd C:\Users\tinez\repos\epics-pvxs-sys
cargo build
```

## Manual Build (If Script Fails)

### Prerequisites

1. **Visual Studio 2015+** with C++ tools
   - https://visualstudio.microsoft.com/downloads/
   
2. **Strawberry Perl** (includes Make)
   - https://strawberryperl.com/
   
3. **CMake** (for building libevent)
   - https://cmake.org/download/
   - Add to PATH during installation
   
4. **Git for Windows**
   - https://git-scm.com/download/win

### Build Steps

**In "x64 Native Tools Command Prompt for VS":**

```cmd
:: 1. Build EPICS Base
cd C:\epics
git clone --recursive https://github.com/epics-base/epics-base.git base
cd base
git checkout R7.0.8
make

:: 2. Build PVXS (with libevent)
cd C:\epics
git clone https://github.com/epics-base/pvxs.git
cd pvxs
git checkout 1.4.1

:: Create CONFIG_SITE.local with:
:: EPICS_BASE = C:/epics/base
:: SHARED_LIBRARIES = YES

:: Build libevent first (requires CMake)
make -C bundle libevent.windows-x64

:: Then build PVXS
make
```

### Set Environment Variables

**In PowerShell:**

```powershell
[Environment]::SetEnvironmentVariable("EPICS_BASE", "C:\epics\base", "User")
[Environment]::SetEnvironmentVariable("EPICS_HOST_ARCH", "windows-x64", "User")
[Environment]::SetEnvironmentVariable("PVXS_DIR", "C:\epics\pvxs", "User")

# Add to PATH
$path = [Environment]::GetEnvironmentVariable("PATH", "User")
$newPath = "$path;C:\epics\base\bin\windows-x64;C:\epics\pvxs\bin\windows-x64"
[Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
```

### Verify

```powershell
# Restart PowerShell, then:
pvxget -h
softIoc
```

## Full Documentation

For complete details, troubleshooting, and explanations:
- [BUILDING_PVXS_WINDOWS.md](BUILDING_PVXS_WINDOWS.md) - Comprehensive guide
- [README.md](README.md) - Main project documentation

## Common Issues

### "make: command not found"
→ Install Strawberry Perl: https://strawberryperl.com/

### "cmake: command not found"
→ Install CMake: https://cmake.org/download/
→ Make sure "Add to PATH" is selected during install

### "cl.exe not found"
→ Use "x64 Native Tools Command Prompt for VS"

### Build fails with errors
→ Check [BUILDING_PVXS_WINDOWS.md](BUILDING_PVXS_WINDOWS.md) troubleshooting section

## What Gets Built

```
C:\epics\
├── base\                    ← EPICS Base
│   ├── bin\windows-x64\     (executables)
│   ├── lib\windows-x64\     (Com.lib, etc.)
│   └── include\             (EPICS headers)
└── pvxs\                    ← PVXS
    ├── bundle\usr\windows-x64\  (libevent DLLs)
    ├── bin\windows-x64\     (pvxget.exe, pvxs.dll)
    ├── lib\windows-x64\     (pvxs.lib)
    └── include\pvxs\        (client.h, etc.)
```

## After Building

```powershell
# Build your Rust wrapper
cd C:\Users\tinez\repos\epics-pvxs-sys
cargo build

# Test it (need running IOC)
cargo run --example simple_get -- test:pv
```

## Estimated Time

- Prerequisites install: 10-20 minutes
- EPICS Base build: 10-30 minutes
- libevent build: 2-5 minutes
- PVXS build: 5-10 minutes
- **Total: ~30-65 minutes**

## Need Help?

- Full guide: [BUILDING_PVXS_WINDOWS.md](BUILDING_PVXS_WINDOWS.md)
- EPICS Tech-Talk: https://epics.anl.gov/tech-talk/
- PVXS Issues: https://github.com/epics-base/pvxs/issues
