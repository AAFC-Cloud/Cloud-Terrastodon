use crate::app::MyApp;
use crate::loadable::Loadable;
use crate::loadable_work::LoadableWorkBuilder;
use cloud_terrastodon_azure::Subscription;
use cloud_terrastodon_azure::fetch_all_subscriptions;
use std::rc::Rc;
use std::sync::Arc;
use tracing::info;

pub fn load_subscriptions(app: &mut MyApp) {
    let tenant_id = app.tenant_id;
    let auth_context = app.auth_context.clone();
    info!("Queueing work to fetch subscriptions");
    let work = LoadableWorkBuilder::<Vec<Arc<Subscription>>>::new()
        .description("Loading Subscriptions")
        .setter(|app, data| app.subscriptions = data.map(Rc::new))
        .work(app.executor.cancellable_scope(move |executor| async move {
            let auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
            let executor = auth_context.auth_context.bind_executor(&executor)?;
            let subs = fetch_all_subscriptions(&auth_context, &executor).await?;
            Ok(subs)
        }))
        .build();
    match work {
        Ok(work) => work.enqueue(app),
        Err(error) => app.subscriptions = Loadable::Failed(error),
    }
}
