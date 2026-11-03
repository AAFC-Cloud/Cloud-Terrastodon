use crate::work_item::relation::AzureDevOpsWorkItemRelationWriteArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemRelationCreateArgs {
    #[facet(flatten)]
    pub args: AzureDevOpsWorkItemRelationWriteArgs,
}

impl AzureDevOpsWorkItemRelationCreateArgs {
    pub async fn invoke(self, auth: &AuthContext, cancellation: &CancellationScope) -> Result<()> {
        self.args.invoke(auth, true, cancellation).await
    }
}
