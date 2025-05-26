use cloud_terrastodon_azure_types::prelude::ResourceGraphQueryResponse;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
#[cfg(debug_assertions)]
use eyre::bail;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
#[cfg(debug_assertions)]
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::debug;

pub struct ResourceGraphHelper {
    query: String,
    cache_behaviour: CacheBehaviour,
    skip: Option<(u64, String)>,
    index: usize,
    #[cfg(debug_assertions)]
    seen_skip_tokens: HashSet<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceGraphQueryRestOptions {
    #[serde(rename = "$skip")]
    skip: u64,
    #[serde(rename = "$top")]
    top: u64,
    #[serde(rename = "$skipToken")]
    skip_token: Option<String>,
    #[serde(rename = "authorizationScopeFilter")]
    authorization_scope_filter: ResourceGraphQueryRestScopeFilterOption,
    #[serde(rename = "resultFormat")]
    result_format: QueryRestResultFormat,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResourceGraphQueryRestScopeFilterOption {
    AtScopeAboveAndBelow,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum QueryRestResultFormat {
    #[serde(rename = "table")]
    Table,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceGraphQueryRestBody {
    query: String,
    options: ResourceGraphQueryRestOptions,
}

impl ResourceGraphHelper {
    pub fn new(query: impl AsRef<str>, mut cache_behaviour: CacheBehaviour) -> Self {
        if let CacheBehaviour::Some { ref mut path, .. } = cache_behaviour {
            let mut with_prefix = PathBuf::from_iter(["az", "resource_graph"]);
            with_prefix.push(&path);
            *path = with_prefix;
        }
        Self {
            query: query.as_ref().to_owned(),
            cache_behaviour,
            skip: None,
            index: 0,
            #[cfg(debug_assertions)]
            seen_skip_tokens: Default::default(),
        }
    }

    #[track_caller]
    pub async fn fetch<T: DeserializeOwned>(
        &mut self,
    ) -> Result<Option<ResourceGraphQueryResponse<T>>> {
        #[cfg(debug_assertions)]
        if let Some((_, token)) = &self.skip
            && !self.seen_skip_tokens.insert(token.to_owned()) {
                bail!("Saw the same skip token twice, infinite loop detected");
            }

        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        // Previously tried using `az graph query` but hit issues with scopes
        // we want the results to be identical to the resource graph explorer in the portal
        // so we must be able to pass authorizationScopeFilter: AtScopeAboveandBelow
        // in the body, so we will use `az rest` instead.

        /*
        cmd.args(["graph", "query", "--graph-query"]);
        cmd.file_arg("query.kql", self.query.to_string());
        if let Some(ref skip_token) = self.skip_token {
            cmd.args(["--skip-token", skip_token]);
        }
        */

        cmd.args(["rest","--method","POST","--url","https://management.azure.com/providers/Microsoft.ResourceGraph/resources?api-version=2022-10-01", "--body"]);
        let batch_size = 1000;
        let (skip, skip_token) = match &self.skip {
            Some((skip, token)) => (*skip, Some(token.to_owned())),
            None => (0u64, None),
        };
        cmd.file_arg(
            "body.json",
            serde_json::to_string_pretty(&ResourceGraphQueryRestBody {
                query: self.query.to_string(),
                options: ResourceGraphQueryRestOptions {
                    skip,
                    top: batch_size,
                    skip_token,
                    authorization_scope_filter:
                        ResourceGraphQueryRestScopeFilterOption::AtScopeAboveAndBelow,
                    result_format: QueryRestResultFormat::Table,
                },
            })?,
        );

        // Set up caching
        if let CacheBehaviour::Some {
            ref path,
            ref valid_for,
        } = self.cache_behaviour
        {
            cmd.use_cache_behaviour(CacheBehaviour::Some {
                path: path.join(self.index.to_string()),
                valid_for: *valid_for,
            });
        }

        // Increment index for the next potential query
        self.index += 1;

        // Run command
        // TODO: handle throttling
        // https://learn.microsoft.com/en-us/azure/governance/resource-graph/overview#throttling
        // https://learn.microsoft.com/en-us/azure/governance/resource-graph/concepts/guidance-for-throttled-requests
        let results = cmd.run::<ResourceGraphQueryResponse<T>>().await?;

        // Update skip token
        if let Some(skip_token) = &results.skip_token {
            self.skip
                .replace((skip + results.count, skip_token.to_owned()));
        } else {
            self.skip.clone_from(&None);
        }

        // // Transform results
        // let results: QueryResponse<T> = results.try_into()?;

        Ok(Some(results))
    }

    #[track_caller]
    pub async fn collect_all<T: DeserializeOwned>(&mut self) -> Result<Vec<T>> {
        let mut all_data = Vec::new();
        debug!("Fetching first batch");
        while let Some(response) = self.fetch().await? {
            all_data.extend(response.data);

            if self.skip.is_none() {
                break;
            }
            debug!("Fetching next batch");
        }

        Ok(all_data)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde::Deserialize;

    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let query = r#"
resourcecontainers
| project name
"#;
        #[derive(Deserialize)]
        struct Row {
            name: String,
        }
        let data = ResourceGraphHelper::new(
            query,
            CacheBehaviour::Some {
                path: PathBuf::from("resource-container-names"),
                valid_for: Duration::from_mins(5),
            },
        )
        .collect_all::<Row>()
        .await?;
        assert!(data.len() > 100);
        for row in data.iter().take(5) {
            println!("- {}", row.name);
            assert!(!row.name.is_empty());
        }
        println!("({} rows omitted)", data.len() - 5);
        Ok(())
    }
}
