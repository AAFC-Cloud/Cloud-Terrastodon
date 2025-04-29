use cloud_terrastodon_core_azure_types::prelude::Subscription;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use eyre::Result;
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
use tracing::info;

use crate::prelude::ResourceGraphHelper;

pub async fn fetch_all_subscriptions() -> Result<Vec<Subscription>> {
    info!("Fetching subscriptions");
    let query = indoc! {r#"
        resourcecontainers
        | where type =~ "Microsoft.Resources/subscriptions"
        | extend parent_management_group_id = strcat("/providers/Microsoft.Management/managementGroups/", properties.managementGroupAncestorsChain[0].name)
        | project 
            name,
            id,
            tenant_id=tenantId,
            parent_management_group_id
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
            println!(
                "- {} ({}) under {}",
                sub.name,
                sub.id,
                sub.parent_management_group_id.name()
            );
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
