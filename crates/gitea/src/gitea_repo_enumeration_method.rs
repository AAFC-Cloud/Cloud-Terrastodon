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
