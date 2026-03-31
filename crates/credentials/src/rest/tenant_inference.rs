use crate::RestService;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::SubscriptionId;
use eyre::Result;
use reqwest::Url;
use std::future::Future;

pub async fn infer_tenant_id_for_request<F, Fut>(
    service: RestService,
    url: &Url,
    resolve_subscription_tenant: F,
) -> Result<Option<AzureTenantId>>
where
    F: FnOnce(SubscriptionId) -> Fut,
    Fut: Future<Output = Result<AzureTenantId>>,
{
    if service != RestService::AzureResourceManager {
        return Ok(None);
    }

    let Some(subscription_id) = extract_arm_subscription_id(url) else {
        return Ok(None);
    };

    Ok(Some(resolve_subscription_tenant(subscription_id).await?))
}

pub fn extract_arm_subscription_id(url: &Url) -> Option<SubscriptionId> {
    let mut segments = url.path_segments()?;
    let first = segments.next()?;
    if !first.eq_ignore_ascii_case("subscriptions") {
        return None;
    }
    let subscription = segments.next()?;
    subscription.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::RestService;
    use super::extract_arm_subscription_id;
    use super::infer_tenant_id_for_request;
    use cloud_terrastodon_azure_types::AzureTenantId;
    use cloud_terrastodon_azure_types::SubscriptionId;
    use reqwest::Url;

    #[test]
    fn extracts_subscription_id_from_arm_subscription_url() {
        let url = Url::parse(
            "https://management.azure.com/subscriptions/11111111-1111-1111-1111-111111111111/locations?api-version=2022-12-01",
        )
        .unwrap();
        let expected = "11111111-1111-1111-1111-111111111111"
            .parse::<SubscriptionId>()
            .unwrap();
        assert_eq!(extract_arm_subscription_id(&url), Some(expected));
    }

    #[test]
    fn ignores_non_subscription_arm_urls() {
        let url = Url::parse(
            "https://management.azure.com/providers/Microsoft.ResourceGraph/resources?api-version=2022-10-01",
        )
        .unwrap();
        assert_eq!(extract_arm_subscription_id(&url), None);
    }

    #[test]
    fn ignores_invalid_subscription_ids_in_arm_urls() {
        let url = Url::parse(
            "https://management.azure.com/subscriptions/not-a-guid/locations?api-version=2022-12-01",
        )
        .unwrap();
        assert_eq!(extract_arm_subscription_id(&url), None);
    }

    #[tokio::test]
    async fn skips_tenant_inference_for_non_arm_services() -> eyre::Result<()> {
        let url = Url::parse("https://graph.microsoft.com/v1.0/organization").unwrap();
        let tenant =
            infer_tenant_id_for_request(RestService::MicrosoftGraph, &url, |_| async move {
                eyre::bail!("resolver should not be called")
            })
            .await?;
        assert_eq!(tenant, None);
        Ok(())
    }

    #[tokio::test]
    async fn infers_tenant_for_arm_subscription_urls() -> eyre::Result<()> {
        let url = Url::parse(
            "https://management.azure.com/subscriptions/11111111-1111-1111-1111-111111111111/locations?api-version=2022-12-01",
        )
        .unwrap();
        let expected = "22222222-2222-2222-2222-222222222222"
            .parse::<AzureTenantId>()
            .unwrap();
        let tenant = infer_tenant_id_for_request(
            RestService::AzureResourceManager,
            &url,
            |_subscription_id| async move { Ok(expected) },
        )
        .await?;
        assert_eq!(tenant, Some(expected));
        Ok(())
    }
}
