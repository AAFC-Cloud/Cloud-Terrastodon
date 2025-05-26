# https://github.com/bevyengine/bevy/blob/8898c9e14221a5e5ed05005927bfc5b185753dab/.github/workflows/post-release.yml
# https://github.com/crate-ci/cargo-release

cargo release 0.9.0 `
--workspace `
--no-tag `
--exclude cloud_terrastodon_azure_devops_rest_client `
--exclude cloud_terrastodon_ui_egui `
--exclude cloud_terrastodon_ui_ratatui `
--exclude cloud_terrastodon_entrypoint `
