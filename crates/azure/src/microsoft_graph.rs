use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::path::PathBuf;

enum NextLink {
    Uninitialized,
    Some(String),
    StopIteration,
}
pub struct MicrosoftGraphHelper {
    url: String,
    cache_behaviour: CacheBehaviour,
}
impl MicrosoftGraphHelper {
    pub fn new(url: impl ToString, mut cache_behaviour: CacheBehaviour) -> Self {
        if let CacheBehaviour::Some { ref mut path, .. } = cache_behaviour {
            let mut with_prefix = PathBuf::from_iter(["ms", "graph", "GET"]);
            with_prefix.push(&path);
            *path = with_prefix;
        }
        MicrosoftGraphHelper {
            url: url.to_string(),
            cache_behaviour,
        }
    }

    /// This doesn't handle 'singleton' responses like https://graph.microsoft.com/v1.0/me
    pub async fn fetch_all<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
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

            // Build command
            let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
            cmd.args(["rest", "--method", "GET", "--url", url]);

            // Set up caching
            if let CacheBehaviour::Some {
                ref path,
                ref valid_for,
            } = self.cache_behaviour
            {
                cmd.use_cache_behaviour(CacheBehaviour::Some {
                    path: path.join(request_index.to_string()),
                    valid_for: *valid_for,
                });
            }

            // Perform request
            let mut response = cmd.run::<Response<T>>().await?;
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
struct Response<T> {
    // #[serde(rename = "@odata.context")]
    // context: String,
    #[serde(rename = "@odata.nextLink")]
    next_link: Option<String>,
    value: Vec<T>,
}
