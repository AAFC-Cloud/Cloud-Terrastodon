#![allow(
    clippy::disallowed_script_idents,
    reason = "Facet's derive generates Unicode identifiers in sibling items, outside the struct's lint scope"
)]
// Prints each Azure Resource Group name using the re-exported Azure module.

use cloud_terrastodon::app::App;
use cloud_terrastodon::app::ApplicationCli;
use cloud_terrastodon::app::GlobalArgs;
use cloud_terrastodon::app::Result;
use cloud_terrastodon::app::figue;
use cloud_terrastodon::azure::AzureTenantArgument;
use cloud_terrastodon::azure::AzureTenantArgumentExt;
use cloud_terrastodon::azure::fetch_all_resource_groups;
use facet::Facet;

/// List resource groups with their subscription names and resource IDs.
#[derive(Facet)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
struct Cli {
    #[facet(flatten)]
    global_args: GlobalArgs,
    /// Tenant ID, tracked alias, or `default`.
    /// Defaults to the authentication source's tenant or the active Azure CLI tenant.
    #[facet(figue::named, default)]
    tenant: AzureTenantArgument<'static>,
}

impl ApplicationCli for Cli {
    fn global_args(&self) -> &GlobalArgs {
        &self.global_args
    }
}

fn main() -> Result<()> {
    App::new("resource_group_list", env!("CARGO_PKG_VERSION")).run(|cli: Cli, context| async move {
        let auth = cli
            .tenant
            .bind_auth_context(&context.auth, &context.cancellation)
            .await?;
        let resource_groups = fetch_all_resource_groups(&auth, &context.cancellation).await?;

        for resource_group in resource_groups {
            // Suspend so App's wrapper can check cancellation before this row.
            tokio::task::yield_now().await;
            println!(
                "{subscription_name} - {resource_group_name} ({full_id})",
                subscription_name = resource_group.subscription_name,
                resource_group_name = resource_group.name,
                full_id = resource_group.id
            );
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_args_consistency() -> Result<()> {
        figue::assert_to_args_consistency::<Cli>(Default::default())?;
        Ok(())
    }

    #[test]
    fn to_args_roundtrip() -> Result<()> {
        figue::assert_to_args_roundtrip::<Cli>(figue::TestToArgsRoundTrip {
            success_count_global: 128,
            max_attempts_global: 10_000,
            ..Default::default()
        })?;
        Ok(())
    }
}
