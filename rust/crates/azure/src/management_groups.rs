use anyhow::Result;
use azure_types::prelude::ManagementGroup;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::debug;

pub async fn fetch_management_groups() -> Result<Vec<ManagementGroup>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.use_cache_dir("az account management-group list");
    cmd.args([
        "account",
        "management-group",
        "list",
        "--no-register",
        "--output",
        "json",
    ]);
    cmd.run().await
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
    let management_groups = fetch_management_groups().await?;

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
            mg.name
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
        let result = fetch_management_groups().await?;
        println!("Found {} management groups:", result.len());
        for mg in result {
            println!("- {} ({})", mg.display_name, mg.name);
        }
        Ok(())
    }
}
