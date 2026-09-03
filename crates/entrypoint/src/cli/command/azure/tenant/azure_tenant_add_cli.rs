use cloud_terrastodon_azure::AzureTenantAlias;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::add_tracked_tenant;
use cloud_terrastodon_azure::add_tracked_tenant_aliases;
use cloud_terrastodon_azure::set_tracked_tenant_auth_source;
use cloud_terrastodon_credentials::AuthSource;
use eyre::Result;
use std::io::Write;

/// Arguments for adding a tracked Azure tenant.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureTenantAddArgs {
    /// Tenant id (GUID) to track.
    #[facet(figue::positional, proxy = String)]
    pub tenant_id: AzureTenantId,

    /// One or more aliases to associate with the tracked tenant.
    #[facet(figue::named, default)]
    pub alias: Vec<String>,

    /// Default authentication source for this tenant.
    #[facet(figue::named, figue::label = "SOURCE")]
    pub auth_source: Option<AuthSource>,
}

impl AzureTenantAddArgs {
    pub async fn invoke(self) -> Result<()> {
        let aliases = self
            .alias
            .into_iter()
            .map(AzureTenantAlias::try_new)
            .collect::<Result<Vec<_>>>()?;
        let tenant = add_tracked_tenant(self.tenant_id).await?;
        if !aliases.is_empty() {
            add_tracked_tenant_aliases(tenant, &aliases).await?;
        }
        if let Some(auth_source) = self.auth_source {
            set_tracked_tenant_auth_source(tenant, auth_source).await?;
        }

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &tenant)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(facet::Facet, Debug)]
    struct ParseArgs {
        #[facet(flatten)]
        args: AzureTenantAddArgs,
    }

    #[test]
    fn parses_repeated_alias_flags() {
        let parsed: ParseArgs = figue::from_slice(&[
            "00000000-0000-0000-0000-000000000000",
            "--alias",
            "Prod",
            "--alias",
            "Dev",
        ])
        .unwrap();

        assert_eq!(parsed.args.alias, vec!["Prod".to_owned(), "Dev".to_owned()]);
        assert_eq!(parsed.args.auth_source, None);
    }

    #[test]
    fn parses_an_auth_source() {
        let parsed: ParseArgs = figue::from_slice(&[
            "00000000-0000-0000-0000-000000000000",
            "--auth-source",
            "browser",
        ])
        .unwrap();

        assert_eq!(parsed.args.auth_source, Some(AuthSource::Browser));
    }
}
