use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_group_members;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_groups;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_license_entitlements;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use cloud_terrastodon_command::ParallelFallibleWorkQueue;
use std::collections::HashMap;

/// Write to stdout the json for a bunch of Azure DevOps info
pub async fn dump_azure_devops() -> eyre::Result<()> {
    let org_url = get_default_organization_url().await?;
    let projects = fetch_all_azure_devops_projects(&org_url).await?;
    let mut payload = HashMap::<_, serde_json::Value>::new();
    payload.insert("projects", serde_json::to_value(&projects)?);

    let users = fetch_azure_devops_license_entitlements(&org_url).await?;
    payload.insert("users", serde_json::to_value(&users)?);

    let mut project_groups = ParallelFallibleWorkQueue::new("fetch_azure_devops_groups", 10);
    for project in projects.iter() {
        let org_url = org_url.clone();
        let project_id = project.id.clone();
        project_groups.enqueue(async move {
            let groups = fetch_azure_devops_groups(&org_url, &project_id).await?;
            eyre::Ok((project_id, groups))
        });
    }

    let project_groups = project_groups
        .join()
        .await?
        .into_iter()
        .collect::<HashMap<_, _>>();
    payload.insert("project_groups", serde_json::to_value(&project_groups)?);

    let mut group_members = ParallelFallibleWorkQueue::new("group_members", 10);
    for group in project_groups.values().flatten().cloned() {
        let org_url = org_url.clone();
        group_members.enqueue(async move {
            let members = fetch_azure_devops_group_members(&org_url, &group.descriptor).await?;
            eyre::Ok((group.descriptor.clone(), members))
        });
    }
    let group_members = group_members
        .join()
        .await?
        .into_iter()
        .collect::<HashMap<_, _>>();
    payload.insert("group_members", serde_json::to_value(&group_members)?);

    // print to stdout
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}
