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

$cargoTomlPath = Join-Path $PSScriptRoot "Cargo.toml"
$cargoTomlContent = Get-Content $cargoTomlPath -Raw

# Update root Cargo.toml version fields
$patternRoot = '"\d+\.\d+\.\d+"(\s*#\s*CT_VERSION)'
$replacementRoot = '"' + $newVersion + '"$1'
$updatedCargoTomlContent = $cargoTomlContent -replace $patternRoot, $replacementRoot
Set-Content -Path $cargoTomlPath -Value $updatedCargoTomlContent
Write-Host "Updated root Cargo.toml to version $newVersion"

# Update all crates' Cargo.toml files
$cratesDir = Join-Path $PSScriptRoot "crates"
Get-ChildItem -Path $cratesDir -Directory | ForEach-Object {
    $crateTomlPath = Join-Path $_.FullName "Cargo.toml"
    if (Test-Path $crateTomlPath) {
        $crateContent = Get-Content $crateTomlPath -Raw
        $patternCrate = '(?m)^(version\s*=\s*)"\d+\.\d+\.\d+"'
        $replacementCrate = '$1"' + $newVersion + '"'
        $updatedCrateContent = $crateContent -replace $patternCrate, $replacementCrate
        Set-Content -Path $crateTomlPath -Value $updatedCrateContent
        Write-Host "Updated $crateTomlPath to version $newVersion"
    }
}
