# Renders the winget and Scoop manifests from their templates with a synthetic release,
# so a broken or renamed placeholder fails here instead of halfway through a release.
#
#   pwsh .scripts/check-packaging-templates.ps1
#
# A real release passes the version and checksum published with the archive
# (render-package-manifests.ps1 -Version <v>); this uses a fixed placeholder version and
# digest precisely so it can run on any checkout, in CI, without a release or a download.

[CmdletBinding()]
param(
    [string]$Version = '0.0.0'
)

$ErrorActionPreference = 'Stop'

$repositoryRoot = Split-Path -Parent $PSScriptRoot

# Inside the repository, because the renderer joins its -OutputDirectory against the
# repository root and a path on another drive has no relative form. `dist-manifests/` is
# ignored by git, so a run that is killed leaves nothing for the maintainer to clean up.
$relativeOutput = 'dist-manifests/check-templates'
$outputDirectory = Join-Path $repositoryRoot $relativeOutput

# Only this subdirectory is removed, never the manifests a real release rendered.
if (Test-Path -LiteralPath $outputDirectory) {
    Remove-Item -LiteralPath $outputDirectory -Recurse -Force
}

try {
    try {
        & (Join-Path $PSScriptRoot 'render-package-manifests.ps1') `
            -Version $Version `
            -Sha256 ('0' * 64) `
            -OutputDirectory $relativeOutput `
            -Validate
    }
    catch {
        # `& script.ps1` runs in this session, so its terminating error has to be turned
        # into an exit code explicitly — otherwise a broken template still "passes".
        Write-Error $_
        exit 1
    }

    $rendered = @(
        Join-Path $outputDirectory 'winget/thep0y.Dwall.yaml'
        Join-Path $outputDirectory 'winget/thep0y.Dwall.installer.yaml'
        Join-Path $outputDirectory 'winget/thep0y.Dwall.locale.en-US.yaml'
        Join-Path $outputDirectory 'scoop/dwall.json'
    )

    foreach ($file in $rendered) {
        if (-not (Test-Path -LiteralPath $file -PathType Leaf)) {
            throw "Expected $file to be rendered"
        }

        if ((Get-Item -LiteralPath $file).Length -eq 0) {
            throw "$file is empty"
        }
    }

    Write-Host "Templates render for $Version"
}
finally {
    if (Test-Path -LiteralPath $outputDirectory) {
        Remove-Item -LiteralPath $outputDirectory -Recurse -Force
    }
}
