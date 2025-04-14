use crate::app::MyApp;
use crate::loadable_work::LoadableWorkBuilder;
use cloud_terrastodon_core_azure::prelude::fetch_all_resource_groups;
use std::rc::Rc;
use tracing::info;

pub fn load_resource_groups(app: &mut MyApp) {
    info!("Queueing work to fetch resource groups");
    LoadableWorkBuilder::new()
        .setter(|app, data| app.resource_groups = data.map(Rc::new))
        .work(async move {
            let resource_groups = fetch_all_resource_groups().await?;
            Ok(resource_groups.into())
        })
        .build()
        .unwrap()
        .enqueue(app);
}
