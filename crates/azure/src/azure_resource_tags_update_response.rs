use cloud_terrastodon_azure_types::AzureResourceTags;
use cloud_terrastodon_azure_types::ResourceTagsId;
use cloud_terrastodon_rest::SerializableRestResponse;
use eyre::Result;
use eyre::bail;
use eyre::ensure;
use http::StatusCode;
use std::time::Duration;
use url::Url;

/// Interprets the synchronous result or Location polling contract of ARM Tags
/// 2022-09-01. A terminal 204 requires reading the final tags at the original URL.
/// [Official contract](https://github.com/Azure/azure-rest-api-specs/blob/main/specification/resources/resource-manager/Microsoft.Resources/resources/stable/2022-09-01/resources.json)
/// and [Location polling](https://learn.microsoft.com/azure/azure-resource-manager/management/async-operations).
#[derive(Debug)]
pub(crate) enum AzureResourceTagsUpdateResponse {
    Complete(AzureResourceTags),
    CompleteWithoutBody,
    Pending {
        location: Url,
        retry_after: Duration,
    },
}

impl AzureResourceTagsUpdateResponse {
    pub(crate) fn from_response(
        response: SerializableRestResponse,
        expected: &ResourceTagsId,
        request_url: &Url,
        polling_url: Option<&Url>,
    ) -> Result<Self> {
        response.error_for_status()?;
        match response.status {
            StatusCode::OK => {
                let tags: AzureResourceTags = response.decode_json_body()?;
                tags.validate_identity(expected)?;
                Ok(Self::Complete(tags))
            }
            StatusCode::NO_CONTENT if polling_url.is_some() => Ok(Self::CompleteWithoutBody),
            StatusCode::ACCEPTED => {
                let location = match response.header("location") {
                    Some(location) => polling_url.unwrap_or(request_url).join(location)?,
                    None => polling_url.cloned().ok_or_else(|| {
                        eyre::eyre!("Tags update returned 202 without a Location header")
                    })?,
                };
                // Polling is authorized only against the same ARM origin. A
                // response header cannot redirect this account to another host.
                ensure!(
                    location.origin() == request_url.origin()
                        && location.username().is_empty()
                        && location.password().is_none()
                        && location.fragment().is_none(),
                    "Tags polling Location must remain on the original ARM origin"
                );
                Ok(Self::Pending {
                    location,
                    retry_after: response
                        .headers
                        .retry_after()
                        .unwrap_or(Duration::from_secs(2)),
                })
            }
            status => {
                bail!("Unexpected tags update response status {status}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderMap;
    use http::HeaderValue;

    #[test]
    fn accepted_responses_reuse_the_poll_location_without_reading_a_body() -> Result<()> {
        let id = "/subscriptions/00000000-0000-4000-8000-000000000001/providers/Microsoft.Resources/tags/default".parse()?;
        let request = Url::parse("https://management.azure.com/tags?api-version=2022-09-01")?;
        let mut headers = HeaderMap::new();
        headers.insert(
            "location",
            HeaderValue::from_static("/operations/synthetic"),
        );
        headers.insert("retry-after", HeaderValue::from_static("7"));
        let initial = SerializableRestResponse::new(StatusCode::ACCEPTED, &headers, String::new());
        let AzureResourceTagsUpdateResponse::Pending {
            location,
            retry_after,
        } = AzureResourceTagsUpdateResponse::from_response(initial, &id, &request, None)?
        else {
            panic!("accepted operation must remain pending");
        };
        assert_eq!(retry_after, Duration::from_secs(7));
        let pending =
            SerializableRestResponse::new(StatusCode::ACCEPTED, &HeaderMap::new(), String::new());
        let AzureResourceTagsUpdateResponse::Pending {
            location: continued,
            ..
        } = AzureResourceTagsUpdateResponse::from_response(
            pending,
            &id,
            &request,
            Some(&location),
        )?
        else {
            panic!("pending poll must remain pending");
        };
        assert_eq!(continued, location);
        let mut headers = HeaderMap::new();
        headers.insert("location", HeaderValue::from_static("?step=2"));
        let next = SerializableRestResponse::new(StatusCode::ACCEPTED, &headers, String::new());
        let AzureResourceTagsUpdateResponse::Pending { location: next, .. } =
            AzureResourceTagsUpdateResponse::from_response(next, &id, &request, Some(&location))?
        else {
            panic!("accepted continuation must remain pending");
        };
        assert_eq!(next.path(), location.path());
        assert_eq!(next.query(), Some("step=2"));
        Ok(())
    }

    #[test]
    fn initial_acceptance_requires_a_safe_poll_destination() -> Result<()> {
        let id = "/subscriptions/00000000-0000-4000-8000-000000000001/providers/Microsoft.Resources/tags/default".parse()?;
        let request = Url::parse("https://management.azure.com/tags")?;
        for location in [
            None,
            Some("https://example.invalid/operation"),
            Some("http://management.azure.com/operation"),
            Some("https://user@management.azure.com/operation"),
            Some("/operation#fragment"),
        ] {
            let mut headers = HeaderMap::new();
            if let Some(location) = location {
                headers.insert("location", HeaderValue::from_str(location)?);
            }
            let response =
                SerializableRestResponse::new(StatusCode::ACCEPTED, &headers, String::new());
            assert!(
                AzureResourceTagsUpdateResponse::from_response(response, &id, &request, None)
                    .is_err()
            );
        }
        Ok(())
    }
}
