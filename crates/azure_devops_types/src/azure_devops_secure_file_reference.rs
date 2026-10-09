use crate::AzureDevOpsSecureFileId;
use arbitrary::Arbitrary;

/// A secure-file resource reference used by a build process.
///
/// Microsoft REST documentation: [containing `BuildDefinition.process` property](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#builddefinition).
/// For this resource-reference shape, see Microsoft's
/// [SecureFileReference interface](https://learn.microsoft.com/en-us/javascript/api/azure-devops-extension-api/securefilereference)
/// and [TypeScript source](https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/BuildInterfaces.ts#L2609).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsSecureFileReference {
    pub id: Option<AzureDevOpsSecureFileId>,
    pub alias: Option<String>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsSecureFileReference);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsSecureFileReference);
