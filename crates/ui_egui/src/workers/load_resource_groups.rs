use crate::app::MyApp;
use cloud_terrastodon_azure::fetch_all_resource_groups;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use std::rc::Rc;
use std::time::Instant;
use tracing::info;

pub fn load_resource_groups(app: &mut MyApp) {
    let tenant_id = app.tenant_id;
    let auth_context = app.auth_context.clone();
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let started_at = Instant::now();
    info!("Queueing work to fetch resource groups");
    let work = LoadableWorkBuilder::<MyApp, _>::new()
        .description("Loading Resource Groups")
        .setter(|app, data| app.resource_groups = data.map(Rc::new))
        .work(async move |scope| {
            let executor = executor.with_cancellation_scope(scope);
            let auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
            let executor = auth_context.auth_context.bind_executor(&executor)?;
            let resource_groups = fetch_all_resource_groups(&auth_context, &executor).await?;
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
            error,
            started_at,
            finished_at: Instant::now(),
        };
    }
}
