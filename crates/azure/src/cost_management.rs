use crate::fetch_root_management_group;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::CostManagementQueryDefinition;
use cloud_terrastodon_azure_types::CostManagementQueryResult;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestResponseBody;
use cloud_terrastodon_rest::SerializableRestResponse;
use eyre::Context;
use std::time::Duration;
use tracing::warn;

const COST_MANAGEMENT_RETRY_BUFFER: Duration = Duration::from_secs(1);
const COST_MANAGEMENT_MAX_THROTTLE_RETRIES: usize = 3;

pub async fn fetch_cost_query_results(
    tenant_id: AzureTenantId,
    query: &CostManagementQueryDefinition,
) -> eyre::Result<CostManagementQueryResult> {
    let root = fetch_root_management_group(tenant_id).await?;
    let url = format!(
        "https://management.azure.com/providers/Microsoft.Management/managementGroups/{}/providers/Microsoft.CostManagement/query?api-version=2021-10-01",
        root.tenant_id
    );
    let request = RestRequest::new(http::Method::POST, url.as_str())?
        .tenant(tenant_id)
        .body(facet_json::to_string_pretty(query).map_err(|error| eyre::eyre!("{error:?}"))?);
    receive_cost_management_response(request).await
}

async fn receive_cost_management_response(
    request: RestRequest,
) -> eyre::Result<CostManagementQueryResult> {
    enum CostManagementResponse {
        Parsed(CostManagementQueryResult),
        Raw(SerializableRestResponse),
    }

    let mut retries = 0usize;

    loop {
        let cache_key = request.cache_key.clone();
        let outcome = request
            .clone()
            .receive_raw_with_decoder(|response| {
                if response.ok {
                    let parsed = facet_json::from_str::<CostManagementQueryResult>(
                        response.into_json_body()?.as_str(),
                    )
                    .map_err(|error| eyre::eyre!("{error:?}"))
                    .wrap_err_with(|| {
                        format!(
                            "Deserializing REST response into {}",
                            std::any::type_name::<CostManagementQueryResult>()
                        )
                    })?;
                    Ok(CostManagementResponse::Parsed(parsed))
                } else {
                    Ok(CostManagementResponse::Raw(response))
                }
            })
            .await?;

        let response = match outcome {
            CostManagementResponse::Parsed(parsed) => return Ok(parsed),
            CostManagementResponse::Raw(response) => response,
        };

        if is_cost_management_throttled(&response) && retries < COST_MANAGEMENT_MAX_THROTTLE_RETRIES
        {
            retries += 1;
            if let Some(cache_key) = cache_key {
                cache_key.invalidate().await?;
            }
            let delay = response
                .headers
                .retry_after()
                .unwrap_or(COST_MANAGEMENT_RETRY_BUFFER)
                + COST_MANAGEMENT_RETRY_BUFFER;
            warn!(
                attempt = retries,
                max_attempts = COST_MANAGEMENT_MAX_THROTTLE_RETRIES,
                reset_in = %humantime::format_duration(delay),
                "Retrying throttled Cost Management request"
            );
            tokio::time::sleep(delay).await;
            continue;
        }

        eyre::bail!(
            "REST call failed with status {}: {}{}",
            response.status,
            response.reason_phrase.as_deref().unwrap_or("Unknown error"),
            format_rest_error_body(&response.body)
        );
    }
}

fn is_cost_management_throttled(response: &SerializableRestResponse) -> bool {
    response.status == http::StatusCode::TOO_MANY_REQUESTS.as_u16()
        || response.headers.retry_after().is_some()
}

fn format_rest_error_body(body: &RestResponseBody) -> String {
    match body {
        RestResponseBody::Json(value) => format!("\nBody: {}", value.as_str()),
        RestResponseBody::Text(text) if text.trim().is_empty() => String::new(),
        RestResponseBody::Text(text) => format!("\nBody: {}", text.trim()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use http::HeaderMap;
    use http::HeaderValue;
    use http::StatusCode;

    #[test]
    fn identifies_cost_management_throttle_responses() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-ms-ratelimit-microsoft.costmanagement-clienttype-retry-after",
            HeaderValue::from_static("2"),
        );

        let throttled_by_status = SerializableRestResponse::new(
            StatusCode::TOO_MANY_REQUESTS,
            &HeaderMap::new(),
            "{}".to_string(),
        );
        let throttled_by_header = SerializableRestResponse::new(
            StatusCode::SERVICE_UNAVAILABLE,
            &headers,
            "{}".to_string(),
        );
        let not_throttled = SerializableRestResponse::new(
            StatusCode::BAD_REQUEST,
            &HeaderMap::new(),
            "{}".to_string(),
        );

        assert!(is_cost_management_throttled(&throttled_by_status));
        assert!(is_cost_management_throttled(&throttled_by_header));
        assert!(!is_cost_management_throttled(&not_throttled));
    }

    #[tokio::test]
    async fn it_works1() -> eyre::Result<()> {
        let query = CostManagementQueryDefinition::new_cost_total_this_month();
        let resp = fetch_cost_query_results(get_test_tenant_id().await?, &query).await?;
        assert_eq!(resp.properties.next_link, None);
        assert!(!resp.properties.columns.is_empty());
        Ok(())
    }
    #[tokio::test]
    async fn it_works2() -> eyre::Result<()> {
        let query = CostManagementQueryDefinition::new_cost_by_day_this_month();
        let resp = fetch_cost_query_results(get_test_tenant_id().await?, &query).await?;
        assert_eq!(resp.properties.next_link, None);
        assert!(!resp.properties.columns.is_empty());
        Ok(())
    }
    #[tokio::test]
    async fn it_works3() -> eyre::Result<()> {
        let query = CostManagementQueryDefinition::new_cost_by_resource_group_this_month();
        let resp = fetch_cost_query_results(get_test_tenant_id().await?, &query).await?;
        assert_eq!(resp.properties.next_link, None);
        assert!(!resp.properties.columns.is_empty());
        Ok(())
    }
}
