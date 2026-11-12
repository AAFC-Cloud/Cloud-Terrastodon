use cloud_terrastodon_app::CliOutputValue;
use cloud_terrastodon_app::OutputFormat;
use cloud_terrastodon_app::write_facet;
use cloud_terrastodon_azure::OAuth2PermissionScope;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use std::io::Write;
use std::sync::Arc;

/// Retain the cached scope records while presenting claims in a stable order.
pub(super) struct AzureEntraOAuth2PermissionGrantClaimListOutput {
    pub scopes: Arc<Vec<OAuth2PermissionScope>>,
}

impl CliOutputValue for AzureEntraOAuth2PermissionGrantClaimListOutput {
    fn write_to(
        &self,
        format: OutputFormat,
        writer: &mut dyn Write,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let mut scopes = self.scopes.iter().collect::<Vec<_>>();
        scopes.sort_by(|left, right| left.value.cmp(&right.value));
        write_facet(&scopes, format, writer, cancellation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_app::CliOutput;
    use cloud_terrastodon_azure::OAuth2PermissionScopeKind;
    use cloud_terrastodon_command::CancellationToken;
    use uuid::Uuid;

    #[test]
    fn presenting_sorted_claims_preserves_the_shared_inventory() -> Result<()> {
        let scopes = Arc::new(
            ["Synthetic.Write", "Synthetic.Read"]
                .into_iter()
                .map(|value| OAuth2PermissionScope {
                    id: Uuid::nil(),
                    admin_consent_description: String::new(),
                    admin_consent_display_name: String::new(),
                    is_enabled: true,
                    kind: OAuth2PermissionScopeKind::User,
                    user_consent_description: String::new(),
                    user_consent_display_name: String::new(),
                    value: value.into(),
                })
                .collect(),
        );
        let output = CliOutput::new(AzureEntraOAuth2PermissionGrantClaimListOutput {
            scopes: Arc::clone(&scopes),
        });
        CancellationToken::new().with_scope(|cancellation| {
            for format in [
                OutputFormat::Json,
                OutputFormat::FacetPretty,
                OutputFormat::Text,
            ] {
                let rendered = output.render(Some(format), false, &cancellation)?.unwrap();
                assert!(
                    rendered.find("Synthetic.Read").unwrap()
                        < rendered.find("Synthetic.Write").unwrap()
                );
            }
            eyre::Ok(())
        })?;
        assert_eq!(scopes[0].value, "Synthetic.Write");
        Ok(())
    }
}
