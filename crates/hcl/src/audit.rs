use crate::prelude::TerraformBlockExtracterPatcher;
use crate::reflow::as_single_body;
use cloud_terrastodon_hcl_types::prelude::TerraformProviderInfo;
use hcl::edit::visit_mut::VisitMut;
use std::collections::HashSet;
use std::path::Path;
use tracing::debug;
use tracing::info;
use tracing::warn;

pub async fn audit(source_dir: &Path) -> eyre::Result<()> {
    info!(?source_dir, "Auditing");

    let mut main_body = as_single_body(source_dir).await?;
    let terraform_block;
    {
        debug!("Extracting terraform config blocks");
        let mut patcher = TerraformBlockExtracterPatcher::default();
        patcher.visit_body_mut(&mut main_body);
        terraform_block = patcher.terraform_block;
    }
    debug!("Boilerplate body extracted: {terraform_block:#?}");

    let providers_with_version_specified: HashSet<String> =
        match &terraform_block.required_providers {
            Some(required_providers) => required_providers.0.keys().cloned().collect(),
            None => Default::default(),
        };

    let mut providers_being_used: HashSet<String> = Default::default();
    // for each resource and data block, get the part before the underscore in the first label of the block
    for structure in main_body {
        debug!(?structure, "Checking structure for provider usage");
        let Some(block) = structure.as_block() else {
            continue;
        };
        // if not resource or data, skip
        if block.ident.as_str() != "resource" && block.ident.as_str() != "data" {
            continue;
        }
        let [kind, _name] = block.labels.as_slice() else {
            warn!(?block, "Block does not have exactly two labels, skipping");
            continue;
        };
        match kind.as_str().split_once('_') {
            Some((before, _after)) => {
                providers_being_used.insert(before.to_string());
            }
            None => {
                warn!(?kind, "Block kind does not have an underscore, skipping");
                continue;
            }
        }
    }

    debug!(
        ?providers_with_version_specified,
        ?providers_being_used,
        "Providers with version specified and being used"
    );

    let mut warned = false;

    // If no backend is specified, warn about it
    if terraform_block.backend.is_none() {
        warn!(
            "No backend is specified in the Terraform configuration. If you lose your state file, you're pooched ."
        );
        warned = true;
    }

    // If a provider has a version specified but is not being used, warn about it
    for provider in providers_with_version_specified.difference(&providers_being_used) {
        warn!(
            "Provider `{provider}` is specified as required but is not being used in the configuration."
        );
        warned = true;
    }

    // If a provider is being used but does not have a version specified, warn about it
    for provider in providers_being_used.difference(&providers_with_version_specified) {
        warn!(
            "Provider `{provider}` is being used but does not have a version specified. This can lead to unexpected behavior."
        );
        warned = true;
    }

    if let Some(required_providers) = &terraform_block.required_providers {
        for (key, provider) in required_providers.0.iter() {
            let url = format!(
                "https://{registry_url}/v1/providers/{namespace}/{provider}",
                registry_url = provider.source.hostname.0,
                namespace = provider.source.namespace.0,
                provider = provider.source.kind.provider_prefix()
            );
            let json = reqwest::Client::new()
                .get(&url)
                .send()
                .await?
                .json::<TerraformProviderInfo>()
                .await?;
            let latest_version = json.versions.last().unwrap();
            let satisfies = provider.version.is_satisfied_by(latest_version);
            if !satisfies {
                warn!(
                    "Provider `{key}` version `{}` does not satisfy the latest version `{}`. Please update your configuration.",
                    provider.version, latest_version
                );
                warned = true;
            } else {
                info!(
                    "Provider `{key}` version `{}` satisfies the latest version `{}`.",
                    provider.version, latest_version
                );
            }
        }
    }

    if !warned {
        info!("Epic config win! You're doing it awesome style! 🔥🔥🔥");
    }
    Ok(())
}
