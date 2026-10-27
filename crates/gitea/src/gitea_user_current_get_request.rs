use crate::GiteaInstanceUrl;
use crate::GiteaUser;
use crate::gitea_api_support::gitea_api_get;
use crate::gitea_api_support::tenant_cache_key_prefix;
use arbitrary::Arbitrary;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use std::borrow::Cow;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct GiteaUserCurrentGetRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
    pub tenant: Cow<'a, GiteaInstanceUrl>,
}

pub fn fetch_current_gitea_user<'a>(
    tenant: &'a GiteaInstanceUrl,
    cancellation: &CancellationScope,
) -> GiteaUserCurrentGetRequest<'a> {
    GiteaUserCurrentGetRequest {
        cancellation: cancellation.clone(),
        tenant: Cow::Borrowed(tenant),
    }
}

impl<'a> Arbitrary<'a> for GiteaUserCurrentGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: CancellationScope::unbound(),
            tenant: Cow::Owned(GiteaInstanceUrl::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for GiteaUserCurrentGetRequest<'a> {
    type Output = GiteaUser;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            tenant_cache_key_prefix(self.tenant.as_ref())
                .join("user")
                .join("current"),
        )
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        gitea_api_get(
            self.tenant.as_ref(),
            "/user",
            Some(self.cache_key()),
            &self.cancellation,
        )
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GiteaUserCurrentGetRequest<'a>, 'a);

cloud_terrastodon_registry::register_thing!(GiteaUserCurrentGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GiteaUserCurrentGetRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
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
            cancellation: CancellationScope::unbound(),
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
