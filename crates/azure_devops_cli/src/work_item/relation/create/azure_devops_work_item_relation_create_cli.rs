use crate::work_item::relation::AzureDevOpsWorkItemRelationWriteArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemRelationCreateArgs {
    #[facet(flatten)]
    pub args: AzureDevOpsWorkItemRelationWriteArgs,
}

impl AzureDevOpsWorkItemRelationCreateArgs {
    pub async fn invoke(
        self,
        auth: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        self.args.invoke(auth, true, executor).await
    }
}
