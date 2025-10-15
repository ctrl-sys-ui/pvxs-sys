# Setup EPICS/PVXS runtime environment for current session
# Run this before running the Rust examples

$ErrorActionPreference = "Stop"

Write-Host "Setting up EPICS/PVXS runtime environment..." -ForegroundColor Yellow

# Your current paths
$epicsBase = "C:\Users\tinez\repos\3rdparty\epics-base"
$pvxsDir = "C:\Users\tinez\repos\3rdparty\pvxs"
$hostArch = "windows-x64"

# Get libevent location (supports EPICS_PVXS_LIBEVENT override)
$libeventDir = $env:EPICS_PVXS_LIBEVENT
if (-not $libeventDir) {
    $libeventDir = Join-Path $pvxsDir "bundle\usr\$hostArch"
    Write-Host "Using default bundled libevent location: $libeventDir" -ForegroundColor Gray
} else {
    Write-Host "Using EPICS_PVXS_LIBEVENT: $libeventDir" -ForegroundColor Gray
}

# Check if directories exist
if (!(Test-Path $epicsBase)) {
    Write-Host "ERROR: EPICS_BASE directory not found: $epicsBase" -ForegroundColor Red
    exit 1
}

if (!(Test-Path $pvxsDir)) {
    Write-Host "ERROR: PVXS directory not found: $pvxsDir" -ForegroundColor Red
    exit 1
}

# DLL paths
$epicsBinPath = Join-Path $epicsBase "bin\$hostArch"
$pvxsBinPath = Join-Path $pvxsDir "bin\$hostArch"
$libeventLibPath = Join-Path $libeventDir "lib"

Write-Host "Checking for required DLLs..." -ForegroundColor Yellow

# Check if bin directories exist
if (!(Test-Path $epicsBinPath)) {
    Write-Host "WARNING: EPICS bin directory not found: $epicsBinPath" -ForegroundColor Yellow
} else {
    Write-Host "  ✓ EPICS bin directory found: $epicsBinPath" -ForegroundColor Green
    $epicsDlls = Get-ChildItem "$epicsBinPath\*.dll" -ErrorAction SilentlyContinue
    Write-Host "    Found $($epicsDlls.Count) EPICS DLLs" -ForegroundColor Gray
}

if (!(Test-Path $pvxsBinPath)) {
    Write-Host "WARNING: PVXS bin directory not found: $pvxsBinPath" -ForegroundColor Yellow
} else {
    Write-Host "  ✓ PVXS bin directory found: $pvxsBinPath" -ForegroundColor Green
    $pvxsDlls = Get-ChildItem "$pvxsBinPath\*.dll" -ErrorAction SilentlyContinue
    Write-Host "    Found $($pvxsDlls.Count) PVXS DLLs" -ForegroundColor Gray
}

if (!(Test-Path $libeventLibPath)) {
    Write-Host "WARNING: libevent lib directory not found: $libeventLibPath" -ForegroundColor Yellow
} else {
    Write-Host "  ✓ libevent lib directory found: $libeventLibPath" -ForegroundColor Green
    $libeventDlls = Get-ChildItem "$libeventLibPath\*.dll" -ErrorAction SilentlyContinue
    Write-Host "    Found $($libeventDlls.Count) libevent DLLs" -ForegroundColor Gray
}

# Set environment variables for current session
$env:EPICS_BASE = $epicsBase
$env:EPICS_HOST_ARCH = $hostArch
$env:EPICS_PVXS = $pvxsDir
$env:EPICS_PVXS_LIBEVENT = $libeventDir

# Add DLL paths to PATH for current session
$pathsToAdd = @()
if (Test-Path $epicsBinPath) {
    $pathsToAdd += $epicsBinPath
}
if (Test-Path $pvxsBinPath) {
    $pathsToAdd += $pvxsBinPath
}
if (Test-Path $libeventLibPath) {
    $pathsToAdd += $libeventLibPath
}

if ($pathsToAdd.Count -gt 0) {
    $newPath = ($pathsToAdd -join ";") + ";" + $env:PATH
    $env:PATH = $newPath
    Write-Host "Added to PATH for current session:" -ForegroundColor Green
    foreach ($path in $pathsToAdd) {
        Write-Host "  $path" -ForegroundColor Gray
    }
} else {
    Write-Host "No DLL paths found to add to PATH" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Environment setup complete!" -ForegroundColor Green
Write-Host "You can now run the examples:" -ForegroundColor Yellow
Write-Host "  cargo run --example simple_info -- `"TEST:PV_Bool`"" -ForegroundColor Gray
Write-Host "  cargo run --example simple_get -- `"TEST:PV_Bool`"" -ForegroundColor Gray
Write-Host "  cargo run --example simple_put -- `"TEST:PV_Bool`" 1" -ForegroundColor Gray