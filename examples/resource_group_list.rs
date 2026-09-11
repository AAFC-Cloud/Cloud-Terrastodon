#![allow(
    clippy::negative_feature_names,
    reason = "The crate depends on an established vendored feature named no-backend"
)]
#![allow(
    clippy::disallowed_script_idents,
    reason = "Facet's derive generates Unicode identifiers in sibling items, outside the struct's lint scope"
)]
// Prints each Azure Resource Group name using the re-exported Azure module.

use cloud_terrastodon::app::{App, ApplicationCli, GlobalArgs, Result, figue};
use cloud_terrastodon::azure::fetch_all_resource_groups;
use cloud_terrastodon::azure::get_default_tenant_id;
use facet::Facet;

/// List resource groups with their subscription names and resource IDs.
/// Uses the default tenant selected in Azure CLI.
#[derive(Facet)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
struct Cli {
    #[facet(flatten)]
    global_args: GlobalArgs,
    #[facet(flatten)]
    #[cfg_attr(test, arbitrary(default))]
    builtins: figue::FigueBuiltins,
}

impl ApplicationCli for Cli {
    fn global_args(&self) -> &GlobalArgs {
        &self.global_args
    }
}

fn main() -> Result<()> {
    App::new("resource_group_list", env!("CARGO_PKG_VERSION")).run(
        |_cli: Cli, context| async move {
            // These helpers do not accept a cancellation token. Check between
            // requests and while printing; an in-flight request is not interrupted.
            context.cancellation.bail_if_cancelled()?;
            let tenant_id = get_default_tenant_id().await?;
            context.cancellation.bail_if_cancelled()?;
            let auth = context.auth.bind_to_azure_tenant(tenant_id)?;
            let resource_groups = fetch_all_resource_groups(&auth).await?;
            context.cancellation.bail_if_cancelled()?;

            for resource_group in resource_groups {
                context.cancellation.bail_if_cancelled()?;
                println!(
                    "{subscription_name} - {resource_group_name} ({full_id})",
                    subscription_name = resource_group.subscription_name,
                    resource_group_name = resource_group.name,
                    full_id = resource_group.id
                );
            }
            Ok(())
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // FigueBuiltins has no PartialEq implementation. As in Figue's own helper
    // tests, keep builtins at their defaults and compare the application data.
    impl PartialEq for Cli {
        fn eq(&self, other: &Self) -> bool {
            self.global_args == other.global_args
        }
    }

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
