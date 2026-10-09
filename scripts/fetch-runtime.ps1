[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$manifestPath = Join-Path $projectRoot 'runtime\manifest.json'
$downloadsPath = Join-Path $projectRoot 'runtime\downloads'
$binPath = Join-Path $projectRoot 'runtime\bin'
$officialReleaseBases = @{
    xray = 'https://github.com/XTLS/Xray-core/releases/download'
    tun2socks = 'https://github.com/xjasonlyu/tun2socks/releases/download'
    wintun = 'https://www.wintun.net/builds'
}

function Report([string]$Status, [string]$Message) {
    Write-Output ("{0}: {1}" -f $Status, $Message)
}

function Get-RequiredSha([object]$Component) {
    if ([string]$Component.archiveSha256 -notmatch '^[A-Fa-f0-9]{64}$') {
        throw "archiveSha256 is not pinned"
    }
    return ([string]$Component.archiveSha256).ToLowerInvariant()
}

function Download-Official([string]$Url, [string]$Destination) {
    $uri = [Uri]$Url
    if ($uri.Scheme -ne 'https') { throw "Only HTTPS sources are allowed: $Url" }
    Invoke-WebRequest -UseBasicParsing -Uri $Url -OutFile $Destination
}

function Find-ExtractedFile([string]$Root, [string]$Name) {
    $matches = @(Get-ChildItem -LiteralPath $Root -File -Recurse | Where-Object { $_.Name -ieq $Name })
    if ($matches.Count -ne 1) { throw "Expected exactly one extracted $Name, found $($matches.Count)" }
    return $matches[0].FullName
}

function Copy-LicenseFiles([string]$ExtractRoot, [string]$Destination, [string]$LicenseUrl) {
    New-Item -ItemType Directory -Force -Path $Destination | Out-Null
    $licenses = @(Get-ChildItem -LiteralPath $ExtractRoot -File -Recurse | Where-Object { $_.Name -match '^(LICENSE|COPYING)(\..*)?$' })
    foreach ($license in $licenses) {
        Copy-Item -LiteralPath $license.FullName -Destination (Join-Path $Destination $license.Name) -Force
    }
    if (($licenses.Count -eq 0) -and $LicenseUrl) {
        Download-Official $LicenseUrl (Join-Path $Destination 'LICENSE')
    } elseif ($licenses.Count -eq 0) {
        Set-Content -LiteralPath (Join-Path $Destination 'NOTICE.txt') -Value 'License notice was not embedded in the release archive; consult the upstream release and manifest license field.' -Encoding utf8
    }
}

function Confirm-ChecksumManifest([object]$Component, [string]$ArchivePath) {
    if ([string]::IsNullOrWhiteSpace([string]$Component.checksumUrl)) { return }
    $checksumPath = Join-Path $downloadsPath ("{0}.checksums.txt" -f $Component.artifact)
    if (-not (Test-Path -LiteralPath $checksumPath -PathType Leaf)) {
        Download-Official $Component.checksumUrl $checksumPath
    }
    $content = Get-Content -LiteralPath $checksumPath -Raw
    $artifactPattern = [regex]::Escape([string]$Component.artifact)
    $match = [regex]::Match($content, "(?m)^\s*([A-Fa-f0-9]{64})\s+\*?$artifactPattern\s*$")
    if (-not $match.Success) { throw "Official checksum manifest does not contain $($Component.artifact)" }
    $listedHash = $match.Groups[1].Value.ToLowerInvariant()
    $expectedHash = Get-RequiredSha $Component
    if ($listedHash -ne $expectedHash) { throw "Manifest archiveSha256 differs from official checksum manifest for $($Component.artifact)" }
    $actualHash = (Get-FileHash -LiteralPath $ArchivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualHash -ne $listedHash) { throw "Downloaded archive differs from official checksum manifest for $($Component.artifact)" }
    Report 'PASS' "$($Component.artifact) official checksum manifest verified"
}

if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) { throw "Manifest missing: $manifestPath" }
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
if ($manifest.schemaVersion -ne 3) { throw 'fetch-runtime requires manifest schemaVersion 3' }

New-Item -ItemType Directory -Force -Path $downloadsPath, $binPath | Out-Null
$components = @('xray', 'tun2socks', 'wintun')
if ($manifest.platform -ne 'windows-x86_64') { throw 'platform must be windows-x86_64' }
$componentNames = @($manifest.components.PSObject.Properties.Name)
if ($componentNames.Count -ne $components.Count -or @($componentNames | Where-Object { $_ -notin $components }).Count -gt 0) {
    throw 'manifest must contain exactly xray, tun2socks and wintun'
}
$runtimeFiles = @{ xray = 'xray.exe'; tun2socks = 'tun2socks.exe'; wintun = 'wintun.dll' }

foreach ($name in $components) {
    $component = $manifest.components.$name
    if ($null -eq $component) { throw "Missing component in manifest: $name" }
    if ([string]::IsNullOrWhiteSpace([string]$component.artifact)) { throw "$name is not pinned; artifact is missing" }
    if ($component.runtimeFile -cne $runtimeFiles[$name]) { throw "$name runtimeFile must be $($runtimeFiles[$name])" }
    if ([string]$component.runtimeSha256 -notmatch '^[A-Fa-f0-9]{64}$') { throw "$name runtimeSha256 is not pinned" }
    $expectedRuntimeHash = ([string]$component.runtimeSha256).ToLowerInvariant()
    if ($name -eq 'wintun') {
        $expectedSource = "$($officialReleaseBases[$name])/$($component.artifact)"
    } else {
        $expectedSource = "$($officialReleaseBases[$name])/$($component.tag)/$($component.artifact)"
    }
    if ([string]$component.sourceUrl -ne $expectedSource) { throw "$name sourceUrl must exactly match its official upstream release URL" }

    $archivePath = Join-Path $downloadsPath $component.artifact
    if (Test-Path -LiteralPath $archivePath -PathType Leaf) {
        $cachedArchiveHash = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
        $expectedArchiveHash = Get-RequiredSha $component
        if ($cachedArchiveHash -ne $expectedArchiveHash) {
            Report 'FAIL HASH_MISMATCH' "$name cached archive SHA256 expected $expectedArchiveHash, got $cachedArchiveHash"
            exit 1
        }
        Confirm-ChecksumManifest $component $archivePath
    }

    $runtimePath = Join-Path $binPath $component.runtimeFile
    if ((Test-Path -LiteralPath $runtimePath -PathType Leaf) -and ([string]$component.runtimeSha256 -match '^[A-Fa-f0-9]{64}$')) {
        $cachedHash = (Get-FileHash -LiteralPath $runtimePath -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($cachedHash -eq ([string]$component.runtimeSha256).ToLowerInvariant()) {
            Report 'PASS CACHED' "$name $($component.version) runtime hash verified"
            continue
        }
        throw "Existing $name runtime does not match manifest runtimeSha256"
    }
    if ((Test-Path -LiteralPath $runtimePath -PathType Leaf) -and [string]::IsNullOrWhiteSpace([string]$component.runtimeSha256)) {
        throw "Existing $name runtime cannot be overwritten because manifest runtimeSha256 is not populated"
    }

    if (-not (Test-Path -LiteralPath $archivePath -PathType Leaf)) {
        Report 'FETCH' "$name $($component.version) from $($component.sourceUrl)"
        $temporaryPath = "$archivePath.partial"
        if (Test-Path -LiteralPath $temporaryPath) { Remove-Item -LiteralPath $temporaryPath -Force }
        Download-Official $component.sourceUrl $temporaryPath
        Move-Item -LiteralPath $temporaryPath -Destination $archivePath -Force
    }

    $archiveHash = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    $expectedArchiveHash = Get-RequiredSha $component
    if ($archiveHash -ne $expectedArchiveHash) {
        Report 'FAIL HASH_MISMATCH' "$name archive SHA256 expected $expectedArchiveHash, got $archiveHash"
        exit 1
    }
    Report 'PASS' "$name archive SHA256 verified"
    Confirm-ChecksumManifest $component $archivePath

    $tempBase = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    $extractRoot = [IO.Path]::GetFullPath((Join-Path $tempBase ("onetunnel-runtime-{0}-{1}" -f $name, [Guid]::NewGuid().ToString('N'))))
    if (-not $extractRoot.StartsWith($tempBase, [StringComparison]::OrdinalIgnoreCase)) { throw 'Extraction path must stay within TEMP' }
    New-Item -ItemType Directory -Force -Path $extractRoot | Out-Null
    try {
        if ($archivePath -match '\.tar\.gz$') {
            tar -xzf $archivePath -C $extractRoot
        } elseif ($archivePath -match '\.zip$') {
            Expand-Archive -LiteralPath $archivePath -DestinationPath $extractRoot -Force
        } else {
            throw "Unsupported archive format: $archivePath"
        }

        $sourceFile = if ($component.extractPath) {
            $candidate = Join-Path $extractRoot ([string]$component.extractPath)
            if (-not (Test-Path -LiteralPath $candidate -PathType Leaf)) { throw "Required extractPath missing: $($component.extractPath)" }
            $candidate
        } else {
            $archiveRuntimeFile = if ($component.extractFile) { [string]$component.extractFile } else { [string]$component.runtimeFile }
            Find-ExtractedFile $extractRoot $archiveRuntimeFile
        }
        $runtimeHash = (Get-FileHash -LiteralPath $sourceFile -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($runtimeHash -ne $expectedRuntimeHash) { throw "$name extracted runtime SHA256 differs from manifest" }
        Copy-Item -LiteralPath $sourceFile -Destination $runtimePath -Force
        Copy-LicenseFiles $extractRoot (Join-Path $projectRoot $component.licenseDirectory) ([string]$component.licenseUrl)
        Report 'PASS' "$name runtime extracted with SHA256 $runtimeHash"
    } finally {
        if (Test-Path -LiteralPath $extractRoot) { Remove-Item -LiteralPath $extractRoot -Recurse -Force }
    }
}

exit 0
