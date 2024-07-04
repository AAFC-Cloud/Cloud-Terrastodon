use crate::prelude::gather_from_subscriptions;
use anyhow::Result;
use azure_types::prelude::ResourceGroup;
use azure_types::prelude::Scope;
use azure_types::prelude::Subscription;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use indicatif::MultiProgress;
use tofu_types::prelude::Sanitizable;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub async fn fetch_all_resource_groups() -> Result<HashMap<Subscription, Vec<ResourceGroup>>> {
    let resource_groups =
        gather_from_subscriptions(async |sub: Subscription, _mp: Arc<MultiProgress>| {
            let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
            cmd.args([
                "group",
                "list",
                "--output",
                "json",
                "--subscription",
                sub.id.short_form(),
            ]);
            cmd.use_cache_dir(PathBuf::from_iter([
                "az group list",
                format!("--subscription {}", sub.name.sanitize()).as_str(),
            ]));
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
