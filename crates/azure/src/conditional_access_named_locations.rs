use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::ConditionalAccessNamedLocation;
use cloud_terrastodon_command::CacheBehaviour;
use std::path::PathBuf;
use std::time::Duration;

pub async fn fetch_all_conditional_access_named_locations()
-> eyre::Result<Vec<ConditionalAccessNamedLocation>> {
    let query = MicrosoftGraphHelper::new(
        "https://graph.microsoft.com/beta/identity/conditionalAccess/namedLocations",
        CacheBehaviour::Some {
            path: PathBuf::from("conditional_access_named_locations"),
            valid_for: Duration::from_hours(24),
        },
    );

    let found = query.fetch_all().await?;
    Ok(found)
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_conditional_access_named_locations;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let found = fetch_all_conditional_access_named_locations().await?;
        println!("Found {} entries", found.len());
        println!("{:#?}", found);
        Ok(())
    }
}
