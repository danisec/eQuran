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

# Runs an interpreter candidate and verifies it is a real Python. Windows ships
# an "app execution alias" stub for python.exe that echoes "Python was not
# found" and exits non-zero unless a real Python hides behind it; requiring a
# successful, answering run here keeps that stub from being chosen.
function Test-Python {
    param([string]$Interpreter, [string[]]$InterpreterArgs = @())

    try {
        $output = & $Interpreter @InterpreterArgs -c "import sys; print(sys.version_info[0], sys.version_info[1])" 2>$null
    } catch {
        return $false
    }
    if ($LASTEXITCODE -ne 0) { return $false }
    # A real interpreter answers with "major minor"; anything else is not one.
    return (($output -join ' ') -match '^\d+ \d+$')
}

# Reports a failure as one clean line (the streamed native output above already
# named the cause) and stops, instead of a multi-line PowerShell error record.
function Fail-Setup {
    param([string]$Message)

    [Console]::Error.WriteLine($Message)
    exit 1
}

function Resolve-Python {
    # Prefer the relocatable CPython shipped next to this script: it is what
    # makes the setup work on a machine with no system Python at all (and keeps
    # it out of the Microsoft Store alias trap below).
    $bundled = Join-Path $PSScriptRoot "python\python.exe"
    if ((Test-Path -LiteralPath $bundled -PathType Leaf) -and (Test-Python -Interpreter $bundled)) {
        $script:pythonExe = $bundled
        return $true
    }

    if ($env:PYTHON_BIN) {
        if (Test-Python -Interpreter $env:PYTHON_BIN) {
            $script:pythonExe = $env:PYTHON_BIN
            return $true
        }
        [Console]::Error.WriteLine("PYTHON_BIN is set but does not run: $env:PYTHON_BIN")
        return $false
    }

    $py = Get-Command py -ErrorAction SilentlyContinue
    if ($py) {
        foreach ($version in $SupportedVersions) {
            if (Test-Python -Interpreter $py.Source -InterpreterArgs @("-$version")) {
                $script:pythonExe = $py.Source
                $script:pythonArgs = @("-$version")
                return $true
            }
        }
        # `py` exists but none of the supported versions is registered; let it
        # pick whatever default it has and surface a real failure at pip time.
        if (Test-Python -Interpreter $py.Source) {
            $script:pythonExe = $py.Source
            return $true
        }
    }

    # A bare python/python3 on PATH may be the Microsoft Store alias, which
    # exists on every Windows machine yet only opens the Store. Test each one by
    # actually running it; the alias fails Test-Python and is skipped.
    foreach ($name in @("python", "python3")) {
        foreach ($command in @(Get-Command $name -All -ErrorAction SilentlyContinue)) {
            if (Test-Python -Interpreter $command.Source) {
                $script:pythonExe = $command.Source
                return $true
            }
        }
    }

    return $false
}

if (-not (Resolve-Python)) {
    $message = @(
        "No usable Python found. The speech runtime needs Python 3.12-3.14 (64-bit) with pip."
        ""
        "If typing python opens the Microsoft Store, turn off the python app execution"
        "alias in Settings > Apps > Advanced app settings > App execution aliases."
        "Then install Python from https://www.python.org/downloads/windows/ with the"
        "py launcher and 'Add python.exe to PATH' enabled, and press Set up voice again."
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
    Fail-Setup "Could not create the virtual environment in $VenvDir. See the line above for the reason."
}

# A venv on Windows keeps its interpreter under Scripts\, so call it directly
# instead of sourcing Activate.ps1 (which is also subject to execution policy).
$venvPython = Join-Path $VenvDir "Scripts\python.exe"

& $venvPython -m pip install --upgrade pip
if ($LASTEXITCODE -ne 0) {
    Fail-Setup "Could not upgrade pip in $VenvDir. See the lines above for the reason."
}

& $venvPython -m pip install -r requirements.txt
if ($LASTEXITCODE -ne 0) {
    Fail-Setup "Could not install the speech runtime into $VenvDir. See the lines above for the reason."
}

# Fail fast instead of discovering a broken runtime on the first playback.
& $venvPython -c "import supertonic, onnxruntime"
if ($LASTEXITCODE -ne 0) {
    Fail-Setup "The installed runtime in $VenvDir cannot import supertonic."
}

Write-Output "Supertonic 3 runtime ready in $VenvDir"
