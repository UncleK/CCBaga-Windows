# Disposable GitHub runner only: install, launch and inspect bundled resources.
param([Parameter(Mandatory=$true)][string]$Installer)
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows') {
    throw 'This smoke test is restricted to disposable GitHub Windows runners.'
}
$install=Join-Path $env:RUNNER_TEMP 'CCBaga Windows smoke'
$data=Join-Path $env:APPDATA 'ai.local.ccbaga.windows'
if ((Test-Path $install) -or (Test-Path $data)) { throw 'Smoke-test paths must be fresh.' }
$hostProcess=$null
try {
    $setup=Start-Process -FilePath (Resolve-Path $Installer) -ArgumentList @('/S',"/D=$install") -PassThru
    if (!$setup.WaitForExit(180000)) { $setup.Kill(); throw 'Installer timed out' }
    if ($setup.ExitCode -ne 0) { throw "Installer exit code $($setup.ExitCode)" }
    foreach ($path in @('requirement-french.exe','browser-runtime/windows/chrome.exe','browser-runtime/windows/icudtl.dat','browser-ui/browser.html','licenses/LICENSE.unicode')) {
        if (!(Test-Path -LiteralPath (Join-Path $install $path))) { throw "Missing installed resource: $path" }
    }
    $expected=(Get-FileHash 'browser-runtime/windows/chrome.exe' -Algorithm SHA256).Hash
    $actual=(Get-FileHash (Join-Path $install 'browser-runtime/windows/chrome.exe') -Algorithm SHA256).Hash
    if ($actual -ne $expected) { throw 'Installed engine differs from the verified input' }
    $hostProcess=Start-Process -FilePath (Join-Path $install 'requirement-french.exe') -PassThru
    $startFile=Join-Path $data 'browser/start.url'
    $deadline=(Get-Date).AddSeconds(60)
    do {
        if ($hostProcess.HasExited) { throw 'Installed host exited during startup' }
        if (Test-Path -LiteralPath $startFile) { break }
        Start-Sleep -Milliseconds 500
    } while ((Get-Date) -lt $deadline)
    if (!(Test-Path -LiteralPath $startFile)) { throw 'Host did not create a browser session' }
    $url=(Get-Content -LiteralPath $startFile -Raw).Trim()
    $uri=[uri]$url
    if ($uri.Scheme -ne 'http' -or $uri.Host -ne '127.0.0.1') { throw 'Expected a local browser session' }
    $page=Invoke-WebRequest -Uri $uri -UseBasicParsing -TimeoutSec 15
    if ($page.StatusCode -ne 200 -or $page.Content -notmatch '<script') { throw 'Bundled browser page is unavailable' }
    $context=Invoke-RestMethod -Uri ([uri]::new($uri,'context')) -TimeoutSec 30
    if ($context.machine.platform -ne 'windows' -or !$context.engine_version) { throw 'Invalid native Windows context' }
    if ($context.machine.PSObject.Properties.Name -contains 'system_proxy') { throw 'Private proxy data was exposed' }
    $deadline=(Get-Date).AddSeconds(30)
    do {
        $browser=@(Get-CimInstance Win32_Process -Filter "Name='chrome.exe'" | Where-Object { $_.ExecutablePath -eq (Join-Path $install 'browser-runtime/windows/chrome.exe') })
        if ($browser.Count) { break }
        Start-Sleep -Milliseconds 500
    } while ((Get-Date) -lt $deadline)
    if (!$browser.Count) { throw 'Installed companion browser did not start' }
    Write-Output 'PASS: current-user NSIS install, engine integrity, host startup, loopback page, Windows context and companion process.'
} finally {
    if ($hostProcess -and !$hostProcess.HasExited) { Stop-Process -Id $hostProcess.Id -Force -ErrorAction SilentlyContinue }
    Get-CimInstance Win32_Process -Filter "Name='chrome.exe'" | Where-Object { $_.ExecutablePath -eq (Join-Path $install 'browser-runtime/windows/chrome.exe') } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
    # The runner is discarded. No real user profile or system settings are changed.
}
