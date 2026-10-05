[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$script:failed = $false

function Write-Check {
    param(
        [string]$Name,
        [ValidateSet('PASS', 'WARN', 'FAIL')][string]$Status,
        [string]$Detail
    )

    if ($Status -eq 'FAIL') { $script:failed = $true }
    Write-Output ("{0} {1}: {2}" -f $Status, $Name, $Detail)
}

try {
    $os = Get-CimInstance -ClassName Win32_OperatingSystem
    $caption = [string]$os.Caption
    $isSupportedWindows = $caption -match 'Windows 10|Windows 11'
    Write-Check 'Windows version' ($(if ($isSupportedWindows) { 'PASS' } else { 'FAIL' })) "$caption (build $($os.BuildNumber))"
} catch {
    Write-Check 'Windows version' 'FAIL' "Unable to query Win32_OperatingSystem: $($_.Exception.Message)"
}

$is64 = [Environment]::Is64BitOperatingSystem
Write-Check 'Architecture' ($(if ($is64) { 'PASS' } else { 'FAIL' })) ([Environment]::OSVersion.VersionString + "; 64-bit=$is64")

$psVersion = $PSVersionTable.PSVersion.ToString()
Write-Check 'PowerShell' 'PASS' $psVersion

foreach ($tool in @('git', 'node', 'npm', 'rustc', 'cargo')) {
    $command = Get-Command $tool -ErrorAction SilentlyContinue
    if ($null -eq $command) {
        Write-Check $tool 'WARN' 'not found on PATH'
        continue
    }
    try {
        $version = (& $command.Source --version 2>&1 | Select-Object -First 1).ToString().Trim()
        Write-Check $tool 'PASS' $version
    } catch {
        Write-Check $tool 'WARN' "found at $($command.Source), version query failed"
    }
}

$msbuild = Get-Command msbuild -ErrorAction SilentlyContinue
$vswhere = Get-Command vswhere -ErrorAction SilentlyContinue
$vsInstall = @(
    'C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe',
    'C:\Program Files\Microsoft Visual Studio\Installer\vswhere.exe'
) | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
if ($msbuild) {
    Write-Check 'Visual Studio Build Tools' 'PASS' "msbuild at $($msbuild.Source)"
} elseif ($vswhere -or $vsInstall) {
    $path = if ($vswhere) { $vswhere.Source } else { $vsInstall }
    Write-Check 'Visual Studio Build Tools' 'WARN' "Visual Studio installer detected at $path; msbuild not on PATH"
} else {
    Write-Check 'Visual Studio Build Tools' 'WARN' 'not detectable; not installed by this script'
}

$webViewKeys = @(
    'HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients',
    'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients',
    'HKCU:\SOFTWARE\Microsoft\EdgeUpdate\Clients'
)
$webView2 = $webViewKeys | Where-Object {
    Test-Path -LiteralPath $_ -and (Get-ChildItem -LiteralPath $_ -ErrorAction SilentlyContinue | Where-Object { $_.GetValue('name') -match 'WebView2' })
} | Select-Object -First 1
if ($webView2) {
    Write-Check 'WebView2' 'PASS' "detected under $webView2"
} else {
    Write-Check 'WebView2' 'WARN' 'not detectable; not installed by this script'
}

$principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
$isAdmin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
Write-Check 'Administrator status' ($(if ($isAdmin) { 'PASS' } else { 'WARN' })) "administrator=$isAdmin; M0 does not require elevation"

if ($script:failed) { exit 1 }
exit 0
