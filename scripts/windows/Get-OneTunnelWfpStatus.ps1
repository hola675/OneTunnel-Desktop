#Requires -Version 5.1
[CmdletBinding()]
param(
    [ValidatePattern('^[a-zA-Z0-9_-]{1,80}$')]
    [string] $DriverServiceName = 'OneTunnelWfp',
    [switch] $AsJson
)
$ErrorActionPreference = 'Stop'
if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw 'Windows is required.'
}
$bfe = Get-Service -Name BFE -ErrorAction SilentlyContinue
$driver = Get-Service -Name $DriverServiceName -ErrorAction SilentlyContinue
$status = [ordered]@{
    bfeRunning = [bool]($bfe -and $bfe.Status -eq 'Running')
    driverServicePresent = [bool]$driver
    driverServiceRunning = [bool]($driver -and $driver.Status -eq 'Running')
    interceptionConfirmed = $false
    explanation = 'Read-only service checks cannot prove WFP callout or network interception.'
}
if ($AsJson) { [pscustomobject]$status | ConvertTo-Json }
else { [pscustomobject]$status | Format-List }
