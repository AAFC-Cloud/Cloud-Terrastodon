use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::GiteaRepoId;
use crate::gitea_api_support::dedupe_repositories;
use crate::try_fetch_gitea_repository_by_id;
use arbitrary::Arbitrary;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::borrow::Cow;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct GiteaRepoScanByIdRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub start_id: u64,
    pub end_id: u64,
}

pub fn fetch_gitea_repositories_by_id_range<'a>(
    tenant: &'a GiteaInstanceUrl,
    start_id: u64,
    end_id: u64,
) -> GiteaRepoScanByIdRequest<'a> {
    GiteaRepoScanByIdRequest {
        tenant: Cow::Borrowed(tenant),
        start_id,
        end_id,
    }
}

impl<'a> Arbitrary<'a> for GiteaRepoScanByIdRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            start_id: u64::arbitrary(u)?,
            end_id: u64::arbitrary(u)?,
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaRepoScanByIdRequest<'a> {
    type Output = Vec<GiteaRepo>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "tea",
            self.tenant.storage_key().as_str(),
            "repositories",
            "scan-by-id",
            self.start_id.to_string().as_str(),
            self.end_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let tenant = self.tenant;
        let mut repositories = Vec::new();
        for repo_id in self.start_id..=self.end_id {
            if let Some(repo) =
                try_fetch_gitea_repository_by_id(tenant.as_ref(), GiteaRepoId::new(repo_id)).await?
            {
                repositories.push(repo);
            }
        }
        Ok(dedupe_repositories(repositories))
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaRepoScanByIdRequest<'a>, 'a);

cloud_terrastodon_registry::register_thing!(GiteaRepoScanByIdRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaRepoScanByIdRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    GiteaRepoScanByIdRequest<'static> => Vec<GiteaRepo>,
    effects = [Read]
);
