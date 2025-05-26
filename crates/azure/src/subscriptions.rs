use cloud_terrastodon_azure_types::prelude::Subscription;
use cloud_terrastodon_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use indoc::indoc;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

use crate::prelude::ResourceGraphHelper;

pub async fn fetch_all_subscriptions() -> Result<Vec<Subscription>> {
    info!("Fetching subscriptions");
    let query = indoc! {r#"
        resourcecontainers
        | where type =~ "Microsoft.Resources/subscriptions"
        | project 
            name,
            id,
            tenant_id=tenantId,
            management_group_ancestors_chain=properties.managementGroupAncestorsChain
    "#};

    let subscriptions = ResourceGraphHelper::new(
        query,
        CacheBehaviour::Some {
            path: PathBuf::from("subscriptions"),
            valid_for: Duration::from_hours(8),
        },
    )
    .collect_all::<Subscription>()
    .await?;
    info!("Found {} subscriptions", subscriptions.len());
    Ok(subscriptions)
}

pub async fn get_active_subscription_id() -> Result<SubscriptionId> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "account",
        "list",
        "--query",
        "[?isDefault].id",
        "--output",
        "json",
    ]);
    let rtn = cmd.run::<[SubscriptionId; 1]>().await?[0];
    Ok(rtn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_subscriptions().await?;
        println!("Found {} subscriptions:", result.len());
        for sub in result {
            println!(
                "- {} ({}) under {}",
                sub.name,
                sub.id,
                sub.management_group_ancestors_chain.first().unwrap().name
            );
        }
        Ok(())
    }

    #[tokio::test]
    pub async fn get_active() -> eyre::Result<()> {
        println!("{}", get_active_subscription_id().await?);
        Ok(())
    }
}
