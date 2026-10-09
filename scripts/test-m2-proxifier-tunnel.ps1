[CmdletBinding()]
param(
    [ValidateSet('ams', 'sgp', 'lax')][string]$Location = 'lax',
    [ValidateRange(1, 2)][int]$ServerIndex = 1,
    [ValidateRange(1, 65535)][int]$SocksPort = 10808,
    [string]$ProxifierLogPath,
    [string]$XrayCorporateRuleName = 'OneTunnel-Xray-Outbound',
    [string]$SelectedAppRuleName = 'OneTunnel-Test-App',
    [switch]$SkipManualSetupWait
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$xray = Join-Path $repoRoot 'runtime\bin\xray.exe'
$evidenceDirectory = Join-Path ([IO.Path]::GetTempPath()) ('OneTunnel\m2-proxifier-' + [guid]::NewGuid().ToString('N'))
[void][IO.Directory]::CreateDirectory($evidenceDirectory)
. (Join-Path $PSScriptRoot 'lib\NativeProcess.ps1')
. (Join-Path $PSScriptRoot 'lib\HttpsProbe.ps1')

$processHandle = $null
$configPath = $null
$logOffset = $null
$script:result = [ordered]@{
    Result = 'PARTIAL'
    Stage = 'STARTUP'
    Failure = $null
    Runtime = 'NOT_RUN'
    Proxifier = 'NOT_DETECTED'
    ProxifierLogLevel = 'UNVERIFIED'
    ProxifierLogEvidence = 'MISSING'
    Location = $Location
    Server = $null
    Socks = "SOCKS5 127.0.0.1:$SocksPort"
    XrayPid = $null
    XrayConfig = 'NOT_RUN'
    SocksListener = 'NOT_RUN'
    XrayTunnel = 'NOT_RUN'
    XrayCorporateProxyRule = 'UNVERIFIED'
    XrayProxyRuleObserved = 'UNVERIFIED'
    SelectedApplication = 'curl.exe'
    SelectedAppProxyArguments = 'NONE'
    SelectedApplicationHttps = 'NOT_RUN'
    SelectedAppOneTunnelRule = 'UNVERIFIED'
    UnselectedApplication = $null
    UnselectedApplicationHttps = 'NOT_RUN'
    UnselectedDefaultDirect = 'UNVERIFIED'
    XraySelfProxyLoop = 'UNKNOWN'
    NormalEgressSHA256 = $null
    SelectedEgressSHA256 = $null
    XrayEgressSHA256 = $null
    SelectedEqualsXray = 'UNVERIFIED'
    SelectedDiffersFromNormal = 'UNVERIFIED'
    Wintun = 'NOT USED'
    Tun2socks = 'NOT USED'
    RoutesModified = 'NO'
    DNSModified = 'NO'
    AdminRequired = 'NO'
    Cleanup = 'NOT_RUN'
    ProxifierModifiedByScript = 'NO'
}

function Get-IpDigest([string]$Value) {
    $ip = $null
    if (-not [Net.IPAddress]::TryParse($Value.Trim(), [ref]$ip)) { throw 'IP_PROBE_INVALID' }
    $bytes = [Text.Encoding]::UTF8.GetBytes($ip.ToString())
    $sha = [Security.Cryptography.SHA256]::Create()
    try { return ([BitConverter]::ToString($sha.ComputeHash($bytes))).Replace('-', '').ToLowerInvariant() }
    finally { $sha.Dispose() }
}

function Invoke-WithCleanProxyEnvironment([scriptblock]$Action) {
    $names = @('HTTP_PROXY','HTTPS_PROXY','ALL_PROXY','NO_PROXY','http_proxy','https_proxy','all_proxy','no_proxy')
    $saved = @{}
    foreach ($name in $names) {
        $saved[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
        [Environment]::SetEnvironmentVariable($name, $null, 'Process')
    }
    try { & $Action }
    finally {
        foreach ($name in $names) { [Environment]::SetEnvironmentVariable($name, $saved[$name], 'Process') }
    }
}

function Invoke-CurlHttpsProbe([string]$Url, [string]$Label) {
    $curl = Get-OneTunnelCurlInfo
    if (-not $curl.BestEffortAvailable) { throw 'CURL_BEST_EFFORT_UNAVAILABLE' }
    $attempts = [Collections.Generic.List[object]]::new()
    $warning = $null
    for ($index = 0; $index -lt 2; $index++) {
        $mode = if ($index -eq 0) { 'STRICT' } else { 'REVOCATION_BEST_EFFORT' }
        # No proxy flags are used for either application probe. -q prevents curlrc settings.
        $arguments = @('-q','--silent','--show-error','--fail','--proto','=https',
            '--connect-timeout','10','--max-time','25')
        if ($index -eq 1) { $arguments += '--ssl-revoke-best-effort' }
        $arguments += $Url
        $native = Invoke-WithCleanProxyEnvironment {
            Invoke-NativeProcess -FilePath $curl.FilePath -Arguments $arguments -TimeoutSeconds 30
        }
        $revocationOffline = $native.ExitCode -eq 35 -and $native.StdErr -match '(?i)CRYPT_E_REVOCATION_OFFLINE|0x80092013'
        $attempts.Add([pscustomobject]@{ Mode = $mode; ExitCode = $native.ExitCode; RevocationOffline = $revocationOffline })
        if ($index -eq 0 -and $revocationOffline) { $warning = 'SCHANNEL_REVOCATION_OFFLINE'; continue }
        if ($native.ExitCode -eq 0) {
            return [pscustomobject]@{
                Classification = if ($index -eq 0) { 'PASS_STRICT' } else { 'PASS_REVOCATION_BEST_EFFORT' }
                Success = $true; IpSHA256 = Get-IpDigest $native.StdOut; Warning = $warning
                Attempts = @($attempts.ToArray()); Label = $Label
            }
        }
        $kind = if ($native.ExitCode -in @(35,51,53,54,58,59,60,64,66,77,80,82,83,90,91,98)) { 'FAIL_TLS' } else { 'FAIL_CONNECTIVITY' }
        return [pscustomobject]@{ Classification = $kind; Success = $false; IpSHA256 = $null; Warning = $warning; Attempts = @($attempts.ToArray()); Label = $Label }
    }
    throw 'HTTPS_PROBE_FAILURE'
}

function Invoke-UnselectedHttpsProbe([string]$Url) {
    $handler = [Net.Http.HttpClientHandler]::new()
    $handler.UseProxy = $false
    $client = [Net.Http.HttpClient]::new($handler)
    try {
        $client.Timeout = [TimeSpan]::FromSeconds(25)
        $response = Invoke-WithCleanProxyEnvironment { $client.GetAsync($Url).GetAwaiter().GetResult() }
        $response.EnsureSuccessStatusCode() | Out-Null
        $body = $response.Content.ReadAsStringAsync().GetAwaiter().GetResult()
        return [pscustomobject]@{ Classification = 'PASS_STRICT'; IpSHA256 = Get-IpDigest $body; Success = $true }
    } finally { $client.Dispose(); $handler.Dispose() }
}

function Test-SocksPortFree([int]$Port) {
    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, $Port)
    try { $listener.Start(); return $true }
    catch { return $false }
    finally { try { $listener.Stop() } catch {} }
}

function Test-XrayOwnLoop([int]$ProcessId, [int]$Port) {
    $connections = @(Get-NetTCPConnection -OwningProcess $ProcessId -ErrorAction SilentlyContinue |
        Where-Object { $_.RemoteAddress -in @('127.0.0.1','::1') -and $_.RemotePort -eq $Port })
    return $connections.Count -gt 0
}

function Get-ProxifierLogSince([string]$Path, [long]$Offset) {
    if (-not $Path -or -not (Test-Path -LiteralPath $Path -PathType Leaf)) { return @() }
    # In-memory read of only the test interval. Lines are never printed, copied, or persisted.
    $stream = [IO.File]::Open($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::ReadWrite)
    try {
        if ($stream.Length -lt $Offset) { return @() } # rotated/truncated during test
        $stream.Position = $Offset
        $reader = [IO.StreamReader]::new($stream, [Text.Encoding]::UTF8, $true, 4096, $true)
        try { $text = $reader.ReadToEnd() } finally { $reader.Dispose() }
        $text = $text.Replace([string][char]0, '')
        return @($text -split "`r?`n" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    } finally { $stream.Dispose() }
}

function Test-LogWindow([string[]]$Lines, [string[]]$Terms) {
    # Proxifier writes one connection decision per line. Never combine
    # neighboring records: that can attribute another process's rule to Xray.
    foreach ($line in $Lines) {
        $allFound = $true
        foreach ($term in $Terms) { if ($line -notmatch [regex]::Escape($term)) { $allFound = $false; break } }
        if ($allFound) { return $true }
    }
    return $false
}

function Test-AnyLogWindow([string[]]$Lines, [string]$ProcessName, [string]$Action) {
    return Test-LogWindow $Lines @($ProcessName, $Action)
}

try {
    $script:result.Stage = 'SOCKS_PORT_CHECK'
    if (-not (Test-SocksPortFree $SocksPort)) { throw 'SOCKS_PORT_IN_USE' }

    $script:result.Stage = 'RUNTIME_VERIFICATION'
    & (Join-Path $PSScriptRoot 'verify-runtime.ps1') | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'RUNTIME_VERIFICATION_FAILURE' }
    $script:result.Runtime = 'PASS_HASHES_ONLY'

    $script:result.Stage = 'PROXIFIER_DETECTION'
    if (@(Get-Process -Name 'Proxifier' -ErrorAction SilentlyContinue).Count -gt 0) {
        $script:result.Proxifier = 'PROXIFIER_DETECTED'
    }

    $script:result.Stage = 'CONFIGURATION'
    $generated = & (Join-Path $PSScriptRoot 'new-xray-config.ps1') -Location $Location -ServerIndex $ServerIndex -SocksPort $SocksPort
    $configPath = $generated.Path
    $script:result.Server = $generated.Server
    $validation = Invoke-NativeProcess -FilePath $xray -Arguments @('run','-test','-config',$configPath)
    if ($validation.ExitCode -ne 0) { throw 'XRAY_CONFIG_FAILURE' }
    $script:result.XrayConfig = 'PASS'

    $script:result.Stage = 'XRAY_START'
    $processHandle = Start-NativeProcess -FilePath $xray -Arguments @('run','-config',$configPath)
    $processId = $processHandle.Process.Id
    $script:result.XrayPid = $processId
    $listenerReady = $false
    $deadline = [DateTime]::UtcNow.AddSeconds(15)
    while ([DateTime]::UtcNow -lt $deadline) {
        if ($processHandle.Process.HasExited) { break }
        $owned = @(Get-NetTCPConnection -State Listen -LocalPort $SocksPort -ErrorAction SilentlyContinue |
            Where-Object { $_.OwningProcess -eq $processId -and $_.LocalAddress -eq '127.0.0.1' })
        if ($owned.Count -gt 0) { $listenerReady = $true; break }
        Start-Sleep -Milliseconds 150
    }
    if (-not $listenerReady) { throw 'SOCKS_LISTENER_FAILURE' }
    $script:result.SocksListener = 'PASS_127.0.0.1_OWNED_BY_XRAY_PID'
    if (Test-XrayOwnLoop $processId $SocksPort) { throw 'PROXIFIER_LOOP' }
    $script:result.XraySelfProxyLoop = 'NONE'

    $curlInfo = Get-OneTunnelCurlInfo
    $script:result.SelectedApplication = $curlInfo.FilePath
    $script:result.UnselectedApplication = (Get-Process -Id $PID).Path
    $script:result.Stage = 'MANUAL_PROXIFIER_SETUP'
    Write-Output ''
    Write-Output 'M2-R.1 manual Proxifier setup (no profile changes will be made):'
    Write-Output '  Localhost/loopback -> DIRECT'
    Write-Output '  xray.exe -> Corporate Proxy'
    Write-Output "  $($curlInfo.FilePath) -> OneTunnel-Xray (SOCKS5 127.0.0.1:$SocksPort)"
    Write-Output '  Default -> DIRECT'
    Write-Output 'Enable Log -> File Log -> Verbose. Do not route xray.exe to OneTunnel-Xray.'
    if (-not $SkipManualSetupWait) {
        [void](Read-Host 'After applying rules and enabling Verbose File Log, press Enter to run all three HTTPS probes')
        if (-not $ProxifierLogPath) { $ProxifierLogPath = Read-Host 'Enter the Proxifier log file path (blank leaves evidence PARTIAL)' }
    }

    if ($ProxifierLogPath -and (Test-Path -LiteralPath $ProxifierLogPath -PathType Leaf)) {
        $fileBefore = Get-Item -LiteralPath $ProxifierLogPath
        $logOffset = $fileBefore.Length
    } elseif ($ProxifierLogPath) {
        $logOffset = 0
    }

    $url = 'https://api.ipify.org'
    $script:result.Stage = 'XRAY_CORPORATE_PROXY_PROBE'
    $xrayProbe = Invoke-OneTunnelHttpsProbe -Url $url -Proxy ("socks5h://127.0.0.1:$SocksPort") -Timeout 25 -ExpectIpAddress
    $script:result.XrayTunnel = $xrayProbe.Classification
    if (-not $xrayProbe.Success) { throw $xrayProbe.Classification }
    $script:result.XrayEgressSHA256 = Get-IpDigest $xrayProbe.StdOut
    if (Test-XrayOwnLoop $processId $SocksPort) { throw 'PROXIFIER_LOOP' }
    $script:result.XraySelfProxyLoop = 'NONE'

    $script:result.Stage = 'UNSELECTED_APP_PROBE'
    $unselected = Invoke-WithCleanProxyEnvironment { Invoke-UnselectedHttpsProbe $url }
    $script:result.UnselectedApplicationHttps = $unselected.Classification
    if (-not $unselected.Success) { throw 'UNSELECTED_HTTPS_FAILURE' }
    $script:result.NormalEgressSHA256 = $unselected.IpSHA256

    $script:result.Stage = 'SELECTED_APP_PROBE'
    # curl receives no proxy option, URI, or proxy environment variable.
    $selected = Invoke-CurlHttpsProbe -Url $url -Label 'selected-app'
    $script:result.SelectedApplicationHttps = $selected.Classification
    if (-not $selected.Success) { throw 'SELECTED_HTTPS_FAILURE' }
    $script:result.SelectedEgressSHA256 = $selected.IpSHA256
    $script:result.SelectedEqualsXray = if ($selected.IpSHA256 -eq $script:result.XrayEgressSHA256) { 'YES' } else { 'NO' }
    $script:result.SelectedDiffersFromNormal = if ($selected.IpSHA256 -ne $script:result.NormalEgressSHA256) { 'YES' } else { 'NO' }

    if ($ProxifierLogPath -and (Test-Path -LiteralPath $ProxifierLogPath -PathType Leaf)) {
        $logLines = Get-ProxifierLogSince $ProxifierLogPath $logOffset
        $script:result.ProxifierLogEvidence = 'READ_ONLY_REDACTED_SUMMARY'
        # Rule names may share the text "OneTunnel-Xray" while routing Xray
        # through the corporate HTTPS proxy. A loop requires evidence of the
        # selected SOCKS rule/action or a live socket from Xray to its own SOCKS.
        $xrayLoop = Test-LogWindow $logLines @('xray.exe',$SelectedAppRuleName) -or
            Test-LogWindow $logLines @('xray.exe','Proxy SOCKS5 localhost')
        $serverTargets = @($generated.Server)
        try { $serverTargets += @([Net.Dns]::GetHostAddresses($generated.Server) | ForEach-Object { $_.IPAddressToString }) } catch {}
        $corporateRule = $false
        foreach ($target in $serverTargets) {
            if (Test-LogWindow $logLines @('xray.exe',$XrayCorporateRuleName,$target)) { $corporateRule = $true; break }
        }
        $selectedRule = Test-LogWindow $logLines @([IO.Path]::GetFileName($curlInfo.FilePath),$SelectedAppRuleName,'api.ipify.org')
        $unselectedName = [IO.Path]::GetFileName($script:result.UnselectedApplication)
        $unselectedDirect = (Test-AnyLogWindow $logLines $unselectedName 'Direct') -or (Test-LogWindow $logLines @($unselectedName,'Default'))
        $selectedWrong = Test-LogWindow $logLines @([IO.Path]::GetFileName($curlInfo.FilePath),'api.ipify.org','Direct')
        $xrayWrong = $false
        foreach ($target in $serverTargets) {
            if ((Test-LogWindow $logLines @('xray.exe',$target,'Direct')) -or (Test-LogWindow $logLines @('xray.exe',$target,'Default'))) { $xrayWrong = $true; break }
        }
        if ($selectedWrong -and -not $selectedRule) { throw 'SELECTED_RULE_NOT_APPLIED' }
        if ($xrayWrong -and -not $corporateRule) { throw 'XRAY_CORPORATE_RULE_NOT_APPLIED' }
        if (-not $unselectedDirect -and (Test-LogWindow $logLines @($unselectedName,'Proxy'))) { throw 'UNSELECTED_DEFAULT_DIRECT_NOT_APPLIED' }
        $script:result.XrayCorporateProxyRule = if ($corporateRule) { 'PASS_LOG_RULE_EVIDENCE' } else { 'UNVERIFIED' }
        $script:result.XrayProxyRuleObserved = if ($xrayLoop) { $SelectedAppRuleName } elseif ($corporateRule) { $XrayCorporateRuleName } else { 'UNVERIFIED' }
        $script:result.SelectedAppOneTunnelRule = if ($selectedRule) { 'PASS_LOG_RULE_EVIDENCE' } else { 'UNVERIFIED' }
        $script:result.UnselectedDefaultDirect = if ($unselectedDirect) { 'PASS_LOG_DIRECT_EVIDENCE' } else { 'UNVERIFIED' }
        if ($xrayLoop -or $corporateRule -or $selectedRule -or $unselectedDirect) { $script:result.ProxifierLogLevel = 'VERBOSE_RULE_EVIDENCE_PRESENT' }
        if ($xrayLoop) {
            $script:result.XraySelfProxyLoop = 'PROXIFIER_LOOP'
            $script:result.XrayCorporateProxyRule = 'FAIL_XRAY_CORPORATE_RULE_NOT_APPLIED'
            throw 'PROXIFIER_LOOP'
        }
        if (Test-XrayOwnLoop $processId $SocksPort) {
            $script:result.XraySelfProxyLoop = 'PROXIFIER_LOOP'
            $script:result.XrayCorporateProxyRule = 'FAIL_XRAY_CORPORATE_RULE_NOT_APPLIED'
            throw 'PROXIFIER_LOOP'
        }
    }

    $gatesPass = $script:result.XrayTunnel -like 'PASS_*' -and
        $script:result.XrayCorporateProxyRule -eq 'PASS_LOG_RULE_EVIDENCE' -and
        $script:result.SelectedAppOneTunnelRule -eq 'PASS_LOG_RULE_EVIDENCE' -and
        $script:result.UnselectedDefaultDirect -eq 'PASS_LOG_DIRECT_EVIDENCE' -and
        $script:result.XraySelfProxyLoop -eq 'NONE'
    if ($gatesPass -and ($script:result.SelectedEqualsXray -eq 'YES' -or $selected.Success)) {
        $script:result.Result = 'PASS'
    } else {
        $script:result.Result = 'PARTIAL'
        $script:result.Failure = if (-not $ProxifierLogPath) { 'PROXIFIER_EVIDENCE_MISSING' } else { 'PROXIFIER_RULE_EVIDENCE_INCOMPLETE' }
    }
    $script:result.Stage = 'COMPLETE'
} catch {
    $script:result.Failure = $_.Exception.Message
    $script:result.Result = 'FAIL'
} finally {
    $cleanupFailed = $false
    if ($processHandle) {
        try {
            Stop-NativeProcess $processHandle
            $processHandle.Process.Dispose()
            $ownedAfter = @(Get-NetTCPConnection -State Listen -LocalPort $SocksPort -ErrorAction SilentlyContinue |
                Where-Object { $_.OwningProcess -eq $processId -and $_.LocalAddress -eq '127.0.0.1' })
            $stillRunning = Get-Process -Id $processId -ErrorAction SilentlyContinue
            if ($ownedAfter.Count -gt 0 -or $stillRunning) { $cleanupFailed = $true }
        } catch { $cleanupFailed = $true }
    }
    if ($configPath -and (Test-Path -LiteralPath $configPath)) {
        try { Remove-Item -LiteralPath $configPath -ErrorAction Stop } catch { $cleanupFailed = $true }
    }
    $script:result.Cleanup = if ($cleanupFailed) { 'FAIL' } elseif ($processHandle) { 'PASS_XRAY_PID_GONE_CONFIG_REMOVED_SOCKS_GONE' } else { 'PASS_NO_XRAY_STARTED' }
    if ($cleanupFailed) { $script:result.Result = 'FAIL'; $script:result.Failure = 'CLEANUP_FAILURE' }
    if ($script:result.Result -eq 'PARTIAL' -and (-not $ProxifierLogPath -or -not (Test-Path -LiteralPath $ProxifierLogPath -PathType Leaf))) {
        $script:result.Failure = 'PROXIFIER_EVIDENCE_MISSING'
    }
    $script:result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $evidenceDirectory 'result.json') -Encoding utf8
}

Write-Output ($script:result | ConvertTo-Json -Depth 8)
Write-Output "Redacted evidence: $evidenceDirectory"
if ($script:result.Result -eq 'FAIL') { exit 1 }
if ($script:result.Result -eq 'PARTIAL') { Write-Output 'PARTIAL PROXIFIER_EVIDENCE_MISSING'; exit 2 }
Write-Output 'PASS: M2-R.1 Proxifier live rule certification'
exit 0
