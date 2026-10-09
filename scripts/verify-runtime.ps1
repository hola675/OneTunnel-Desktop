[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$manifestPath = Join-Path $PSScriptRoot '..\runtime\manifest.json'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$failed = $false

function Report([string]$Status, [string]$Message) {
    Write-Output ("{0}: {1}" -f $Status, $Message)
}

function Fail([string]$Status, [string]$Message) {
    Report $Status $Message
    $script:failed = $true
}

function Is-Sha256([object]$Value) {
    return (-not [string]::IsNullOrWhiteSpace([string]$Value)) -and ([string]$Value -match '^[A-Fa-f0-9]{64}$')
}

if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
    Fail 'MANIFEST_INVALID' "manifest missing: $manifestPath"
    exit 1
}

try {
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
} catch {
    Fail 'MANIFEST_INVALID' "manifest is not valid JSON: $($_.Exception.Message)"
    exit 1
}

if ($manifest.schemaVersion -ne 3) { Fail 'MANIFEST_INVALID' 'schemaVersion must be 3' }
if ($manifest.platform -ne 'windows-x86_64') { Fail 'WRONG_PLATFORM' 'platform must be windows-x86_64' }

$expected = @{
    xray = 'XTLS/Xray-core'
    tun2socks = 'xjasonlyu/tun2socks'
    wintun = 'Wintun'
}
$officialReleaseBases = @{
    xray = 'https://github.com/XTLS/Xray-core/releases/download'
    tun2socks = 'https://github.com/xjasonlyu/tun2socks/releases/download'
    wintun = 'https://www.wintun.net/builds'
}

$components = @('xray', 'tun2socks', 'wintun')
$componentNames = @($manifest.components.PSObject.Properties.Name)
if ($componentNames.Count -ne $components.Count -or @($componentNames | Where-Object { $_ -notin $components }).Count -gt 0) {
    Fail 'MANIFEST_INVALID' 'components must be exactly xray, tun2socks and wintun'
}
$runtimeFiles = @{ xray = 'xray.exe'; tun2socks = 'tun2socks.exe'; wintun = 'wintun.dll' }

foreach ($name in $components) {
    $component = $manifest.components.$name
    if ($null -eq $component) {
        Fail 'MANIFEST_INVALID' "missing component: $name"
        continue
    }

    if ($component.upstream -ne $expected[$name]) { Fail 'MANIFEST_INVALID' "$name upstream mismatch" }
    if ([string]::IsNullOrWhiteSpace([string]$component.version) -or [string]$component.version -match '^(latest|master|main|nightly|dev)$') {
        Fail 'UNPINNED' "$name version is not pinned"
    }
    if ([string]::IsNullOrWhiteSpace([string]$component.tag) -or [string]$component.tag -match '^(latest|master|main|nightly|dev)$') {
        Fail 'UNPINNED' "$name tag is not pinned"
    }
    if ([string]::IsNullOrWhiteSpace([string]$component.architecture)) { Fail 'MANIFEST_INVALID' "$name architecture is missing" }
    if ([string]::IsNullOrWhiteSpace([string]$component.artifact)) { Fail 'UNPINNED' "$name artifact is missing" }
    $expectedSource = if ($name -eq 'wintun') {
        "$($officialReleaseBases[$name])/$($component.artifact)"
    } else {
        "$($officialReleaseBases[$name])/$($component.tag)/$($component.artifact)"
    }
    if ([string]$component.sourceUrl -ne $expectedSource) { Fail 'MANIFEST_INVALID' "$name sourceUrl must exactly match its official upstream release URL" }
    if ($component.checksumUrl -and ([string]$component.checksumUrl -notmatch '^https://')) { Fail 'MANIFEST_INVALID' "$name checksumUrl must be HTTPS" }
    if ($component.licenseUrl -and ([string]$component.licenseUrl -notmatch '^https://raw\.githubusercontent\.com/')) { Fail 'MANIFEST_INVALID' "$name licenseUrl must use upstream raw GitHub HTTPS" }
    if (-not (Is-Sha256 $component.archiveSha256)) { Fail 'UNPINNED' "$name archiveSha256 is missing or invalid" }
    if ($component.runtimeFile -cne $runtimeFiles[$name]) {
        Fail 'MANIFEST_INVALID' "$name runtimeFile must be $($runtimeFiles[$name])"
        continue
    }
    if ([string]::IsNullOrWhiteSpace([string]$component.verificationCommand)) { Fail 'MANIFEST_INVALID' "$name verificationCommand is missing" }
    if (-not (Is-Sha256 $component.runtimeSha256)) { Fail 'UNPINNED' "$name runtimeSha256 is missing or invalid" }
    if ([string]::IsNullOrWhiteSpace([string]$component.license)) { Fail 'MANIFEST_INVALID' "$name license is missing" }
    if ([string]::IsNullOrWhiteSpace([string]$component.licenseDirectory)) { Fail 'MANIFEST_INVALID' "$name licenseDirectory is missing" }
    $licensePath = Join-Path $projectRoot $component.licenseDirectory
    if (-not (Test-Path -LiteralPath $licensePath -PathType Container)) { Fail 'MISSING' "$name license directory missing" }
    elseif (@(Get-ChildItem -LiteralPath $licensePath -File).Count -eq 0) { Fail 'MISSING' "$name license directory is empty" }

    $runtimePath = Join-Path $projectRoot (Join-Path 'runtime\bin' $component.runtimeFile)
    if (-not (Test-Path -LiteralPath $runtimePath -PathType Leaf)) {
        Fail 'MISSING' "$name runtime missing: $($component.runtimeFile)"
        continue
    }

    $actual = (Get-FileHash -LiteralPath $runtimePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne ([string]$component.runtimeSha256).ToLowerInvariant()) {
        Fail 'HASH_MISMATCH' "$name runtime SHA256 differs from manifest"
        continue
    }
    Report 'PASS' "$name runtime verified ($($component.version))"
}

if ($failed) {
    exit 1
}
Report 'PASS' 'runtime manifest and installed runtime hashes are valid'
exit 0
