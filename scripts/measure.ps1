param(
    [Parameter(Mandatory)][int]$ProcessId,
    [Parameter(Mandatory)][ValidateSet('idle', 'scroll')][string]$Phase,
    [ValidateRange(1, 300)][int]$Seconds = 30,
    [Parameter(Mandatory)][string]$OutputPath
)

$ErrorActionPreference = 'Stop'

function Get-ProbeSnapshot {
    $processTree = @(Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId)
    $probeIds = [System.Collections.Generic.HashSet[int]]::new()
    [void]$probeIds.Add($ProcessId)
    do {
        $added = 0
        foreach ($entry in $processTree) {
            if ($probeIds.Contains([int]$entry.ParentProcessId) -and $probeIds.Add([int]$entry.ProcessId)) {
                $added++
            }
        }
    } while ($added -gt 0)

    $processes = @(Get-Process -Id @($probeIds) -ErrorAction SilentlyContinue)
    if ($ProcessId -notin $processes.Id) {
        throw "Probe process $ProcessId is not running."
    }
    $cpu = @{}
    $privateBytes = 0L
    $workingSet = 0L
    foreach ($process in $processes) {
        $cpu[$process.Id] = $process.TotalProcessorTime.TotalSeconds
        $privateBytes += $process.PrivateMemorySize64
        $workingSet += $process.WorkingSet64
    }
    $gpuMemory = @(Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUProcessMemory |
        Where-Object { $_.Name -match '^pid_(\d+)_' -and $probeIds.Contains([int]$Matches[1]) })
    $gpuDedicated = if ($gpuMemory.Count) { ($gpuMemory | Measure-Object DedicatedUsage -Sum).Sum } else { $null }
    $gpuShared = if ($gpuMemory.Count) { ($gpuMemory | Measure-Object SharedUsage -Sum).Sum } else { $null }
    [pscustomobject]@{
        Cpu = $cpu; PrivateBytes = $privateBytes; WorkingSet = $workingSet
        GpuDedicated = $gpuDedicated; GpuShared = $gpuShared
    }
}

$previous = Get-ProbeSnapshot
$timer = [System.Diagnostics.Stopwatch]::StartNew()
$previousTime = 0.0
$samples = [System.Collections.Generic.List[object]]::new()
while ($timer.Elapsed.TotalSeconds -lt $Seconds) {
    Start-Sleep -Milliseconds 1000
    $current = Get-ProbeSnapshot
    $now = $timer.Elapsed.TotalSeconds
    $changed = $current.Cpu.Count -ne $previous.Cpu.Count
    $cpuDelta = 0.0
    foreach ($id in $current.Cpu.Keys) {
        if (-not $previous.Cpu.ContainsKey($id)) {
            $changed = $true
        } else {
            $cpuDelta += $current.Cpu[$id] - $previous.Cpu[$id]
        }
    }
    $cpuPercent = if ($changed) { $null } else { [Math]::Round(100 * $cpuDelta / ($now - $previousTime), 3) }
    $samples.Add([pscustomobject]@{
        ElapsedSeconds = [Math]::Round($now, 3)
        Phase = $Phase
        ProcessCount = $current.Cpu.Count
        PrivateMiB = [Math]::Round($current.PrivateBytes / 1MB, 3)
        WorkingSetMiB = [Math]::Round($current.WorkingSet / 1MB, 3)
        GpuDedicatedMiB = if ($null -ne $current.GpuDedicated) { [Math]::Round($current.GpuDedicated / 1MB, 3) } else { $null }
        GpuSharedMiB = if ($null -ne $current.GpuShared) { [Math]::Round($current.GpuShared / 1MB, 3) } else { $null }
        CpuOneCorePercent = $cpuPercent
        ProcessSetChanged = $changed
    })
    $previous = $current
    $previousTime = $now
}
$samples | Export-Csv -LiteralPath $OutputPath -NoTypeInformation -Encoding utf8
Write-Output "Saved $($samples.Count) samples to $OutputPath"
