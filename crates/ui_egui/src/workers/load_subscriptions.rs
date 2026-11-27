use crate::app::MyApp;
use cloud_terrastodon_azure::AzureSubscriptionListRequest;
use cloud_terrastodon_azure::Subscription;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

pub fn load_subscriptions(app: &mut MyApp) {
    load_subscriptions_with_invalidation(app, false);
}

pub(crate) fn load_subscriptions_with_invalidation(app: &mut MyApp, refresh: bool) {
    let account_generation = app.azure.account_generation;
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    let executor = app.executor.clone();
    let work_cache = app.work_cache.clone();
    let cancellation = executor.cancellation().clone();
    let started_at = Instant::now();
    let enqueue_result = LoadableWorkBuilder::<MyApp, Vec<Arc<Subscription>>>::new()
        .description("Loading Subscriptions")
        .setter(move |app, data| {
            if let Some(azure) = app.azure_content_mut(account_generation) {
                azure.subscriptions = data.map(Rc::new);
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            let subs = AzureSubscriptionListRequest { authentication }
                .execute(&executor)
                .with_invalidation(refresh)
                .await?;
            Ok(subs)
        })
        .build()
        .enqueue(
            &app.work_tracker.clone(),
            app.tx.clone(),
            app,
            &cancellation,
        );
    if let Err(error) = enqueue_result {
        app.azure.subscriptions = Loadable::Failed {
            error: error.into(),
            started_at,
            finished_at: Instant::now(),
        };
    }
}
