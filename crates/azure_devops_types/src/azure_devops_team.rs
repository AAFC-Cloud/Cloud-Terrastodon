use crate::AzureDevOpsProjectId;
use crate::AzureDevOpsTeamId;
use arbitrary::Arbitrary;
use cloud_terrastodon_hcl_types::AzureDevOpsResourceBlockKind;
use cloud_terrastodon_hcl_types::HclImportBlock;
use cloud_terrastodon_hcl_types::HclProviderReference;
use cloud_terrastodon_hcl_types::ResourceBlockReference;
use cloud_terrastodon_hcl_types::Sanitizable;

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsTeam {
    pub description: String,
    pub id: AzureDevOpsTeamId,
    pub identity_url: String,
    pub name: String,
    pub project_id: AzureDevOpsProjectId,
    pub project_name: String,
    pub url: String,
}

impl From<AzureDevOpsTeam> for HclImportBlock {
    fn from(team: AzureDevOpsTeam) -> Self {
        HclImportBlock {
            provider: HclProviderReference::Inherited,
            id: format!("{}/{}", team.project_id, *team.id),
            to: ResourceBlockReference::AzureDevOps {
                kind: AzureDevOpsResourceBlockKind::Team,
                name: format!("project_{}_team_{}", team.project_name, team.name).sanitize(),
            },
        }
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsTeam);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsTeam);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsTeam>);
