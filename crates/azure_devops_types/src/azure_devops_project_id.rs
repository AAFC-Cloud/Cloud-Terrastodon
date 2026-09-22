use arbitrary::Arbitrary;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Clone, Hash, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct AzureDevOpsProjectId(Uuid);

cloud_terrastodon_azure_types::impl_uuid_newtype!(AzureDevOpsProjectId);

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectId);

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn it_works() {
        let id = Uuid::new_v4().to_string();
        let _project_id = id.parse::<AzureDevOpsProjectId>().unwrap();
    }
}
