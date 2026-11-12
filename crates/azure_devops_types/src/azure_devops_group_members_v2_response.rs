use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;
use facet::Facet;
use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct AzureDevOpsGroupMembersV2Response(pub ArbitraryJson);

impl<'a> Arbitrary<'a> for AzureDevOpsGroupMembersV2Response {
    fn arbitrary(_u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        // Keep the response's object envelope when generating offline values.
        Ok(Self(ArbitraryJson::object()))
    }
}

impl Deref for AzureDevOpsGroupMembersV2Response {
    type Target = ArbitraryJson;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsGroupMembersV2Response);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGroupMembersV2Response);
