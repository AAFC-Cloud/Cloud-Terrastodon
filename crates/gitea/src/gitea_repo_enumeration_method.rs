use facet::Facet;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Facet)]
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

