use crate::app::MyApp;
use crate::loadable_work::LoadableWorkBuilder;
use cloud_terrastodon_core_azure::prelude::fetch_all_subscriptions;
use tracing::info;

pub fn load_subscriptions(app: &mut MyApp) {
    info!("Queueing work to fetch subscriptions");
    LoadableWorkBuilder::new()
        .field(|app| &mut app.subscriptions)
        .work(async move {
            let subs = fetch_all_subscriptions().await?;
            // default to not-expanded
            Ok(subs.into_iter().map(|sub| (false, sub)).collect())
        })
        .build()
        .unwrap()
        .enqueue(app);
}
