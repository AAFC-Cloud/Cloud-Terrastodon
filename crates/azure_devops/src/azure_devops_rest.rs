use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::SerializableRestResponse;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use reqwest::Url;

/// Attach an already-refined Azure DevOps authentication context to a REST
/// request without discarding an explicitly selected bearer-token tenant.
pub(crate) fn authenticate_azure_devops_request(
    request: RestRequest,
    auth_context: &AzureDevOpsAuthContext,
) -> Result<RestRequest> {
    match auth_context {
        AzureDevOpsAuthContext::None => {
            bail!("Azure DevOps authentication is not configured for this request")
        }
        AzureDevOpsAuthContext::Bearer(context) => Ok(request
            .auth_context(&context.auth_context)
            .tenant(context.tenant_id)),
        AzureDevOpsAuthContext::AzureCli(context)
        | AzureDevOpsAuthContext::PersonalAccessToken(context) => Ok(request.auth_context(context)),
    }
}

/// Builds an Azure DevOps REST URL while keeping query parameters encoded by
/// `Url`. The host is selected per API area because Azure DevOps exposes core,
/// graph, and entitlement APIs on different service hosts.
pub(crate) fn azure_devops_api_url(
    organization: &AzureDevOpsOrganizationUrl,
    host: &str,
    path: &str,
    query: &[(&str, &str)],
) -> Result<String> {
    let base = if organization.is_visual_studio_com_format() {
        organization.expanded_form()
    } else {
        format!("https://{host}/{}", organization.organization_name.as_ref())
    };
    let mut url = Url::parse(&format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    ))?;
    {
        let mut pairs = url.query_pairs_mut();
        for (name, value) in query {
            pairs.append_pair(name, value);
        }
    }
    Ok(url.to_string())
}

/// Gives each page in a paginated request its own cache entry beneath the
/// command's base key. Invalidating the base key therefore invalidates page 0
/// and every continuation page together.
pub(crate) fn page_cache_key(cache_key: &CacheKey, page_index: usize) -> CacheKey {
    CacheKey {
        path: cache_key.path.join(page_index.to_string()),
        valid_for: cache_key.valid_for,
    }
}

pub(crate) async fn receive_azure_devops_page<T>(
    request: RestRequest,
) -> Result<(T, Option<String>)>
where
    T: Facet<'static> + Send + 'static,
{
    let response = request.receive_raw().await?;
    parse_azure_devops_page(response)
}

pub(crate) fn parse_azure_devops_page<T>(
    response: SerializableRestResponse,
) -> Result<(T, Option<String>)>
where
    T: Facet<'static>,
{
    if !response.ok {
        bail!(
            "Azure DevOps REST call failed with status {}: {}",
            response.status,
            response.reason_phrase.as_deref().unwrap_or("Unknown error")
        );
    }
    let continuation = response
        .header("x-ms-continuationtoken")
        .or_else(|| response.header("x-ms-continuation-token"))
        .map(str::to_owned);
    let value = facet_json::from_str::<T>(response.into_json_body()?.as_str())
        .map_err(|error| eyre::eyre!("{error:?}"))?;
    Ok((value, continuation))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_credentials::AuthContext;
    use cloud_terrastodon_credentials::AuthSource;
    use reqwest::StatusCode;
    use reqwest::header::HeaderMap;
    use reqwest::header::HeaderValue;

    #[test]
    fn encodes_api_query_values() -> Result<()> {
        let organization = "example".parse::<AzureDevOpsOrganizationUrl>()?;
        let url = azure_devops_api_url(
            &organization,
            "dev.azure.com",
            "_apis/projects",
            &[("api-version", "7.1"), ("name", "a project")],
        )?;
        let parsed = Url::parse(&url)?;
        assert_eq!(
            parsed
                .query_pairs()
                .find(|(name, _)| name == "name")
                .map(|(_, value)| value.into_owned()),
            Some("a project".to_string())
        );
        Ok(())
    }

    #[test]
    fn page_cache_keys_share_an_invalidatable_base_path() {
        let base = CacheKey::new("az/devops/projects");
        let first = page_cache_key(&base, 0);
        let continuation = page_cache_key(&base, 1);
        assert_eq!(first.path, base.path.join("0"));
        assert_eq!(continuation.path, base.path.join("1"));
        assert!(continuation.path.starts_with(&base.path));
    }

    #[test]
    fn refined_bearer_context_applies_its_tenant_to_rest_requests() -> Result<()> {
        let tenant_id = "11111111-1111-1111-1111-111111111111".parse()?;
        let auth_context = AuthContext::explicit(AuthSource::Browser);
        let auth_context = AzureDevOpsAuthContext::for_tenant(&auth_context, tenant_id)?;
        let request = RestRequest::new(
            reqwest::Method::GET,
            "https://dev.azure.com/example/_apis/projects?api-version=7.1",
        )?;

        let request = authenticate_azure_devops_request(request, &auth_context)?;

        assert_eq!(request.tenant, Some(tenant_id));
        assert_eq!(
            request.auth_context.as_ref().and_then(AuthContext::source),
            Some(AuthSource::Browser)
        );
        Ok(())
    }

    #[test]
    fn missing_authentication_is_rejected_locally() -> Result<()> {
        let request = RestRequest::new(
            reqwest::Method::GET,
            "https://dev.azure.com/example/_apis/projects?api-version=7.1",
        )?;

        let error = authenticate_azure_devops_request(request, &AzureDevOpsAuthContext::None)
            .expect_err("missing authentication should fail before request execution");

        assert!(error.to_string().contains("not configured"));
        Ok(())
    }

    #[derive(Debug, Facet, PartialEq)]
    struct FixturePage {
        count: u32,
        value: Vec<String>,
    }

    #[test]
    fn parses_fixture_page_and_continuation_header() -> Result<()> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-ms-continuationtoken",
            HeaderValue::from_static("next page"),
        );
        let response = SerializableRestResponse::new(
            StatusCode::OK,
            &headers,
            r#"{"count":1,"value":["first"]}"#.to_string(),
        );
        let (page, continuation) = parse_azure_devops_page::<FixturePage>(response)?;
        assert_eq!(
            page,
            FixturePage {
                count: 1,
                value: vec!["first".to_string()]
            }
        );
        assert_eq!(continuation.as_deref(), Some("next page"));
        Ok(())
    }
}
