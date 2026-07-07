use crate::GiteaInstanceUrl;
use crate::GiteaRepoEnumerationComparisonReport;
use crate::GiteaRepoEnumerationMethod;
use crate::GiteaRepoEnumerationMethodReport;
use crate::GiteaRepoId;
use crate::fetch_all_gitea_organization_repositories;
use crate::fetch_all_gitea_organizations;
use crate::fetch_all_gitea_repositories;
use crate::fetch_all_gitea_repositories_via_search;
use crate::fetch_all_gitea_user_repositories;
use crate::fetch_all_gitea_users;
use crate::fetch_current_user_gitea_repositories;
use crate::fetch_gitea_repositories_by_id_range;
use arbitrary::Arbitrary;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct GiteaRepoEnumerationAnalysisRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub max_repo_id: u64,
}

pub fn analyze_gitea_repo_enumeration<'a>(
    tenant: &'a GiteaInstanceUrl,
    max_repo_id: u64,
) -> GiteaRepoEnumerationAnalysisRequest<'a> {
    GiteaRepoEnumerationAnalysisRequest {
        tenant: Cow::Borrowed(tenant),
        max_repo_id,
    }
}

impl<'a> Arbitrary<'a> for GiteaRepoEnumerationAnalysisRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            max_repo_id: u64::arbitrary(u)?,
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaRepoEnumerationAnalysisRequest<'a> {
    type Output = GiteaRepoEnumerationComparisonReport;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "tea",
            self.tenant.storage_key().as_str(),
            "repositories",
            "enumeration-analysis",
            self.max_repo_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let tenant = self.tenant;
        let organizations = fetch_all_gitea_organizations(tenant.as_ref()).await?;
        let mut org_repositories = Vec::new();
        for organization in &organizations {
            org_repositories.extend(
                fetch_all_gitea_organization_repositories(tenant.as_ref(), &organization.username)
                    .await?,
            );
        }

        let users = fetch_all_gitea_users(tenant.as_ref()).await?;
        let mut user_repositories = Vec::new();
        for user in &users {
            user_repositories
                .extend(fetch_all_gitea_user_repositories(tenant.as_ref(), &user.login).await?);
        }

        let current_user_repositories =
            fetch_current_user_gitea_repositories(tenant.as_ref()).await?;
        let search_repositories = fetch_all_gitea_repositories_via_search(tenant.as_ref()).await?;
        let id_range_repositories =
            fetch_gitea_repositories_by_id_range(tenant.as_ref(), 1, self.max_repo_id).await?;
        let combined_repositories = fetch_all_gitea_repositories(tenant.as_ref()).await?;

        let organizations_report = method_report(
            GiteaRepoEnumerationMethod::Organizations,
            organizations.len() + 1,
            org_repositories.iter().map(|repo| repo.id),
        );
        let users_report = method_report(
            GiteaRepoEnumerationMethod::Users,
            users.len() + 1,
            user_repositories.iter().map(|repo| repo.id),
        );
        let current_user_report = method_report(
            GiteaRepoEnumerationMethod::CurrentUser,
            1,
            current_user_repositories.iter().map(|repo| repo.id),
        );
        let search_report = method_report(
            GiteaRepoEnumerationMethod::Search,
            1,
            search_repositories.iter().map(|repo| repo.id),
        );
        let id_range_report = method_report(
            GiteaRepoEnumerationMethod::IdRange,
            self.max_repo_id as usize,
            id_range_repositories.iter().map(|repo| repo.id),
        );
        let combined_report = method_report(
            GiteaRepoEnumerationMethod::Combined,
            organizations_report.request_count
                + users_report.request_count
                + current_user_report.request_count
                + search_report.request_count,
            combined_repositories.iter().map(|repo| repo.id),
        );

        Ok(GiteaRepoEnumerationComparisonReport {
            search_missing_from_combined: difference(
                &search_report.repo_ids,
                &combined_report.repo_ids,
            ),
            organizations_missing_from_search: difference(
                &organizations_report.repo_ids,
                &search_report.repo_ids,
            ),
            users_missing_from_search: difference(&users_report.repo_ids, &search_report.repo_ids),
            current_user_missing_from_search: difference(
                &current_user_report.repo_ids,
                &search_report.repo_ids,
            ),
            id_range_missing_from_search: difference(
                &id_range_report.repo_ids,
                &search_report.repo_ids,
            ),
            organizations: organizations_report,
            users: users_report,
            current_user: current_user_report,
            search: search_report,
            id_range: id_range_report,
            combined: combined_report,
        })
    }
}

fn method_report(
    method: GiteaRepoEnumerationMethod,
    request_count: usize,
    repo_ids: impl IntoIterator<Item = GiteaRepoId>,
) -> GiteaRepoEnumerationMethodReport {
    let mut ids = repo_ids
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    ids.sort();
    GiteaRepoEnumerationMethodReport {
        method,
        request_count,
        repo_count: ids.len(),
        repo_ids: ids,
    }
}

fn difference(left: &[GiteaRepoId], right: &[GiteaRepoId]) -> Vec<GiteaRepoId> {
    let right = right.iter().copied().collect::<BTreeSet<_>>();
    left.iter()
        .copied()
        .filter(|repo_id| !right.contains(repo_id))
        .collect()
}

cloud_terrastodon_command::impl_cacheable_into_future!(
    GiteaRepoEnumerationAnalysisRequest<'a>,
    'a
);

cloud_terrastodon_registry::register_thing!(GiteaRepoEnumerationAnalysisRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaRepoEnumerationAnalysisRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    GiteaRepoEnumerationAnalysisRequest<'static> => GiteaRepoEnumerationComparisonReport,
    effects = [Read]
);
