use anyhow::bail;
use anyhow::Result;
use azure_types::prelude::QueryResponse;
use command::prelude::CacheBehaviour;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::PathBuf;
use tracing::debug;

pub struct QueryBuilder {
    query: String,
    cache_behaviour: CacheBehaviour,
    skip: Option<(u64, String)>,
    index: usize,
    #[cfg(debug_assertions)]
    seen_skip_tokens: HashSet<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRestOptions {
    #[serde(rename = "$skip")]
    skip: u64,
    #[serde(rename = "$top")]
    top: u64,
    #[serde(rename = "$skipToken")]
    skip_token: Option<String>,
    #[serde(rename = "authorizationScopeFilter")]
    authorization_scope_filter: QueryRestScopeFilterOption,
    #[serde(rename = "resultFormat")]
    result_format: QueryRestResultFormat,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum QueryRestScopeFilterOption {
    AtScopeAboveAndBelow,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum QueryRestResultFormat {
    #[serde(rename = "table")]
    Table,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRestBody {
    query: String,
    options: QueryRestOptions,
}

impl QueryBuilder {
    pub fn new(query: String, mut cache_behaviour: CacheBehaviour) -> Self {
        if let CacheBehaviour::Some { ref mut path, .. } = cache_behaviour {
            let mut segment = OsString::new();
            segment.push("az graph query --graph-query ");
            segment.push(path.as_os_str());
            *path = PathBuf::from(segment);
        }
        Self {
            query,
            cache_behaviour,
            skip: None,
            index: 0,
            #[cfg(debug_assertions)]
            seen_skip_tokens: Default::default(),
        }
    }

    pub async fn fetch<T: DeserializeOwned>(&mut self) -> Result<Option<QueryResponse<T>>> {
        #[cfg(debug_assertions)]
        if let Some((_, token)) = &self.skip {
            if !self.seen_skip_tokens.insert(token.to_owned()) {
                bail!("Saw the same skip token twice, infinite loop detected");
            }
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
            serde_json::to_string_pretty(&QueryRestBody {
                query: self.query.to_string(),
                options: QueryRestOptions {
                    skip,
                    top: batch_size,
                    skip_token,
                    authorization_scope_filter: QueryRestScopeFilterOption::AtScopeAboveAndBelow,
                    result_format: QueryRestResultFormat::Table,
                },
            })?,
        );

        // Set up caching
        if let CacheBehaviour::Some {
            ref mut path,
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
        let results = cmd.run::<QueryResponse<T>>().await?;

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
        let data = QueryBuilder::new(
            query.to_string(),
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
