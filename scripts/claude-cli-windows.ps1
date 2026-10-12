# Fixed action from JSON stdin. Preserve configuration and custom launchers.
$action = [Console]::In.ReadToEnd() | ConvertFrom-Json
if ($action -notin @('read','uninstall','reinstall')) { throw 'Invalid CLI action' }
$npm = Get-Command npm.cmd -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
function Read-Cli {
    $command = Get-Command claude.exe,claude.cmd -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    $package = $null
    if ($npm) {
        $root = (& $npm.Source root -g 2>$null | Out-String).Trim()
        if ($LASTEXITCODE -ne 0) { throw 'Cannot read npm root' }
        $manifest = Join-Path $root '@anthropic-ai\claude-code\package.json'
        if (Test-Path -LiteralPath $manifest) { $package = Get-Content -LiteralPath $manifest -Raw | ConvertFrom-Json }
    }
    if ($package -and $package.name -eq '@anthropic-ai/claude-code') {
        $prefix = (& $npm.Source prefix -g 2>$null | Out-String).Trim()
        if ($LASTEXITCODE -ne 0) { throw 'Cannot read npm prefix' }
        $expected = Join-Path $prefix 'claude.cmd'
        if ($command -and $command.Source -ne $expected) { return @{method='custom';path=$command.Source;version=''} }
        if (!(Test-Path -LiteralPath $expected)) { throw 'npm launcher missing' }
        $launcher = Get-Content -LiteralPath $expected -Raw
        if ($launcher -notmatch '@anthropic-ai[\\/]claude-code[\\/]cli\.js') { throw 'Unknown launcher' }
        $version = (& $expected --version 2>$null | Out-String).Trim()
        if ($LASTEXITCODE -ne 0 -or !$version) { throw 'CLI version check failed' }
        return @{method='npm';path=$expected;version=$version}
    }
    if ($command) { return @{method='native';path=$command.Source;version=''} }
    $native = Join-Path $env:USERPROFILE '.local\bin\claude.exe'
    if (Test-Path -LiteralPath $native) { return @{method='native';path=$native;version=''} }
    return @{method='';path='';version=''}
}
$before = Read-Cli
if ($action -ne 'read') {
    if (!$npm) { throw 'Node.js/npm required' }
    if ($before.method -and $before.method -ne 'npm') { throw 'Use original installer' }
    if ($before.method -eq 'npm') {
        $null = & $npm.Source uninstall --global '@anthropic-ai/claude-code' 2>&1
        if ($LASTEXITCODE -ne 0) { throw 'npm uninstall failed' }
    }
    if ($action -eq 'reinstall') {
        $null = & $npm.Source install --global '@anthropic-ai/claude-code' 2>&1
        if ($LASTEXITCODE -ne 0) { throw 'npm install failed' }
    }
}
$after = Read-Cli
if ($action -eq 'reinstall' -and ($after.method -ne 'npm' -or !$after.version)) { throw 'Reinstall verification failed' }
if ($action -eq 'uninstall' -and $after.method) { throw 'CLI still installed' }
$after | ConvertTo-Json -Compress
