use super::azure_entra_oauth2_permission_grant_claim_list_output::AzureEntraOAuth2PermissionGrantClaimListOutput;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::EntraOAuth2PermissionScopeListRequest;
use cloud_terrastodon_azure::EntraServicePrincipal;
use cloud_terrastodon_azure::EntraServicePrincipalListRequest;
use cloud_terrastodon_azure::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure::EntraServicePrincipalRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use tracing::info;

/// List the delegated OAuth2 permission claims exposed by a service principal.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraOAuth2PermissionGrantClaimListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Resource service principal object id, app id, display name, or service principal name.
    #[facet(figue::positional)]
    pub service_principal: String,
}

impl AzureEntraOAuth2PermissionGrantClaimListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let service_principal_id =
            resolve_service_principal_id(authentication, self.service_principal.trim(), executor)
                .await?;

        info!(
            tenant_id = %authentication.tenant_id,
            %service_principal_id,
            "Fetching delegated OAuth2 permission claims"
        );
        let scopes = EntraOAuth2PermissionScopeListRequest {
            service_principal_id: (service_principal_id),
            authentication,
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::new(
            AzureEntraOAuth2PermissionGrantClaimListOutput { scopes },
        ))
    }
}

async fn resolve_service_principal_id(
    authentication: AzureBearerAuthenticationSelection,
    needle: &str,
    executor: &CloudTerrastodonExecutor,
) -> Result<EntraServicePrincipalObjectId> {
    if let Ok(service_principal_id) = needle.parse::<EntraServicePrincipalObjectId>() {
        match (EntraServicePrincipalRequest {
            service_principal_id: (service_principal_id),
            authentication,
        })
        .execute(executor)
        .await
        {
            Ok(_) => return Ok(service_principal_id),
            Err(error) => {
                info!(
                    %error,
                    "Object-id lookup did not match; checking alternate service-principal identifiers"
                );
            }
        }
    }

    let service_principals = EntraServicePrincipalListRequest { authentication }
        .execute(executor)
        .await?;
    let mut matches = service_principals
        .iter()
        .filter(|service_principal| matches_service_principal(service_principal, needle))
        .collect::<Vec<_>>();

    match matches.len() {
        0 => {
            bail!("No service principal found matching '{}'.", needle);
        }
        1 => Ok(matches.remove(0).id),
        _ => {
            matches.sort_by_key(|service_principal| service_principal.id.to_string());
            let ids = matches
                .iter()
                .map(|service_principal| service_principal.id.to_string())
                .collect::<Vec<_>>()
                .join("\n  ");
            bail!(
                "Multiple service principals matched '{}'. Use a full object id.\n  {}",
                needle,
                ids
            );
        }
    }
}

fn matches_service_principal(service_principal: &EntraServicePrincipal, needle: &str) -> bool {
    service_principal.id.to_string() == needle
        || service_principal.app_id.to_string() == needle
        || service_principal.display_name.eq_ignore_ascii_case(needle)
        || service_principal
            .app_display_name
            .as_deref()
            .map(|value| value.eq_ignore_ascii_case(needle))
            .unwrap_or(false)
        || service_principal
            .service_principal_names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(needle))
}

#[cfg(test)]
mod tests {
    use super::matches_service_principal;
    use cloud_terrastodon_azure::EntraServicePrincipal;

    fn sample_service_principal() -> EntraServicePrincipal {
        EntraServicePrincipal {
            account_enabled: true,
            add_ins: Vec::new(),
            alternative_names: Vec::new(),
            app_description: None,
            app_display_name: None,
            app_id: "22222222-2222-2222-2222-222222222222"
                .parse()
                .expect("fixture app id should parse"),
            app_owner_organization_id: None,
            app_role_assignment_required: false,
            app_roles: Vec::new(),
            application_template_id: None,
            created_date_time: chrono::DateTime::from_timestamp(1_700_000_000, 0)
                .expect("fixture timestamp should be valid"),
            deleted_date_time: None,
            description: None,
            disabled_by_microsoft_status: None,
            display_name: "Contoso Test Service".to_string(),
            homepage: None,
            id: "33333333-3333-3333-3333-333333333333"
                .parse()
                .expect("fixture object id should parse"),
            info: None,
            key_credentials: Vec::new(),
            login_url: None,
            logout_url: None,
            notes: None,
            notification_email_addresses: Vec::new(),
            oauth2_permission_scopes: Vec::new(),
            password_credentials: Vec::new(),
            preferred_single_sign_on_mode: None,
            preferred_token_signing_key_thumbprint: None,
            reply_urls: Vec::new(),
            resource_specific_application_permissions: Vec::new(),
            saml_single_sign_on_settings: None,
            service_principal_names: Vec::new(),
            service_principal_type: "Application".to_string(),
            sign_in_audience: None,
            tags: Vec::new(),
            token_encryption_key_id: None,
            verified_publisher: None,
        }
    }

    #[test]
    fn matches_by_app_id() {
        let mut service_principal = sample_service_principal();
        assert!(!matches_service_principal(
            &service_principal,
            "11111111-1111-1111-1111-111111111111"
        ));
        service_principal.app_id = "11111111-1111-1111-1111-111111111111"
            .parse()
            .expect("test app id should parse");

        assert!(matches_service_principal(
            &service_principal,
            "11111111-1111-1111-1111-111111111111"
        ));
    }

    #[test]
    fn matches_by_display_name_case_insensitively() {
        let mut service_principal = sample_service_principal();
        assert!(!matches_service_principal(
            &service_principal,
            "microsoft graph"
        ));
        service_principal.display_name = "Microsoft Graph".to_string();

        assert!(matches_service_principal(
            &service_principal,
            "microsoft graph"
        ));
    }
}
