use crate::GiteaInstanceUrl;
use crate::GiteaOrganizationListRequest;
use crate::GiteaOrganizationRepoListRequest;
use crate::GiteaRepoEnumerationComparisonReport;
use crate::GiteaRepoEnumerationMethod;
use crate::GiteaRepoEnumerationMethodReport;
use crate::GiteaRepoId;
use crate::GiteaRepoListRequest;
use crate::GiteaRepoScanByIdRequest;
use crate::GiteaRepoSearchRequest;
use crate::GiteaUserCurrentRepoListRequest;
use crate::GiteaUserListRequest;
use crate::GiteaUserRepoListRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::time::Duration;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct GiteaRepoEnumerationAnalysisRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub account_id: CloudTerrastodonAccountId,
    pub max_repo_id: u64,
}

impl<'a> Arbitrary<'a> for GiteaRepoEnumerationAnalysisRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            account_id: CloudTerrastodonAccountId::arbitrary(u)?,
            max_repo_id: u64::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for GiteaRepoEnumerationAnalysisRequest<'_> {}

impl HasCacheKey for GiteaRepoEnumerationAnalysisRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "tea",
                self.tenant.storage_key().as_str(),
                "repositories",
                "enumeration-analysis",
                self.max_repo_id.to_string().as_str(),
            ])?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for GiteaRepoEnumerationAnalysisRequest<'_> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&AuthenticationSelection::Gitea {
            account_id: self.account_id,
        })
    }

    type Error = eyre::Report;
    type Output = GiteaRepoEnumerationComparisonReport;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            let organizations = GiteaOrganizationListRequest {
                account_id: self.account_id,
                tenant: Cow::Borrowed(self.tenant.as_ref()),
            }
            .execute(&executor)
            .await?;
            let mut org_repositories = Vec::new();
            for organization in &organizations {
                executor.cancellation().bail_if_cancelled()?;
                org_repositories.extend(
                    GiteaOrganizationRepoListRequest {
                        account_id: self.account_id,
                        tenant: Cow::Borrowed(self.tenant.as_ref()),
                        organization_name: Cow::Borrowed(&organization.username),
                    }
                    .execute(&executor)
                    .await?,
                );
            }

            let users = GiteaUserListRequest {
                account_id: self.account_id,
                tenant: Cow::Borrowed(self.tenant.as_ref()),
            }
            .execute(&executor)
            .await?;
            let mut user_repositories = Vec::new();
            for user in &users {
                executor.cancellation().bail_if_cancelled()?;
                user_repositories.extend(
                    GiteaUserRepoListRequest {
                        account_id: self.account_id,
                        tenant: Cow::Borrowed(self.tenant.as_ref()),
                        username: Cow::Borrowed(&user.login),
                    }
                    .execute(&executor)
                    .await?,
                );
            }

            let current_user_repositories = GiteaUserCurrentRepoListRequest {
                account_id: self.account_id,
                tenant: Cow::Borrowed(self.tenant.as_ref()),
            }
            .execute(&executor)
            .await?;
            let search_repositories = GiteaRepoSearchRequest {
                account_id: self.account_id,
                tenant: Cow::Borrowed(self.tenant.as_ref()),
            }
            .execute(&executor)
            .await?;
            let id_range_repositories = GiteaRepoScanByIdRequest {
                account_id: self.account_id,
                tenant: Cow::Borrowed(self.tenant.as_ref()),
                start_id: (1),
                end_id: (self.max_repo_id),
            }
            .execute(&executor)
            .await?;
            let combined_repositories = GiteaRepoListRequest {
                account_id: self.account_id,
                tenant: Cow::Borrowed(self.tenant.as_ref()),
            }
            .execute(&executor)
            .await?;

            executor.cancellation().bail_if_cancelled()?;
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
                users_missing_from_search: difference(
                    &users_report.repo_ids,
                    &search_report.repo_ids,
                ),
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

cloud_terrastodon_registry::register_thing!(GiteaRepoEnumerationAnalysisRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaRepoEnumerationAnalysisRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    GiteaRepoEnumerationAnalysisRequest<'static> => GiteaRepoEnumerationComparisonReport,
    effects = [Read]
);

cloud_terrastodon_registry_execution::register_runtime_test!(
    GiteaRepoEnumerationAnalysisRequest<'static>,
    name = "gitea-repo-enumeration-analysis",
    cached
);
