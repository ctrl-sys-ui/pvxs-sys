# Build PVXS only - EPICS Base already installed
param(
    [string]$PvxsVersion = "1.4.1",
    [string]$InstallDir = $env:EPICS_PVXS
)

$ErrorActionPreference = "Stop"

Write-Host "======================================== " -ForegroundColor Cyan
Write-Host "PVXS Build Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Load Visual Studio environment
Write-Host "Loading Visual Studio environment..." -ForegroundColor Yellow

$vsPaths = @(
    "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat",
    "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvarsall.bat",
    "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvarsall.bat",
    "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvarsall.bat",
    "C:\Program Files (x86)\Microsoft Visual Studio\2019\Professional\VC\Auxiliary\Build\vcvarsall.bat",
    "C:\Program Files (x86)\Microsoft Visual Studio\2019\Enterprise\VC\Auxiliary\Build\vcvarsall.bat"
)

$vcvarsPath = $null
foreach ($path in $vsPaths) {
    if (Test-Path $path) {
        $vcvarsPath = $path
        break
    }
}

if (-not $vcvarsPath) {
    Write-Host "ERROR: Visual Studio not found!" -ForegroundColor Red
    Write-Host "Please install Visual Studio with C++ development tools" -ForegroundColor Yellow
    exit 1
}

Write-Host "Found Visual Studio at: $vcvarsPath" -ForegroundColor Green

# Load environment variables from vcvarsall.bat
$tempFile = [System.IO.Path]::GetTempFileName()
cmd /c "`"$vcvarsPath`" x64 && set > `"$tempFile`""

Get-Content $tempFile | ForEach-Object {
    if ($_ -match '^(.*?)=(.*)$') {
        $varName = $matches[1]
        $varValue = $matches[2]
        Set-Item -Path "Env:\$varName" -Value $varValue
    }
}

Remove-Item $tempFile
Write-Host "Visual Studio environment loaded successfully" -ForegroundColor Green
Write-Host ""

# Check EPICS_BASE
if (-not $env:EPICS_BASE) {
    Write-Host "ERROR: EPICS_BASE not set!" -ForegroundColor Red
    exit 1
}

Write-Host "Found EPICS_BASE: $env:EPICS_BASE" -ForegroundColor Green

# Verify EPICS is built
$hostArch = if ($env:EPICS_HOST_ARCH) { $env:EPICS_HOST_ARCH } else { "windows-x64" }
$comLib = Join-Path $env:EPICS_BASE "lib\$hostArch\Com.lib"

if (-not (Test-Path $comLib)) {
    Write-Host "ERROR: EPICS Base not built! Missing: $comLib" -ForegroundColor Red
    exit 1
}

Write-Host "EPICS Base is built" -ForegroundColor Green
Write-Host ""

# Determine install location
if (-not $InstallDir) {
    $InstallDir = Join-Path (Split-Path $env:EPICS_BASE -Parent) "pvxs"
}

Write-Host "Will install to: $InstallDir" -ForegroundColor Yellow
Write-Host ""

# Check tools
Write-Host "Checking tools..." -ForegroundColor Yellow

# Check git
if (Get-Command git -ErrorAction SilentlyContinue) {
    Write-Host "  git found" -ForegroundColor Green
} else {
    Write-Host "  git MISSING" -ForegroundColor Red
    exit 1
}

# Check perl
if (Get-Command perl -ErrorAction SilentlyContinue) {
    Write-Host "  perl found" -ForegroundColor Green
} else {
    Write-Host "  perl MISSING" -ForegroundColor Red
    exit 1
}

# Check make (try both 'make' and 'gnumake')
$makeCmd = $null
if (Get-Command make -ErrorAction SilentlyContinue) {
    $makeCmd = "make"
    Write-Host "  make found" -ForegroundColor Green
} elseif (Get-Command gnumake -ErrorAction SilentlyContinue) {
    $makeCmd = "gnumake"
    Write-Host "  gnumake found (will use as make)" -ForegroundColor Green
} else {
    Write-Host "  make/gnumake MISSING" -ForegroundColor Red
    exit 1
}

Write-Host ""

# Get PVXS source
Write-Host "Getting PVXS source..." -ForegroundColor Yellow
$parent = Split-Path $InstallDir -Parent

if (-not (Test-Path $parent)) {
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
}

if (Test-Path $InstallDir) {
    Write-Host "Updating existing PVXS..." -ForegroundColor Yellow
    Push-Location $InstallDir
    git fetch --all --tags
} else {
    Write-Host "Cloning PVXS..." -ForegroundColor Yellow
    Push-Location $parent
    git clone https://github.com/epics-base/pvxs.git (Split-Path $InstallDir -Leaf)
    Push-Location (Split-Path $InstallDir -Leaf)
}

git checkout $PvxsVersion
Write-Host ""

# Configure
Write-Host "Configuring build..." -ForegroundColor Yellow
$baseUnix = $env:EPICS_BASE -replace '\\', '/'

# Set EPICS_BASE in configure/RELEASE file
$releaseFile = "configure\RELEASE"
if (Test-Path $releaseFile) {
    $content = Get-Content $releaseFile
    $newContent = $content | ForEach-Object {
        if ($_ -match '^\s*#?\s*EPICS_BASE\s*=') {
            "EPICS_BASE = $baseUnix"
        } else {
            $_
        }
    }
    $newContent | Out-File -FilePath $releaseFile -Encoding ASCII -Force
    Write-Host "Updated configure/RELEASE with EPICS_BASE" -ForegroundColor Green
}

# Also create CONFIG_SITE.local for additional settings
@"
SHARED_LIBRARIES = YES
STATIC_BUILD = NO
"@ | Out-File -FilePath "CONFIG_SITE.local" -Encoding ASCII -Force

Write-Host ""

# Build libevent first
Write-Host "Building bundled libevent..." -ForegroundColor Yellow
Push-Location bundle
& $makeCmd libevent.windows-x64

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "libevent build FAILED!" -ForegroundColor Red
    Pop-Location
    Pop-Location
    exit 1
}
Pop-Location
Write-Host "libevent build SUCCESS!" -ForegroundColor Green
Write-Host ""

# Build PVXS
Write-Host "Building PVXS (5-10 minutes)..." -ForegroundColor Yellow
Write-Host "Using: $makeCmd" -ForegroundColor Gray
Write-Host "EPICS_BASE = $env:EPICS_BASE" -ForegroundColor Gray
Write-Host "EPICS_HOST_ARCH = $hostArch" -ForegroundColor Gray

# Make sure environment variables are set for the build
$env:EPICS_HOST_ARCH = $hostArch

& $makeCmd clean 2>&1 | Out-Null
& $makeCmd -j4

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "PVXS build FAILED!" -ForegroundColor Red
    Pop-Location
    exit 1
}

Write-Host ""
Write-Host "Build SUCCESS!" -ForegroundColor Green
Pop-Location

# Set environment
Write-Host ""
Write-Host "Setting environment..." -ForegroundColor Yellow
[Environment]::SetEnvironmentVariable("EPICS_PVXS", $InstallDir, "User")

$binPath = Join-Path $InstallDir "bin\$hostArch"
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")

if ($currentPath -notlike "*$binPath*") {
    [Environment]::SetEnvironmentVariable("PATH", "$currentPath;$binPath", "User")
}

$env:EPICS_PVXS = $InstallDir
$env:PATH = "$binPath;$env:PATH"

Write-Host "EPICS_PVXS = $InstallDir" -ForegroundColor Green
Write-Host ""

# Verify
Write-Host "Verifying..." -ForegroundColor Yellow
$files = @(
    "bin\$hostArch\pvxget.exe",
    "lib\$hostArch\pvxs.lib",
    "bin\$hostArch\pvxs.dll",
    "include\pvxs\client.h"
)

foreach ($file in $files) {
    $path = Join-Path $InstallDir $file
    if (Test-Path $path) {
        Write-Host "  $file OK" -ForegroundColor Green
    } else {
        Write-Host "  $file MISSING" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "DONE!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next:" -ForegroundColor Yellow
Write-Host "  1. Restart PowerShell" -ForegroundColor Gray
Write-Host "  2. Test: pvxget -h" -ForegroundColor Gray
Write-Host "  3. cargo build" -ForegroundColor Gray
Write-Host ""
