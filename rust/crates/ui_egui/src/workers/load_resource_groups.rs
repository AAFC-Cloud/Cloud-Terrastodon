use crate::app::MyApp;
use crate::loadable_work::LoadableWorkBuilder;
use cloud_terrastodon_core_azure::prelude::fetch_all_resource_groups;
use itertools::Itertools;
use tracing::info;

pub fn load_resource_groups(app: &mut MyApp) {
    info!("Queueing work to fetch resource groups");
    LoadableWorkBuilder::new()
        .field(|app| &mut app.resource_groups)
        .work(async move {
            let resource_groups = fetch_all_resource_groups().await?;
            // default to not-expanded
            Ok(resource_groups
                .into_iter()
                .map(|resource_group| (false, resource_group))
                .into_group_map_by(|(_checked, rg)| rg.subscription_id.clone()))
        })
        .build()
        .unwrap()
        .enqueue(app);
}
