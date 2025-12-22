use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::Group;
use cloud_terrastodon_command::CacheBehaviour;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

pub async fn fetch_all_security_groups() -> Result<Vec<Group>> {
    debug!("Fetching security groups");
    let query = MicrosoftGraphHelper::new(
        "https://graph.microsoft.com/v1.0/groups?$select=id,displayName,description,securityEnabled,isAssignableToRole&$filter=securityEnabled eq true",
        CacheBehaviour::Some {
            path: PathBuf::from_iter(["ms", "graph", "GET", "security_groups"]),
            valid_for: Duration::from_hours(2),
        },
    );
    let groups: Vec<Group> = query.fetch_all().await?;
    debug!("Found {} security groups", groups.len());
    Ok(groups)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::fetch_all_security_groups;
    use cloud_terrastodon_azure_types::prelude::Group;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let groups: Vec<Group> = fetch_all_security_groups().await?;
        println!("Found {} groups", groups.len());
        assert!(groups.len() > 1);
        Ok(())
    }
}
