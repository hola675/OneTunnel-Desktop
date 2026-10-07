[CmdletBinding()]
param(
    [ValidateSet('ams', 'sgp', 'lax')][string]$Location = 'lax',
    [ValidateRange(1, 2)][int]$ServerIndex = 1,
    [ValidateRange(0, 65535)][int]$SocksPort = 0
)
$ErrorActionPreference = 'Stop'
$contract = Get-Content -LiteralPath (Join-Path $PSScriptRoot '..\tests\fixtures\1vpn\free-contract.json') -Raw | ConvertFrom-Json
$selected = $contract.locations | Where-Object cityCode -eq $Location
$server = $selected.servers[$ServerIndex - 1]
if ($SocksPort -eq 0) {
    $reservation = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    try { $reservation.Start(); $SocksPort = $reservation.LocalEndpoint.Port }
    finally { $reservation.Stop() }
}
$config = [ordered]@{
    log = @{ loglevel = 'warning' }
    inbounds = @(@{ listen = '127.0.0.1'; port = $SocksPort; protocol = 'socks'; settings = @{ udp = $false; auth = 'noauth' } })
    outbounds = @(@{
        protocol = 'vless'; tag = 'proxy'
        settings = @{ vnext = @(@{
            address = $server.host; port = 443
            users = @(@{ id = $contract.uuid; flow = 'xtls-rprx-vision'; encryption = 'none' })
        }) }
        streamSettings = @{
            network = 'tcp'; security = 'reality'
            realitySettings = @{ fingerprint = 'chrome'; serverName = $server.realityServerName; publicKey = $contract.publicKey; shortId = $contract.shortId; spiderX = '' }
        }
    })
}
$directory = Join-Path ([IO.Path]::GetTempPath()) 'OneTunnel\poc'
[void][IO.Directory]::CreateDirectory($directory)
$path = Join-Path $directory ('xray-direct-' + [guid]::NewGuid().ToString('N') + '.json')
$config | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $path -Encoding utf8
[pscustomobject]@{ Path = $path; SocksPort = $SocksPort; Location = $selected.city; Server = $server.host; RealityServerName = $server.realityServerName }
