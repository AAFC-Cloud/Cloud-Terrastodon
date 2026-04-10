# build with all cores
cargo build --tests
# test with fewer workers to avoid rate limits lol
$env:CLOUD_TERRASTODON_REAUTH="DENY"
cargo test --workspace --jobs 2
