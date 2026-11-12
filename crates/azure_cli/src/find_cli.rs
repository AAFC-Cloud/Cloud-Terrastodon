use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureResourceListRequest;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use nucleo::Nucleo;
use nucleo::pattern::CaseMatching;
use nucleo::pattern::Normalization;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::info;

/// Find Azure resources where the serialized JSON contains the provided text.
#[derive(Facet, Debug, Clone)]
pub struct AzureFindArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Text to search for in each resource JSON payload.
    #[facet(figue::positional)]
    pub text: String,
}

impl AzureFindArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(needle = %self.text, "Fetching all Azure resources");
        let resources = AzureResourceListRequest { authentication }
            .execute(executor)
            .await?;
        info!(count = resources.len(), "Fetched Azure resources");

        let serialized_resources = resources
            .iter()
            .map(|value| facet_json::to_string(value).map_err(|error| eyre::eyre!("{error:?}")))
            .collect::<Result<Vec<_>, _>>()?;

        let mut nucleo: Nucleo<usize> =
            Nucleo::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 1);
        for idx in 0..resources.len() {
            nucleo.injector().push(idx, |x, cols| {
                cols[0] = serialized_resources[*x].as_str().into();
            });
        }

        nucleo.pattern.reparse(
            0,
            &self.text,
            CaseMatching::Ignore,
            Normalization::Smart,
            false,
        );
        let _ = nucleo.tick(10_000);

        let snapshot = nucleo.snapshot();
        let matched_indices: HashSet<usize> =
            snapshot.matched_items(..).map(|item| *item.data).collect();
        let matches: Vec<Arc<Resource>> = resources
            .into_iter()
            .enumerate()
            .filter_map(|(idx, resource)| matched_indices.contains(&idx).then_some(resource))
            .collect();

        info!(needle = %self.text, matches = matches.len(), "Completed resource JSON search");

        Ok(CliOutput::facet(matches))
    }
}
