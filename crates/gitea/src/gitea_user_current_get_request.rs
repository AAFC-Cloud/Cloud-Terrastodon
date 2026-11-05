use crate::GiteaInstanceUrl;
use crate::GiteaUser;
use crate::gitea_api_support::gitea_api_get;
use crate::gitea_api_support::tenant_cache_key_prefix;
use arbitrary::Arbitrary;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use std::borrow::Cow;
use std::time::Duration;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct GiteaUserCurrentGetRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
}

pub fn fetch_current_gitea_user<'a>(
    tenant: &'a GiteaInstanceUrl,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, GiteaUserCurrentGetRequest<'a>> {
    executor.execute(GiteaUserCurrentGetRequest {
        tenant: Cow::Borrowed(tenant),
    })
}

impl<'a> Arbitrary<'a> for GiteaUserCurrentGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
        })
    }
}

impl InvalidatesOwnCacheKey for GiteaUserCurrentGetRequest<'_> {}

impl HasCacheKey for GiteaUserCurrentGetRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: tenant_cache_key_prefix(self.tenant.as_ref())?
                .join("user")?
                .join("current")?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for GiteaUserCurrentGetRequest<'_> {
    type Output = GiteaUser;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            gitea_api_get(
                self.tenant.as_ref(),
                "/user",
                Some(self.cache_key()?),
                &executor,
            )
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(GiteaUserCurrentGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaUserCurrentGetRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    GiteaUserCurrentGetRequest<'static> => GiteaUser,
    effects = [Read]
);

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_registry::RuntimeValue;

    #[test]
    fn reflected_current_user_clone_retains_its_owned_instance() -> eyre::Result<()> {
        let original = RuntimeValue::from_box(Box::new(GiteaUserCurrentGetRequest {
            tenant: Cow::Owned("https://offline-clone.example.invalid".parse()?),
        }))?;
        let cloned = original
            .try_clone()?
            .into_box::<GiteaUserCurrentGetRequest<'static>>()?
            .downcast::<GiteaUserCurrentGetRequest<'static>>()
            .expect("reflected cloning preserves the request type");
        drop(original);
        assert_eq!(
            cloned.tenant.as_ref().as_ref(),
            "https://offline-clone.example.invalid"
        );
        assert!(matches!(cloned.tenant, Cow::Owned(_)));
        Ok(())
    }
}
