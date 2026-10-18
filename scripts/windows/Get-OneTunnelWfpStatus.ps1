#Requires -Version 5.1
<#
.SYNOPSIS
  Read-only WFP / OneTunnel driver and network readiness diagnostic.
.DESCRIPTION
  Inspects Windows BFE and signed OneTunnel driver service without modifying Windows.
  This script does not install, start, stop, or configure any driver.
#>
[CmdletBinding()]
param(
    [string] $DriverServiceName = 'OneTunnelWfp',
    [switch] $AsJson
)
$ErrorActionPreference = 'Stop'
$results = [ordered]@{
    timestampUtc = [DateTime]::UtcNow.ToString('o')
    platform = [Environment]::OSVersion.Platform.ToString()
    bfeRunning = $false
    driverServicePresent = $false
    driverServiceRunning = $false
    driverBinaryPath = $null
    driverSignatureValid = $false
    interceptionConfirmed = $false
    note = 'Service inspection cannot prove WFP callout registration or real traffic redirection.'
}
if (-not [Environment]::OSVersion.Platform.ToString().StartsWith('Win')) {
    throw 'Windows is required.'
}
$bfe = Get-Service -Name BFE -ErrorAction SilentlyContinue
$results.bfeRunning = [bool]($bfe -and $bfe.Status -eq 'Running')
$svc = Get-CimInstance Win32_SystemDriver -Filter "Name='$($DriverServiceName.Replace("'","''"))'" -ErrorAction SilentlyContinue
if ($svc) {
    $results.driverServicePresent = $true
    $results.driverServiceRunning = ($svc.State -eq 'Running')
    $binary = [Environment]::ExpandEnvironmentVariables($svc.PathName.Trim('"'))
    if ($binary.StartsWith('\SystemRoot\',[StringComparison]::OrdinalIgnoreCase)) {
        $binary = Join-Path $env:SystemRoot $binary.Substring(12)
    }
    if (Test-Path -LiteralPath $binary -PathType Leaf) {
        $results.driverBinaryPath = $binary
        $sig = Get-AuthenticodeSignature -LiteralPath $binary
        $results.driverSignatureValid = $sig.Status -eq 'Valid'
    }
}
if ($AsJson) { [pscustomobject]$results | ConvertTo-Json -Depth 4 }
else { [pscustomobject]$results | Format-List }
