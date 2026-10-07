[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot '..\scripts\lib\NativeProcess.ps1')
$shellPath = (Get-Process -Id $PID).Path

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw "FAIL: $Message" }
}
function Invoke-TestChild([string]$Code) {
    $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($Code))
    Invoke-NativeProcess $shellPath @('-NoProfile', '-EncodedCommand', $encoded)
}
foreach ($case in @(
    @{ Name = 'stdout only'; Code = "[Console]::Out.Write('stdout'); exit 0"; Out = 'stdout'; Err = ''; Exit = 0 },
    @{ Name = 'stderr only'; Code = "[Console]::Error.Write('stderr'); exit 0"; Out = ''; Err = 'stderr'; Exit = 0 },
    @{ Name = 'both streams / nonzero'; Code = "[Console]::Out.Write('stdout'); [Console]::Error.Write('stderr'); exit 7"; Out = 'stdout'; Err = 'stderr'; Exit = 7 }
)) {
    $result = Invoke-TestChild $case.Code
    Assert-True ($result.ExitCode -eq $case.Exit) "$($case.Name) exit code"
    Assert-True ($result.StdOut -eq $case.Out) "$($case.Name) stdout"
    Assert-True ($result.StdErr -eq $case.Err) "$($case.Name) stderr"
    Assert-True ($result.CombinedOutput.Contains($case.Out) -and $result.CombinedOutput.Contains($case.Err)) "$($case.Name) combined output"
    Write-Output "PASS: $($case.Name)"
}
$result = Invoke-TestChild "[Console]::Out.Write(('o' * 131072)); [Console]::Error.Write(('e' * 131072)); exit 0"
Assert-True ($result.StdOut.Length -eq 131072 -and $result.StdErr.Length -eq 131072) 'concurrent pipe draining'
Write-Output 'PASS: large stdout and stderr without deadlock'

$tempPath = Join-Path ([IO.Path]::GetTempPath()) ('onetunnel-argv-' + [guid]::NewGuid() + '.ps1')
try {
    # Check the native argument vector. powershell.exe 5.1 drops empty values
    # when binding -File script $args, independently of native quoting.
    '[Console]::Out.Write((ConvertTo-Json -InputObject @([Environment]::GetCommandLineArgs() | Select-Object -Skip 4) -Compress))' | Set-Content -LiteralPath $tempPath -Encoding UTF8
    $values = @('', 'two words', 'quote"inside', 'C:\trailing space\', 'slashes\\"quote', '& literal; $value')
    $result = Invoke-NativeProcess $shellPath (@('-NoProfile', '-File', $tempPath) + $values)
    $actual = ConvertFrom-Json -InputObject $result.StdOut
    Assert-True ($result.ExitCode -eq 0 -and $actual.Count -eq $values.Count) 'argument count'
    for ($i = 0; $i -lt $values.Count; $i++) { Assert-True ($actual[$i] -ceq $values[$i]) "argument $i roundtrip" }
    Write-Output 'PASS: empty/space/quote/backslash/literal argument roundtrip'
} finally { Remove-Item -LiteralPath $tempPath }
Write-Output "PASS: native helper regression on PowerShell $($PSVersionTable.PSVersion)"
