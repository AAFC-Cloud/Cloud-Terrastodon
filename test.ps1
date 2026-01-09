# build with all cores
cargo build --tests
# test with fewer workers to avoid rate limits lol
cargo test --workspace --jobs 2
