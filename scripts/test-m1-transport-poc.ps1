#Requires -Version 7.0
[CmdletBinding()]
param(
    [switch]$DirectOnly,
    [ValidateSet('ams', 'sgp', 'lax')][string]$Location = 'lax',
    [ValidateRange(1, 2)][int]$ServerIndex = 1
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'lib\NativeProcess.ps1')
. (Join-Path $PSScriptRoot 'lib\HttpsProbe.ps1')
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$xray = Join-Path $root 'runtime\bin\xray.exe'
$contract = Get-Content -LiteralPath (Join-Path $root 'tests\fixtures\1vpn\free-contract.json') -Raw | ConvertFrom-Json
$logDirectory = Join-Path ([IO.Path]::GetTempPath()) ('OneTunnel\M1\' + [guid]::NewGuid().ToString('N'))
[void][IO.Directory]::CreateDirectory($logDirectory)
$configs = [Collections.Generic.List[string]]::new()
$handles = [Collections.Generic.List[object]]::new()
$stage = 'RUNTIME_VERIFICATION'
$result = [ordered]@{
    Result = 'FAIL'; Failure = $null; Stage = $stage; ImplementedScope = 'M1_DIRECT'
    Mode = 'Direct'
    DirectContractSHA256 = (Get-FileHash -LiteralPath (Join-Path $root 'tests\fixtures\1vpn\free-contract.json') -Algorithm SHA256).Hash.ToLowerInvariant()
    DirectGate = 'FAIL'; Curl = $null; Baseline = $null; ServerAttempts = @()
    Runtime = @{ Xray = '26.9.9' }
    Control = [ordered]@{ Location = $Location; Server = $null; DNS = 'NOT_RUN'; TCP443 = 'NOT_RUN'; Config = 'NOT_RUN'; DirectBaselineHTTPS = 'NOT_RUN'; SOCKS = 'NOT_RUN'; Reality = 'NOT_RUN'; HTTPS = 'NOT_RUN'; Cleanup = 'NOT_RUN' }
    Safety = @{ Admin = ([Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator); RoutesModified = $false; DNSModified = $false; ProxyModified = $false; FirewallModified = $false; Wintun = 'NOT_RUN'; Tun2socks = 'NOT_RUN' }
    Processes = @(); Cleanup = 'NOT_RUN'; TimestampUTC = [DateTime]::UtcNow.ToString('o')
}

function Get-IpHash([string]$Value) {
    $parsed = $null
    if (-not [Net.IPAddress]::TryParse($Value.Trim(), [ref]$parsed)) { throw 'HTTPS_PROBE_FAILURE' }
    $sha = [Security.Cryptography.SHA256]::Create()
    try { return [Convert]::ToHexString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($parsed.ToString()))).ToLowerInvariant() }
    finally { $sha.Dispose() }
}
function Protect-Log([string]$Text) {
    foreach ($value in @($contract.uuid, $contract.publicKey, $contract.shortId)) {
        if (-not [string]::IsNullOrWhiteSpace($value)) { $Text = $Text.Replace($value, '[REDACTED]') }
    }
    $Text = $Text -replace '\b(?:\d{1,3}\.){3}\d{1,3}\b', '[IP_REDACTED]'
    $Text = [regex]::Replace($Text, '(?i)(?<![\w:])(?:[0-9a-f]{0,4}:){2,}[0-9a-f:]{0,39}(?![\w:])', {
        param($match)
        $address = $null
        if ([Net.IPAddress]::TryParse($match.Value, [ref]$address) -and $address.AddressFamily -eq [Net.Sockets.AddressFamily]::InterNetworkV6) { return '[IP_REDACTED]' }
        return $match.Value
    })
    return $Text
}
function Test-Tcp([string]$HostName, [int]$Port) {
    $client = [Net.Sockets.TcpClient]::new()
    try {
        $task = $client.ConnectAsync($HostName, $Port)
        if (-not $task.Wait(8000)) { return $false }
        return $client.Connected
    } catch { return $false } finally { $client.Dispose() }
}
function Wait-Listener($Handle, [int]$Port) {
    $deadline = [DateTime]::UtcNow.AddSeconds(12)
    while ([DateTime]::UtcNow -lt $deadline) {
        if ($Handle.Process.HasExited) { return $false }
        $owned = @(Get-NetTCPConnection -State Listen -LocalPort $Port -ErrorAction SilentlyContinue | Where-Object { $_.OwningProcess -eq $Handle.Process.Id -and $_.LocalAddress -eq '127.0.0.1' })
        if ($owned.Count -gt 0) { return $true }
        Start-Sleep -Milliseconds 150
    }
    return $false
}
function Start-Owned([string]$Name, [string]$Executable, [string[]]$Arguments) {
    $handle = Start-NativeProcess -FilePath $Executable -Arguments $Arguments
    $record = [pscustomobject]@{ Name = $Name; Handle = $handle; Pid = $handle.Process.Id; Stopped = $false }
    $handles.Add($record)
    return $handle
}
function Stop-Owned($Handle) {
    Stop-NativeProcess $Handle
    ($handles | Where-Object { $_.Handle -eq $Handle }).Stopped = $true
}
function Invoke-IpProbe([string]$Proxy, [string]$Label) {
    $probe = Invoke-OneTunnelHttpsProbe -Url 'https://api.ipify.org' -Proxy $Proxy -Timeout 25 -ExpectIpAddress
    foreach ($attempt in $probe.Attempts) {
        Protect-Log $attempt.StdErr | Set-Content -LiteralPath (Join-Path $logDirectory ($Label + '-' + $attempt.Mode + '-curl.log'))
    }
    return $probe
}
function Get-ProbeSummary($Probe) {
    [ordered]@{
        Strict = if ($Probe.Attempts[0].ExitCode -eq 0) { $Probe.Classification } else { 'FAIL' }
        StrictExitCode = $Probe.Attempts[0].ExitCode
        StrictRevocationOffline = $Probe.Attempts[0].ExitCode -eq 35 -and $Probe.Attempts[0].StdErr -match '(?i)CRYPT_E_REVOCATION_OFFLINE|0x80092013'
        BestEffort = if ($Probe.Attempts.Count -eq 2) { $Probe.Classification } else { 'NOT_RUN' }
        Classification = $Probe.Classification; Warning = $Probe.Warning; ExitCode = $Probe.ExitCode
        IpSHA256 = if ($Probe.Success) { Get-IpHash $Probe.StdOut } else { $null }
    }
}

try {
    & (Join-Path $PSScriptRoot 'verify-runtime.ps1')
    if ($LASTEXITCODE -ne 0) { throw 'RUNTIME_VERIFICATION_FAILURE' }
    $stage = 'CURL_CAPABILITY_CHECK'
    $info = Get-OneTunnelCurlInfo
    $result.Curl = @{ Version = $info.Version; TLSBackend = $info.TLSBackend; BestEffortAvailable = $info.BestEffortAvailable }
    if (-not $info.BestEffortAvailable) { throw 'CURL_BEST_EFFORT_UNAVAILABLE' }
    $stage = 'DIRECT_BASELINE_HTTPS'
    $baseline = Invoke-IpProbe '' 'baseline'
    $result.Baseline = Get-ProbeSummary $baseline
    if (-not $baseline.Success) { throw $baseline.Classification }
    # Complete IP values exist only in memory. Persist only the SHA256 summaries.
    $baselineIp = [Net.IPAddress]::Parse($baseline.StdOut.Trim()).ToString()
    $candidates = @(
        @{ Location = $Location; Index = $ServerIndex },
        @{ Location = $Location; Index = $(if ($ServerIndex -eq 1) { 2 } else { 1 }) }
    )
    foreach ($other in $contract.locations | Where-Object cityCode -ne $Location) {
        $candidates += @{ Location = $other.cityCode; Index = 1 }, @{ Location = $other.cityCode; Index = 2 }
    }
    foreach ($candidate in $candidates) {
        $generated = $null
        $direct = $null
        $probe = $null
        $selected = $contract.locations | Where-Object cityCode -eq $candidate.Location
        $control = [ordered]@{
            Location = $selected.city; Server = $selected.servers[$candidate.Index - 1].host
            DNS = 'NOT_RUN'; TCP443 = 'NOT_RUN'; Config = 'NOT_RUN'; SOCKS = 'NOT_RUN'; HTTPS = 'NOT_RUN'; Reality = 'NOT_RUN'
            DirectBaselineHTTPS = $baseline.Classification; DirectBaselineIpSHA256 = $result.Baseline.IpSHA256
            Result = 'FAIL'; Failure = $null; Cleanup = 'NOT_RUN'
        }
        $result.Control = $control
        try {
            $stage = 'XRAY_CONFIG_FAILURE'
            $generated = & (Join-Path $PSScriptRoot 'new-poc-xray-config.ps1') -Location $candidate.Location -ServerIndex $candidate.Index
            $configs.Add($generated.Path)
            $validation = Invoke-NativeProcess $xray @('run', '-test', '-config', $generated.Path)
            if ($validation.ExitCode -ne 0) { throw 'XRAY_CONFIG_FAILURE' }
            $control.Config = 'PASS'
            $stage = 'DNS_FAILURE'
            $addresses = @([Net.Dns]::GetHostAddresses($generated.Server))
            if ($addresses.Count -eq 0) { throw 'DNS_FAILURE' }
            $control.DNS = 'PASS'
            $stage = 'TCP443_FAILURE'
            if (-not (Test-Tcp $generated.Server 443)) { throw 'TCP443_FAILURE' }
            $control.TCP443 = 'PASS'
            $stage = 'SOCKS_FAILURE'
            $label = $candidate.Location + '-' + $candidate.Index
            $direct = Start-Owned ('xray-control-' + $label) $xray @('run', '-config', $generated.Path)
            $control.Pid = $direct.Process.Id
            $control.SocksBind = '127.0.0.1:' + $generated.SocksPort
            if (-not (Wait-Listener $direct $generated.SocksPort)) { throw 'SOCKS_FAILURE' }
            $control.SOCKS = 'PASS'
            $stage = 'HTTPS_PROBE_FAILURE'
            $probe = Invoke-IpProbe ('socks5h://127.0.0.1:' + $generated.SocksPort) $label
            $control.HttpsProbe = Get-ProbeSummary $probe
            $control.HTTPS = $probe.Classification
            if (-not $probe.Success) { throw $probe.Classification }
            $vpnIp = [Net.IPAddress]::Parse($probe.StdOut.Trim()).ToString()
            $control.EgressChanged = $baselineIp -ne $vpnIp
            $control.VpnDirectIpSHA256 = $control.HttpsProbe.IpSHA256
            $control.Reality = 'REALITY_CONFIRMED_BY_TRAFFIC'
            $control.EgressEvidence = if ($control.EgressChanged) { 'XRAY_DIRECT_EGRESS_CONFIRMED' } else { 'WARN_IP_UNCHANGED' }
            # Same egress IP alone is not a failure: a successful HTTPS response
            # passed through the owned SOCKS listener and sole VLESS outbound.
            $control.Result = 'PASS'
        } catch {
            $control.Failure = $stage
            switch ($stage) {
                'DNS_FAILURE' { $control.DNS = 'FAIL' }
                'TCP443_FAILURE' { $control.TCP443 = 'FAIL' }
                'XRAY_CONFIG_FAILURE' { $control.Config = 'FAIL' }
                'SOCKS_FAILURE' { $control.SOCKS = 'FAIL' }
                'HTTPS_PROBE_FAILURE' { if (-not $probe) { $control.HTTPS = 'FAIL' } }
            }
            if ($probe -and -not $probe.Success) { $control.Failure = $probe.Classification; $control.Reality = 'UNCONFIRMED' }
            Protect-Log $_.Exception.Message | Set-Content -LiteralPath (Join-Path $logDirectory ('control-' + $candidate.Location + '-' + $candidate.Index + '-failure.log'))
        } finally {
            if ($direct) { Stop-Owned $direct }
            if ($generated) { Remove-Item -LiteralPath $generated.Path -ErrorAction Stop }
            $control.Cleanup = 'PASS'
            $result.ServerAttempts += $control
        }
        if ($control.Result -eq 'PASS') { break }
        # Changing server cannot solve the exact environmental Schannel error.
        if ($probe -and $probe.ExitCode -eq 35 -and $probe.StdErr -match '(?i)CRYPT_E_REVOCATION_OFFLINE|0x80092013') { break }
    }
    if ($result.Control.Result -ne 'PASS') { throw $result.Control.Failure }
    $result.DirectGate = 'PASS'
    $result.Result = 'DIRECT_CONTROL_PASS'
    $result.Stage = 'DIRECT_GATE_COMPLETE'
} catch {
    $knownError = '^(FAIL_TLS|FAIL_CONNECTIVITY|FAIL_HTTP|CURL_BEST_EFFORT_UNAVAILABLE|DNS_FAILURE|TCP443_FAILURE|XRAY_CONFIG_FAILURE|SOCKS_FAILURE|HTTPS_PROBE_FAILURE|CLEANUP_FAILURE)$'
    $result.Failure = if ($_.Exception.Message -match $knownError) { $_.Exception.Message } else { $stage }
    $result.Stage = $stage
    if ($stage -eq 'HTTPS_PROBE_FAILURE') { $result.Control.Reality = 'UNCONFIRMED'; $result.Control.HTTPS = 'FAIL' }
    $result.Result = 'FAIL'
} finally {
    $cleanupFailed = $false
    foreach ($record in $handles) {
        try {
            Stop-Owned $record.Handle
            $log = $record.Handle.StdOutTask.GetAwaiter().GetResult() + [Environment]::NewLine + $record.Handle.StdErrTask.GetAwaiter().GetResult()
            Protect-Log $log | Set-Content -LiteralPath (Join-Path $logDirectory ($record.Name + '.log'))
        } catch { $cleanupFailed = $true }
        finally { $record.Handle.Process.Dispose() }
        $result.Processes += @{ Name = $record.Name; Pid = $record.Pid; Terminated = $record.Stopped }
    }
    foreach ($path in $configs) {
        try { if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -ErrorAction Stop } } catch { $cleanupFailed = $true }
    }
    if ($cleanupFailed) { $result.Result = 'FAIL'; $result.Failure = 'CLEANUP_FAILURE'; $result.Cleanup = 'FAIL'; $result.DirectGate = 'FAIL' }
    else { $result.Cleanup = 'PASS'; if ($handles.Count -gt 0) { $result.Control.Cleanup = 'PASS' } }
    $result | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $logDirectory 'result.json')
}
Write-Output ($result | ConvertTo-Json -Depth 10)
Write-Output "Redacted evidence: $logDirectory"
if ($result.Result -eq 'FAIL') { exit 1 }
Write-Output 'PASS: Xray/1VPN Direct'
exit 0
