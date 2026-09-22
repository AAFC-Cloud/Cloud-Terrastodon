use crate::AzureDevOpsProject;
use crate::AzureDevOpsRepoId;
use cloud_terrastodon_hcl_types::AzureDevOpsResourceBlockKind;
use cloud_terrastodon_hcl_types::HclImportBlock;
use cloud_terrastodon_hcl_types::HclProviderReference;
use cloud_terrastodon_hcl_types::ResourceBlockReference;
use cloud_terrastodon_hcl_types::Sanitizable;
use facet_json::RawJson;

#[derive(Debug, Eq, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsRepo {
    pub default_branch: Option<String>,

    pub id: AzureDevOpsRepoId,

    pub is_disabled: bool,

    pub is_fork: Option<bool>,

    pub is_in_maintenance: bool,

    pub name: String,

    pub parent_repository: Option<RawJson<'static>>,

    pub project: AzureDevOpsProject,

    pub remote_url: String,

    pub size: u64,

    pub ssh_url: String,

    pub url: String,

    pub valid_remote_urls: Option<RawJson<'static>>,

    pub web_url: String,
}

impl From<AzureDevOpsRepo> for HclImportBlock {
    fn from(repo: AzureDevOpsRepo) -> Self {
        HclImportBlock {
            provider: HclProviderReference::Inherited,
            id: format!("{}/{}", repo.project.id, *repo.id),
            to: ResourceBlockReference::AzureDevOps {
                kind: AzureDevOpsResourceBlockKind::Repo,
                name: format!("project_{}_repo_{}", repo.project.name, repo.name).sanitize(),
            },
        }
    }
}
