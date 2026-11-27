Write-Host -ForegroundColor Yellow "Checking for direct Serde dependencies..."
$metadataJson = cargo metadata --locked --no-deps --format-version 1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$serdeDependencies = foreach ($package in ($metadataJson | ConvertFrom-Json).packages) {
    foreach ($dependency in $package.dependencies) {
        if ($dependency.name -in @("serde", "serde_core", "serde_derive", "serde_json")) {
            "$($package.name) depends on $($dependency.name)"
        }
    }
}
if ($serdeDependencies) {
    throw "Use Facet in workspace crates. $($serdeDependencies -join '. ')."
}

Write-Host -ForegroundColor Yellow "Running check check..."
cargo check --all --tests --examples --workspace
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host -ForegroundColor Yellow "Running format check..."
rustup run nightly -- cargo fmt --all
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host -ForegroundColor Yellow "Running clippy lint check..."
cargo clippy --workspace --all-targets --all-features -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Write-Host -ForegroundColor Yellow "Running build..."
# cargo build --all-features --verbose
# if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Write-Host -ForegroundColor Yellow "Running tests..."
# cargo test --all-features --verbose
# if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
