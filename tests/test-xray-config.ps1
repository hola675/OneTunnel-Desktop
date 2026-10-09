#Requires -Version 7.0
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot '..\scripts\lib\NativeProcess.ps1')
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$contract = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'fixtures\1vpn\free-contract.json') -Raw | ConvertFrom-Json
& (Join-Path $PSScriptRoot 'contracts\1vpn\test-free-contract.ps1')
function Assert-True([bool]$Condition, [string]$Message) { if (-not $Condition) { throw "FAIL: $Message" } }
foreach ($location in $contract.locations) {
    for ($index = 1; $index -le 2; $index++) {
        $generated = & (Join-Path $root 'scripts\new-xray-config.ps1') -Location $location.cityCode -ServerIndex $index
        try {
            $config = Get-Content -LiteralPath $generated.Path -Raw | ConvertFrom-Json
            Assert-True ($generated.Path.StartsWith((Join-Path ([IO.Path]::GetTempPath()) 'OneTunnel\xray'), [StringComparison]::OrdinalIgnoreCase)) 'config stays under TEMP'
            Assert-True ($config.inbounds.Count -eq 1 -and $config.inbounds[0].listen -eq '127.0.0.1' -and $config.inbounds[0].protocol -eq 'socks' -and -not $config.inbounds[0].settings.udp) 'SOCKS-only loopback / no UDP'
            Assert-True ($config.outbounds.Count -eq 1 -and $config.outbounds[0].protocol -eq 'vless' -and -not $config.dns -and -not $config.routing) 'minimal VLESS-only outbound / no custom DNS or routing'
            $outbound = $config.outbounds[0]
            Assert-True ($outbound.settings.vnext[0].address -eq $location.servers[$index - 1].host -and $outbound.settings.vnext[0].port -eq 443) 'exact Free destination'
            Assert-True ($outbound.settings.vnext[0].users[0].id -eq $contract.uuid -and $outbound.settings.vnext[0].users[0].flow -eq 'xtls-rprx-vision') 'Free UUID and Vision'
            Assert-True ($outbound.streamSettings.security -eq 'reality' -and $outbound.streamSettings.network -eq 'tcp' -and $outbound.streamSettings.realitySettings.fingerprint -eq 'chrome') 'Reality/TCP/chrome'
            Assert-True ($outbound.streamSettings.realitySettings.serverName -eq $location.servers[$index - 1].realityServerName -and $outbound.streamSettings.realitySettings.publicKey -eq $contract.publicKey -and $outbound.streamSettings.realitySettings.shortId -eq $contract.shortId) 'exact Free Reality parameters'
            $validation = Invoke-NativeProcess (Join-Path $root 'runtime\bin\xray.exe') @('run', '-test', '-config', $generated.Path)
            Assert-True ($validation.ExitCode -eq 0) 'offline Xray validation'
            Write-Output "PASS: $($location.cityCode) server $index provenance and Xray config"
        } finally { Remove-Item -LiteralPath $generated.Path }
    }
}
Write-Output 'PASS: Free fixture and Direct configs; no network processes started'
