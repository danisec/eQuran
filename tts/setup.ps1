# Prepares the local Supertonic 3 runtime used for translation and tafsir audio.
# Windows counterpart of setup.sh. The app runs it with VENV_DIR pointed at a
# writable data directory (so no elevation is needed), or a reader runs it by
# hand and the venv lands next to this script.
#
# Note on error handling: this deliberately keeps the default
# $ErrorActionPreference. Under 'Stop', Windows PowerShell 5.1 promotes any
# native command's stderr line into a terminating error, and pip writes progress
# and warnings to stderr on a perfectly successful install. Each native call is
# checked through $LASTEXITCODE instead.
[CmdletBinding()]
param(
    [string]$VenvDir = $env:VENV_DIR
)

Set-Location -LiteralPath $PSScriptRoot

if ([string]::IsNullOrWhiteSpace($VenvDir)) {
    $VenvDir = ".venv"
}

# onnxruntime publishes wheels only up to a certain CPython version, and the
# newest system Python is often past it. The `py` launcher can select any
# installed version, so try the supported ones newest first.
$SupportedVersions = @("3.14", "3.13", "3.12")

$pythonExe = $null
$pythonArgs = @()

function Test-PythonLauncher {
    param([string]$Launcher, [string]$Version)

    & $Launcher "-$Version" -c "import sys" 2>$null | Out-Null
    return $LASTEXITCODE -eq 0
}

function Resolve-Python {
    if ($env:PYTHON_BIN) {
        $script:pythonExe = $env:PYTHON_BIN
        return $true
    }

    $py = Get-Command py -ErrorAction SilentlyContinue
    if ($py) {
        foreach ($version in $SupportedVersions) {
            if (Test-PythonLauncher -Launcher $py.Source -Version $version) {
                $script:pythonExe = $py.Source
                $script:pythonArgs = @("-$version")
                return $true
            }
        }
        # No supported interpreter registered, but `py` exists: let it pick the
        # default and let the pip step report an unusable version if that fails.
        $script:pythonExe = $py.Source
        return $true
    }

    foreach ($name in @("python", "python3")) {
        $command = Get-Command $name -ErrorAction SilentlyContinue
        if ($command) {
            $script:pythonExe = $command.Source
            return $true
        }
    }

    return $false
}

if (-not (Resolve-Python)) {
    $message = @(
        "No supported Python found. onnxruntime requires Python 3.12, 3.13 or 3.14."
        ""
        "Install one of them from https://www.python.org/downloads/windows/, make sure"
        "the py launcher or python is on PATH, then run this script again."
    ) -join "`n"
    [Console]::Error.WriteLine($message)
    exit 1
}

Write-Output "Using Python: $pythonExe $($pythonArgs -join ' ')"
Write-Output "Installing the runtime into $VenvDir"

# `py -3.14 -m venv` picks that exact interpreter; a bare `python -m venv` uses
# whatever is on PATH. Splatting handles both without a separate code path.
& $pythonExe @pythonArgs -m venv $VenvDir
if ($LASTEXITCODE -ne 0) {
    throw "Could not create the virtual environment in $VenvDir"
}

# A venv on Windows keeps its interpreter under Scripts\, so call it directly
# instead of sourcing Activate.ps1 (which is also subject to execution policy).
$venvPython = Join-Path $VenvDir "Scripts\python.exe"

& $venvPython -m pip install --upgrade pip
if ($LASTEXITCODE -ne 0) {
    throw "Could not upgrade pip in $VenvDir"
}

& $venvPython -m pip install -r requirements.txt
if ($LASTEXITCODE -ne 0) {
    throw "Could not install the speech runtime into $VenvDir"
}

# Fail fast instead of discovering a broken runtime on the first playback.
& $venvPython -c "import supertonic, onnxruntime"
if ($LASTEXITCODE -ne 0) {
    throw "The installed runtime in $VenvDir cannot import supertonic"
}

Write-Output "Supertonic 3 runtime ready in $VenvDir"
