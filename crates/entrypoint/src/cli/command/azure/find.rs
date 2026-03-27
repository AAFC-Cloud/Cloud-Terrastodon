use clap::Args;
use cloud_terrastodon_azure::prelude::AzureTenantArgument;
use cloud_terrastodon_azure::prelude::AzureTenantArgumentExt;
use cloud_terrastodon_azure::prelude::Resource;
use cloud_terrastodon_azure::prelude::fetch_all_resources;
use eyre::Result;
use nucleo::Nucleo;
use nucleo::pattern::CaseMatching;
use nucleo::pattern::Normalization;
use std::collections::HashSet;
use std::io::Write;
use std::sync::Arc;
use tracing::info;

/// Find Azure resources where the serialized JSON contains the provided text.
#[derive(Args, Debug, Clone)]
pub struct AzureFindArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// Text to search for in each resource JSON payload.
    pub text: String,
}

impl AzureFindArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(needle = %self.text, "Fetching all Azure resources");
        let resources = fetch_all_resources(tenant_id).await?;
        info!(count = resources.len(), "Fetched Azure resources");

        let serialized_resources = resources
            .iter()
            .map(serde_json::to_string)
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
        let matches: Vec<Resource> = resources
            .into_iter()
            .enumerate()
            .filter_map(|(idx, resource)| matched_indices.contains(&idx).then_some(resource))
            .collect();

        info!(needle = %self.text, matches = matches.len(), "Completed resource JSON search");

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &matches)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
