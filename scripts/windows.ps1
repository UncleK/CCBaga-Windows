param([ValidateSet('Check','Prepare','Dev','Build','Test')][string]$Action='Dev')
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
if ($env:OS -ne 'Windows_NT') { throw 'Run on Windows 10/11 x64.' }
if ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne 'X64') { throw 'Windows x64 is required.' }
Set-Location (Split-Path -Parent $PSScriptRoot)
function Run-Checked([string]$Program,[string[]]$Arguments) {
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed with exit code $LASTEXITCODE" }
}
foreach ($tool in @('node','bun','cargo','rustc','python')) {
    if (!(Get-Command $tool -ErrorAction SilentlyContinue)) { throw "Missing $tool. See README.windows.md." }
    Run-Checked $tool @('--version')
}
Run-Checked 'python' @('-c','import sys; assert sys.version_info >= (3,11), "Python 3.11+ required"')
$vswhere=Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
if (!(Test-Path $vswhere) -or !(& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath)) {
    throw 'Install Visual Studio 2022 Build Tools with Desktop development with C++, MSVC and Windows SDK.'
}
if ($Action -eq 'Check') { exit 0 }
Run-Checked 'bun' @('install','--frozen-lockfile')
Run-Checked 'python' @('scripts/fetch-browser-windows.py')
Run-Checked 'bun' @('run','build')
if ($Action -eq 'Prepare') { exit 0 }
switch ($Action) {
    'Dev' { Run-Checked 'bun' @('run','tauri','dev') }
    'Build' { Run-Checked 'bun' @('run','tauri','build','--bundles','nsis','--features','custom-protocol') }
    'Test' {
        Run-Checked 'bun' @('test')
        Run-Checked 'python' @('-m','unittest','discover','-s','scripts','-p','test_windows_*.py')
        Run-Checked 'cargo' @('test','--locked','--manifest-path','src-tauri/Cargo.toml','--features','custom-protocol')
    }
}
