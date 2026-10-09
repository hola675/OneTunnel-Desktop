$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
$contract = Get-Content -LiteralPath (Join-Path $root 'tests\fixtures\1vpn\free-contract.json') -Raw | ConvertFrom-Json
$constants = Get-Content -LiteralPath (Join-Path $root $contract.sources[0]) -Raw
$locations = Get-Content -LiteralPath (Join-Path $root $contract.sources[1]) -Raw
function Assert-True([bool]$Condition, [string]$Message) { if (-not $Condition) { throw "FAIL: $Message" } }
foreach ($pair in @(@('uuid', 'FREE_UUID'), @('publicKey', 'FREE_PUBLIC_KEY'), @('shortId', 'FREE_SHORT_ID'))) {
    $match = [regex]::Match($constants, $pair[1] + '\s*=\s*"([^"]+)"')
    Assert-True ($match.Success -and $match.Groups[1].Value -ceq $contract.($pair[0])) ('Free public ' + $pair[0] + ' provenance')
}
$sourceServers = @([regex]::Matches($locations, 'host\s*=\s*"([^"]+)"\s*,\s*realityServerName\s*=\s*"([^"]+)"'))
$fixtureServers = @($contract.locations.servers)
Assert-True ($sourceServers.Count -eq 6 -and $fixtureServers.Count -eq 6) 'six Free servers only'
for ($i = 0; $i -lt 6; $i++) {
    Assert-True ($fixtureServers[$i].host -ceq $sourceServers[$i].Groups[1].Value -and $fixtureServers[$i].realityServerName -ceq $sourceServers[$i].Groups[2].Value) 'Free server/SNI pairing'
}
Write-Output 'PASS: public Free contract matches the untouched 1VPN snapshot'
