use crate::app::MyApp;
use cloud_terrastodon_azure::Subscription;
use cloud_terrastodon_azure::fetch_all_subscriptions;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

pub fn load_subscriptions(app: &mut MyApp) {
    let tenant_id = app.tenant_id;
    let auth_context = app.auth_context.clone();
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let started_at = Instant::now();
    info!("Queueing work to fetch subscriptions");
    let work = LoadableWorkBuilder::<MyApp, Vec<Arc<Subscription>>>::new()
        .description("Loading Subscriptions")
        .setter(|app, data| app.subscriptions = data.map(Rc::new))
        .work(async move |scope| {
            let executor = executor.with_cancellation_scope(scope);
            let auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
            let executor = auth_context.auth_context.bind_executor(&executor)?;
            let subs = fetch_all_subscriptions(&auth_context, &executor).await?;
            Ok(subs)
        })
        .build()
        .and_then(|work| {
            work.enqueue(
                &app.work_tracker.clone(),
                app.tx.clone(),
                app,
                &cancellation,
            )
        });
    if let Err(error) = work {
        app.subscriptions = Loadable::Failed {
            error,
            started_at,
            finished_at: Instant::now(),
        };
    }
}
