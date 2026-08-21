#!/usr/bin/pwsh
cargo install --path . --locked
Write-Host "Now in path:"
cloud_terrastodon --version
