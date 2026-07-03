use arbitrary::Arbitrary;
use crate::GiteaRepoEnumerationMethod;
use crate::GiteaRepoId;
use facet::Facet;

#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, Facet)]
pub struct GiteaRepoEnumerationMethodReport {
    pub method: GiteaRepoEnumerationMethod,
    pub request_count: usize,
    pub repo_count: usize,
    pub repo_ids: Vec<GiteaRepoId>,
}

#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, Facet)]
pub struct GiteaRepoEnumerationComparisonReport {
    pub organizations: GiteaRepoEnumerationMethodReport,
    pub users: GiteaRepoEnumerationMethodReport,
    pub current_user: GiteaRepoEnumerationMethodReport,
    pub search: GiteaRepoEnumerationMethodReport,
    pub id_range: GiteaRepoEnumerationMethodReport,
    pub combined: GiteaRepoEnumerationMethodReport,
    pub search_missing_from_combined: Vec<GiteaRepoId>,
    pub organizations_missing_from_search: Vec<GiteaRepoId>,
    pub users_missing_from_search: Vec<GiteaRepoId>,
    pub current_user_missing_from_search: Vec<GiteaRepoId>,
    pub id_range_missing_from_search: Vec<GiteaRepoId>,
}

cloud_terrastodon_registry::register_thing!(GiteaRepoEnumerationComparisonReport);
cloud_terrastodon_registry::register_arbitrary!(GiteaRepoEnumerationComparisonReport);

cloud_terrastodon_registry::register_thing!(GiteaRepoEnumerationMethodReport);
cloud_terrastodon_registry::register_arbitrary!(GiteaRepoEnumerationMethodReport);

