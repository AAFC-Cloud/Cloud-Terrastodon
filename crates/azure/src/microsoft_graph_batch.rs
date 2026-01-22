use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::FromCommandOutput;
use eyre::bail;
use http::Method;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Serialize)]
pub struct MicrosoftGraphBatchRequest<REQ: Serialize> {
    /// The requests to be made in the batch
    pub requests: Vec<MicrosoftGraphBatchRequestEntry<REQ>>,
    /// The IDs of the requests, in the order the requests were added
    pub ids: Vec<String>,
    /// The key to use for caching the batch request
    #[serde(skip)]
    pub cache_key: Option<CacheKey>,
}
impl<T: Serialize> Default for MicrosoftGraphBatchRequest<T> {
    fn default() -> Self {
        MicrosoftGraphBatchRequest {
            requests: Vec::new(),
            ids: Vec::new(),
            cache_key: None,
        }
    }
}
impl<REQ: Serialize> MicrosoftGraphBatchRequest<REQ> {
    pub fn new() -> Self {
        MicrosoftGraphBatchRequest::default()
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
    ) -> eyre::Result<MicrosoftGraphBatchResponse<RESP>> {
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["rest", "--method", "POST", "--url"]);
        cmd.args(["https://graph.microsoft.com/v1.0/$batch"]);
        cmd.args(["--body"]);
        cmd.azure_file_arg("body.json", serde_json::to_string_pretty(&self)?);
        cmd.use_cache(self.cache_key);
        let mut response = cmd.run::<MicrosoftGraphBatchResponse<RESP>>().await?;
        // reorder the responses to match the order of the requests
        response.responses.sort_by_key(|r| {
            self.ids
                .iter()
                .position(|id| id == &r.id)
                .unwrap_or(usize::MAX)
        });
        Ok(response)
    }
}

#[derive(Debug, Serialize)]
pub struct MicrosoftGraphBatchRequestEntry<T> {
    pub id: String,
    #[serde(
        deserialize_with = "cloud_terrastodon_azure_types::serde_helpers::deserialize_using_from_str",
        serialize_with = "cloud_terrastodon_azure_types::serde_helpers::serialize_using_asref_str"
    )]
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
            id: id,
            method: Method::GET,
            url: Self::prepare_url(url),
            headers: HashMap::new(),
            body: None,
        }
    }
    
    pub fn prepare_url(url: String) -> String {
        url.trim_start_matches("https://graph.microsoft.com/v1.0").to_string()
    }
}

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: FromCommandOutput"))]
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

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: FromCommandOutput"))]
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
impl<'de, T: FromCommandOutput> Deserialize<'de> for MicrosoftGraphBatchResponseEntryBody<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut v = serde_json::Value::deserialize(deserializer)?;
        if let Some(error) = v.get_mut("error") {
            let err = serde_json::from_value::<MicrosoftGraphBatchResponseEntryError>(
                std::mem::take(error),
            )
            .map_err(serde::de::Error::custom)?;
            Ok(MicrosoftGraphBatchResponseEntryBody::Error(err))
        } else {
            let t = serde_json::from_value::<T>(v).map_err(serde::de::Error::custom)?;
            Ok(MicrosoftGraphBatchResponseEntryBody::Success(t))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MicrosoftGraphBatchResponseEntryError {
    pub code: String,
    pub message: String,
    pub inner_error: Option<HashMap<String, serde_json::Value>>,
}

#[cfg(test)]
mod test {
    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        // let mut batch = MicrosoftGraphBatchRequest::new();

        Ok(())
    }
}
