use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzurePolicyAssignmentListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use nucleo::Config;
use nucleo::Matcher;
use nucleo::pattern::AtomKind;
use nucleo::pattern::CaseMatching;
use nucleo::pattern::Normalization;
use nucleo::pattern::Pattern;
use tracing::info;

/// Arguments for listing Azure policy assignments.
#[derive(Facet, Debug, Clone)]
pub struct AzurePolicyAssignmentListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
    /// Filter policy assignments to those whose name satisfies the provided Nucleo fuzzy expression
    #[facet(figue::named)]
    pub name: Option<String>,
}

impl AzurePolicyAssignmentListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!("Fetching Azure policy assignments...");
        let mut policy_assignments = AzurePolicyAssignmentListRequest { authentication }
            .execute(executor)
            .await?;
        info!(
            count = policy_assignments.len(),
            "Fetched Azure policy assignments",
        );

        if let Some(name) = self.name.as_ref() {
            ensure!(!name.is_empty(), "The name pattern is required");
            let pattern = Pattern::new(
                name,
                CaseMatching::Smart,
                Normalization::Smart,
                AtomKind::Substring,
            );
            let mut matcher = Matcher::new(Config::DEFAULT);
            policy_assignments.retain(|policy_assignment| {
                !pattern
                    .match_list([policy_assignment.name.as_str()], &mut matcher)
                    .is_empty()
            });
        }

        Ok(CliOutput::facet(policy_assignments))
    }
}
