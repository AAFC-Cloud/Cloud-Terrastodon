use crate::cli::azure_devops::work_item::relation::AzureDevOpsWorkItemRelationWriteArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemRelationCreateArgs {
    #[facet(flatten)]
    pub args: AzureDevOpsWorkItemRelationWriteArgs,
}

impl AzureDevOpsWorkItemRelationCreateArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        self.args.invoke(auth, true).await
    }
}
