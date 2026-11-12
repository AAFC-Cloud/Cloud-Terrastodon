use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_gitea::GiteaOrganizationArgument;
use cloud_terrastodon_gitea::GiteaOrganizationListRequest;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;

#[derive(Facet, Debug, Clone)]
pub struct GiteaOrgShowArgs {
    /// Organization id or name.
    #[facet(figue::positional, proxy = String)]
    pub organization: GiteaOrganizationArgument<'static>,

    /// Tracked tenant URL or alias to query. Select an instance explicitly; account defaults do not select service scope.
    #[facet(figue::named, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
    /// Account ID or alias for this instance; omission selects its configured default.
    #[facet(figue::named)]
    pub account: Option<CloudTerrastodonAccountArgument>,
}

impl GiteaOrgShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let tenant = self.tenant.resolve(executor.cancellation()).await?;
        let account = self.account.as_ref().map(ToString::to_string);
        let account_id =
            executor.resolve_gitea_authentication(tenant.as_ref(), account.as_deref())?;
        let organizations = GiteaOrganizationListRequest {
            account_id,
            tenant: Cow::Borrowed(&tenant),
        }
        .execute(executor)
        .await?;
        let mut matches = organizations
            .into_iter()
            .filter(|organization| self.organization.matches(organization))
            .collect::<Vec<_>>();
        match matches.len() {
            1 => Ok(CliOutput::facet(matches.remove(0))),
            0 => {
                bail!(
                    "No Gitea organization found matching '{}'.",
                    self.organization
                );
            }
            _ => {
                bail!(
                    "Multiple Gitea organizations matched '{}'. Please specify the numeric id.",
                    self.organization
                );
            }
        }
    }
}
