use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraServicePrincipal;
use cloud_terrastodon_azure::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure::fetch_all_service_principals;
use cloud_terrastodon_azure::fetch_service_principal;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use eyre::bail;
use std::io::Write;
use tracing::info;

/// Show a single Entra (Azure AD) service principal.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraSpShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Service principal object id, app id, display name, or service principal name.
    #[facet(figue::positional)]
    pub service_principal: String,
}

impl AzureEntraSpShowArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let tenant_auth_context = self.tenant.bind_auth_context(auth_context).await?;
        info!(needle = %self.service_principal, tenant_id = %tenant_auth_context.tenant_id, "Fetching service principals");
        let needle = self.service_principal.trim();

        if let Ok(service_principal_id) = needle.parse::<EntraServicePrincipalObjectId>() {
            match fetch_service_principal(service_principal_id, &tenant_auth_context).await {
                Ok(service_principal) => {
                    let stdout = std::io::stdout();
                    let mut handle = stdout.lock();
                    cloud_terrastodon_command::to_writer_pretty(&mut handle, &service_principal)?;
                    handle.write_all(b"\n")?;
                    return Ok(());
                }
                Err(error) => {
                    info!(
                        %error,
                        "Object-id lookup did not match; checking alternate service-principal identifiers"
                    );
                }
            }
        }

        let service_principals = fetch_all_service_principals(&tenant_auth_context).await?;
        info!(
            count = service_principals.len(),
            "Fetched service principals"
        );

        let mut matches = service_principals
            .into_iter()
            .filter(|service_principal| matches_service_principal(service_principal, needle))
            .collect::<Vec<_>>();

        match matches.len() {
            0 => bail!("No service principal found matching '{}'.", needle),
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                cloud_terrastodon_command::to_writer_pretty(&mut handle, &matches.remove(0))?;
                handle.write_all(b"\n")?;
                Ok(())
            }
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
                )
            }
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
    fn matches_by_service_principal_name() {
        let mut service_principal = sample_service_principal();
        assert!(!matches_service_principal(
            &service_principal,
            "api://graph-client"
        ));
        service_principal.service_principal_names = vec!["api://graph-client".to_string()];

        assert!(matches_service_principal(
            &service_principal,
            "api://graph-client"
        ));
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
}
