# Native CLI capture shared by Windows PowerShell 5.1 and PowerShell 7.
function ConvertTo-NativeArgument {
    param([AllowEmptyString()][string]$Value)
    # Windows CommandLineToArgvW/CRT quoting, including quotes and trailing slashes.
    $escaped = [regex]::Replace($Value, '(\\*)"', '$1$1\"')
    $escaped = [regex]::Replace($escaped, '(\\+)$', '$1$1')
    return '"' + $escaped + '"'
}

function Start-NativeProcess {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$FilePath,
        [string[]]$Arguments = @()
    )
    $process = New-Object System.Diagnostics.Process
    $process.StartInfo.FileName = $FilePath
    $process.StartInfo.Arguments = (($Arguments | ForEach-Object { ConvertTo-NativeArgument $_ }) -join ' ')
    $process.StartInfo.UseShellExecute = $false
    $process.StartInfo.RedirectStandardOutput = $true
    $process.StartInfo.RedirectStandardError = $true
    $process.StartInfo.CreateNoWindow = $true
    try {
        [void]$process.Start()
        # Drain both pipes concurrently to avoid blocking when either pipe fills.
        [pscustomobject]@{
            Process = $process
            StdOutTask = $process.StandardOutput.ReadToEndAsync()
            StdErrTask = $process.StandardError.ReadToEndAsync()
        }
    } catch {
        $process.Dispose()
        throw
    }
}

function Stop-NativeProcess {
    param([Parameter(Mandatory)]$Handle)
    # Only the Process object created by Start-NativeProcess is ever stopped.
    if (-not $Handle.Process.HasExited) { $Handle.Process.Kill() }
    if (-not $Handle.Process.WaitForExit(5000)) { throw 'Native process cleanup timed out.' }
}

function Invoke-NativeProcess {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$FilePath,
        [string[]]$Arguments = @(),
        [ValidateRange(1, 3600)][int]$TimeoutSeconds = 60
    )
    $handle = Start-NativeProcess -FilePath $FilePath -Arguments $Arguments
    try {
        if (-not $handle.Process.WaitForExit($TimeoutSeconds * 1000)) {
            Stop-NativeProcess $handle
            throw "Native process timed out after $TimeoutSeconds seconds."
        }
        $stdout = $handle.StdOutTask.GetAwaiter().GetResult()
        $stderr = $handle.StdErrTask.GetAwaiter().GetResult()
        [pscustomobject]@{
            ExitCode = $handle.Process.ExitCode
            StdOut = $stdout
            StdErr = $stderr
            # Stream ordering is not preserved; capability probes search both streams.
            CombinedOutput = $stdout + [Environment]::NewLine + $stderr
        }
    } finally {
        try { Stop-NativeProcess $handle } finally { $handle.Process.Dispose() }
    }
}
