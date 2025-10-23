# Build script for creating portable Unpackrr distribution (Windows)
# Usage: .\build-portable.ps1 [-Version "0.1.0"]
# Example: .\build-portable.ps1 -Version "0.1.0"

param(
    [string]$Version = "0.1.0"
)

$ErrorActionPreference = "Stop"

# Directories
$ProjectRoot = $PSScriptRoot
$DistDir = Join-Path $ProjectRoot "dist"
$BuildDir = Join-Path $DistDir "unpackrr-$Version"
$BinaryName = "unpackrr.exe"

Write-Host "========================================" -ForegroundColor Blue
Write-Host "Unpackrr Portable Build Script" -ForegroundColor Blue
Write-Host "========================================" -ForegroundColor Blue
Write-Host "Version: " -NoNewline -ForegroundColor Green
Write-Host $Version
Write-Host "Platform: " -NoNewline -ForegroundColor Green
Write-Host "windows-x86_64"
Write-Host "Build Directory: " -NoNewline -ForegroundColor Green
Write-Host $BuildDir
Write-Host ""

# Step 1: Clean previous builds
Write-Host "[1/7] " -NoNewline -ForegroundColor Yellow
Write-Host "Cleaning previous builds..."
if (Test-Path $DistDir) {
    Remove-Item -Path $DistDir -Recurse -Force
}
New-Item -ItemType Directory -Path $BuildDir -Force | Out-Null

# Step 2: Build release binary
Write-Host "[2/7] " -NoNewline -ForegroundColor Yellow
Write-Host "Building release binary..."
Set-Location $ProjectRoot
cargo build --release

$ReleaseBinary = Join-Path $ProjectRoot "target\release\$BinaryName"
if (-not (Test-Path $ReleaseBinary)) {
    Write-Host "ERROR: " -NoNewline -ForegroundColor Red
    Write-Host "Release binary not found at $ReleaseBinary"
    exit 1
}

# Step 3: Copy binary
Write-Host "[3/7] " -NoNewline -ForegroundColor Yellow
Write-Host "Copying binary..."
Copy-Item $ReleaseBinary -Destination $BuildDir

# Step 4: Copy documentation
Write-Host "[4/7] " -NoNewline -ForegroundColor Yellow
Write-Host "Copying documentation..."
Copy-Item (Join-Path $ProjectRoot "README.md") -Destination $BuildDir
Copy-Item (Join-Path $ProjectRoot "THIRD_PARTY_LICENSES.md") -Destination $BuildDir

# Copy main license (GPL-3.0)
$LicensePath = Join-Path $ProjectRoot "..\LICENSE"
if (Test-Path $LicensePath) {
    Copy-Item $LicensePath -Destination (Join-Path $BuildDir "LICENSE")
} elseif (Test-Path (Join-Path $ProjectRoot "LICENSE")) {
    Copy-Item (Join-Path $ProjectRoot "LICENSE") -Destination (Join-Path $BuildDir "LICENSE")
} else {
    Write-Host "WARNING: " -NoNewline -ForegroundColor Yellow
    Write-Host "LICENSE file not found, creating placeholder..."
    $LicenseContent = @"
Unpackrr - BA2 Batch Unpacker (Rust Edition)
Copyright (C) 2024-2025 evildarkarchon

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
"@
    Set-Content -Path (Join-Path $BuildDir "LICENSE") -Value $LicenseContent
}

# Step 5: Check for BSArch.exe
Write-Host "[5/7] " -NoNewline -ForegroundColor Yellow
Write-Host "Checking for BSArch.exe..."
$BSArchPath = Join-Path $ProjectRoot "BSArch.exe"
$BSArchParentPath = Join-Path $ProjectRoot "..\BSArch.exe"

if (Test-Path $BSArchPath) {
    Copy-Item $BSArchPath -Destination $BuildDir
    Write-Host "✓ " -NoNewline -ForegroundColor Green
    Write-Host "BSArch.exe included"
} elseif (Test-Path $BSArchParentPath) {
    Copy-Item $BSArchParentPath -Destination $BuildDir
    Write-Host "✓ " -NoNewline -ForegroundColor Green
    Write-Host "BSArch.exe included"
} else {
    Write-Host "WARNING: " -NoNewline -ForegroundColor Yellow
    Write-Host "BSArch.exe not found in project directory"
    Write-Host "         Please download from https://github.com/TES5Edit/TES5Edit" -ForegroundColor Yellow
    Write-Host "         and place in: $BuildDir" -ForegroundColor Yellow

    $BSArchNotice = @"
BSArch.exe is required for Unpackrr to function.

Please download BSArch.exe from:
https://github.com/TES5Edit/TES5Edit/releases

Place BSArch.exe in the same directory as unpackrr.exe

BSArch.exe is licensed under MPL-2.0 (Mozilla Public License 2.0).
See THIRD_PARTY_LICENSES.md for details.
"@
    Set-Content -Path (Join-Path $BuildDir "BSARCH_NEEDED.txt") -Value $BSArchNotice
}

# Step 6: Create version info file
Write-Host "[6/7] " -NoNewline -ForegroundColor Yellow
Write-Host "Creating version info..."
$BuildDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-dd HH:mm:ss UTC")
$RustVersion = & rustc --version
$VersionInfo = @"
Unpackrr Version $Version
Build Date: $BuildDate
Platform: windows-x86_64
Rust Version: $RustVersion
"@
Set-Content -Path (Join-Path $BuildDir "VERSION.txt") -Value $VersionInfo

# Step 7: Create distribution archive
Write-Host "[7/7] " -NoNewline -ForegroundColor Yellow
Write-Host "Creating distribution archive..."
$ArchiveName = "unpackrr-$Version-windows-x86_64.zip"
$ArchivePath = Join-Path $DistDir $ArchiveName

Compress-Archive -Path $BuildDir -DestinationPath $ArchivePath -Force
Write-Host "✓ " -NoNewline -ForegroundColor Green
Write-Host "Created: $ArchivePath"

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "Build Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host "Distribution directory: " -NoNewline -ForegroundColor Green
Write-Host $BuildDir
Write-Host "Archive: " -NoNewline -ForegroundColor Green
Write-Host $ArchivePath

# Calculate sizes
$BinarySize = [math]::Round((Get-Item (Join-Path $BuildDir $BinaryName)).Length / 1MB, 2)
Write-Host "Binary size: " -NoNewline -ForegroundColor Green
Write-Host "$BinarySize MB"

$BSArchExe = Join-Path $BuildDir "BSArch.exe"
if (Test-Path $BSArchExe) {
    $BSArchSize = [math]::Round((Get-Item $BSArchExe).Length / 1MB, 2)
    Write-Host "BSArch.exe size: " -NoNewline -ForegroundColor Green
    Write-Host "$BSArchSize MB"
}

$ArchiveSize = [math]::Round((Get-Item $ArchivePath).Length / 1MB, 2)
Write-Host "Archive size: " -NoNewline -ForegroundColor Green
Write-Host "$ArchiveSize MB"

Write-Host ""
Write-Host "Next steps:" -ForegroundColor Blue
Write-Host "  1. Test the distribution: cd $BuildDir && .\$BinaryName"
if (-not (Test-Path $BSArchExe)) {
    Write-Host "  2. " -NoNewline
    Write-Host "Download and add BSArch.exe to $BuildDir" -ForegroundColor Yellow
}
Write-Host "  3. Create release on GitHub with the archive"
Write-Host ""
