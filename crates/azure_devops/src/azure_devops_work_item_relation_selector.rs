use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItem;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemRelationTypeName;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemUrl;
use eyre::Result;
use eyre::ensure;

#[derive(Debug, Clone, arbitrary::Arbitrary, facet::Facet)]
pub struct AzureDevOpsWorkItemRelationSelector {
    pub rel: AzureDevOpsWorkItemRelationTypeName,
    pub target: AzureDevOpsWorkItemUrl<'static>,
}

impl AzureDevOpsWorkItemRelationSelector {
    pub(crate) fn indices(
        &self,
        organization: &AzureDevOpsOrganizationUrl,
        item: &AzureDevOpsWorkItem,
    ) -> Result<Vec<usize>> {
        let relations = item.relations.as_deref().unwrap_or_default();
        let indices: Vec<_> = relations
            .iter()
            .enumerate()
            .filter(|(_, relation)| {
                if relation.rel != self.rel {
                    return false;
                }
                relation
                    .url
                    .work_item()
                    .is_some_and(|url| url == &self.target)
                    || matches!(
                        (
                            relation.url.work_item_id_for_organization(organization),
                            self.target.work_item_id_for_organization(organization)
                        ),
                        (Ok(a), Ok(b)) if a == b
                    )
            })
            .map(|(index, _)| index)
            .collect();
        Ok(indices)
    }

    pub fn index(
        &self,
        organization: &AzureDevOpsOrganizationUrl,
        item: &AzureDevOpsWorkItem,
    ) -> Result<usize> {
        let indices = self.indices(organization, item)?;
        ensure!(indices.len() == 1, "Expected exactly one matching relation");
        Ok(indices[0])
    }
}
