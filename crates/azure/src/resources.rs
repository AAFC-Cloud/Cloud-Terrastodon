// $x = az rest --method GET --url 'https://management.azure.com/subscriptions/{subscription_id}/providers?api-version=2021-04-01&' | ConvertFrom-Json
// $x.value | % { $n = $_.namespace; $_.resourceTypes | % { "$n/$($_.resourceType)" } } | fzf

use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::Resource;
use cloud_terrastodon_command::CacheBehaviour;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

pub async fn fetch_all_resources() -> Result<Vec<Resource>> {
    debug!(fetching = "resources");
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
    debug!(count = resources.len(), "Retrieved resources");
    Ok(resources)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure_types::prelude::ResourceType;
    use cloud_terrastodon_azure_types::prelude::Scope;
    use cloud_terrastodon_azure_types::prelude::ScopeImplKind;
    use itertools::Itertools;
    use std::collections::HashMap;

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
        assert!(!resources.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn count() -> Result<()> {
        let resources = fetch_all_resources().await?;
        let ids: HashMap<ScopeImplKind, i32> =
            resources
                .iter()
                .map(|res| res.id.kind())
                .fold(HashMap::default(), |mut acc, kind| {
                    *acc.entry(kind).or_insert(0) += 1;
                    acc
                });

        for (k, v) in ids
            .iter()
            .filter(|x| *x.0 != ScopeImplKind::Unknown)
            .sorted_by(|a, b| b.1.cmp(a.1))
        {
            println!("{:?}: {}", k, v);
        }

        // print unknown count
        println!();
        println!(
            "{:?}: {}",
            ScopeImplKind::Unknown,
            ids.get(&ScopeImplKind::Unknown)
                .cloned()
                .unwrap_or_default()
        );

        let unknown_kinds: HashMap<ResourceType, i32> = resources
            .iter()
            .filter(|res| res.id.kind() == ScopeImplKind::Unknown)
            .map(|res| res.kind.clone())
            .fold(HashMap::new(), |mut acc, kind| {
                *acc.entry(kind).or_insert(0) += 1;
                acc
            });
        // print descending order
        for (k, v) in unknown_kinds.iter().sorted_by(|a, b| b.1.cmp(a.1)) {
            println!("{k}: {v}");
        }

        Ok(())
    }
}
