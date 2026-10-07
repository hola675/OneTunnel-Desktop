. (Join-Path $PSScriptRoot 'NativeProcess.ps1')

function Get-OneTunnelCurlInfo {
    $path = (Get-Command curl.exe -ErrorAction Stop).Source
    $version = Invoke-NativeProcess -FilePath $path -Arguments @('--version')
    $help = Invoke-NativeProcess -FilePath $path -Arguments @('--help', 'all')
    if ($version.ExitCode -ne 0 -or $help.ExitCode -ne 0) { throw 'CURL_CAPABILITY_PROBE_FAILURE' }
    $match = [regex]::Match($version.StdOut, '^curl\s+(\S+)')
    [pscustomobject]@{
        FilePath = $path
        Version = $match.Groups[1].Value
        TLSBackend = if ($version.StdOut -match '\bSchannel\b') { 'Schannel' } else { 'OTHER' }
        BestEffortAvailable = $help.CombinedOutput -match '(?m)^\s+--ssl-revoke-best-effort\s'
    }
}

function Invoke-OneTunnelHttpsProbe {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][uri]$Url,
        [AllowEmptyString()][string]$Proxy = '',
        [ValidateRange(1, 300)][int]$Timeout = 25,
        [switch]$ExpectIpAddress
    )
    if (-not $Url.IsAbsoluteUri -or $Url.Scheme -ne 'https' -or $Url.UserInfo) {
        throw 'HTTPS probes require an absolute HTTPS URL without credentials.'
    }
    $curlPath = (Get-Command curl.exe -ErrorAction Stop).Source
    # -q is deliberately first: no user curlrc can change the TLS policy.
    # Explicit proxy/noproxy also prevents inherited proxy environment variables
    # from silently changing the baseline or bypassing the requested SOCKS proxy.
    $arguments = @('-q', '--silent', '--show-error', '--fail', '--proto', '=https',
        '--connect-timeout', [string][Math]::Min(10, $Timeout), '--max-time', [string]$Timeout,
        '--proxy', $Proxy, '--noproxy', $(if ($Proxy) { '' } else { '*' }))
    $attempts = [Collections.Generic.List[object]]::new()
    $warning = $null
    for ($attemptIndex = 0; $attemptIndex -lt 2; $attemptIndex++) {
        $mode = if ($attemptIndex -eq 0) { 'STRICT' } else { 'REVOCATION_BEST_EFFORT' }
        $attemptArguments = @($arguments)
        if ($attemptIndex -eq 1) { $attemptArguments += '--ssl-revoke-best-effort' }
        try {
            $native = Invoke-NativeProcess -FilePath $curlPath -Arguments ($attemptArguments + @($Url.AbsoluteUri)) -TimeoutSeconds ($Timeout + 5)
        } catch {
            if ($_.Exception.Message -notmatch '^Native process timed out after \d+ seconds\.$') { throw }
            $native = [pscustomobject]@{ ExitCode = 28; StdOut = ''; StdErr = 'Native process timeout.' }
        }
        $attempts.Add([pscustomobject]@{ Mode = $mode; ExitCode = $native.ExitCode; StdOut = $native.StdOut; StdErr = $native.StdErr })
        $exactRevocationError = $native.ExitCode -eq 35 -and $native.StdErr -match '(?i)CRYPT_E_REVOCATION_OFFLINE|0x80092013'
        if ($attemptIndex -eq 0 -and $exactRevocationError) {
            $warning = 'SCHANNEL_REVOCATION_OFFLINE'
            continue
        }
        if ($native.ExitCode -eq 0) {
            $validResponse = -not [string]::IsNullOrWhiteSpace($native.StdOut)
            if ($ExpectIpAddress) {
                $parsedIp = $null
                $validResponse = [Net.IPAddress]::TryParse($native.StdOut.Trim(), [ref]$parsedIp)
            }
            $classification = if (-not $validResponse) { 'FAIL_HTTP' }
                elseif ($attemptIndex -eq 0) { 'PASS_STRICT' }
                else { 'PASS_REVOCATION_BEST_EFFORT' }
        } elseif ($native.ExitCode -in @(35, 51, 53, 54, 58, 59, 60, 64, 66, 77, 80, 82, 83, 90, 91, 98)) {
            $classification = 'FAIL_TLS'
        } elseif ($native.ExitCode -eq 22 -or $native.ExitCode -eq 47) {
            $classification = 'FAIL_HTTP'
        } else {
            $classification = 'FAIL_CONNECTIVITY'
        }
        return [pscustomobject]@{
            Classification = $classification
            Success = $classification -in @('PASS_STRICT', 'PASS_REVOCATION_BEST_EFFORT')
            Warning = $warning
            ExitCode = $native.ExitCode
            StdOut = $native.StdOut
            StdErr = $native.StdErr
            Attempts = @($attempts.ToArray())
        }
    }
}
