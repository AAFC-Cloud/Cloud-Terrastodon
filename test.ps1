# build with all cores
cargo build --tests
# test with only 4 workers to avoid rate limits lol
cargo test --workspace --jobs 4
