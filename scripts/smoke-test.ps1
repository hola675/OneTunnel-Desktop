[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$binPath = Join-Path $projectRoot 'runtime\bin'
$fixturePath = Join-Path $projectRoot 'tests\fixtures\xray\reality-validation.json'

& (Join-Path $PSScriptRoot 'verify-runtime.ps1')
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

function Run-Offline([string]$Name, [string]$Executable, [string[]]$Arguments) {
    & $Executable @Arguments *> $null
    if ($LASTEXITCODE -ne 0) {
        Write-Output "FAIL: $Name offline probe exit code $LASTEXITCODE"
        $script:failed = $true
    } else {
        Write-Output "PASS: $Name offline probe"
    }
}

function Test-HelpContains([string]$Name, [string]$Executable, [string[]]$Arguments, [string[]]$RequiredText) {
    $helpText = (& $Executable @Arguments 2>&1 | Out-String)
    if ($LASTEXITCODE -ne 0) {
        Write-Output "FAIL: $Name help probe exit code $LASTEXITCODE"
        $script:failed = $true
        return
    }
    $missing = @($RequiredText | Where-Object { $helpText -notmatch [regex]::Escape($_) })
    if ($missing.Count -gt 0) {
        Write-Output "FAIL: $Name missing CLI capability: $($missing -join ', ')"
        $script:failed = $true
    } else {
        Write-Output "PASS: $Name required CLI capabilities"
    }
}

$script:failed = $false
Run-Offline 'wstunnel version' (Join-Path $binPath 'wstunnel.exe') @('--version')
Test-HelpContains 'wstunnel' (Join-Path $binPath 'wstunnel.exe') @('client', '--help') @('-L, --local-to-remote', 'wss://', '--http-proxy', '--tls-verify-certificate', '--http-upgrade-path-prefix', '--dns-resolver', '--dns-resolver-prefer-ipv4')
Run-Offline 'tun2socks version' (Join-Path $binPath 'tun2socks.exe') @('--version')
Test-HelpContains 'tun2socks' (Join-Path $binPath 'tun2socks.exe') @('--help') @('--device', '--proxy')
Run-Offline 'xray version' (Join-Path $binPath 'xray.exe') @('version')
Run-Offline 'xray help' (Join-Path $binPath 'xray.exe') @('help')
Run-Offline 'xray config validation' (Join-Path $binPath 'xray.exe') @('run', '-test', '-config', $fixturePath)

if ($script:failed) { exit 1 }
Write-Output 'PASS: wintun.dll verified by verify-runtime; no DLL load or adapter creation attempted.'
Write-Output 'PASS: runtime smoke test completed offline; no VPN connection attempted.'
exit 0
