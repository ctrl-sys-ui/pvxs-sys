# Quick Build - PVXS Only (You Already Have EPICS Base)

Since you already have EPICS Base installed, just run this:

## Step 1: Build PVXS

```powershell
# Make sure EPICS_BASE is set
echo $env:EPICS_BASE
# Should show your EPICS Base path

# Run the build script
.\build-pvxs-only.ps1
```

The script will:
- ✅ Verify EPICS Base is available
- ✅ Clone PVXS from GitHub
- ✅ Build libevent dependency (using CMake)
- ✅ Build PVXS (5-10 minutes)
- ✅ Set `EPICS_PVXS` environment variable
- ✅ Add PVXS to PATH

## Step 2: Restart PowerShell

Close and reopen PowerShell to load the new environment variables.

## Step 3: Verify PVXS

```powershell
# Test PVXS tools
pvxget -h

# Check environment
echo $env:EPICS_BASE
echo $env:EPICS_PVXS
echo $env:EPICS_HOST_ARCH
```

## Step 4: Build Your Rust Wrapper

```powershell
cd C:\Users\tinez\repos\epics-pvxs-sys
cargo build
```

That's it! 🎉

## If Build Fails

Make sure you have all prerequisites installed:

### Required Tools:
1. **Visual Studio** (with C++ tools)
2. **Strawberry Perl** (includes Make)
3. **CMake** (for building libevent)
4. **Git for Windows**

Check if tools are available:
```powershell
cl.exe        # Visual Studio compiler
make --version
cmake --version
git --version
```

### Option 1: Use VS Developer Command Prompt
1. Start Menu → "x64 Native Tools Command Prompt for VS"
2. Navigate to: `cd C:\Users\tinez\repos\epics-pvxs-sys`
3. Run: `powershell -ExecutionPolicy Bypass -File .\build-pvxs-only.ps1`

### Option 2: Load VS Tools in Current PowerShell
```powershell
# Adjust path for your VS version
cmd /c '"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat" x64 && set' | ForEach-Object {
    if ($_ -match "^(.*?)=(.*)$") {
        Set-Item -Path "Env:$($matches[1])" -Value $matches[2]
    }
}

# Then run the build script
.\build-pvxs-only.ps1
```

## Environment Variables Used

The build script and Rust Build.rs will look for PVXS in this order:
1. `EPICS_PVXS` (recommended)
2. `PVXS_DIR` (also supported)
3. `PVXS_BASE` (also supported)
4. Default to EPICS_BASE location

## What Gets Built

```
$env:EPICS_PVXS\              (or parent of EPICS_BASE\pvxs)
├── bundle\usr\windows-x64\   ← libevent (built by CMake)
│   └── lib\
│       ├── event_core.dll
│       └── event.dll
├── bin\windows-x64\
│   ├── pvxget.exe
│   ├── pvxput.exe
│   ├── pvxmonitor.exe
│   └── pvxs.dll
├── lib\windows-x64\
│   └── pvxs.lib
└── include\pvxs\
    ├── client.h
    ├── server.h
    └── ...
```

## Estimated Time
- libevent build: **2-5 minutes** (via CMake)
- PVXS build: **5-10 minutes**
- Rust wrapper build: **1-2 minutes**
- **Total: ~8-17 minutes**

## Need Help?
- Full Windows guide: [BUILDING_PVXS_WINDOWS.md](BUILDING_PVXS_WINDOWS.md)
- Complete documentation: [README.md](README.md)
