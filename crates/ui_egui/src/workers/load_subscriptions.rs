use crate::app::MyApp;
use crate::loadable_work::LoadableWorkBuilder;
use cloud_terrastodon_azure::prelude::Subscription;
use cloud_terrastodon_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_azure::prelude::get_default_tenant_id;
use std::rc::Rc;
use tracing::info;

pub fn load_subscriptions(app: &mut MyApp) {
    info!("Queueing work to fetch subscriptions");
    LoadableWorkBuilder::<Vec<Subscription>>::new()
        .description("Loading Subscriptions")
        .setter(|app, data| app.subscriptions = data.map(Rc::new))
        .work(async move {
            let tenant_id = get_default_tenant_id().await?;
            let subs = fetch_all_subscriptions(tenant_id).await?;
            Ok(subs)
        })
        .build()
        .unwrap()
        .enqueue(app);
}
