use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraUserArgument;
use cloud_terrastodon_azure::fetch_entra_user;
use cloud_terrastodon_azure::fetch_group_members;
use cloud_terrastodon_azure::remove_group_member;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseAssignmentSource;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_group_license_entitlements;
use cloud_terrastodon_azure_devops::fetch_azure_devops_user_license_entitlements;
use cloud_terrastodon_azure_devops::update_azure_devops_user_license_entitlement;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use eyre::Context;
use eyre::ContextCompat;
use eyre::Result;
use tracing::debug;
use tracing::info;

/// Demote a user to Stakeholder license level, including removing the user from any groups involved in transitive license assignment.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserRevokeArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    #[facet(figue::named, proxy = String)]
    pub user: AzureDevOpsUserArgument<'static>,

    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureDevOpsLicenseEntitlementUserRevokeArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let tenant_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let tenant_id = tenant_auth_context.tenant_id;
        let user_predicate = self.user.as_predicate()?;

        let azure_devops_auth_context =
            AzureDevOpsAuthContext::for_tenant(auth_context, tenant_id)?;
        let entitlements = fetch_azure_devops_user_license_entitlements(
            &self.org,
            &azure_devops_auth_context,
            cancellation,
        )
        .await?;

        let user_entitlement = entitlements
            .into_iter()
            .find(|e| user_predicate(e))
            .wrap_err_with(|| format!("No license entitlement found matching {self:?}"))?;

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
            // Membership listings can expose only object IDs. Resolve the selected
            // user once, rather than interpreting an unavailable UPN as absence.
            let entra_user_id = fetch_entra_user(
                EntraUserArgument::try_from(user_principal_name.as_str())?,
                &tenant_auth_context,
                cancellation,
            )
            .with_invalidation(true)
            .await
            .wrap_err("Resolving the Entra user before removing license-granting memberships")?
            .id;
            // Find the rule that is granting the license and remove the user from that group

            // Fetch groups that are granting licenses
            let group_entitlements =
                fetch_azure_devops_group_license_entitlements(&self.org, cancellation).await?;

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
                let group_entra_members =
                    fetch_group_members(group_entra_id, &tenant_auth_context, cancellation)
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

                    cancellation
                        .deferred(
                            "finishing group-member removal and invalidating its inventory",
                            |protected| {
                                let tenant_auth_context = &tenant_auth_context;
                                async move {
                                    remove_group_member(group_entra_id, entra_user_id, &protected)
                                        .await?;
                                    fetch_group_members(
                                        group_entra_id,
                                        tenant_auth_context,
                                        &protected,
                                    )
                                    .invalidate()
                                    .await
                                }
                            },
                        )
                        .await?;
                }
            }
        } else {
            // Direct assignment - just downgrade the license
            update_azure_devops_user_license_entitlement(
                &self.org,
                user_entitlement.user_id,
                AzureDevOpsLicenseType::AccountStakeholder,
                cancellation,
            )
            .await?;
        }

        Ok(())
    }
}
