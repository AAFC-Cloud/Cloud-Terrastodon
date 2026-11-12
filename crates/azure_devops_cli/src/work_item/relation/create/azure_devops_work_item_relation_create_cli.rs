use crate::work_item::relation::AzureDevOpsWorkItemRelationWriteArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemRelationCreateArgs {
    #[facet(flatten)]
    pub args: AzureDevOpsWorkItemRelationWriteArgs,
}

impl AzureDevOpsWorkItemRelationCreateArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.args.invoke(true, executor).await
    }
}
