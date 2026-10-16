<#
.SYNOPSIS
    Packages the standalone (headless) daemon into a release archive.

.DESCRIPTION
    Produces Dwall_<Version>_x64-daemon.zip containing the daemon binary, its
    licence, the headless and custom-theme documentation and a checksum list.

    The release workflows call this after the Tauri build, which produces
    target/release/dwall.exe through `bun run build:daemon`; it can be run locally
    the same way. The daemon needs no WebView2 runtime, so this archive is the
    "just the engine" download for portable, headless and managed installs.

.EXAMPLE
    ./.scripts/package-daemon.ps1 -Version 0.2.5
    ./.scripts/package-daemon.ps1 -Version nightly
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Version,

    [string]$BinaryPath = "target/release/dwall.exe",

    [string]$OutputDirectory = "dist-daemon"
)

$ErrorActionPreference = "Stop"

# Resolve against the repository root (.scripts/..), so the script behaves the
# same when called from the repository root in CI and from anywhere locally.
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$binary = Join-Path $repositoryRoot $BinaryPath

if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) {
    throw "Daemon binary not found: $binary. Build it first (bun run build:daemon)."
}

$archiveName = "Dwall_${Version}_x64-daemon.zip"
$outputRoot = Join-Path $repositoryRoot $OutputDirectory
$stagingDirectory = Join-Path $outputRoot "staging"
$archivePath = Join-Path $outputRoot $archiveName

if (Test-Path -LiteralPath $stagingDirectory) {
    Remove-Item -LiteralPath $stagingDirectory -Recurse -Force
}
New-Item -ItemType Directory -Path $stagingDirectory -Force | Out-Null

Copy-Item -LiteralPath $binary -Destination (Join-Path $stagingDirectory "dwall.exe")

foreach ($file in @("LICENSE", "docs/headless.md", "docs/custom-themes.md")) {
    $source = Join-Path $repositoryRoot $file
    if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
        throw "Required file not found: $source"
    }

    $destination = Join-Path $stagingDirectory $file
    New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force | Out-Null
    Copy-Item -LiteralPath $source -Destination $destination
}

$readme = @"
Dwall daemon $Version
$(('=' * (16 + $Version.Length)))

This archive contains the Dwall daemon on its own: no settings window, no
WebView2 runtime, about 600 KB. It reads %APPDATA%\dwall\config.toml (creating a
commented default file on the first run) and applies the wallpaper that matches
the current solar position.

Quick start
-----------
    dwall.exe --check-config        validate the configuration and exit
    dwall.exe --once                apply one update cycle and exit
    dwall.exe --install-autostart   start automatically at sign-in
    dwall.exe --help                every option and exit code

Documentation
-------------
    docs/headless.md        configuration, commands, auto-start, logs, uninstalling
    docs/custom-themes.md   the theme.toml format for your own wallpaper sets

The settings window is a separate download; it shares the same configuration
file and can manage this daemon as well.
"@
Set-Content -LiteralPath (Join-Path $stagingDirectory "README.txt") -Value $readme -Encoding utf8

# Checksums, so a download can be verified without trusting the transport.
$checksums = Get-ChildItem -LiteralPath $stagingDirectory -Recurse -File |
    Where-Object { $_.Name -ne "SHA256SUM.txt" } |
    Sort-Object FullName |
    ForEach-Object {
        $relative = [System.IO.Path]::GetRelativePath($stagingDirectory, $_.FullName) -replace '\\', '/'
        $hash = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        "$hash  $relative"
    }
Set-Content -LiteralPath (Join-Path $stagingDirectory "SHA256SUM.txt") -Value $checksums -Encoding utf8

Remove-Item -LiteralPath $archivePath -Force -ErrorAction SilentlyContinue
Compress-Archive -Path (Join-Path $stagingDirectory "*") -DestinationPath $archivePath -CompressionLevel Optimal
Remove-Item -LiteralPath $stagingDirectory -Recurse -Force

# A checksum next to the archive is what lets a download be verified at all (the
# list inside the archive can only describe its contents after extraction).
$archiveHash = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
Set-Content -LiteralPath "$archivePath.sha256" -Value "$archiveHash  $archiveName" -Encoding ascii

$size = [math]::Round((Get-Item -LiteralPath $archivePath).Length / 1KB, 1)
Write-Host "Packaged $archiveName ($size KB)"
Get-ChildItem -LiteralPath $outputRoot -File | ForEach-Object { Write-Host "  $($_.FullName)" }
