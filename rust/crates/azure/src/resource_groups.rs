use anyhow::Result;
use azure_types::prelude::Subscription;
use azure_types::resource_groups::ResourceGroup;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use indicatif::MultiProgress;
use std::path::PathBuf;
use std::sync::Arc;

use crate::prelude::gather_from_subscriptions;
use crate::prelude::SubscriptionMap;

pub async fn fetch_all_resource_groups() -> Result<SubscriptionMap<Vec<ResourceGroup>>> {
    let resource_groups =
        gather_from_subscriptions(async |sub: Subscription, _mp: Arc<MultiProgress>| {
            let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
            cmd.args([
                "group",
                "list",
                "--output",
                "json",
                "--subscription",
                sub.id.to_string().as_ref(),
            ]);
            let mut cache = PathBuf::new();
            cache.push("ignore");
            cache.push(format!("az group list --subscription {}", sub.name));
            cmd.use_cache_dir(Some(cache));
            cmd.run().await
        })
        .await?;
    Ok(resource_groups)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test_log::test(tokio::test)]
    async fn it_works() -> Result<()> {
        let result = fetch_all_resource_groups().await?;
        println!("Found {} resource groups:", result.len());
        for (sub, groups) in result {
            println!("Subscription: {}", sub.name);
            for group in groups {
                println!(" - {}", group);
            }
        }
        Ok(())
    }
}
