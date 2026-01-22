use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsLicenseKind;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_user_license_entitlements;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use cloud_terrastodon_azure_devops::prelude::update_azure_devops_user_license_entitlement;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use tracing::info;

#[derive(Args, Debug, Clone)]
/// Update an Azure DevOps user's license entitlement.
pub struct AzureDevOpsLicenseEntitlementUserUpdateTuiArgs {}

impl AzureDevOpsLicenseEntitlementUserUpdateTuiArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;

        let chosen_entitlements = PickerTui::new()
            .set_header("Azure DevOps License Entitlements")
            .pick_many_reloadable(async |invalidate| {
                fetch_azure_devops_user_license_entitlements(&org_url)
                    .with_invalidation(invalidate)
                    .await
                    .map(|ents| {
                        ents.into_iter()
                            .map(|e| Choice {
                                key: format!(
                                    "{} <{}> ({})",
                                    e.user.display_name, e.user.unique_name, e.user.id
                                ),
                                value: e,
                            })
                            .collect::<Vec<_>>()
                    })
            })
            .await?;

        // Choose a single license kind
        let license = PickerTui::new()
            .set_header("Azure DevOps License Kind")
            .pick_one(AzureDevOpsLicenseKind::VARIANTS)?;

        for entitlement in chosen_entitlements {
            let resp = update_azure_devops_user_license_entitlement(
                &org_url,
                entitlement.user_id.clone(),
                license.clone(),
            )
            .await?;
            info!(
                %entitlement.user_id,
                %license,
                ?resp,
                "Successfully updated license entitlement for user"
            );
        }

        Ok(())
    }
}
