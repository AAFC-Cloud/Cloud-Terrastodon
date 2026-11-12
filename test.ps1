# Cargo tests are offline checks: they must not consult configured accounts,
# invoke installed cloud tools, or contact services. Runtime verification is
# explicitly selected with `ct test list` and `ct test run <pattern>`.
cargo test --workspace --offline --locked
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
