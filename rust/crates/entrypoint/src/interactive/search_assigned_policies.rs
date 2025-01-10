use anyhow::Context;
use cloud_terrastodon_core_azure::prelude::fetch_all_policy_assignments;
use cloud_terrastodon_core_azure::prelude::fetch_all_policy_definitions;
use cloud_terrastodon_core_azure::prelude::fetch_all_policy_set_definitions;
use tokio::try_join;
use tracing::info;

pub async fn search_assigned_policies() -> anyhow::Result<()> {
    info!("Fetching policy assignments and definitions");
    let (policy_assignments, policy_definitions, policy_set_definitions) = match try_join!(
        fetch_all_policy_assignments(),
        fetch_all_policy_definitions(),
        fetch_all_policy_set_definitions()
    ) {
        Ok(x) => x,
        Err(e) => return Err(e).context("failed to fetch policy data"),
    };

    info!("Found {} policy assignments", policy_assignments.len());
    info!("Found {} policy definitions", policy_definitions.len());
    info!(
        "Found {} policy set definitions",
        policy_set_definitions.len()
    );

    Ok(())
}
