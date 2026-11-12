use crate::app::MyApp;
use cloud_terrastodon_azure::AzureSubscriptionListRequest;
use cloud_terrastodon_azure::Subscription;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

pub fn load_subscriptions(app: &mut MyApp) {
    let authentication = app.authentication;
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let started_at = Instant::now();
    info!("Queueing work to fetch subscriptions");
    let work = LoadableWorkBuilder::<MyApp, Vec<Arc<Subscription>>>::new()
        .description("Loading Subscriptions")
        .setter(|app, data| app.subscriptions = data.map(Rc::new))
        .work(async move |scope| {
            let executor = executor.with_cancellation_scope(scope);
            let subs = AzureSubscriptionListRequest { authentication }
                .execute(&executor)
                .await?;
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
