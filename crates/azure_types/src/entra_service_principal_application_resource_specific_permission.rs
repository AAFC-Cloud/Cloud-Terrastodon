use arbitrary::Arbitrary;
use facet::Facet;

#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraServicePrincipalApplicationResourceSpecificPermission {
    pub description: String,
    pub display_name: String,
    pub id: String,
    pub is_enabled: bool,
    pub value: String,
}

cloud_terrastodon_registry::register_thing!(
    EntraServicePrincipalApplicationResourceSpecificPermission
);
cloud_terrastodon_registry::register_arbitrary!(
    EntraServicePrincipalApplicationResourceSpecificPermission
);
