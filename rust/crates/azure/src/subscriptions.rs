use anyhow::Result;
use azure_types::prelude::Subscription;
use azure_types::prelude::SubscriptionId;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::debug;

pub async fn fetch_subscriptions() -> Result<Vec<Subscription>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["account", "list", "--output", "json"]);
    let mut cache = PathBuf::new();
    cache.push("ignore");
    cache.push("az account list");
    cmd.use_cache_dir(Some(cache));
    cmd.run().await
}

#[derive(Debug)]
pub struct SubscriptionBins<T: Sized> {
    results: HashMap<SubscriptionId, Vec<T>>,
    subscriptions: HashMap<SubscriptionId, Subscription>,
}
impl<T> Default for SubscriptionBins<T> {
    fn default() -> Self {
        Self {
            results: Default::default(),
            subscriptions: Default::default(),
        }
    }
}
impl<T> SubscriptionBins<T> {
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }
    pub fn into_iter(mut self) -> impl Iterator<Item = (Subscription, Vec<T>)> {
        self.results.into_iter().map(move |(k, v)| {
            let subscription = self.subscriptions.remove(&k).unwrap();
            (subscription, v)
        })
    }
}


pub async fn gather_from_subscriptions<T>(
    fetcher: impl Fn(Subscription, Arc<MultiProgress>) -> Pin<Box<dyn Future<Output = Result<Vec<T>>> + Send>>
        + Send
        + Sync
        + Clone
        + 'static,
) -> Result<SubscriptionBins<T>>
where
    T: Send + 'static,
{
    // Fetch subscriptions
    debug!("Fetching subscriptions");
    let subscriptions = fetch_subscriptions().await?;

    // Set up progress bars
    // https://github.com/console-rs/indicatif/blob/main/examples/multi-tree.rs
    let mp = Arc::new(MultiProgress::new());
    let pb_sub = mp.add(ProgressBar::new(subscriptions.len() as u64));
    pb_sub.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:30.cyan/blue} {pos:>7}/{len:7} {msg}")?,
    );

    let mut work_pool: JoinSet<(Subscription, Result<Vec<T>>)> = JoinSet::new();
    debug!("Launching fetchers");
    for subscription in subscriptions {
        let fetcher = fetcher.clone();
        let mp = mp.clone();
        work_pool.spawn(async move { (subscription.clone(), (fetcher)(subscription, mp).await) });
    }

    debug!("Collecting results");
    let mut rtn = SubscriptionBins::<T>::default();
    while let Some(res) = work_pool.join_next().await {
        let (sub, res) = res?;
        let res = res?;

        pb_sub.inc(1);
        pb_sub.set_message(format!(
            "Found {} things from subscription {}",
            res.len(),
            sub.name
        ));

        rtn.results.insert(sub.id.clone(), res);
        rtn.subscriptions.insert(sub.id.clone(), sub);
    }

    pb_sub.finish_with_message(format!(
        "Obtained {} things from {} subscriptions.",
        rtn.results.values().map(|x| x.len()).sum::<usize>(),
        rtn.subscriptions.len(),
    ));

    Ok(rtn)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rand::Rng;
    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_subscriptions().await?;
        println!("Found {} subscriptions:", result.len());
        for sub in result {
            println!("- {} ({})", sub.name, sub.id);
        }
        Ok(())
    }

    async fn dummy_fetcher(sub: Subscription, _mp: Arc<MultiProgress>) -> Result<Vec<String>> {
        let delay = rand::thread_rng().gen_range(200..=3000);
        sleep(Duration::from_millis(delay)).await;
        Ok(vec![format!("item1 from {}", sub.name), format!("item2 from {}", sub.name)])
    }


    #[tokio::test]
    async fn test_gather_from_subscriptions() -> Result<()> {
        // let dummy_fetcher =
        //     async |sub: Subscription, _mp: Arc<MultiProgress>| -> Result<Vec<String>> {
        //         sleep(Duration::from_millis(100)).await;
        //         Ok(vec![
        //             format!("item1 from {}", sub.name),
        //             format!("item2 from {}", sub.name),
        //         ])
        //     };
        let dummy_fetcher = |sub: Subscription, mp: Arc<MultiProgress>| {
            Box::pin(dummy_fetcher(sub, mp)) as Pin<Box<dyn Future<Output = Result<Vec<String>>> + Send>>
        };
        let results = gather_from_subscriptions(dummy_fetcher).await?;
        assert!(!results.is_empty());

        for (sub, items) in results.into_iter() {
            println!("Subscription: {}", sub.name);
            assert_eq!(items.len(),2);
            for item in items {
                println!(" - {}", item);
            }
        }

        Ok(())
    }
}
