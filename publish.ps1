[CmdletBinding()]
param(
    # Include all three fork workspaces for the first coordinated Teamy release.
    [string[]]$ManifestPath = @((Join-Path $PSScriptRoot 'Cargo.toml')),
    [switch]$DryRun,
    [switch]$AllowDirty
)

$ErrorActionPreference = 'Stop'
$releasePackages = @{}

foreach ($manifest in $ManifestPath) {
    $metadataJson = cargo metadata --manifest-path $manifest --format-version 1 --no-deps
    if ($LASTEXITCODE -ne 0) { throw "Could not read Cargo metadata for $manifest" }
    $metadata = $metadataJson | ConvertFrom-Json
    foreach ($package in $metadata.packages) {
        if ($package.id -notin $metadata.workspace_members) { continue }
        if ($null -ne $package.publish -and $package.publish.Count -eq 0) { continue }
        if ($package.name -notmatch '^(cloud_terrastodon($|_)|teamy-facet($|-)|teamy-figue($|-))') { continue }
        if ($releasePackages.ContainsKey($package.name)) {
            if ($releasePackages[$package.name].manifest_path -ne $package.manifest_path) {
                throw "Two workspaces supply $($package.name); choose one release source"
            }
            continue
        }
        $releasePackages[$package.name] = $package
    }
}

if ($releasePackages.Count -eq 0) { throw 'No publishable Cloud Terrastodon or Teamy packages selected' }

# Development dependencies may form cycles; publication follows production and
# build dependencies. Packaged consumer validation is a separate release gate.
$remainingDependencies = @{}
foreach ($packageName in $releasePackages.Keys) {
    $remainingDependencies[$packageName] = @(
        $releasePackages[$packageName].dependencies |
            Where-Object { $_.kind -ne 'dev' -and $releasePackages.ContainsKey($_.name) } |
            Select-Object -ExpandProperty name -Unique
    )
}
$publicationOrder = [System.Collections.Generic.List[string]]::new()
while ($remainingDependencies.Count -gt 0) {
    $readyPackages = @($remainingDependencies.Keys | Where-Object { $remainingDependencies[$_].Count -eq 0 } | Sort-Object)
    if ($readyPackages.Count -eq 0) {
        throw "Production dependency cycle: $(@($remainingDependencies.Keys | Sort-Object) -join ', ')"
    }
    foreach ($packageName in $readyPackages) {
        $publicationOrder.Add($packageName)
        $remainingDependencies.Remove($packageName)
    }
    foreach ($packageName in @($remainingDependencies.Keys)) {
        $remainingDependencies[$packageName] = @($remainingDependencies[$packageName] | Where-Object { $_ -notin $readyPackages })
    }
}

foreach ($packageName in $publicationOrder) {
    $package = $releasePackages[$packageName]
    if ($DryRun) {
        Write-Output "$($package.name) $($package.version) $($package.manifest_path)"
        continue
    }

    # Check the exact version, not the registry's latest stable version. A 404
    # means first publication is allowed; authentication/network errors must fail.
    $versionUri = 'https://crates.io/api/v1/crates/{0}/{1}' -f
        [Uri]::EscapeDataString($package.name), [Uri]::EscapeDataString($package.version)
    $versionExists = $false
    try {
        $null = Invoke-RestMethod -Uri $versionUri -Headers @{ 'User-Agent' = 'Cloud-Terrastodon-release' }
        $versionExists = $true
    }
    catch {
        if ($null -eq $_.Exception.Response -or [int]$_.Exception.Response.StatusCode -ne 404) { throw }
    }
    if ($versionExists) {
        Write-Output "Already published: $($package.name) $($package.version)"
        continue
    }

    Write-Output "Publishing $($package.name) $($package.version)"
    $publishArguments = @('publish', '--registry', 'crates-io', '--manifest-path', $package.manifest_path, '--locked')
    if ($AllowDirty) { $publishArguments += '--allow-dirty' }
    & cargo @publishArguments
    if ($LASTEXITCODE -ne 0) { throw "Publication failed: $($package.name) $($package.version)" }
}
