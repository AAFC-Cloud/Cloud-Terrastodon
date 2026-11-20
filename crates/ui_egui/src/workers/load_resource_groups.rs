use crate::app::MyApp;
use cloud_terrastodon_azure::AzureResourceGroupListRequest;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use std::rc::Rc;
use std::time::Instant;
use tracing::info;

pub fn load_resource_groups(app: &mut MyApp, refresh: bool) {
    let account_generation = app.account_generation;
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    let executor = app.executor.clone();
    let work_cache = app.work_cache.clone();
    let cancellation = executor.cancellation().clone();
    let started_at = Instant::now();
    info!("Queueing work to fetch resource groups");
    let work = LoadableWorkBuilder::<MyApp, _>::new()
        .description("Loading Resource Groups")
        .setter(move |app, data| {
            if app.account_generation == account_generation {
                app.resource_groups = data.map(Rc::new);
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            let resource_groups = AzureResourceGroupListRequest { authentication }
                .execute(&executor)
                .with_invalidation(refresh)
                .await?;
            Ok(resource_groups.into())
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
        app.resource_groups = Loadable::Failed {
            error: error.into(),
            started_at,
            finished_at: Instant::now(),
        };
    }
}
