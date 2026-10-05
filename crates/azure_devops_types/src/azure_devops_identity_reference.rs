#![allow(
    deprecated,
    reason = "Generated reflection in this wire-record module must retain deprecated fields"
)]

use crate::AzureDevOpsDescriptor;
use crate::AzureDevOpsIdentityDirectoryAlias;
use crate::AzureDevOpsIdentityDisplayName;
use crate::AzureDevOpsIdentityId;
use crate::AzureDevOpsIdentityImageUrl;
use crate::AzureDevOpsIdentityUniqueName;
use crate::AzureDevOpsIdentityUrl;
use crate::AzureDevOpsReferenceLinkUrl;
use crate::AzureDevOpsReferenceLinks;
use arbitrary::Arbitrary;

/// A shallow reference to an Azure DevOps identity, including people, groups,
/// and processes. Missing identity metadata remains unknown.
///
/// Models Microsoft's [IdentityRef 7.1 schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/list?view=azure-devops-rest-7.1#identityref),
/// including source, image and profile URLs, hypermedia links and legacy provider
/// metadata. Microsoft's [IdentityRef](https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/common/VSSInterfaces.ts)
/// and [GraphSubjectBase](https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/GraphInterfaces.ts)
/// definitions allow metadata to be absent. Debug decoding still rejects unmodeled fields.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsIdentityReference {
    pub id: Option<AzureDevOpsIdentityId>,
    pub descriptor: Option<AzureDevOpsDescriptor>,
    pub display_name: Option<AzureDevOpsIdentityDisplayName>,
    pub url: Option<AzureDevOpsIdentityUrl>,
    #[facet(rename = "_links")]
    pub links: Option<AzureDevOpsReferenceLinks>,
    /// Legacy provider name, which is not necessarily an email address.
    #[deprecated(
        note = "Deprecated by Microsoft; use the Graph user's domain and principalName instead"
    )]
    pub unique_name: Option<AzureDevOpsIdentityUniqueName>,
    /// Deprecated in favor of the `avatar` relation in `links`.
    #[deprecated(note = "Deprecated by Microsoft; use the avatar relation in links instead")]
    pub image_url: Option<AzureDevOpsIdentityImageUrl>,
    /// Legacy directory alias, when supplied by the provider.
    #[deprecated(
        note = "Deprecated by Microsoft; retrieve directoryAlias from the Graph user referenced by the self relation in links"
    )]
    pub directory_alias: Option<AzureDevOpsIdentityDirectoryAlias>,
    #[deprecated(
        note = "Deprecated by Microsoft; retrieve membership state through the Graph user's membershipState link"
    )]
    pub inactive: Option<bool>,
    #[deprecated(
        note = "Deprecated by Microsoft; infer AAD user/group identity from descriptor instead"
    )]
    pub is_aad_identity: Option<bool>,
    #[deprecated(
        note = "Deprecated by Microsoft; infer whether the identity is a group from descriptor instead"
    )]
    pub is_container: Option<bool>,
    pub is_deleted_in_origin: Option<bool>,
    /// Legacy profile link, usually omitted by the service.
    #[deprecated(
        note = "Deprecated by Microsoft; this legacy profile property is usually unused and no replacement is documented"
    )]
    pub profile_url: Option<AzureDevOpsReferenceLinkUrl>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsIdentityReference);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsIdentityReference);
