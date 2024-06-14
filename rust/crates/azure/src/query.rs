use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Result;
use azure_types::prelude::QueryResponse;
use command::prelude::CacheBehaviour;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use serde::de::DeserializeOwned;

pub struct QueryBuilder {
    query: Rc<String>,
    cache_behaviour: CacheBehaviour,
    skip_token: Option<String>,
    index: usize,
}

impl QueryBuilder {
    pub fn new(query: String, mut cache_behaviour: CacheBehaviour) -> Self {
        if let CacheBehaviour::Some { ref mut path, .. } = cache_behaviour {
            *path = PathBuf::from("az graph query").join(&path);
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
        cmd.args(["graph", "query", "--graph-query"]);
        cmd.file_arg("query.kql", self.query.to_string());
        if let Some(ref skip_token) = self.skip_token {
            cmd.args(["--skip-token", skip_token]);
        }

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
        self.skip_token = results.skip_token.clone();
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
        }
        Ok(())
    }
}
