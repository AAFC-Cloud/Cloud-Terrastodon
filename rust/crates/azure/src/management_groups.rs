use anyhow::bail;
use anyhow::Result;
use cloud_terrastodon_core_azure_types::prelude::ManagementGroup;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use indoc::indoc;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;
use tracing::debug;
use tracing::error;
use tracing::info;

use crate::prelude::ResourceGraphHelper;

pub async fn fetch_root_management_group() -> Result<ManagementGroup> {
    info!("Fetching root management group");
    let found = fetch_all_management_groups()
        .await?
        .into_iter()
        .find(|mg| mg.name() == mg.tenant_id.to_string());
    match found {
        Some(management_group) => {
            info!("Found root management group");
            Ok(management_group)
        }
        None => {
            let msg = "Failed to find a management group with name matching the tenant ID";
            error!(msg);
            bail!(msg);
        }
    }
}

pub async fn fetch_all_management_groups() -> Result<Vec<ManagementGroup>> {
    info!("Fetching management groups");
    let query = indoc! {r#"
        resourcecontainers
        | where type =~ "Microsoft.Management/managementGroups"
        | project 
            tenant_id=tenantId,
            id,
            display_name=properties.displayName,
            parent_id=properties.details.parent.id
    "#};

    let management_groups = ResourceGraphHelper::new(
        query,
        CacheBehaviour::Some {
            path: PathBuf::from("management_groups"),
            valid_for: Duration::from_hours(8),
        },
    )
    .collect_all::<ManagementGroup>()
    .await?;
    info!("Found {} management groups", management_groups.len());
    Ok(management_groups)
}

pub async fn gather_from_management_groups<T, F, Fut>(
    fetcher: F,
) -> Result<HashMap<ManagementGroup, Vec<T>>>
where
    T: Send + 'static,
    F: Fn(ManagementGroup, Arc<MultiProgress>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Vec<T>>> + Send + 'static,
{
    // Fetch subscriptions
    debug!("Fetching management groups");
    let management_groups = fetch_all_management_groups().await?;

    // Set up progress bars
    // https://github.com/console-rs/indicatif/blob/main/examples/multi-tree.rs
    let mp = Arc::new(MultiProgress::new());
    let pb_mg = mp.add(ProgressBar::new(management_groups.len() as u64));
    pb_mg.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:30.cyan/blue} {pos:>7}/{len:7} {msg}")?,
    );

    let fetcher = Arc::new(fetcher);
    let mut work_pool: JoinSet<(ManagementGroup, Result<Vec<T>>)> = JoinSet::new();
    debug!("Launching fetchers");
    for mg in management_groups {
        let fetcher = fetcher.clone();
        let mp = mp.clone();
        debug!("Spawning work pool entry for {}", mg);
        work_pool.spawn(async move { (mg.clone(), fetcher(mg, mp).await) });
    }
    pb_mg.tick();

    debug!("Collecting results");
    let mut rtn = HashMap::<ManagementGroup, Vec<T>>::default();
    while let Some(res) = work_pool.join_next().await {
        let (mg, res) = res?;
        let res = res?;

        pb_mg.inc(1);
        pb_mg.set_message(format!(
            "Found {} things from management group {}",
            res.len(),
            mg.name()
        ));

        rtn.insert(mg, res);
    }

    pb_mg.finish_with_message(format!(
        "Obtained {} things from {} management groups.",
        rtn.values().map(|x| x.len()).sum::<usize>(),
        rtn.keys().len(),
    ));

    Ok(rtn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_management_groups().await?;
        println!("Found {} management groups:", result.len());
        for mg in result {
            println!("- {} ({})", mg.display_name, mg.name());
        }
        Ok(())
    }
}
