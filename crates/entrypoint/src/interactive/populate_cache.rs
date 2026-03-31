use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::fetch_all_entra_users;
use cloud_terrastodon_azure::fetch_all_policy_assignments;
use cloud_terrastodon_azure::fetch_all_policy_definitions;
use cloud_terrastodon_azure::fetch_all_policy_set_definitions;
use cloud_terrastodon_azure::fetch_all_resource_groups;
use cloud_terrastodon_azure::fetch_all_role_assignments;
use eyre::Result;
use indicatif::ProgressBar;
use tokio::task::JoinSet;
pub async fn populate_cache(tenant_id: AzureTenantId) -> Result<()> {
    let mut work: JoinSet<(&str, bool)> = JoinSet::new();
    work.spawn(async move {
        (
            "fetch_all_policy_assignments",
            fetch_all_policy_assignments(tenant_id).await.is_ok(),
        )
    });
    work.spawn(async move {
        (
            "fetch_all_policy_definitions",
            fetch_all_policy_definitions(tenant_id).await.is_ok(),
        )
    });
    work.spawn(async move {
        (
            "fetch_all_policy_set_definitions",
            fetch_all_policy_set_definitions(tenant_id).await.is_ok(),
        )
    });
    work.spawn(async move {
        (
            "fetch_all_resource_groups",
            fetch_all_resource_groups(tenant_id).await.is_ok(),
        )
    });
    work.spawn(async move {
        (
            "fetch_all_role_assignments",
            fetch_all_role_assignments(tenant_id).await.is_ok(),
        )
    });
    work.spawn(async move {
        (
            "fetch_all_users",
            fetch_all_entra_users(tenant_id).await.is_ok(),
        )
    });
    let pb = ProgressBar::new(work.len() as u64);
    // pb.set_style(
    //     ProgressStyle::default_bar()
    //         .template("[{wide_bar} {pos}/{len} {msg}")?,
    // );
    pb.tick();
    while let Some(x) = work.join_next().await {
        let (operation, success) = x?;
        pb.inc(1);
        let msg = format!(
            "{} {}",
            operation,
            if success { "succeeded" } else { "failed" }
        );
        // pb.set_message(msg);
        pb.println(msg);
    }
    pb.finish_with_message("Cache population complete");
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use cloud_terrastodon_azure::get_test_tenant_id;

    #[test_log::test(tokio::test)]
    #[ignore]
    async fn it_works() -> Result<()> {
        populate_cache(get_test_tenant_id().await?).await?;
        Ok(())
    }
}
