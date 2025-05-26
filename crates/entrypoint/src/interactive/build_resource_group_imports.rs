use cloud_terrastodon_azure::prelude::ResourceGroup;
use cloud_terrastodon_azure::prelude::Subscription;
use cloud_terrastodon_azure::prelude::SubscriptionId;
use cloud_terrastodon_azure::prelude::fetch_all_resource_groups;
use cloud_terrastodon_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_hcl::prelude::HCLImportBlock;
use cloud_terrastodon_hcl::prelude::HCLProviderReference;
use cloud_terrastodon_hcl::prelude::HCLWriter;
use cloud_terrastodon_hcl::prelude::ProviderKind;
use cloud_terrastodon_hcl::prelude::Sanitizable;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick_many;
use eyre::Result;
use eyre::eyre;
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::info;

pub struct SubRGPair<'a> {
    subscription: &'a Subscription,
    resource_group: ResourceGroup,
}
impl std::fmt::Display for SubRGPair<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.resource_group.name)
    }
}

pub async fn build_resource_group_imports() -> Result<()> {
    info!("Fetching resource groups");
    let subscriptions = fetch_all_subscriptions()
        .await?
        .into_iter()
        .map(|sub| (sub.id, sub))
        .collect::<HashMap<SubscriptionId, Subscription>>();

    let resource_groups = fetch_all_resource_groups().await?;

    info!("Prompting for which to import");
    let mut choices: Vec<Choice<SubRGPair>> = Vec::with_capacity(resource_groups.len());
    for rg in resource_groups {
        let sub = subscriptions
            .get(&rg.subscription_id)
            .ok_or_else(|| eyre!("could not find subscription for resource group {rg:?}"))?;
        let choice = SubRGPair {
            subscription: sub,
            resource_group: rg,
        };
        choices.push(choice.into());
    }
    let chosen = pick_many(FzfArgs {
        choices,
        prompt: Some("Groups to import: ".to_string()),
        ..Default::default()
    })?;

    info!("Building import blocks");
    let mut used_subscriptions = HashSet::new();
    let mut imports = Vec::with_capacity(chosen.len());
    for Choice { value: pair, .. } in chosen {
        let mut block: HCLImportBlock = pair.resource_group.into();
        block.provider = HCLProviderReference::Alias {
            kind: ProviderKind::AzureRM,
            name: pair.subscription.name.sanitize(),
        };
        imports.push(block);

        used_subscriptions.insert(pair.subscription);
    }
    if imports.is_empty() {
        return Err(eyre!("Imports should not be empty"));
    }

    info!("Writing import blocks");
    HCLWriter::new(AppDir::Imports.join("resource_group_imports.tf"))
        .overwrite(imports)
        .await?
        .format_file()
        .await?;

    let mut providers = Vec::new();
    for sub in used_subscriptions {
        let provider = sub.clone().into_provider_block();
        providers.push(provider);
    }
    HCLWriter::new(AppDir::Imports.join("boilerplate.tf"))
        .merge(providers)
        .await?
        .format_file()
        .await?;

    Ok(())
}
