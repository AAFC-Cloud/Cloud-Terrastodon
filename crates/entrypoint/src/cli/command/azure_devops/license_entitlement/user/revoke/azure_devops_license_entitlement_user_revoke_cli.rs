use std::env::current_exe;

use crate::cli::CloudTerrastodonCommand;
use crate::cli::ToArgs;
use crate::cli::azure::AzureArgs;
use crate::cli::azure::azure_command::AzureCommand;
use crate::cli::azure::entra::AzureEntraArgs;
use crate::cli::azure::entra::AzureEntraCommand;
use crate::cli::azure::entra::AzureEntraGroupArgs;
use crate::cli::azure::entra::group::AzureEntraGroupCommand;
use crate::cli::azure::entra::group::AzureEntraGroupMemberArgs;
use crate::cli::azure::entra::group::member::AzureEntraGroupMemberCommand;
use crate::cli::azure::entra::group::member::AzureEntraGroupMemberRemoveArgs;
use clap::Args;
use cloud_terrastodon_azure::prelude::GroupId;
use cloud_terrastodon_azure::prelude::GroupMemberRemoveRequest;
use cloud_terrastodon_azure::prelude::Principal;
use cloud_terrastodon_azure::prelude::fetch_group_members;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsLicenseAssignmentSource;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsUserId;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsUserLicenseEntitlement;
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
    pub devops_user_id: Option<AzureDevOpsUserId>,
    #[arg(long)]
    pub user_email: Option<String>,
}

impl AzureDevOpsLicenseEntitlementUserRevokeArgs {
    pub async fn invoke(self) -> Result<()> {
        let user_matcher: Box<dyn Fn(&AzureDevOpsUserLicenseEntitlement) -> bool> =
            match (&self.devops_user_id, &self.user_email) {
                (None, None) => {
                    bail!("No user filter was provided");
                }
                (Some(devops_user_id), None) => Box::new(move |e| e.user_id == *devops_user_id),
                (None, Some(user_email)) => {
                    Box::new(move |e| e.user.unique_name.eq_ignore_ascii_case(&user_email))
                }
                (Some(devops_user_id), Some(user_email)) => Box::new(move |e| {
                    e.user_id == *devops_user_id
                        && e.user.unique_name.eq_ignore_ascii_case(&user_email)
                }),
            };

        let org_url = get_default_organization_url().await?;

        let entitlements = fetch_azure_devops_user_license_entitlements(&org_url).await?;

        let user_entitlement = entitlements
            .into_iter()
            .find(|e| user_matcher(e))
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

        // If the assignment isn't group-rule based, nothing to do here.
        if user_entitlement.assignment_source != AzureDevOpsLicenseAssignmentSource::GroupRule {
            bail!(
                "User's entitlement is not assigned by group rules (assignment source: {:?}) - todo: impl direct revocation",
                user_entitlement.assignment_source
            );
        }

        // Fetch groups that are granting licenses
        let group_entitlements = fetch_azure_devops_group_license_entitlements(&org_url).await?;

        let mut actions = Vec::new();

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
                .filter_map(|p: &Principal| match p.as_user() {
                    Some(user) => (user.user_principal_name == user_principal_name).then_some(user),
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

            if let Some(user) = user_in_group {
                info!(
                    user_devops_id = %user_entitlement.user_id,
                    user_display_name = %user_entitlement.user.display_name,
                    user_devops_license = ?user_entitlement.license,
                    group_display_name = %group_license_entitlement.group.display_name,
                    group_origin_id = %group_license_entitlement.group.origin_id,
                    "User is granted license via this group - remove the user from this group to revoke the license"
                );
                actions.push(GroupMemberRemoveRequest {
                    group_id: group_entra_id,
                    member_id: user.id.into(),
                });
                // actions.push(CloudTerrastodonCommand::Azure(AzureArgs {
                //     command: AzureCommand::Entra(AzureEntraArgs {
                //         command: AzureEntraCommand::Group(AzureEntraGroupArgs {
                //             command: AzureEntraGroupCommand::Member(AzureEntraGroupMemberArgs {
                //                 command: AzureEntraGroupMemberCommand::Remove(
                //                     AzureEntraGroupMemberRemoveArgs {
                //                         group_id: group_entra_id,
                //                         member_id: user.id.into(),
                //                     },
                //                 ),
                //             }),
                //         }),
                //     }),
                // }))
            }
        }

        // for action in actions {
        //     println!(
        //         "{} {}",
        //         current_exe()?.to_string_lossy(),
        //         action
        //             .to_args()
        //             .iter()
        //             .map(|s| s.to_string_lossy())
        //             .collect::<Vec<_>>()
        //             .join(" ")
        //     );
        // }

        Ok(())
    }
}
