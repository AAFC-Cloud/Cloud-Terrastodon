use clap::Args;
use cloud_terrastodon_azure::prelude::GroupId;
use cloud_terrastodon_azure::prelude::Principal;
use cloud_terrastodon_azure::prelude::fetch_group_members;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsLicenseAssignmentSource;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsUserId;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_group_license_entitlements;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_user_license_entitlements;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use eyre::bail;
use tracing::debug;
use tracing::info;

/// Find group-based license assignments that grant the provided user a given license.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserRevokeArgs {
    /// User id to inspect (GUID or value accepted by `AzureDevOpsUserId`).
    #[arg(long)]
    pub user_id: AzureDevOpsUserId,
}

impl AzureDevOpsLicenseEntitlementUserRevokeArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;

        let entitlements = fetch_azure_devops_user_license_entitlements(&org_url).await?;

        let user_entitlement = entitlements
            .into_iter()
            .find(|e| e.user_id == self.user_id)
            .ok_or_else(|| {
                eyre::eyre!("No license entitlement found for user id {}", self.user_id)
            })?;
        debug!(?user_entitlement, "Found entitlement");
        let user_principal_name = user_entitlement.user.unique_name;

        if matches!(
            user_entitlement.license,
            AzureDevOpsLicenseType::AccountStakeholder | AzureDevOpsLicenseType::None
        ) {
            info!(
                user_devops_id = %self.user_id,
                user_display_name = %user_entitlement.user.display_name,
                user_devops_license = ?user_entitlement.license,
                "User does not have a premium license, skipping revocation"
            );
            return Ok(());
        }

        // If the assignment isn't group-rule based, nothing to do here.
        if user_entitlement.assignment_source != AzureDevOpsLicenseAssignmentSource::GroupRule {
            bail!(
                "User's entitlement is not assigned by group rules (assignment source: {:?}) - todo: impl direct revocation",
                user_entitlement.assignment_source
            );
        }

        // Fetch groups that are granting licenses
        let group_entitlements = fetch_azure_devops_group_license_entitlements(&org_url).await?;

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
                .parse::<GroupId>()?;
            let group_entra_members = fetch_group_members(group_entra_id).await?;
            let user_in_group = group_entra_members
                .iter()
                .any(|p: &Principal| match p.as_user() {
                    Some(u) => u.user_principal_name == user_principal_name,
                    None => false,
                });
            debug!(
                group_display_name = %group_license_entitlement.group.display_name,
                group_origin_id = %group_license_entitlement.group.origin_id,
                group_member_count = group_entra_members.len(),
                user_in_group,
                "Checked if user is in group granting license"
            );

            if user_in_group {
                info!(
                    user_devops_id = %self.user_id,
                    user_display_name = %user_entitlement.user.display_name,
                    user_devops_license = ?user_entitlement.license,
                    group_display_name = %group_license_entitlement.group.display_name,
                    group_origin_id = %group_license_entitlement.group.origin_id,
                    "User is granted license via this group - remove the user from this group to revoke the license"
                );
            }
        }

        Ok(())
    }
}
