use eyre::Result;
use eyre::ensure;
use url::Url;

/// ACR continuation may be in a Link header or the response's link property.
/// Keep continuation on the same read operation and preserve its API version.
/// [Get repositories response headers](https://learn.microsoft.com/en-us/rest/api/registry-dataplane/container-registry/get-repositories?view=rest-registry-dataplane-2021-07-01#responses).
pub(crate) fn next_registry_page(
    current: &Url,
    header: Option<&str>,
    body: Option<&str>,
) -> Result<Option<Url>> {
    let Some(link) = header.or(body) else {
        return Ok(None);
    };
    let link = link.trim();
    ensure!(
        !link.is_empty(),
        "registry returned an empty continuation link"
    );
    let reference = if let Some(link) = link.strip_prefix('<') {
        let (reference, attributes) = link.split_once('>').ok_or_else(|| {
            eyre::eyre!("registry continuation Link header is missing its closing delimiter")
        })?;
        ensure!(
            attributes.trim().is_empty()
                || attributes
                    .split(';')
                    .skip(1)
                    .any(|attribute| matches!(attribute.trim(), "rel=\"next\"" | "rel=next")),
            "registry continuation Link header is not a next-page link"
        );
        reference
    } else {
        link
    };
    let mut next = current.join(reference)?;
    ensure!(
        next.origin() == current.origin()
            && next.path() == current.path()
            && next.username().is_empty()
            && next.password().is_none()
            && next.fragment().is_none(),
        "registry continuation must remain on the selected registry operation"
    );
    if !next.query_pairs().any(|(key, _)| key == "api-version")
        && let Some((_, version)) = current.query_pairs().find(|(key, _)| key == "api-version")
    {
        next.query_pairs_mut().append_pair("api-version", &version);
    }
    Ok(Some(next))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolves_relative_page_without_losing_version_or_crossing_auth_boundary() {
        let current =
            Url::parse("https://fixture.azurecr.io/acr/v1/_catalog?api-version=2021-07-01")
                .unwrap();
        let next = next_registry_page(&current, Some("<?last=sample>; rel=\"next\""), None)
            .unwrap()
            .unwrap();
        assert!(
            next.query_pairs()
                .any(|(key, value)| key == "api-version" && value == "2021-07-01")
        );
        assert!(
            next.query_pairs()
                .any(|(key, value)| key == "last" && value == "sample")
        );
        for invalid in [
            "https://other.azurecr.io/acr/v1/_catalog",
            "/oauth2/token",
            "//user@fixture.azurecr.io/acr/v1/_catalog",
        ] {
            assert!(next_registry_page(&current, None, Some(invalid)).is_err());
        }
    }
}
