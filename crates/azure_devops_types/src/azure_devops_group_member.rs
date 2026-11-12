use crate::AzureDevOpsDescriptor;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;
use cloud_terrastodon_registry::ARBITRARY_REGISTRATION_POINTS;
use cloud_terrastodon_registry::ArbitraryBytes;
use cloud_terrastodon_registry::ArbitraryRegistration;
use cloud_terrastodon_registry::RegistrationSite;
use cloud_terrastodon_registry::arbitrary_from_bytes;
use cloud_terrastodon_registry::distributed_slice;
use cloud_terrastodon_registry::runtime_from_boxed;
use facet::Facet;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsGroupMember {
    pub description: Option<String>,
    pub descriptor: AzureDevOpsDescriptor,
    pub display_name: String,
    pub domain: String,
    pub legacy_descriptor: Option<ArbitraryJson>,
    pub mail_address: Option<String>,
    pub origin: String,
    pub origin_id: String,
    pub principal_name: String,
    pub subject_kind: String,
    pub url: String,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsGroupMember);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGroupMember);

// Subject lookup indexes each returned member by that member's descriptor.
// Generating HashMap keys and values independently would break that relationship.
#[distributed_slice(ARBITRARY_REGISTRATION_POINTS)]
static ARBITRARY_GROUP_MEMBERS: ArbitraryRegistration = ArbitraryRegistration::new(
    Arc::<HashMap<AzureDevOpsDescriptor, Arc<AzureDevOpsGroupMember>>>::SHAPE,
    |input| {
        let bytes = input.downcast_mut::<ArbitraryBytes>().ok_or_else(|| {
            eyre::eyre!("group-member generation requires mutable ArbitraryBytes")
        })?;
        let members = arbitrary_from_bytes::<Vec<Arc<AzureDevOpsGroupMember>>>(bytes)?;
        let members = members
            .into_iter()
            .map(|member| (member.descriptor.clone(), member))
            .collect::<HashMap<_, _>>();
        Ok(Box::new(Arc::new(members)) as Box<dyn Any + Send>)
    },
    runtime_from_boxed::<Arc<HashMap<AzureDevOpsDescriptor, Arc<AzureDevOpsGroupMember>>>>,
    RegistrationSite::new(file!(), line!()),
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_subject_lookup_indexes_members_by_their_own_descriptors() {
        let mut entropy = vec![0; 4096];
        entropy[0] = 1; // Admit one member; the remainder supplies empty optional data.
        let generated = ARBITRARY_GROUP_MEMBERS
            .generate(&mut ArbitraryBytes::new(entropy))
            .unwrap();
        let members = generated
            .downcast::<Arc<HashMap<AzureDevOpsDescriptor, Arc<AzureDevOpsGroupMember>>>>()
            .unwrap();
        assert!(!members.is_empty());
        for (descriptor, member) in members.iter() {
            assert_eq!(descriptor, &member.descriptor);
        }
    }
}
