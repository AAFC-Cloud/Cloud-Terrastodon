use crate::prelude::HttpMethod;
use cloud_terrastodon_azure_types::prelude::uuid::Uuid;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Serialize)]
pub struct MicrosoftGraphBatchRequest<REQ: Serialize> {
    /// The requests to be made in the batch
    requests: Vec<MicrosoftGraphBatchRequestEntry<REQ>>,
    /// The IDs of the requests, in the order the requests were added
    ids: Vec<String>,
}
impl<T: Serialize> Default for MicrosoftGraphBatchRequest<T> {
    fn default() -> Self {
        MicrosoftGraphBatchRequest {
            requests: Vec::new(),
            ids: Vec::new(),
        }
    }
}
impl<REQ: Serialize> MicrosoftGraphBatchRequest<REQ> {
    pub fn new() -> Self {
        MicrosoftGraphBatchRequest::default()
    }
    pub fn add(&mut self, entry: MicrosoftGraphBatchRequestEntry<REQ>) {
        self.ids.push(entry.id.clone());
        self.requests.push(entry);
    }
    pub fn add_get(&mut self, url: String) {
        let id = Uuid::new_v4().to_string();
        self.ids.push(id.clone());
        self.requests.push(MicrosoftGraphBatchRequestEntry::new(
            id,
            HttpMethod::GET,
            url,
            HashMap::new(),
            None,
        ));
    }
    pub fn add_all<T: Into<MicrosoftGraphBatchRequestEntry<REQ>>>(&mut self, entries: impl IntoIterator<Item = T>) {
        for entry in entries {
            self.add(entry.into());
        }
    }
    pub async fn send<RESP: DeserializeOwned>(self) -> eyre::Result<MicrosoftGraphBatchResponse<RESP>> {
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["rest", "--method", "POST", "--url"]);
        cmd.args(["https://graph.microsoft.com/v1.0/$batch"]);
        cmd.args(["--body"]);
        cmd.file_arg("body.json", serde_json::to_string(&self)?);
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
    pub method: HttpMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<T>,
}

impl<T> MicrosoftGraphBatchRequestEntry<T> {
    pub fn new(
        id: String,
        method: HttpMethod,
        url: String,
        headers: HashMap<String, String>,
        body: Option<T>,
    ) -> Self {
        MicrosoftGraphBatchRequestEntry {
            id,
            method,
            url,
            headers,
            body,
        }
    }
    pub fn new_get(id: String, url: String) -> Self {
        MicrosoftGraphBatchRequestEntry {
            id,
            method: HttpMethod::GET,
            url,
            headers: HashMap::new(),
            body: None,
        }
    }
}



#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct MicrosoftGraphBatchResponse<T: DeserializeOwned> {
    pub responses: Vec<MicrosoftGraphBatchResponseEntry<T>>,
}
impl<T: DeserializeOwned> Deref for MicrosoftGraphBatchResponse<T> {
    type Target = Vec<MicrosoftGraphBatchResponseEntry<T>>;
    fn deref(&self) -> &Self::Target {
        &self.responses
    }
}
impl<T: DeserializeOwned> DerefMut for MicrosoftGraphBatchResponse<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.responses
    }
}

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct MicrosoftGraphBatchResponseEntry<T: DeserializeOwned> {
    pub id: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: MicrosoftGraphBatchResponseEntryBody<T>,
}

#[derive(Debug)]
pub enum MicrosoftGraphBatchResponseEntryBody<T: DeserializeOwned> {
    Success(T),
    Error(MicrosoftGraphBatchResponseEntryError),
}
impl<'de, T: DeserializeOwned> Deserialize<'de> for MicrosoftGraphBatchResponseEntryBody<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer)?;
        if v.get("error").is_some() {
            let err = serde_json::from_value::<MicrosoftGraphBatchResponseEntryError>(v)
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