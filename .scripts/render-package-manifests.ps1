<#
.SYNOPSIS
    Renders the winget and Scoop manifests for a published daemon archive.

.DESCRIPTION
    Fills .packaging/**/*.template with the release's version, download URL and
    archive checksum, so submitting to winget-pkgs or a Scoop bucket is a copy
    rather than hand-editing three YAML files and a hash.

    Defaults to the checksum produced by package-daemon.ps1, so the usual flow
    after a release is:

        ./.scripts/package-daemon.ps1 -Version 0.2.5
        ./.scripts/render-package-manifests.ps1 -Version 0.2.5 -Validate

    `winget validate` runs only when the winget CLI is available (it is not
    installed on every machine or CI image); the Scoop manifest is always parsed.

.EXAMPLE
    ./.scripts/render-package-manifests.ps1 -Version 0.2.5
    ./.scripts/render-package-manifests.ps1 -Version 0.2.5 -Validate
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Version,

    [string]$Identifier = "thep0y.Dwall",

    [string]$Repository = "dwall-rs/dwall",

    # Defaults to the sibling checksum file published with the archive.
    [string]$Sha256,

    [string]$OutputDirectory = "dist-manifests",

    [switch]$Validate
)

$ErrorActionPreference = "Stop"

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$archiveName = "Dwall_${Version}_x64-daemon.zip"
$archiveUrl = "https://github.com/$Repository/releases/download/v${Version}/$archiveName"

if (-not $Sha256) {
    $checksumFile = Join-Path $repositoryRoot "dist-daemon/$archiveName.sha256"

    if (-not (Test-Path -LiteralPath $checksumFile -PathType Leaf)) {
        throw "Checksum not found: $checksumFile. Run ./.scripts/package-daemon.ps1 -Version $Version first, or pass -Sha256."
    }

    $Sha256 = ((Get-Content -LiteralPath $checksumFile -First 1) -split '\s+')[0]
}

if ($Sha256 -notmatch '^[0-9a-fA-F]{64}$') {
    throw "Not a SHA-256 digest: $Sha256"
}

# winget and Scoop both write digests in upper case.
$Sha256 = $Sha256.ToUpperInvariant()

$outputRoot = Join-Path $repositoryRoot $OutputDirectory
$wingetDirectory = Join-Path $outputRoot "winget"
$scoopDirectory = Join-Path $outputRoot "scoop"

foreach ($directory in @($wingetDirectory, $scoopDirectory)) {
    if (Test-Path -LiteralPath $directory) {
        Remove-Item -LiteralPath $directory -Recurse -Force
    }
    New-Item -ItemType Directory -Path $directory -Force | Out-Null
}

$replacements = [ordered]@{
    "__IDENTIFIER__" = $Identifier
    "__VERSION__"    = $Version
    "__REPOSITORY__" = $Repository
    "__URL__"        = $archiveUrl
    "__SHA256__"     = $Sha256
    "__DATE__"       = (Get-Date -Format "yyyy-MM-dd")
}

function Render-Template {
    param([string]$TemplatePath, [string]$DestinationPath)

    $content = Get-Content -LiteralPath $TemplatePath -Raw
    foreach ($key in $replacements.Keys) {
        $content = $content.Replace($key, $replacements[$key])
    }

    if ($content -match '__[A-Z_]+__') {
        throw "Unreplaced placeholder in $TemplatePath"
    }

    Set-Content -LiteralPath $DestinationPath -Value $content -NoNewline -Encoding utf8
    Write-Host "  $([System.IO.Path]::GetRelativePath($repositoryRoot, $DestinationPath))"
}

Write-Host "Rendering manifests for $Identifier $Version ($archiveName)"
Render-Template -TemplatePath (Join-Path $repositoryRoot ".packaging/winget/version.yaml.template") -DestinationPath (Join-Path $wingetDirectory "$Identifier.yaml")
Render-Template -TemplatePath (Join-Path $repositoryRoot ".packaging/winget/installer.yaml.template") -DestinationPath (Join-Path $wingetDirectory "$Identifier.installer.yaml")
Render-Template -TemplatePath (Join-Path $repositoryRoot ".packaging/winget/locale.en-US.yaml.template") -DestinationPath (Join-Path $wingetDirectory "$Identifier.locale.en-US.yaml")
Render-Template -TemplatePath (Join-Path $repositoryRoot ".packaging/scoop/dwall.json.template") -DestinationPath (Join-Path $scoopDirectory "dwall.json")

# The Scoop manifest is JSON, so it can always be checked here.
Get-Content -LiteralPath (Join-Path $scoopDirectory "dwall.json") -Raw | ConvertFrom-Json | Out-Null
Write-Host "  Scoop manifest parses"

if ($Validate) {
    $winget = Get-Command winget -ErrorAction SilentlyContinue
    if ($winget) {
        Write-Host "Validating with $($winget.Source)"
        & $winget.Source validate --manifest $wingetDirectory
        if ($LASTEXITCODE -ne 0) {
            throw "winget validate failed with exit code $LASTEXITCODE"
        }
        Write-Host "  winget manifest valid"
    }
    else {
        Write-Warning "winget is not installed; skipping 'winget validate' (the manifest was still rendered)"
    }
}
