#Requires -Version 5.1
<#
.SYNOPSIS
  Audit a separately built and signed OneTunnel WFP driver package.
.DESCRIPTION
  Does not install anything unless -Install is explicitly supplied.
  OneTunnel does NOT currently ship a WFP callout driver; the script is
  intended only for a future independently reviewed and certified release.
#>
[CmdletBinding(SupportsShouldProcess = $true, ConfirmImpact = 'High')]
param(
    [Parameter(Mandatory = $true)]
    [ValidateScript({ Test-Path -LiteralPath $_ -PathType Leaf })]
    [string] $InfPath,

    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[a-fA-F0-9]{64}$')]
    [string] $ExpectedInfSha256,

    [switch] $Install
)

$ErrorActionPreference = 'Stop'
$inf = (Resolve-Path -LiteralPath $InfPath).Path
if ([IO.Path]::GetFileName($inf) -ine 'OneTunnelWfp.inf') {
    throw 'Only the separately certified OneTunnelWfp.inf driver package is accepted.'
}
$infDir = Split-Path -Parent $inf
$actualHash = (Get-FileHash -LiteralPath $inf -Algorithm SHA256).Hash
if ($actualHash -ine $ExpectedInfSha256) {
    throw 'INF SHA-256 does not match the independently verified expected digest.'
}
$catalog = Join-Path $infDir 'OneTunnelWfp.cat'
$binary = Join-Path $infDir 'OneTunnelWfp.sys'
foreach ($path in @($catalog, $binary)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Required signed driver artifact missing: $path"
    }
    $signature = Get-AuthenticodeSignature -LiteralPath $path
    if ($signature.Status -ne 'Valid' -or -not $signature.SignerCertificate) {
        throw "Invalid or missing Authenticode signature: $path"
    }
}
$service = Get-Service -Name BFE -ErrorAction SilentlyContinue
if (-not $service -or $service.Status -ne 'Running') {
    throw 'Windows Base Filtering Engine (BFE) is not running.'
}
Write-Host 'BFE is running; driver package files and signatures were checked.'
Write-Host 'No WFP driver is currently included in OneTunnel. Validate publisher, chain, INF/CAT hashes and Microsoft attestation externally before approving.'
if (-not $Install) {
    Write-Host 'AUDIT ONLY. No driver installation or network change performed.'
    return
}
$principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Administrator rights are required for driver installation.'
}
if ($PSCmdlet.ShouldProcess($inf, 'Install signed OneTunnel WFP driver package via PnPUtil')) {
    & "$env:SystemRoot\System32\pnputil.exe" /add-driver $inf /install
    if ($LASTEXITCODE -ne 0) {
        throw "PnPUtil driver installation failed with exit code $LASTEXITCODE."
    }
    Write-Host 'Driver package submitted to Windows. WFP redirection remains DISABLED until the independently built service registers and verifies its callouts.'
}
