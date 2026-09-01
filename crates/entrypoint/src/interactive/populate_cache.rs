use cloud_terrastodon_azure::fetch_all_entra_users;
use cloud_terrastodon_azure::fetch_all_policy_assignments;
use cloud_terrastodon_azure::fetch_all_policy_definitions;
use cloud_terrastodon_azure::fetch_all_policy_set_definitions;
use cloud_terrastodon_azure::fetch_all_resource_groups;
use cloud_terrastodon_azure::fetch_all_role_assignments;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use indicatif::ProgressBar;
use tokio::task::JoinSet;
pub async fn populate_cache(auth_context: &AzureTenantAuthContext) -> Result<()> {
    let mut work: JoinSet<(&str, bool)> = JoinSet::new();
    let auth_context_for_policy_assignments = auth_context.clone();
    work.spawn(async move {
        (
            "fetch_all_policy_assignments",
            fetch_all_policy_assignments(&auth_context_for_policy_assignments)
                .await
                .is_ok(),
        )
    });
    let auth_context_for_policy_definitions = auth_context.clone();
    work.spawn(async move {
        (
            "fetch_all_policy_definitions",
            fetch_all_policy_definitions(&auth_context_for_policy_definitions)
                .await
                .is_ok(),
        )
    });
    let auth_context_for_policy_set_definitions = auth_context.clone();
    work.spawn(async move {
        (
            "fetch_all_policy_set_definitions",
            fetch_all_policy_set_definitions(&auth_context_for_policy_set_definitions)
                .await
                .is_ok(),
        )
    });
    let auth_context_for_resource_groups = auth_context.clone();
    work.spawn(async move {
        (
            "fetch_all_resource_groups",
            fetch_all_resource_groups(&auth_context_for_resource_groups)
                .await
                .is_ok(),
        )
    });
    let auth_context_for_role_assignments = auth_context.clone();
    work.spawn(async move {
        (
            "fetch_all_role_assignments",
            fetch_all_role_assignments(&auth_context_for_role_assignments)
                .await
                .is_ok(),
        )
    });
    let auth_context_for_users = auth_context.clone();
    work.spawn(async move {
        (
            "fetch_all_users",
            fetch_all_entra_users(&auth_context_for_users).await.is_ok(),
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
    use cloud_terrastodon_credentials::AuthContext;

    #[test_log::test(tokio::test)]
    #[ignore]
    async fn it_works() -> Result<()> {
        let auth_context =
            AuthContext::explicit_azure_cli().bind_to_azure_tenant(get_test_tenant_id().await?)?;
        populate_cache(&auth_context).await?;
        Ok(())
    }
}
