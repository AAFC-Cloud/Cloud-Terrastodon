use std::ffi::OsString;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Result;
use azure_types::prelude::QueryResponse;
use command::prelude::CacheBehaviour;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

pub struct QueryBuilder {
    query: Rc<String>,
    cache_behaviour: CacheBehaviour,
    skip_token: Option<String>,
    index: usize,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRestOptions {
    #[serde(rename = "$skip")]
    skip: u32,
    #[serde(rename = "$top")]
    top: u32,
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
            segment.push("--graph-query ");
            segment.push(path.as_os_str());
            *path = PathBuf::from("az graph query").join(segment);
        }
        Self {
            query: Rc::new(query),
            cache_behaviour,
            skip_token: None,
            index: 0,
        }
    }

    pub async fn fetch<T: DeserializeOwned>(&mut self) -> Result<Option<QueryResponse<T>>> {
        // Assemble command
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
        cmd.file_arg(
            "body.json",
            serde_json::to_string_pretty(&QueryRestBody {
                query: self.query.to_string(),
                options: QueryRestOptions {
                    skip: if self.skip_token.is_some() {
                        batch_size
                    } else {
                        0
                    },
                    top: batch_size,
                    skip_token: self.skip_token.to_owned(),
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
                valid_for: valid_for.clone(),
            });
        }

        // Increment index for the next potential query
        self.index += 1;

        // Run command
        let results = cmd.run::<QueryResponse<T>>().await?;

        // Update skip token
        self.skip_token = results.skip_token.clone();
        
        // // Transform results
        // let results: QueryResponse<T> = results.try_into()?;

        Ok(Some(results))
    }

    pub async fn collect_all<T: DeserializeOwned>(&mut self) -> Result<Vec<T>> {
        let mut all_data = Vec::new();

        while let Some(response) = self.fetch().await? {
            all_data.extend(response.data);

            if self.skip_token.is_none() {
                break;
            }
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
