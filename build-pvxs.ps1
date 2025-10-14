# Build EPICS Base and PVXS on Windows
# Run this script in PowerShell with Administrator privileges

param(
    [string]$EpicsRoot = "C:\epics",
    [string]$EpicsBaseVersion = "R7.0.8",
    [string]$PvxsVersion = "1.4.1",
    [switch]$SkipEpicsBase,
    [switch]$SkipPvxs
)

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "EPICS Base + PVXS Build Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check prerequisites
Write-Host "Checking prerequisites..." -ForegroundColor Yellow

$prerequisites = @{
    "git" = "Git for Windows"
    "perl" = "Strawberry Perl"
    "make" = "GNU Make (from Strawberry Perl)"
}

$missingTools = @()
foreach ($tool in $prerequisites.Keys) {
    try {
        $null = Get-Command $tool -ErrorAction Stop
        Write-Host "  ✓ $tool found" -ForegroundColor Green
    } catch {
        Write-Host "  ✗ $tool not found" -ForegroundColor Red
        $missingTools += $prerequisites[$tool]
    }
}

# Check for Visual Studio
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsPath = & $vsWhere -latest -property installationPath
    if ($vsPath) {
        Write-Host "  ✓ Visual Studio found: $vsPath" -ForegroundColor Green
    }
} else {
    Write-Host "  ✗ Visual Studio not found" -ForegroundColor Red
    $missingTools += "Visual Studio 2015 or newer with C++ tools"
}

if ($missingTools.Count -gt 0) {
    Write-Host ""
    Write-Host "Missing prerequisites:" -ForegroundColor Red
    foreach ($tool in $missingTools) {
        Write-Host "  - $tool" -ForegroundColor Red
    }
    Write-Host ""
    Write-Host "Please install missing tools and run this script again." -ForegroundColor Yellow
    Write-Host "See BUILDING_PVXS_WINDOWS.md for installation instructions." -ForegroundColor Yellow
    exit 1
}

Write-Host ""

# Create EPICS directory
Write-Host "Creating EPICS directory: $EpicsRoot" -ForegroundColor Yellow
New-Item -ItemType Directory -Path $EpicsRoot -Force | Out-Null

# Build EPICS Base
if (-not $SkipEpicsBase) {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "Building EPICS Base $EpicsBaseVersion" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    
    $baseDir = Join-Path $EpicsRoot "base"
    
    if (Test-Path $baseDir) {
        Write-Host "EPICS Base directory exists. Updating..." -ForegroundColor Yellow
        Push-Location $baseDir
        git fetch --all --tags
    } else {
        Write-Host "Cloning EPICS Base..." -ForegroundColor Yellow
        Push-Location $EpicsRoot
        git clone --recursive https://github.com/epics-base/epics-base.git base
        Push-Location base
    }
    
    Write-Host "Checking out version $EpicsBaseVersion..." -ForegroundColor Yellow
    git checkout $EpicsBaseVersion
    
    Write-Host "Building EPICS Base (this may take 10-30 minutes)..." -ForegroundColor Yellow
    Write-Host "Note: You may see many compiler warnings, this is normal." -ForegroundColor Gray
    
    # Build using make
    & make clean
    & make -j4
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ EPICS Base built successfully!" -ForegroundColor Green
    } else {
        Write-Host "✗ EPICS Base build failed!" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    
    Pop-Location
} else {
    Write-Host "Skipping EPICS Base build" -ForegroundColor Gray
}

# Build PVXS
if (-not $SkipPvxs) {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "Building PVXS $PvxsVersion" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    
    $pvxsDir = Join-Path $EpicsRoot "pvxs"
    
    if (Test-Path $pvxsDir) {
        Write-Host "PVXS directory exists. Updating..." -ForegroundColor Yellow
        Push-Location $pvxsDir
        git fetch --all --tags
    } else {
        Write-Host "Cloning PVXS..." -ForegroundColor Yellow
        Push-Location $EpicsRoot
        git clone https://github.com/epics-base/pvxs.git
        Push-Location pvxs
    }
    
    Write-Host "Checking out version $PvxsVersion..." -ForegroundColor Yellow
    git checkout $PvxsVersion
    
    # Create CONFIG_SITE.local
    Write-Host "Creating CONFIG_SITE.local..." -ForegroundColor Yellow
    $baseDir = Join-Path $EpicsRoot "base"
    $configContent = @"
# EPICS Base location
EPICS_BASE = $($baseDir -replace '\\', '/')

# Build shared libraries
SHARED_LIBRARIES = YES
STATIC_BUILD = NO
"@
    $configContent | Out-File -FilePath "CONFIG_SITE.local" -Encoding ASCII -Force
    
    Write-Host "Building PVXS (this may take 5-10 minutes)..." -ForegroundColor Yellow
    
    & make clean
    & make -j4
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ PVXS built successfully!" -ForegroundColor Green
    } else {
        Write-Host "✗ PVXS build failed!" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    
    Pop-Location
} else {
    Write-Host "Skipping PVXS build" -ForegroundColor Gray
}

# Set environment variables
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Setting Environment Variables" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$baseDir = Join-Path $EpicsRoot "base"
$pvxsDir = Join-Path $EpicsRoot "pvxs"
$hostArch = "windows-x64"

Write-Host "Setting user environment variables..." -ForegroundColor Yellow
[Environment]::SetEnvironmentVariable("EPICS_BASE", $baseDir, "User")
[Environment]::SetEnvironmentVariable("EPICS_HOST_ARCH", $hostArch, "User")
[Environment]::SetEnvironmentVariable("PVXS_DIR", $pvxsDir, "User")

# Add to PATH
$currentPath = [Environment]::SetEnvironmentVariable("PATH", "User")
$binPaths = @(
    (Join-Path $baseDir "bin\$hostArch"),
    (Join-Path $pvxsDir "bin\$hostArch")
)

foreach ($binPath in $binPaths) {
    if ($currentPath -notlike "*$binPath*") {
        Write-Host "Adding to PATH: $binPath" -ForegroundColor Yellow
        $newPath = "$currentPath;$binPath"
        [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
        $currentPath = $newPath
    }
}

# Set for current session
$env:EPICS_BASE = $baseDir
$env:EPICS_HOST_ARCH = $hostArch
$env:PVXS_DIR = $pvxsDir
$env:PATH = "$($binPaths[0]);$($binPaths[1]);$env:PATH"

Write-Host ""
Write-Host "✓ Environment variables set:" -ForegroundColor Green
Write-Host "  EPICS_BASE = $baseDir" -ForegroundColor Gray
Write-Host "  EPICS_HOST_ARCH = $hostArch" -ForegroundColor Gray
Write-Host "  PVXS_DIR = $pvxsDir" -ForegroundColor Gray

# Verify installation
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Verifying Installation" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$testCommands = @{
    "softIoc.exe" = "EPICS Base softIoc"
    "pvxget.exe" = "PVXS pvxget tool"
}

foreach ($cmd in $testCommands.Keys) {
    try {
        $cmdPath = (Join-Path $baseDir "bin\$hostArch\$cmd")
        if (-not (Test-Path $cmdPath)) {
            $cmdPath = (Join-Path $pvxsDir "bin\$hostArch\$cmd")
        }
        
        if (Test-Path $cmdPath) {
            Write-Host "  ✓ $($testCommands[$cmd]) found" -ForegroundColor Green
        } else {
            Write-Host "  ✗ $($testCommands[$cmd]) not found" -ForegroundColor Red
        }
    } catch {
        Write-Host "  ✗ $($testCommands[$cmd]) not found" -ForegroundColor Red
    }
}

# Check libraries
Write-Host ""
Write-Host "Checking libraries:" -ForegroundColor Yellow
$pvxsLib = Join-Path $pvxsDir "lib\$hostArch\pvxs.lib"
$pvxsDll = Join-Path $pvxsDir "bin\$hostArch\pvxs.dll"

if (Test-Path $pvxsLib) {
    Write-Host "  ✓ pvxs.lib found" -ForegroundColor Green
} else {
    Write-Host "  ✗ pvxs.lib not found at $pvxsLib" -ForegroundColor Red
}

if (Test-Path $pvxsDll) {
    Write-Host "  ✓ pvxs.dll found" -ForegroundColor Green
} else {
    Write-Host "  ✗ pvxs.dll not found at $pvxsDll" -ForegroundColor Red
}

# Check headers
$pvxsHeader = Join-Path $pvxsDir "include\pvxs\client.h"
if (Test-Path $pvxsHeader) {
    Write-Host "  ✓ PVXS headers found" -ForegroundColor Green
} else {
    Write-Host "  ✗ PVXS headers not found at $pvxsHeader" -ForegroundColor Red
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Build Complete!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Restart PowerShell to load new environment variables" -ForegroundColor Gray
Write-Host "  2. cd C:\Users\tinez\repos\epics-pvxs-sys" -ForegroundColor Gray
Write-Host "  3. cargo build" -ForegroundColor Gray
Write-Host ""
Write-Host "To test PVXS:" -ForegroundColor Yellow
Write-Host "  pvxget -h" -ForegroundColor Gray
Write-Host ""
Write-Host "See BUILDING_PVXS_WINDOWS.md for more information." -ForegroundColor Gray
