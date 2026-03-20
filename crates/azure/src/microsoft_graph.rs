use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::FromCommandOutput;
use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use eyre::Result;
use serde::Deserialize;

enum NextLink {
    Uninitialized,
    Some(String),
    StopIteration,
}
pub struct MicrosoftGraphHelper {
    url: String,
    cache_key: Option<CacheKey>,
    tenant_id: Option<AzureTenantId>,
}
impl MicrosoftGraphHelper {
    pub fn new(url: impl ToString, cache_key: Option<CacheKey>) -> Self {
        MicrosoftGraphHelper {
            url: url.to_string(),
            cache_key,
            tenant_id: None,
        }
    }

    pub fn tenant_id(mut self, tenant_id: AzureTenantId) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    fn get_command(&self, url: &str) -> CommandBuilder {
        let mut cmd = match self.tenant_id {
            Some(tenant_id) => {
                let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
                let tenant_id = tenant_id.to_string();
                cmd.args(["rest", "--method", "GET", "--url", url, "--tenant", tenant_id.as_str()]);
                cmd
            }
            None => {
                let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
                cmd.args(["rest", "--method", "GET", "--url"]);
                cmd.azure_file_arg("url.txt", url.to_string());
                cmd
            }
        };
        cmd.use_cache(self.cache_key.clone());
        cmd
    }

    pub async fn fetch_one<T: FromCommandOutput>(self) -> Result<T> {
        // Perform request
        let response = self.get_command(&self.url).run::<T>().await?;
        Ok(response)
    }

    /// This doesn't handle 'singleton' responses like https://graph.microsoft.com/v1.0/me
    pub async fn fetch_all<T: FromCommandOutput>(&self) -> Result<Vec<T>> {
        let mut results = Vec::new();
        let mut next_link = NextLink::Uninitialized;
        let mut request_index = 0;
        loop {
            // Determine URL, considering pagination
            let url = match &next_link {
                NextLink::Uninitialized => &self.url,
                NextLink::Some(x) => x,
                NextLink::StopIteration => break,
            };

            let mut cmd = self.get_command(url);
            if let Some(ref cache_key) = self.cache_key {
                cmd.cache(CacheKey {
                    path: cache_key.path.join(request_index.to_string()),
                    valid_for: cache_key.valid_for,
                });
            }

            // Perform request
            let mut response = cmd.run::<MicrosoftGraphResponse<T>>().await?;
            request_index += 1;

            // Update next link for pagination
            next_link = match response.next_link {
                Some(url) => NextLink::Some(url),
                None => NextLink::StopIteration,
            };

            // Update results
            results.append(&mut response.value);
        }
        Ok(results)
    }
}

#[derive(Debug, Deserialize)]
pub struct MicrosoftGraphResponse<T> {
    // #[serde(rename = "@odata.context")]
    // context: String,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
    pub value: Vec<T>,
}
