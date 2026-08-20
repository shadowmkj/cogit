# ==============================================================================
# Cogit Installer Script for Windows (PowerShell)
# ==============================================================================
# Usage:
#   irm https://raw.githubusercontent.com/shadowmkj/cogit/main/scripts/install.ps1 | iex
# ==============================================================================

$ErrorActionPreference = "Stop"

$Repo = "shadowmkj/cogit"
$BinName = "cogit.exe"
$InstallDir = Join-Path $HOME ".cogit\bin"

Write-Host "🦀 Installing Cogit for Windows..." -ForegroundColor Cyan

# 1. Detect Architecture
$Arch = if ([System.Environment]::Is64BitOperatingSystem) {
    if ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -eq [System.Runtime.InteropServices.Architecture]::Arm64) {
        "aarch64"
    } else {
        "x86_64"
    }
} else {
    Write-Error "Unsupported 32-bit Windows architecture."
    exit 1
}

$Target = "$Arch-pc-windows-msvc"
$DownloadUrl = "https://github.com/$Repo/releases/latest/download/cogit-$Target.zip"

# 2. Download and Extract
$TempZip = Join-Path $env:TEMP "cogit-$Target.zip"
$TempExtract = Join-Path $env:TEMP "cogit-extract-$([System.Guid]::NewGuid())"

Write-Host "📦 Downloading Cogit for $Target..."
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempZip

Expand-Archive -Path $TempZip -DestinationPath $TempExtract -Force

# 3. Copy to destination
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$ExtractedBin = Get-ChildItem -Path $TempExtract -Filter $BinName -Recurse | Select-Object -First 1

if ($ExtractedBin) {
    Copy-Item -Path $ExtractedBin.FullName -Destination (Join-Path $InstallDir $BinName) -Force
} else {
    Write-Error "Could not find $BinName in downloaded release archive."
    exit 1
}

# 4. Clean up temp files
Remove-Item -Path $TempZip -Force -ErrorAction SilentlyContinue
Remove-Item -Path $TempExtract -Recurse -Force -ErrorAction SilentlyContinue

# 5. Ensure in User PATH
$UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -split ";" -notcontains $InstallDir) {
    [System.Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path += ";$InstallDir"
    Write-Host "✨ Added $InstallDir to User PATH." -ForegroundColor Green
}

Write-Host "✅ Cogit successfully installed to $InstallDir\$BinName!" -ForegroundColor Green
Write-Host "Restart your terminal or run '$BinName --help' to get started."
