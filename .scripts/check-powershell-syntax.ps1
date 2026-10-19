# Parses every PowerShell script next to this file and fails on a syntax error.
#
# The release scripts (package-daemon.ps1, render-package-manifests.ps1) only run inside
# publish.yml, so a typo in them is discovered while a release is being cut. This uses
# PowerShell's own parser — no module install, no network — so it runs anywhere pwsh does.
#
#   pwsh .scripts/check-powershell-syntax.ps1
#
# Exit code 0 when every script parses, 1 with one line per error otherwise.

[CmdletBinding()]
param(
    # Defaults to the directory holding this script.
    [string]$Directory = $PSScriptRoot
)

$ErrorActionPreference = 'Stop'

$scripts = @(Get-ChildItem -LiteralPath $Directory -Filter '*.ps1' -File |
    Where-Object { $_.FullName -ne $PSCommandPath })

if ($scripts.Count -eq 0) {
    Write-Error "No PowerShell scripts found in $Directory"
    exit 1
}

$failed = @()

foreach ($script in $scripts) {
    $errors = $null
    $null = [System.Management.Automation.Language.Parser]::ParseFile(
        $script.FullName,
        [ref]$null,
        [ref]$errors
    )

    foreach ($parseError in @($errors)) {
        Write-Host "$($script.Name):$($parseError.Extent.StartLineNumber):$($parseError.Extent.StartColumnNumber): $($parseError.Message)"
        $failed += $script.Name
    }
}

if ($failed.Count -gt 0) {
    Write-Error "$($failed.Count) syntax error(s) in $($scripts.Count) script(s)"
    exit 1
}

Write-Host "$($scripts.Count) script(s) parse"
exit 0
