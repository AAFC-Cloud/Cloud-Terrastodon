use crate::AzureDevOpsReferenceLink;
use arbitrary::Arbitrary;
use std::collections::BTreeMap;

/// Azure DevOps hypermedia references keyed by relation name, such as `avatar`.
///
/// See Microsoft's [ReferenceLinks 7.1 schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/list?view=azure-devops-rest-7.1#referencelinks).
/// Relation names are an open dictionary. Each value is a typed link record;
/// the wire representation is the relation dictionary itself.
#[derive(Debug, Clone, Default, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsReferenceLinks(pub BTreeMap<String, AzureDevOpsReferenceLink>);

cloud_terrastodon_registry::register_thing!(AzureDevOpsReferenceLinks);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsReferenceLinks);
