[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$manifestPath = Join-Path $PSScriptRoot '..\runtime\manifest.json'
$failed = $false

function Report([string]$Status, [string]$Message) {
    Write-Output ("{0}: {1}" -f $Status, $Message)
}

if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
    Report 'FAIL' "manifest missing: $manifestPath"
    exit 1
}

try {
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
} catch {
    Report 'FAIL' "manifest is not valid JSON: $($_.Exception.Message)"
    exit 1
}

if ($manifest.schemaVersion -ne 1) {
    Report 'FAIL' 'schemaVersion must be 1'
    $failed = $true
}
if ($manifest.platform -ne 'windows-x86_64') {
    Report 'FAIL' 'platform must be windows-x86_64'
    $failed = $true
}

$expected = @{
    wstunnel = 'erebe/wstunnel'
    xray = 'XTLS/Xray-core'
    tun2socks = 'xjasonlyu/tun2socks'
    wintun = 'WireGuard/wintun'
}

foreach ($name in $expected.Keys) {
    $component = $manifest.components.$name
    if ($null -eq $component) {
        Report 'FAIL' "missing component: $name"
        $failed = $true
        continue
    }
    if ($component.source -ne $expected[$name]) {
        Report 'FAIL' "$name source mismatch: expected $($expected[$name])"
        $failed = $true
    }
    if ([string]::IsNullOrWhiteSpace([string]$component.version) -or [string]$component.version -eq 'latest') {
        Report 'NOT_PINNED' "$name version is not pinned"
    }
    if ([string]::IsNullOrWhiteSpace([string]$component.sha256)) {
        Report 'NOT_PINNED' "$name SHA256 is missing"
    } elseif ([string]$component.sha256 -notmatch '^[A-Fa-f0-9]{64}$') {
        Report 'FAIL' "$name SHA256 must contain 64 hexadecimal characters"
        $failed = $true
    }
}

if (-not $failed) {
    Report 'PASS' 'manifest schema and component keys are valid'
    exit 0
}
Report 'FAIL' 'manifest validation failed'
exit 1
