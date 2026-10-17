# BornoFree Automated Release Packaging Script
param(
    [string]$Version = "1.0.1"
)

$ErrorActionPreference = "Stop"
$rootDir = (Resolve-Path "$PSScriptRoot\..").Path
$distDir = Join-Path $rootDir "dist"
$pkgName = "BornoFree-v$Version-Windows-x64"
$pkgDir = Join-Path $distDir $pkgName
$zipFile = Join-Path $distDir "$pkgName.zip"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   BornoFree v$Version Release Packaging System" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 0. Stop any running instances first
Stop-Process -Name "BornoFree", "BornoFree-Setup" -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300

# 1. Clean previous dist
if (Test-Path $pkgDir) {
    Remove-Item -Path $pkgDir -Recurse -Force
}
if (Test-Path $zipFile) {
    Remove-Item -Path $zipFile -Force
}
New-Item -ItemType Directory -Path $pkgDir -Force | Out-Null

# 2. Build Release Binaries
Write-Host "`n[1/5] Building Standalone Engine (BornoFree.exe)..." -ForegroundColor Yellow
Set-Location $rootDir

& cargo build --release --bin BornoFree
if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo build of BornoFree failed with exit code $LASTEXITCODE"
    exit 1
}

$releaseExe = Join-Path $rootDir "target\release\BornoFree.exe"
if (-not (Test-Path $releaseExe)) {
    Write-Error "Release binary not found at $releaseExe"
    exit 1
}

# Copy to root so BornoFree-Setup can embed it
Copy-Item $releaseExe (Join-Path $rootDir "BornoFree.exe") -Force

Write-Host "`n[2/5] Building Graphical Setup Wizard (BornoFree-Setup.exe)..." -ForegroundColor Yellow
& cargo build --release --bin BornoFree-Setup
if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo build of BornoFree-Setup failed with exit code $LASTEXITCODE"
    exit 1
}

$setupExe = Join-Path $rootDir "target\release\BornoFree-Setup.exe"
if (-not (Test-Path $setupExe)) {
    Write-Error "Setup binary not found at $setupExe"
    exit 1
}

# 3. Copy files to Package Directory and dist/
Write-Host "`n[3/5] Assembling release directory: $pkgName..." -ForegroundColor Yellow
Copy-Item $releaseExe (Join-Path $pkgDir "BornoFree.exe") -Force
Copy-Item $setupExe (Join-Path $pkgDir "BornoFree-Setup.exe") -Force
Copy-Item (Join-Path $rootDir "scripts\Install-BornoFree.cmd") (Join-Path $pkgDir "Install-BornoFree.cmd") -Force
Copy-Item (Join-Path $rootDir "scripts\Uninstall-BornoFree.cmd") (Join-Path $pkgDir "Uninstall-BornoFree.cmd") -Force
Copy-Item (Join-Path $rootDir "scripts\README.txt") (Join-Path $pkgDir "README.txt") -Force
if (Test-Path (Join-Path $rootDir "LICENSE")) {
    Copy-Item (Join-Path $rootDir "LICENSE") (Join-Path $pkgDir "LICENSE.txt") -Force
}

# Standalone files directly in dist/ for 1-click download
Copy-Item $releaseExe (Join-Path $distDir "BornoFree.exe") -Force
Copy-Item $setupExe (Join-Path $distDir "BornoFree-Setup.exe") -Force

# 4. Create ZIP Archive
Write-Host "`n[4/5] Creating ZIP Archive: $pkgName.zip..." -ForegroundColor Yellow
if (Get-Command tar.exe -ErrorAction SilentlyContinue) {
    & tar.exe -a -c -f $zipFile -C $distDir $pkgName
} else {
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [System.IO.Compression.ZipFile]::CreateFromDirectory($pkgDir, $zipFile)
}

# 5. Generate SHA256 Checksums
Write-Host "`n[5/5] Calculating SHA256 Checksums..." -ForegroundColor Yellow
$hashExe = (Get-FileHash (Join-Path $distDir "BornoFree.exe") -Algorithm SHA256).Hash
$hashSetup = (Get-FileHash (Join-Path $distDir "BornoFree-Setup.exe") -Algorithm SHA256).Hash
$hashZip = (Get-FileHash $zipFile -Algorithm SHA256).Hash

$shaFile = Join-Path $distDir "SHA256SUMS.txt"
$shaContent = @"
$hashSetup  BornoFree-Setup.exe
$hashExe    BornoFree.exe
$hashZip    $pkgName.zip
"@
Set-Content -Path $shaFile -Value $shaContent

$exeSizeMB = [math]::Round((Get-Item (Join-Path $distDir "BornoFree.exe")).Length / 1MB, 2)
$setupSizeMB = [math]::Round((Get-Item (Join-Path $distDir "BornoFree-Setup.exe")).Length / 1MB, 2)
$zipSizeMB = [math]::Round((Get-Item $zipFile).Length / 1MB, 2)

Write-Host "`n==========================================================" -ForegroundColor Green
Write-Host "   PACKAGING COMPLETE & READY FOR DOWNLOAD!            " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
Write-Host "Setup Wizard   : dist\BornoFree-Setup.exe ($setupSizeMB MB)" -ForegroundColor White
Write-Host "Standalone EXE : dist\BornoFree.exe ($exeSizeMB MB)" -ForegroundColor White
Write-Host "Release Folder : dist\$pkgName" -ForegroundColor White
Write-Host "Portable ZIP   : dist\$pkgName.zip ($zipSizeMB MB)" -ForegroundColor White
Write-Host "Checksum File  : dist\SHA256SUMS.txt" -ForegroundColor White
Write-Host "==========================================================" -ForegroundColor Green
