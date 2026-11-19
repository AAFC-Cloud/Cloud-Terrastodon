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
    let account_generation = app.account_generation;
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    let executor = app.executor.clone();
    let work_cache = app.work_cache.clone();
    let cancellation = executor.cancellation().clone();
    let started_at = Instant::now();
    info!("Queueing work to fetch subscriptions");
    let work = LoadableWorkBuilder::<MyApp, Vec<Arc<Subscription>>>::new()
        .description("Loading Subscriptions")
        .setter(move |app, data| {
            if app.account_generation == account_generation {
                app.subscriptions = data.map(Rc::new);
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
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
            error: error.into(),
            started_at,
            finished_at: Instant::now(),
        };
    }
}
