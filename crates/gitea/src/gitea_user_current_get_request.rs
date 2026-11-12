use crate::GiteaInstanceUrl;
use crate::GiteaUser;
use crate::gitea_api_support::gitea_api_get;
use crate::gitea_api_support::tenant_cache_key_prefix;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::borrow::Cow;
use std::time::Duration;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct GiteaUserCurrentGetRequest<'a> {
    pub tenant: Cow<'a, GiteaInstanceUrl>,
    pub account_id: CloudTerrastodonAccountId,
}

impl<'a> Arbitrary<'a> for GiteaUserCurrentGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
            account_id: CloudTerrastodonAccountId::arbitrary(u)?,
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
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&AuthenticationSelection::Gitea {
            account_id: self.account_id,
        })
    }

    type Error = eyre::Report;
    type Output = GiteaUser;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            gitea_api_get(
                self.tenant.as_ref(),
                self.account_id,
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
            account_id: "00000000-0000-0000-0000-000000000001".parse()?,
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

cloud_terrastodon_registry_execution::register_runtime_test!(
    GiteaUserCurrentGetRequest<'static>,
    name = "gitea-user-current-get",
    cached
);
