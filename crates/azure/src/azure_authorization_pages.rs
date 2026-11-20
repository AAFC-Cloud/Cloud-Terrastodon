use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureAuthorizationListPage;
use cloud_terrastodon_azure_types::PercentDecodeExt;
use cloud_terrastodon_azure_types::ScopeImpl;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use facet::Facet;
use http::Method;
use std::collections::HashSet;
use std::future::Future;
use std::panic::Location;
use std::sync::Arc;
use tracing::Instrument;
use url::Url;

/// Follow Azure Authorization collection pages without allowing continuation
/// links to change the ARM credential destination or selected collection.
#[track_caller]
pub(crate) fn collect_azure_authorization_pages<T: Facet<'static> + Send + Sync + 'static>(
    url: Url,
    key: CacheKey,
    authentication: AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
    owner: CacheOperationOwner,
) -> impl Future<Output = Result<Vec<Arc<T>>>> + '_ {
    let caller = Location::caller();
    async move {
        let collection_path = url.path().to_owned();
        let pages = key.join("pages")?;
        let mut next = Some(url);
        let mut visited = HashSet::new();
        let mut values = Vec::new();
        while let Some(url) = next {
            executor.cancellation().bail_if_cancelled()?;
            ensure!(
                valid_continuation(&url, &collection_path),
                "Authorization continuation URL leaves the selected ARM collection"
            );
            ensure!(
                visited.insert(url.clone()),
                "Authorization pagination repeated a previously requested URL"
            );
            let page_key = pages.join(blake3::hash(url.as_str().as_bytes()).to_hex().as_str())?;
            let page = RestRequest::from_method_and_url(Method::GET, url)?
                .authentication(authentication.into())
                .cache(page_key)
                .receive_cached::<AzureAuthorizationListPage<T>>(executor, owner)
                .await
                .wrap_err_with(|| {
                    format!("Reading Azure Authorization page requested at {caller}")
                })?;
            executor.cancellation().bail_if_cancelled()?;
            values.extend(page.value.iter().cloned());
            next = page.next_link.clone();
        }
        Ok(values)
    }
    .in_current_span()
}

fn valid_continuation(url: &Url, collection_path: &str) -> bool {
    let actual = url.path().split('/');
    let expected = collection_path.split('/');
    url.scheme() == "https"
        && url.host_str() == Some("management.azure.com")
        && url.port_or_known_default() == Some(443)
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none()
        && actual.clone().count() == expected.clone().count()
        && actual.zip(expected).all(|(left, right)| {
            left.percent_decode()
                .ok()
                .zip(right.percent_decode().ok())
                .is_some_and(|(left, right)| left.eq_ignore_ascii_case(&right))
        })
}

pub(crate) fn arbitrary_authorization_scope(
    input: &mut Unstructured<'_>,
) -> arbitrary::Result<ScopeImpl> {
    if bool::arbitrary(input)? {
        Ok(ScopeImpl::Subscription(Arbitrary::arbitrary(input)?))
    } else {
        Ok(ScopeImpl::ResourceGroup(Arbitrary::arbitrary(input)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continuation_stays_in_the_same_public_arm_collection() -> Result<()> {
        let path = "/subscriptions/00000000-0000-0000-0000-000000000001/providers/Microsoft.Authorization/roleAssignments";
        assert!(valid_continuation(
            &format!("https://management.azure.com{path}?%24skiptoken=fixture").parse()?,
            path
        ));
        let group_path = "/subscriptions/00000000-0000-0000-0000-000000000001/resourceGroups/example(group)/providers/Microsoft.Authorization/roleAssignments";
        let encoded = group_path.replace("(group)", "%28group%29");
        assert!(valid_continuation(
            &format!("https://management.azure.com{encoded}").parse()?,
            group_path
        ));
        for invalid in [
            format!("https://example.invalid{path}"),
            format!("https://caller@management.azure.com{path}"),
            format!("https://management.azure.com:444{path}"),
            format!("https://management.azure.com{path}/fixture"),
        ] {
            assert!(!valid_continuation(&invalid.parse()?, path));
        }
        Ok(())
    }
}
