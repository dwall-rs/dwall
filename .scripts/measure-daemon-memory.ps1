<#
.SYNOPSIS
    Measures the daemon's resident memory footprint under a reproducible workload.

.DESCRIPTION
    Starts a daemon with a scratch configuration, samples its private bytes, working
    set, handle count and thread count, stops it, and prints a plateau summary. The
    documentation in docs/technical-comparison.md quotes numbers produced by this
    script, so it is the "say how the number was obtained" half of those claims.

    Two modes, because the footprint depends on which of the heavy WinRT paths the
    process has used — not on how long it has run:

      steady  Random mode with the lock screen enabled, like a real installation.
              The wallpaper is normally already the one the sun selects, so no cycle
              changes anything: this is the resting footprint.
      apply   Fixed mode forcing -Theme on every monitor, so the first cycle really
              applies a wallpaper: this is the footprint after the WinRT wallpaper
              and lock-screen paths have run.

    The interval is 1 second instead of the default 15 or 600: a cycle is cheap, and
    the question is whether anything grows per cycle. Five minutes at one cycle per
    second covers ~300 cycles — the same work as about two days at the 600-second
    cadence of a real configuration — while still doing real work (monitor
    enumeration, theme frame selection, the COM wallpaper path, the lock-screen
    compare). Colour-scheme switching is forced off so a measurement cannot flip the
    machine's light/dark theme; the user's own configuration file is never touched.

.PARAMETER Mode
    `steady` (default) or `apply`; see the description.

.PARAMETER Binaries
    Executables to measure, one run each. Defaults to the release build in this
    repository and, when it exists, the installed copy under Program Files — so the
    released build and the working tree can be compared in one invocation.

.PARAMETER Samples
    How many samples to take per binary (default 20).

.PARAMETER SampleSeconds
    Seconds between samples (default 15, i.e. five minutes per binary).

.PARAMETER Theme
    Theme forced by `-Mode apply`; defaults to the first theme directory found.

.PARAMETER OutputDirectory
    Where the raw samples are written (default: `%TEMP%\dwall_memtest`).

.EXAMPLE
    ./.scripts/measure-daemon-memory.ps1

.EXAMPLE
    ./.scripts/measure-daemon-memory.ps1 -Mode apply -Samples 40 -SampleSeconds 30
#>
[CmdletBinding()]
param(
    [ValidateSet('steady', 'apply')]
    [string]$Mode = 'steady',

    [switch]$RefreshPositionEveryCycle,

    [string[]]$Binaries = @(),

    [int]$Samples = 20,

    [int]$SampleSeconds = 15,

    [string]$Theme,

    [string]$OutputDirectory = (Join-Path $env:TEMP 'dwall_memtest')
)

$ErrorActionPreference = 'Stop'

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$userConfig = Join-Path $env:APPDATA 'dwall\config.toml'
$daemonLog = Join-Path $env:LOCALAPPDATA 'com.thep0y.dwall\log\dwall.log'

if ($Binaries.Count -eq 0) {
    $Binaries = @(Join-Path $repositoryRoot 'target/release/dwall.exe')
    $installed = Join-Path ${env:ProgramFiles} 'Dwall Settings\dwall.exe'
    if (Test-Path -LiteralPath $installed -PathType Leaf) {
        $Binaries += $installed
    }
}

foreach ($binary in $Binaries) {
    if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) {
        throw "Daemon binary not found: $binary. Build it first (bun run build:daemon)."
    }
}

# The theme directories come from the real configuration: a measurement that pointed
# at an empty directory would exercise nothing.
function Get-ConfigPath {
    param([string]$Key, [string]$Default)

    if (-not (Test-Path -LiteralPath $userConfig -PathType Leaf)) {
        return $Default
    }

    $match = Select-String -LiteralPath $userConfig -Pattern "^$Key\s*=\s*'([^']+)'" -List
    if ($match) { return $match.Matches[0].Groups[1].Value }

    $match = Select-String -LiteralPath $userConfig -Pattern "^$Key\s*=\s*`"([^`"]+)`"" -List
    if ($match) { return $match.Matches[0].Groups[1].Value }

    return $Default
}

$themesDirectory = Get-ConfigPath -Key 'themes_directory' -Default (Join-Path $env:APPDATA 'dwall\themes')
$customizedDirectory = Get-ConfigPath -Key 'customized_themes_directory' -Default (Join-Path $env:APPDATA 'dwall\customize')

if (-not $Theme) {
    $Theme = Get-ChildItem -LiteralPath $themesDirectory -Directory -ErrorAction SilentlyContinue |
        Where-Object { Test-Path -LiteralPath (Join-Path $_.FullName 'solar.json') } |
        Select-Object -First 1 -ExpandProperty Name
}
if (-not $Theme) {
    throw "No theme found under $themesDirectory; pass -Theme explicitly."
}

New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
$configPath = Join-Path $OutputDirectory "config-$Mode.toml"

$wallpaperMode = if ($Mode -eq 'apply') {
    "[wallpaper_mode]`nmode = `"fixed`"`nmonitor_specific_wallpapers = `"$Theme`""
} else {
    "[wallpaper_mode]`nmode = `"random`"`npool = [`"$Theme`"]"
}

$refreshPosition = if ($RefreshPositionEveryCycle) { 'true' } else { 'false' }

@"
title_bar_color_follows_windows_theme = false
image_format = "jpeg"
auto_detect_color_scheme = false
lock_screen_wallpaper_enabled = true
themes_directory = '$themesDirectory'
customized_themes_directory = '$customizedDirectory'
interval = 1

[position_source]
type = "AUTOMATIC"
update_on_each_calculation = $refreshPosition
cache_minutes = 30

$wallpaperMode
"@ | Set-Content -LiteralPath $configPath -Encoding ascii

function Measure-Daemon {
    param([string]$Binary, [string]$Label)

    Write-Host "measuring $Label ($Binary)"
    $startedAt = (Get-Date).ToString('yyyy-MM-ddTHH:mm:ss')
    $process = Start-Process -FilePath $Binary -ArgumentList '--config', $configPath -PassThru
    Start-Sleep -Seconds 3

    $rows = @()
    for ($i = 0; $i -lt $Samples; $i++) {
        $running = Get-Process -Id $process.Id -ErrorAction SilentlyContinue
        if (-not $running) {
            Write-Warning "$Label exited after $i samples"
            break
        }

        # Not `$samples`: PowerShell variable names are case-insensitive, so a local
        # `$samples` would overwrite the `$Samples` parameter this loop counts to.
        $rows += [pscustomobject]@{
            Label        = $Label
            Sample       = $i
            PrivateMB    = [math]::Round($running.PrivateMemorySize64 / 1MB, 2)
            WorkingSetMB = [math]::Round($running.WorkingSet64 / 1MB, 2)
            Handles      = $running.HandleCount
            Threads      = $running.Threads.Count
        }

        Start-Sleep -Seconds $SampleSeconds
    }

    if (Get-Process -Id $process.Id -ErrorAction SilentlyContinue) {
        Stop-Process -Id $process.Id -Force
        $process.WaitForExit(5000) | Out-Null
    }
    Start-Sleep -Seconds 1

    # What the run actually did, read back from the daemon's own log. Without this the
    # numbers cannot be attributed: `-Mode apply` changes nothing when the wallpaper
    # on screen is already `-Theme`, and the two states differ by several megabytes.
    $applied = 0
    $lockScreen = 0
    $geolocations = 0
    if (Test-Path -LiteralPath $daemonLog -PathType Leaf) {
        $lines = Get-Content -LiteralPath $daemonLog |
            Where-Object { $_ -match '^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})' -and $matches[1] -ge $startedAt }
        $applied = ($lines | Select-String -Pattern 'Applied wallpaper' -SimpleMatch | Measure-Object).Count
        $lockScreen = ($lines | Select-String -Pattern 'lock screen' -SimpleMatch | Measure-Object).Count
        # The altitude warning is the only per-reading line that survives the release
        # log level, and it only fires when the device reports altitude 0 — a cross-check
        # for the position refreshes, not a general geolocation counter.
        $geolocations = ($lines | Select-String -Pattern 'barometric pressure sensor' -SimpleMatch | Measure-Object).Count
    }

    # Every row carries it: the summary groups the rows after the plateau cut, so a
    # property only on the first sample would be missing exactly there.
    foreach ($row in $rows) {
        $row | Add-Member -NotePropertyName AppliedWallpapers -NotePropertyValue $applied
        $row | Add-Member -NotePropertyName LockScreenLines -NotePropertyValue $lockScreen
        $row | Add-Member -NotePropertyName GeolocationWarnings -NotePropertyValue $geolocations
    }

    return $rows
}

$all = @()
foreach ($binary in $Binaries) {
    $label = [System.IO.Path]::GetFileName((Split-Path -Parent $binary)) + '/' + [System.IO.Path]::GetFileName($binary)
    $all += Measure-Daemon -Binary $binary -Label $label
}

$samplesPath = Join-Path $OutputDirectory "samples-$Mode.txt"
$all | Export-Csv -LiteralPath $samplesPath -NoTypeInformation -Encoding utf8

# The plateau is the last third of the samples: the first cycles are what loads the
# WinRT/COM machinery, so including them would describe startup rather than the state
# the process settles into.
$plateauStart = [math]::Floor($Samples * 2 / 3)
Write-Host ''
Write-Host 'plateau (last third of the samples):'
$all |
    Where-Object { $_.Sample -ge $plateauStart } |
    Group-Object Label |
    ForEach-Object {
        $private = $_.Group.PrivateMB | Measure-Object -Minimum -Maximum -Average
        $handles = $_.Group.Handles | Measure-Object -Minimum -Maximum
        $threads = $_.Group.Threads | Measure-Object -Minimum -Maximum
        $applied = $_.Group[0].AppliedWallpapers
        $lockScreen = $_.Group[0].LockScreenLines
        $geolocations = $_.Group[0].GeolocationWarnings
        '{0,-28} private {1:N2}–{2:N2} MB (avg {3:N2})  handles {4}–{5}  threads {6}–{7}  working set {8:N1} MB  applied {9}  lock-screen lines {10}  geolocation warnings {11}' -f `
            $_.Name, $private.Minimum, $private.Maximum, $private.Average, `
            $handles.Minimum, $handles.Maximum, $threads.Minimum, $threads.Maximum, `
            ($_.Group.WorkingSetMB | Measure-Object -Average).Average, $applied, $lockScreen, $geolocations
    }

Write-Host ''
Write-Host 'The states are not interchangeable: a run that applied 0 wallpapers measures the'
Write-Host 'resting footprint, one that applied some measures the footprint afterwards.'
Write-Host "raw samples: $samplesPath"

