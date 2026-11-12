use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraGroupMemberRemoveRequest;
use cloud_terrastodon_azure::EntraGroupMembersListRequest;
use cloud_terrastodon_azure::EntraUserArgument;
use cloud_terrastodon_azure::EntraUserGetRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsAccountLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsGroupLicenseEntitlementsRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseAssignmentSource;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsMsdnLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlementListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlementUpdateRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Context;
use eyre::ContextCompat;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use tracing::debug;
use tracing::info;

/// Demote a user to Stakeholder license level, including removing the user from any groups involved in transitive license assignment.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserRevokeArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    #[facet(figue::named, proxy = String)]
    pub user: AzureDevOpsUserArgument<'static>,

    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureDevOpsLicenseEntitlementUserRevokeArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let user_predicate = self.user.as_predicate()?;

        let entitlements = AzureDevOpsUserLicenseEntitlementListRequest {
            org_url: Cow::Borrowed(&self.org),
            auth_selection: authentication.into(),
        }
        .execute(executor)
        .await?;

        let user_entitlement = entitlements
            .into_iter()
            .find(|e| user_predicate(e))
            .wrap_err_with(|| format!("No license entitlement found matching {self:?}"))?;

        debug!(?user_entitlement, "Found entitlement");
        let user_principal_name = &user_entitlement.user.unique_name;

        if matches!(
            &user_entitlement.license,
            AzureDevOpsLicenseType::Account(
                AzureDevOpsAccountLicenseType::Stakeholder | AzureDevOpsAccountLicenseType::None,
            ) | AzureDevOpsLicenseType::Msdn(AzureDevOpsMsdnLicenseType::None)
        ) {
            info!(
                user_devops_id = %user_entitlement.user_id,
                user_display_name = %user_entitlement.user.display_name,
                user_devops_license = ?user_entitlement.license,
                "User does not have a premium license, skipping revocation"
            );
            return Ok(CliOutput::none());
        }

        if user_entitlement.assignment_source == AzureDevOpsLicenseAssignmentSource::GroupRule {
            // Membership listings can expose only object IDs. Resolve the selected
            // user once, rather than interpreting an unavailable UPN as absence.
            let entra_user_id = EntraUserGetRequest {
                lookup: (EntraUserArgument::try_from(user_principal_name.as_str())?),
                authentication,
            }
            .execute(executor)
            .with_invalidation(true)
            .await
            .wrap_err("Resolving the Entra user before removing license-granting memberships")?
            .id;
            // Find the rule that is granting the license and remove the user from that group

            // Fetch groups that are granting licenses
            let group_entitlements = AzureDevOpsGroupLicenseEntitlementsRequest {
                org_url: Cow::Borrowed(&self.org),
                auth_selection: authentication.into(),
            }
            .execute(executor)
            .await?;

            // Identify groups which grant the license that the user has
            for group_license_entitlement in group_entitlements.iter() {
                let matches = group_license_entitlement
                    .license_rule
                    .matches_license(&user_entitlement.license);
                debug!(
                    group_display_name = %group_license_entitlement.group.display_name,
                    group_origin_id = %group_license_entitlement.group.origin_id,
                    group_licensing_source = ?group_license_entitlement.license_rule.licensing_source,
                    group_account_license_type = ?group_license_entitlement.license_rule.account_license_type,
                    group_msdn_license_type = ?group_license_entitlement.license_rule.msdn_license_type,
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
                let group_entra_members = EntraGroupMembersListRequest {
                    group_id: (group_entra_id),
                    authentication,
                }
                .execute(executor)
                .with_invalidation(true)
                .await?;
                let user_in_group = group_entra_members
                    .iter()
                    .any(|principal| principal.id() == entra_user_id);
                debug!(
                    group_display_name = %group_license_entitlement.group.display_name,
                    group_origin_id = %group_license_entitlement.group.origin_id,
                    group_member_count = group_entra_members.len(),
                    user_in_group,
                    "Checked if user is in group granting license"
                );

                if user_in_group {
                    info!(
                        user_devops_id = %user_entitlement.user_id,
                        user_display_name = %user_entitlement.user.display_name,
                        user_devops_license = ?user_entitlement.license,
                        group_display_name = %group_license_entitlement.group.display_name,
                        group_origin_id = %group_license_entitlement.group.origin_id,
                        "User is granted license via this group - removing the user from this group to revoke the license"
                    );

                    executor
                        .deferred(
                            "finishing group-member removal and invalidating its inventory",
                            async |protected| {
                                EntraGroupMemberRemoveRequest {
                                    authentication,
                                    group_id: (group_entra_id),
                                    member_id: (entra_user_id).into(),
                                }
                                .execute(&protected)
                                .await?;
                                EntraGroupMembersListRequest {
                                    group_id: (group_entra_id),
                                    authentication,
                                }
                                .execute(&protected)
                                .invalidate()
                                .await
                            },
                        )
                        .await?;
                }
            }
        } else {
            // Direct assignment - just downgrade the license
            AzureDevOpsUserLicenseEntitlementUpdateRequest {
                org_url: Cow::Borrowed(&self.org),
                auth_selection: authentication.into(),
                user: (user_entitlement.user_id).into(),
                license_kind: (AzureDevOpsLicenseType::Account(
                    AzureDevOpsAccountLicenseType::Stakeholder,
                )),
            }
            .execute(executor)
            .await?;
        }

        Ok(CliOutput::none())
    }
}
