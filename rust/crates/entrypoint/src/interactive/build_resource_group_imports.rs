use eyre::eyre;
use eyre::Result;
use cloud_terrastodon_core_azure::prelude::fetch_all_resource_groups;
use cloud_terrastodon_core_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_core_azure::prelude::ResourceGroup;
use cloud_terrastodon_core_azure::prelude::Subscription;
use cloud_terrastodon_core_azure::prelude::SubscriptionId;
use cloud_terrastodon_core_pathing::AppDir;
use cloud_terrastodon_core_tofu::prelude::Sanitizable;
use cloud_terrastodon_core_tofu::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu::prelude::TofuProviderKind;
use cloud_terrastodon_core_tofu::prelude::TofuProviderReference;
use cloud_terrastodon_core_tofu::prelude::TofuWriter;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
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
        .map(|sub| (sub.id.clone(), sub))
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
        header: None,
    })?;

    info!("Building import blocks");
    let mut used_subscriptions = HashSet::new();
    let mut imports = Vec::with_capacity(chosen.len());
    for Choice { value: pair, .. } in chosen {
        let mut block: TofuImportBlock = pair.resource_group.into();
        block.provider = TofuProviderReference::Alias {
            kind: TofuProviderKind::AzureRM,
            name: pair.subscription.name.sanitize(),
        };
        imports.push(block);

        used_subscriptions.insert(pair.subscription);
    }
    if imports.is_empty() {
        return Err(eyre!("Imports should not be empty"));
    }

    info!("Writing import blocks");
    TofuWriter::new(AppDir::Imports.join("resource_group_imports.tf"))
        .overwrite(imports)
        .await?
        .format()
        .await?;

    let mut providers = Vec::new();
    for sub in used_subscriptions {
        let provider = sub.clone().into_provider_block();
        providers.push(provider);
    }
    TofuWriter::new(AppDir::Imports.join("boilerplate.tf"))
        .merge(providers)
        .await?
        .format()
        .await?;

    Ok(())
}
