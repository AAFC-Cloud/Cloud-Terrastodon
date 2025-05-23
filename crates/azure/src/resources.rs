// $x = az rest --method GET --url 'https://management.azure.com/subscriptions/{subscription_id}/providers?api-version=2021-04-01&' | ConvertFrom-Json
// $x.value | % { $n = $_.namespace; $_.resourceTypes | % { "$n/$($_.resourceType)" } } | fzf

use std::path::PathBuf;
use std::time::Duration;

use cloud_terrastodon_azure_types::prelude::Resource;
use cloud_terrastodon_command::CacheBehaviour;
use eyre::Result;
use tracing::info;

use crate::prelude::ResourceGraphHelper;

pub async fn fetch_all_resources() -> Result<Vec<Resource>> {
    let resources = ResourceGraphHelper::new(
        r#"
resources 
| union resourcecontainers
| project
    id,
    ['kind'] = type,
    name,
    display_name=properties.displayName,
    tags
"#,
        CacheBehaviour::Some {
            path: PathBuf::from("resources"),
            valid_for: Duration::from_mins(120),
        },
    )
    .collect_all()
    .await?;
    info!("Found {} resources", resources.len());
    Ok(resources)
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

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

    #[tokio::test]
    async fn resource_groups() -> Result<()> {
        let resources = fetch_all_resources()
            .await?
            .into_iter()
            .filter(|res| res.kind.is_resource_group())
            .collect_vec();

        for res in resources.iter() {
            println!("{res:?}");
        }
        assert!(resources.len() > 0);
        Ok(())
    }
}
