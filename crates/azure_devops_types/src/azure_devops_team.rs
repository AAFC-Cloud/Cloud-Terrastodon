use crate::AzureDevOpsProjectId;
use arbitrary::Arbitrary;
use cloud_terrastodon_hcl_types::AzureDevOpsResourceBlockKind;
use cloud_terrastodon_hcl_types::HclImportBlock;
use cloud_terrastodon_hcl_types::HclProviderReference;
use cloud_terrastodon_hcl_types::ResourceBlockReference;
use cloud_terrastodon_hcl_types::Sanitizable;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Clone, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct AzureDevOpsTeamId(Uuid);
impl std::fmt::Display for AzureDevOpsTeamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for AzureDevOpsTeamId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AzureDevOpsTeamId {
    pub fn new(uuid: Uuid) -> AzureDevOpsTeamId {
        AzureDevOpsTeamId(uuid)
    }
}

impl From<&AzureDevOpsTeamId> for String {
    fn from(value: &AzureDevOpsTeamId) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for AzureDevOpsTeamId {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl FromStr for AzureDevOpsTeamId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)?;
        Ok(AzureDevOpsTeamId::new(uuid))
    }
}

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

cloud_terrastodon_registry::register_thing!(AzureDevOpsTeamId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsTeamId);
cloud_terrastodon_registry::register_thing!(AzureDevOpsTeam);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsTeam);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsTeam>);
