$meta = cargo metadata --format-version 1 --no-deps | ConvertFrom-Json
$ct_packages = $meta.packages | Where-Object { $_.name.StartsWith("cloud_terrastodon")}

# Build dependency graph (only internal dependencies)
$dependencies = @{}
$all_package_names = $ct_packages | ForEach-Object { $_.name }

foreach ($package in $ct_packages) {
    $internal_deps = $package.dependencies | Where-Object { 
        $_.name -in $all_package_names 
    } | ForEach-Object { $_.name }
    
    $dependencies[$package.name] = $internal_deps
}

# Topological sort using Kahn's algorithm
$sorted_packages = @()
$in_degree = @{}
$queue = New-Object System.Collections.Queue

# Initialize in-degrees to 0
foreach ($pkg_name in $all_package_names) {
    $in_degree[$pkg_name] = 0
}

# Calculate in-degrees - if A depends on B, then A's in-degree increases
foreach ($pkg_name in $dependencies.Keys) {
    $in_degree[$pkg_name] = $dependencies[$pkg_name].Count
}

# Find packages with no dependencies (in-degree = 0)
foreach ($pkg_name in $all_package_names) {
    if ($in_degree[$pkg_name] -eq 0) {
        $queue.Enqueue($pkg_name)
    }
}

# Process packages in topological order
while ($queue.Count -gt 0) {
    $current = $queue.Dequeue()
    $sorted_packages += $current
    
    # For each package that depends on the current package, reduce its in-degree
    foreach ($pkg_name in $dependencies.Keys) {
        if ($current -in $dependencies[$pkg_name]) {
            $in_degree[$pkg_name]--
            if ($in_degree[$pkg_name] -eq 0) {
                $queue.Enqueue($pkg_name)
            }
        }
    }
}

# Check for circular dependencies
if ($sorted_packages.Count -ne $all_package_names.Count) {
    Write-Warning "Circular dependency detected! Some packages were not processed."
    $unprocessed = $all_package_names | Where-Object { $_ -notin $sorted_packages }
    Write-Host "Unprocessed packages: $($unprocessed -join ', ')"
    exit 1
}

# Process each package in dependency order
foreach ($pkg_name in $sorted_packages) {
    $package = $ct_packages | Where-Object { $_.name -eq $pkg_name }
    Write-Host "Processing: $($package.name) v$($package.version)"
    
    # Check if package exists on crates.io
    Write-Host "  Checking if package exists on crates.io..."
    $checkResult = cargo info $package.name --registry crates-io 2>$null
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  Package not found on crates.io, skipping..." -ForegroundColor Yellow
        continue
    }
    
    Write-Host "  Package exists on crates.io, publishing..." -ForegroundColor Green
    
    # Navigate to package directory and publish
    $packagePath = Split-Path $package.manifest_path -Parent
    Push-Location $packagePath
    
    try {
        cargo publish
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Failed to publish $($package.name)"
            Pop-Location
            exit 1
        }
        Write-Host "  Successfully published $($package.name)" -ForegroundColor Green
    }
    finally {
        Pop-Location
    }
}

Write-Host "All packages processed successfully!" -ForegroundColor Green