use crate::work_item::relation::AzureDevOpsWorkItemRelationWriteArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemRelationUpdateArgs {
    #[facet(flatten)]
    pub args: AzureDevOpsWorkItemRelationWriteArgs,
}

impl AzureDevOpsWorkItemRelationUpdateArgs {
    pub async fn invoke(
        self,
        auth: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        self.args.invoke(auth, false, executor).await
    }
}
