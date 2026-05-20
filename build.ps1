param(
    [switch]$SkipInstall,
    [switch]$KeepRunningApp
)

$ErrorActionPreference = "Stop"

function Write-Step {
    param([string]$Message)
    Write-Host ""
    Write-Host "==> $Message" -ForegroundColor Cyan
}

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

$env:Path = "C:\Program Files\nodejs;C:\Users\Aries\.cargo\bin;$env:Path"

Write-Step "Checking toolchain"
node --version | Write-Host
npm.cmd --version | Write-Host
cargo --version | Write-Host

if (-not $SkipInstall -and -not (Test-Path -LiteralPath (Join-Path $root "node_modules"))) {
    Write-Step "Installing npm dependencies"
    npm.cmd install
    if ($LASTEXITCODE -ne 0) {
        throw "npm install failed with exit code $LASTEXITCODE"
    }
}

if (-not $KeepRunningApp) {
    $releaseExe = Join-Path $root "src-tauri\target\release\todo-float.exe"
    $runningApps = Get-Process todo-float -ErrorAction SilentlyContinue | Where-Object {
        $_.Path -and ([System.IO.Path]::GetFullPath($_.Path) -eq [System.IO.Path]::GetFullPath($releaseExe))
    }

    if ($runningApps) {
        Write-Step "Closing running release app before build"
        foreach ($app in $runningApps) {
            $null = $app.CloseMainWindow()
        }
        Start-Sleep -Milliseconds 800

        $runningApps = Get-Process todo-float -ErrorAction SilentlyContinue | Where-Object {
            $_.Path -and ([System.IO.Path]::GetFullPath($_.Path) -eq [System.IO.Path]::GetFullPath($releaseExe))
        }
        foreach ($app in $runningApps) {
            Stop-Process -Id $app.Id -Force
        }
    }
}

Write-Step "Building Todo Float release bundles"
npm.cmd run tauri build
if ($LASTEXITCODE -ne 0) {
    throw "tauri build failed with exit code $LASTEXITCODE"
}

Write-Step "Build outputs"
$outputs = @(
    "src-tauri\target\release\todo-float.exe",
    "src-tauri\target\release\bundle\nsis\Todo Float_0.1.0_x64-setup.exe",
    "src-tauri\target\release\bundle\msi\Todo Float_0.1.0_x64_en-US.msi"
)

foreach ($relativePath in $outputs) {
    $path = Join-Path $root $relativePath
    if (Test-Path -LiteralPath $path) {
        $item = Get-Item -LiteralPath $path
        Write-Host ("{0}  ({1:N1} MB)" -f $item.FullName, ($item.Length / 1MB))
    } else {
        Write-Host "Missing: $path" -ForegroundColor Yellow
    }
}
