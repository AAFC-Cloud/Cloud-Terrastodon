use arbitrary::Arbitrary;
use facet::Facet;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Arbitrary, Facet)]
#[repr(C)]
pub enum GiteaRepoEnumerationMethod {
    Organizations,
    Users,
    CurrentUser,
    Search,
    IdRange,
    Combined,
}

cloud_terrastodon_registry::register_thing!(GiteaRepoEnumerationMethod);
cloud_terrastodon_registry::register_arbitrary!(GiteaRepoEnumerationMethod);

