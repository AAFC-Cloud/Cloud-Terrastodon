// $x = az rest --method GET --url 'https://management.azure.com/subscriptions/{subscription_id}/providers?api-version=2021-04-01&' | ConvertFrom-Json
// $x.value | % { $n = $_.namespace; $_.resourceTypes | % { "$n/$($_.resourceType)" } } | fzf

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use azure_types::prelude::Resource;
use command::prelude::CacheBehaviour;

use crate::prelude::QueryBuilder;

pub async fn fetch_all_resources() -> Result<Vec<Resource>> {
    let resources = QueryBuilder::new(
        r#"
resources 
| union resourcecontainers
| project id, ['kind'] = type
"#
        .to_string(),
        CacheBehaviour::Some {
            path: PathBuf::from("resources"),
            valid_for: Duration::from_mins(5),
        },
    )
    .collect_all()
    .await?;
    Ok(resources)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let resources = fetch_all_resources().await?;
        for res in resources.iter().take(10) {
            println!("{res:?}");
        }
        assert!(resources.len() > 2000);
        Ok(())
    }
}
