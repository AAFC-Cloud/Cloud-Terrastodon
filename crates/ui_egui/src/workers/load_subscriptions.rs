use crate::app::MyApp;
use crate::loadable_work::LoadableWorkBuilder;
use cloud_terrastodon_azure::Subscription;
use cloud_terrastodon_azure::fetch_all_subscriptions;
use std::rc::Rc;
use tracing::info;

pub fn load_subscriptions(app: &mut MyApp) {
    let tenant_id = app.tenant_id;
    let auth_context = app.auth_context.clone();
    info!("Queueing work to fetch subscriptions");
    LoadableWorkBuilder::<Vec<Subscription>>::new()
        .description("Loading Subscriptions")
        .setter(|app, data| app.subscriptions = data.map(Rc::new))
        .work(async move {
            let subs = fetch_all_subscriptions(tenant_id, &auth_context).await?;
            Ok(subs)
        })
        .build()
        .unwrap()
        .enqueue(app);
}
