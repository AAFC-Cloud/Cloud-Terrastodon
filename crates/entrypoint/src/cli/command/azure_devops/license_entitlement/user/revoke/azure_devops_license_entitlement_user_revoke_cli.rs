use crate::cli::azure_devops::license_entitlement::user::AzureDevOpsLicenseEntitlementUserMatcher;
use clap::Args;
use cloud_terrastodon_azure::prelude::EntraGroupId;
use cloud_terrastodon_azure::prelude::Principal;
use cloud_terrastodon_azure::prelude::fetch_group_members;
use cloud_terrastodon_azure::prelude::remove_group_member;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsLicenseAssignmentSource;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_group_license_entitlements;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_user_license_entitlements;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use cloud_terrastodon_azure_devops::prelude::update_azure_devops_user_license_entitlement;
use eyre::Result;
use tracing::debug;
use tracing::info;

/// Find group-based license assignments that grant the provided user a given license.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserRevokeArgs {
    #[clap(flatten)]
    pub user_matcher: AzureDevOpsLicenseEntitlementUserMatcher,
}

impl AzureDevOpsLicenseEntitlementUserRevokeArgs {
    pub async fn invoke(self) -> Result<()> {
        let user_predicate = self.user_matcher.as_predicate()?;

        let org_url = get_default_organization_url().await?;

        let entitlements = fetch_azure_devops_user_license_entitlements(&org_url).await?;

        let user_entitlement = entitlements
            .into_iter()
            .find(|e| user_predicate(e))
            .ok_or_else(|| eyre::eyre!("No license entitlement found matching {self:?}",))?;

        debug!(?user_entitlement, "Found entitlement");
        let user_principal_name = user_entitlement.user.unique_name;

        if matches!(
            user_entitlement.license,
            AzureDevOpsLicenseType::AccountStakeholder | AzureDevOpsLicenseType::None
        ) {
            info!(
                user_devops_id = %user_entitlement.user_id,
                user_display_name = %user_entitlement.user.display_name,
                user_devops_license = ?user_entitlement.license,
                "User does not have a premium license, skipping revocation"
            );
            return Ok(());
        }

        if user_entitlement.assignment_source == AzureDevOpsLicenseAssignmentSource::GroupRule {
            // Find the rule that is granting the license and remove the user from that group

            // Fetch groups that are granting licenses
            let group_entitlements =
                fetch_azure_devops_group_license_entitlements(&org_url).await?;

            // Identify groups which grant the license that the user has
            for group_license_entitlement in group_entitlements {
                let matches = group_license_entitlement.license_rule.account_license_type
                    == user_entitlement.license;
                debug!(
                    group_display_name = %group_license_entitlement.group.display_name,
                    group_origin_id = %group_license_entitlement.group.origin_id,
                    group_license_type = ?group_license_entitlement.license_rule.account_license_type,
                    matches,
                    "Checking group entitlement against user license"
                );
                if !matches {
                    continue;
                }

                let group_entra_id = group_license_entitlement
                    .group
                    .origin_id
                    .parse::<EntraGroupId>()?;
                let group_entra_members = fetch_group_members(group_entra_id).await?;
                let user_in_group = group_entra_members
                    .iter()
                    .filter_map(|p: &Principal| match p.as_user() {
                        Some(user) => {
                            (user.user_principal_name == user_principal_name).then_some(user)
                        }
                        None => None,
                    })
                    .next();
                debug!(
                    group_display_name = %group_license_entitlement.group.display_name,
                    group_origin_id = %group_license_entitlement.group.origin_id,
                    group_member_count = group_entra_members.len(),
                    user_in_group = user_in_group.is_some(),
                    "Checked if user is in group granting license"
                );

                if let Some(entra_user) = user_in_group {
                    info!(
                        user_devops_id = %user_entitlement.user_id,
                        user_display_name = %user_entitlement.user.display_name,
                        user_devops_license = ?user_entitlement.license,
                        group_display_name = %group_license_entitlement.group.display_name,
                        group_origin_id = %group_license_entitlement.group.origin_id,
                        "User is granted license via this group - removing the user from this group to revoke the license"
                    );

                    remove_group_member(group_entra_id, entra_user.id).await?;
                }
            }
        } else {
            // Direct assignment - just downgrade the license
            update_azure_devops_user_license_entitlement(
                &org_url,
                &user_entitlement.user_id,
                AzureDevOpsLicenseType::AccountStakeholder,
            )
            .await?;
        }

        Ok(())
    }
}
