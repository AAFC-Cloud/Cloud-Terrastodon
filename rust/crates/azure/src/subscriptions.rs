use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use cloud_terrasotodon_core_azure_types::prelude::Subscription;
use cloud_terrasotodon_core_command::prelude::CommandBuilder;
use cloud_terrasotodon_core_command::prelude::CommandKind;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use itertools::Itertools;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::debug;

pub async fn fetch_all_subscriptions() -> Result<Vec<Subscription>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["account", "list", "--output", "json"]);
    cmd.use_cache_dir("az account list");

    let subs = cmd.run::<Vec<Subscription>>().await?;
    if subs.is_empty() {
        cmd.bust_cache()
            .await
            .context("Busting cache because we expect more subscriptions once logged in")?;
        bail!("no subscriptions found, are you logged in?");
    }
    let tenant_id = subs
        .iter()
        .filter(|s| s.is_default)
        .map(|s| s.tenant_id.clone())
        .next();

    let tenant_id = match tenant_id {
        Some(tenant_id) => tenant_id,
        None => {
            cmd.bust_cache().await.context("Busting cache because 	subscription list details will change once user activates a subscription")?;
            bail!("No subscription active, can't determine filter");
        }
    };
    Ok(subs
        .into_iter()
        .filter(|sub| sub.tenant_id == tenant_id)
        .collect_vec())
}

pub async fn gather_from_subscriptions<T, F, Fut>(
    fetcher: F,
) -> Result<HashMap<Subscription, Vec<T>>>
where
    T: Send + 'static,
    F: Fn(Subscription, Arc<MultiProgress>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Vec<T>>> + Send + 'static,
{
    // Fetch subscriptions
    debug!("Fetching subscriptions");
    let subscriptions = fetch_all_subscriptions().await?;

    // Set up progress bars
    // https://github.com/console-rs/indicatif/blob/main/examples/multi-tree.rs
    let mp = Arc::new(MultiProgress::new());
    let pb_sub = mp.add(ProgressBar::new(subscriptions.len() as u64));
    pb_sub.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:30.cyan/blue} {pos:>7}/{len:7} {msg}")?,
    );
    #[cfg(test)]
    pb_sub.set_draw_target(indicatif::ProgressDrawTarget::hidden());

    let fetcher = Arc::new(fetcher);
    let mut work_pool: JoinSet<(Subscription, Result<Vec<T>>)> = JoinSet::new();
    debug!("Launching fetchers");
    for subscription in subscriptions {
        let fetcher = fetcher.clone();
        let mp = mp.clone();
        debug!("Spawning work pool entry for {}", subscription);
        work_pool.spawn(async move { (subscription.clone(), fetcher(subscription, mp).await) });
    }
    pb_sub.tick();

    debug!("Collecting results");
    let mut rtn = HashMap::<Subscription, Vec<T>>::default();
    while let Some(res) = work_pool.join_next().await {
        let (sub, res) = res?;
        let res = res?;

        pb_sub.inc(1);
        pb_sub.set_message(format!(
            "Found {} things from subscription {}",
            res.len(),
            sub.name
        ));

        rtn.insert(sub, res);
    }

    pb_sub.finish_with_message(format!(
        "Obtained {} things from {} subscriptions.",
        rtn.values().map(|x| x.len()).sum::<usize>(),
        rtn.keys().len(),
    ));

    Ok(rtn)
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use std::time::Duration;
    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_subscriptions().await?;
        println!("Found {} subscriptions:", result.len());
        for sub in result {
            println!("- {} ({})", sub.name, sub.id);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_gather_from_subscriptions() -> Result<()> {
        let results =
            gather_from_subscriptions(async |sub: Subscription, _mp: Arc<MultiProgress>| {
                let delay = rand::thread_rng().gen_range(200..=3000);
                sleep(Duration::from_millis(delay)).await;
                Ok(vec![
                    format!("item1 from {}", sub.name),
                    format!("item2 from {}", sub.name),
                ])
            })
            .await?;
        assert!(!results.is_empty());

        for (sub, items) in results.into_iter() {
            println!("Subscription: {}", sub.name);
            assert_eq!(items.len(), 2);
            for item in items {
                println!(" - {}", item);
            }
        }

        Ok(())
    }
}
