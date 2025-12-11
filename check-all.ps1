Write-Host -ForegroundColor Yellow "Running format check..."
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host -ForegroundColor Yellow "Running clippy lint check..."
cargo clippy --all-targets --all-features -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Write-Host -ForegroundColor Yellow "Running build..."
# cargo build --all-features --verbose
# if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Write-Host -ForegroundColor Yellow "Running tests..."
# cargo test --all-features --verbose
# if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }