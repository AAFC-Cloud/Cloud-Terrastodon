Write-Host -ForegroundColor Yellow "Running check check..."
cargo check --all --tests --examples --workspace
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host -ForegroundColor Yellow "Running format check..."
rustup run nightly -- cargo fmt --all
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