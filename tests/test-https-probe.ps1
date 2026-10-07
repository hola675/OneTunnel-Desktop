$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot '..\scripts\lib\HttpsProbe.ps1')

# Replace only the native execution boundary. No child processes or Internet.
function Invoke-NativeProcess {
    param([string]$FilePath, [string[]]$Arguments, [int]$TimeoutSeconds)
    $script:calls.Add(@($Arguments))
    if ($script:responses.Count -eq 0) { throw 'Unexpected extra HTTPS attempt' }
    $response = $script:responses.Dequeue()
    if ($response -is [string]) { throw $response }
    return $response
}
function New-Response([int]$Code, [string]$Out = '', [string]$Err = '') {
    [pscustomobject]@{ ExitCode = $Code; StdOut = $Out; StdErr = $Err }
}
function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw "FAIL: $Message" }
}
$cases = @(
    @{ Name = 'strict success'; Responses = @((New-Response 0 '203.0.113.1')); Expected = 'PASS_STRICT'; Count = 1 },
    @{ Name = 'named revocation error retry'; Responses = @((New-Response 35 '' 'CRYPT_E_REVOCATION_OFFLINE'), (New-Response 0 '203.0.113.2')); Expected = 'PASS_REVOCATION_BEST_EFFORT'; Count = 2 },
    @{ Name = 'hex revocation error retry'; Responses = @((New-Response 35 '' '0x80092013'), (New-Response 0 '2001:db8::1')); Expected = 'PASS_REVOCATION_BEST_EFFORT'; Count = 2 },
    @{ Name = 'generic TLS / no retry'; Responses = @((New-Response 35 '' 'TLS handshake failed')); Expected = 'FAIL_TLS'; Count = 1 },
    @{ Name = 'untrusted certificate / no retry'; Responses = @((New-Response 60 '' 'certificate not trusted')); Expected = 'FAIL_TLS'; Count = 1 },
    @{ Name = 'curl timeout / no retry'; Responses = @((New-Response 28 '' 'Operation timed out')); Expected = 'FAIL_CONNECTIVITY'; Count = 1 },
    @{ Name = 'native timeout / no retry'; Responses = @('Native process timed out after 30 seconds.'); Expected = 'FAIL_CONNECTIVITY'; Count = 1 },
    @{ Name = 'marker with wrong exit / no retry'; Responses = @((New-Response 7 '' '0x80092013')); Expected = 'FAIL_CONNECTIVITY'; Count = 1 },
    @{ Name = 'marker on stdout only / no retry'; Responses = @((New-Response 35 'CRYPT_E_REVOCATION_OFFLINE' 'generic TLS')); Expected = 'FAIL_TLS'; Count = 1 },
    @{ Name = 'retry still fails / no third attempt'; Responses = @((New-Response 35 '' 'CRYPT_E_REVOCATION_OFFLINE'), (New-Response 35 '' 'CRYPT_E_REVOCATION_OFFLINE')); Expected = 'FAIL_TLS'; Count = 2 },
    @{ Name = 'HTTP error / no retry'; Responses = @((New-Response 22 '' 'HTTP 403')); Expected = 'FAIL_HTTP'; Count = 1 },
    @{ Name = 'invalid IP / no retry'; Responses = @((New-Response 0 '<html>not an IP</html>')); Expected = 'FAIL_HTTP'; Count = 1 }
)
foreach ($case in $cases) {
    $script:responses = [Collections.Generic.Queue[object]]::new()
    foreach ($response in $case.Responses) { $script:responses.Enqueue($response) }
    $script:calls = [Collections.Generic.List[object]]::new()
    $result = Invoke-OneTunnelHttpsProbe -Url 'https://example.com' -Timeout 25 -ExpectIpAddress
    Assert-True ($result.Classification -eq $case.Expected) "$($case.Name) classification"
    Assert-True ($calls.Count -eq $case.Count -and $responses.Count -eq 0) "$($case.Name) attempt count"
    Assert-True ($result.Attempts.Count -eq $case.Count -and $result.ExitCode -eq $result.Attempts[-1].ExitCode -and $result.StdOut -ceq $result.Attempts[-1].StdOut -and $result.StdErr -ceq $result.Attempts[-1].StdErr) "$($case.Name) separate stream capture"
    for ($i = 0; $i -lt $calls.Count; $i++) {
        Assert-True ($calls[$i][0] -eq '-q') 'curlrc ignored'
        Assert-True (-not ($calls[$i] | Where-Object { $_ -in @('-k', '--insecure', '--ssl-no-revoke') })) 'no forbidden TLS flags'
        Assert-True (($calls[$i] -contains '--ssl-revoke-best-effort') -eq ($i -eq 1)) 'best-effort only on retry'
    }
    Assert-True (($result.Warning -eq 'SCHANNEL_REVOCATION_OFFLINE') -eq ($case.Count -eq 2)) 'revocation warning'
    Write-Output "PASS: $($case.Name)"
}
$script:responses.Enqueue((New-Response 0 '203.0.113.3'))
$script:calls.Clear()
$result = Invoke-OneTunnelHttpsProbe -Url 'https://example.com' -Proxy 'socks5h://127.0.0.1:12345' -ExpectIpAddress
$proxyIndex = [Array]::IndexOf($calls[0], '--proxy')
$noProxyIndex = [Array]::IndexOf($calls[0], '--noproxy')
Assert-True ($calls[0][$proxyIndex + 1] -eq 'socks5h://127.0.0.1:12345' -and $calls[0][$noProxyIndex + 1] -eq '') 'explicit SOCKS / no inherited bypass'
Write-Output 'PASS: HTTPS classifier and retry policy tested without Internet'
