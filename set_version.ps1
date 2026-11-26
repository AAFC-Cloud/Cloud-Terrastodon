param(
    [string]$VersionParameter
)

$ErrorActionPreference = "Stop"

Function Get-VersionFromUser {
    param(
        [string]$InitialVersion
    )

    if (-not [string]::IsNullOrEmpty($InitialVersion)) {
        if ($InitialVersion -match '^\d+\.\d+\.\d+$') {
            Write-Host "Using provided version: $InitialVersion"
            return $InitialVersion
        }
        else {
            Write-Warning "Invalid version format provided as parameter: '$InitialVersion'. Please use the format X.Y.Z (e.g., 1.2.3)."
            # Fall through to prompt
        }
    }

    while ($true) {
        $versionInput = Read-Host "Enter the new version (e.g., 1.2.3)"
        if ($versionInput -match '^\d+\.\d+\.\d+$') {
            return $versionInput
        }
        else {
            Write-Warning "Invalid version format. Please use the format X.Y.Z (e.g., 1.2.3)"
        }
    }
}

$newVersion = Get-VersionFromUser -InitialVersion $VersionParameter
Write-Host "Updating to version: $newVersion"

$memberRoots = 'crates', 'examples', 'tools' | ForEach-Object { Join-Path $PSScriptRoot $_ }
$manifestPaths = @(
    Join-Path $PSScriptRoot 'Cargo.toml'
    Get-ChildItem -LiteralPath $memberRoots -Directory | ForEach-Object {
        Join-Path $_.FullName 'Cargo.toml'
    }
)

# Update marked package versions and dependency requirements, preserving exact pins.
$pattern = '(?m)(\bversion[ \t]*=[ \t]*"=?)\d+\.\d+\.\d+("[^\r\n]*#[ \t]*CT_VERSION[ \t]*\r?$)'
$replacement = '${1}' + $newVersion + '${2}'
foreach ($manifestPath in $manifestPaths) {
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) { continue }
    $content = [IO.File]::ReadAllText($manifestPath)
    $updatedContent = $content -creplace $pattern, $replacement
    if ($updatedContent -ceq $content) { continue }
    [IO.File]::WriteAllText($manifestPath, $updatedContent)
    Write-Host "Updated $manifestPath to version $newVersion"
}
