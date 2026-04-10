use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraServicePrincipal;
use cloud_terrastodon_azure::fetch_all_service_principals;
use eyre::Result;
use eyre::bail;
use std::io::Write;
use tracing::info;

/// Show a single Entra (Azure AD) service principal.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraSpShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// Service principal object id, app id, display name, or service principal name.
    pub service_principal: String,
}

impl AzureEntraSpShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(needle = %self.service_principal, %tenant_id, "Fetching service principals");
        let service_principals = fetch_all_service_principals(tenant_id).await?;
        info!(
            count = service_principals.len(),
            "Fetched service principals"
        );

        let needle = self.service_principal.trim();
        let mut matches = service_principals
            .into_iter()
            .filter(|service_principal| matches_service_principal(service_principal, needle))
            .collect::<Vec<_>>();

        match matches.len() {
            0 => bail!("No service principal found matching '{}'.", needle),
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                serde_json::to_writer_pretty(&mut handle, &matches.remove(0))?;
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
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use cloud_terrastodon_azure::EntraServicePrincipal;

    fn sample_service_principal() -> EntraServicePrincipal {
        let data = (0u8..=255).cycle().take(4096).collect::<Vec<_>>();
        let mut unstructured = Unstructured::new(&data);
        EntraServicePrincipal::arbitrary(&mut unstructured)
            .expect("sample service principal should be generated from arbitrary")
    }

    #[test]
    fn matches_by_service_principal_name() {
        let mut service_principal = sample_service_principal();
        service_principal.service_principal_names = vec!["api://graph-client".to_string()];

        assert!(matches_service_principal(
            &service_principal,
            "api://graph-client"
        ));
    }

    #[test]
    fn matches_by_app_id() {
        let mut service_principal = sample_service_principal();
        service_principal.app_id = "11111111-1111-1111-1111-111111111111"
            .parse()
            .expect("test app id should parse");

        assert!(matches_service_principal(
            &service_principal,
            "11111111-1111-1111-1111-111111111111"
        ));
    }
}
