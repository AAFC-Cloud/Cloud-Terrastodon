use crate::app::MyApp;
use crate::loadable_work::LoadableWorkBuilder;
use cloud_terrastodon_core_azure::prelude::Subscription;
use cloud_terrastodon_core_azure::prelude::fetch_all_subscriptions;
use std::rc::Rc;
use tracing::info;

pub fn load_subscriptions(app: &mut MyApp) {
    info!("Queueing work to fetch subscriptions");
    LoadableWorkBuilder::<Vec<Subscription>>::new()
        .setter(|app, data| app.subscriptions = data.map(Rc::new))
        .work(async move {
            let subs = fetch_all_subscriptions().await?;
            Ok(subs)
        })
        .build()
        .unwrap()
        .enqueue(app);
}
