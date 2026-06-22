use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::FromCommandOutput;
use cloud_terrastodon_rest::RestRequest;
use eyre::bail;
use facet::Facet;
use facet_json::RawJson;
use http::Method;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

pub struct MicrosoftGraphBatchRequest<REQ> {
    /// The requests to be made in the batch
    pub requests: Vec<MicrosoftGraphBatchRequestEntry<REQ>>,
    /// The IDs of the requests, in the order the requests were added
    pub ids: Vec<String>,
    /// The key to use for caching the batch request
    pub cache_key: Option<CacheKey>,
    pub tenant_id: AzureTenantId,
}
impl<REQ> MicrosoftGraphBatchRequest<REQ> {
    pub fn new(tenant_id: AzureTenantId) -> Self {
        MicrosoftGraphBatchRequest {
            requests: Vec::new(),
            ids: Vec::new(),
            cache_key: None,
            tenant_id,
        }
    }
    pub fn add(&mut self, entry: impl Into<MicrosoftGraphBatchRequestEntry<REQ>>) {
        let entry = entry.into();
        self.ids.push(entry.id.clone());
        self.requests.push(entry);
    }
    pub fn add_all<T: Into<MicrosoftGraphBatchRequestEntry<REQ>>>(
        &mut self,
        entries: impl IntoIterator<Item = T>,
    ) {
        for entry in entries {
            self.add(entry);
        }
    }
    pub fn cache(&mut self, cache_key: CacheKey) -> &mut Self {
        self.cache_key = Some(cache_key);
        self
    }
    pub fn use_cache(&mut self, cache_key: Option<CacheKey>) -> &mut Self {
        self.cache_key = cache_key;
        self
    }
    pub async fn send<RESP: FromCommandOutput>(
        self,
    ) -> eyre::Result<MicrosoftGraphBatchResponse<RESP>>
    where
        REQ: Facet<'static>,
    {
        let Self {
            requests,
            ids,
            cache_key,
            tenant_id,
        } = self;
        let body = MicrosoftGraphBatchRequestWire {
            requests: requests
                .into_iter()
                .map(MicrosoftGraphBatchRequestEntryWire::from)
                .collect(),
        };
        let mut request =
            RestRequest::new(Method::POST, "https://graph.microsoft.com/v1.0/$batch")?
                .tenant(tenant_id)
                .body(
                    facet_json::to_string_pretty(&body)
                        .map_err(|error| eyre::eyre!("{error:?}"))?,
                );
        request.cache_key = cache_key;
        let response = request.receive::<MicrosoftGraphBatchWireResponse>().await?;
        let mut response = response.into_typed::<RESP>()?;
        // reorder the responses to match the order of the requests
        response
            .responses
            .sort_by_key(|r| ids.iter().position(|id| id == &r.id).unwrap_or(usize::MAX));
        Ok(response)
    }
}

#[derive(Debug)]
pub struct MicrosoftGraphBatchRequestEntry<T> {
    pub id: String,
    pub method: Method,
    pub url: String,
    pub headers: HashMap<String, String>,
    /// None if this is a GET request
    pub body: Option<T>,
}

impl<T> MicrosoftGraphBatchRequestEntry<T> {
    pub fn new(
        id: String,
        method: Method,
        url: String,
        headers: HashMap<String, String>,
        body: Option<T>,
    ) -> Self {
        MicrosoftGraphBatchRequestEntry {
            id,
            method,
            url: Self::prepare_url(url),
            headers,
            body,
        }
    }

    /// We want the ID to be consistent such that caching change detection doesn't get unnecessarily invalidated.
    pub fn new_get(id: String, url: String) -> Self {
        MicrosoftGraphBatchRequestEntry {
            id,
            method: Method::GET,
            url: Self::prepare_url(url),
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn prepare_url(url: String) -> String {
        url.trim_start_matches("https://graph.microsoft.com/v1.0")
            .to_string()
    }
}

#[derive(Debug, facet::Facet)]
struct MicrosoftGraphBatchRequestWire<T> {
    requests: Vec<MicrosoftGraphBatchRequestEntryWire<T>>,
}

#[derive(Debug, facet::Facet)]
struct MicrosoftGraphBatchRequestEntryWire<T> {
    id: String,
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Option<T>,
}

impl<T> From<MicrosoftGraphBatchRequestEntry<T>> for MicrosoftGraphBatchRequestEntryWire<T> {
    fn from(value: MicrosoftGraphBatchRequestEntry<T>) -> Self {
        Self {
            id: value.id,
            method: value.method.as_str().to_string(),
            url: value.url,
            headers: value.headers,
            body: value.body,
        }
    }
}

#[derive(Debug)]
pub struct MicrosoftGraphBatchResponse<T: FromCommandOutput> {
    pub responses: Vec<MicrosoftGraphBatchResponseEntry<T>>,
}
impl<T: FromCommandOutput> Deref for MicrosoftGraphBatchResponse<T> {
    type Target = Vec<MicrosoftGraphBatchResponseEntry<T>>;
    fn deref(&self) -> &Self::Target {
        &self.responses
    }
}
impl<T: FromCommandOutput> DerefMut for MicrosoftGraphBatchResponse<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.responses
    }
}

#[derive(Debug)]
pub struct MicrosoftGraphBatchResponseEntry<T: FromCommandOutput> {
    pub id: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: MicrosoftGraphBatchResponseEntryBody<T>,
}
impl<T: FromCommandOutput> MicrosoftGraphBatchResponseEntry<T> {
    pub fn into_body(self) -> eyre::Result<T> {
        match self.body {
            MicrosoftGraphBatchResponseEntryBody::Success(t) => Ok(t),
            MicrosoftGraphBatchResponseEntryBody::Error(e) => bail!(
                "Microsoft Graph API error for request {} (status {}): {} - {}",
                self.id,
                self.status,
                e.code,
                e.message
            ),
        }
    }
}

#[derive(Debug)]
pub enum MicrosoftGraphBatchResponseEntryBody<T: FromCommandOutput> {
    Success(T),
    Error(MicrosoftGraphBatchResponseEntryError),
}

#[derive(Debug, facet::Facet)]
struct MicrosoftGraphBatchWireResponse {
    responses: Vec<MicrosoftGraphBatchWireResponseEntry>,
}

impl MicrosoftGraphBatchWireResponse {
    fn into_typed<T: FromCommandOutput>(self) -> eyre::Result<MicrosoftGraphBatchResponse<T>> {
        let responses = self
            .responses
            .into_iter()
            .map(MicrosoftGraphBatchWireResponseEntry::into_typed)
            .collect::<eyre::Result<_>>()?;
        Ok(MicrosoftGraphBatchResponse { responses })
    }
}

#[derive(Debug, facet::Facet)]
struct MicrosoftGraphBatchWireResponseEntry {
    id: String,
    status: u16,
    headers: HashMap<String, String>,
    body: RawJson<'static>,
}

impl MicrosoftGraphBatchWireResponseEntry {
    fn into_typed<T: FromCommandOutput>(self) -> eyre::Result<MicrosoftGraphBatchResponseEntry<T>> {
        let body = MicrosoftGraphBatchResponseEntryBodyProxy(self.body).try_into()?;
        Ok(MicrosoftGraphBatchResponseEntry {
            id: self.id,
            status: self.status,
            headers: self.headers,
            body,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, facet::Facet)]
#[facet(transparent)]
pub struct MicrosoftGraphBatchResponseEntryBodyProxy(RawJson<'static>);

#[derive(Debug, facet::Facet)]
struct MicrosoftGraphBatchErrorEnvelope {
    error: MicrosoftGraphBatchResponseEntryError,
}

impl<T: FromCommandOutput> TryFrom<MicrosoftGraphBatchResponseEntryBodyProxy>
    for MicrosoftGraphBatchResponseEntryBody<T>
{
    type Error = eyre::Report;

    fn try_from(
        value: MicrosoftGraphBatchResponseEntryBodyProxy,
    ) -> Result<Self, <Self as TryFrom<MicrosoftGraphBatchResponseEntryBodyProxy>>::Error> {
        let raw = value.0;
        if let Ok(envelope) = facet_json::from_str::<MicrosoftGraphBatchErrorEnvelope>(raw.as_str())
        {
            return Ok(Self::Error(envelope.error));
        }
        let value = facet_json::from_str::<T>(raw.as_str()).map_err(|error| {
            eyre::eyre!(
                "failed to deserialize Microsoft Graph batch body as {}: {:?}",
                std::any::type_name::<T>(),
                error
            )
        })?;
        Ok(Self::Success(value))
    }
}

impl<T: FromCommandOutput> TryFrom<&MicrosoftGraphBatchResponseEntryBody<T>>
    for MicrosoftGraphBatchResponseEntryBodyProxy
{
    type Error = eyre::Report;

    fn try_from(value: &MicrosoftGraphBatchResponseEntryBody<T>) -> Result<Self, Self::Error> {
        let json = match value {
            MicrosoftGraphBatchResponseEntryBody::Success(value) => {
                facet_json::to_string(value).map_err(|error| eyre::eyre!("{error:?}"))?
            }
            MicrosoftGraphBatchResponseEntryBody::Error(error) => {
                facet_json::to_string(&MicrosoftGraphBatchErrorEnvelope {
                    error: error.clone(),
                })
                .map_err(|error| eyre::eyre!("{error:?}"))?
            }
        };
        Ok(Self(RawJson::from_owned(json)))
    }
}

#[derive(Clone, Debug, facet::Facet)]
pub struct MicrosoftGraphBatchResponseEntryError {
    pub code: String,
    pub message: String,
    pub inner_error: Option<HashMap<String, RawJson<'static>>>,
}

#[cfg(test)]
mod test {
    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        // let mut batch = MicrosoftGraphBatchRequest::new(
        //     cloud_terrastodon_azure_types::AzureTenantId::nil(),
        // );

        Ok(())
    }
}
