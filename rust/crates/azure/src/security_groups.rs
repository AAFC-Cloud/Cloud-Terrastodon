use anyhow::Result;
use cloud_terrastodon_core_azure_types::prelude::Group;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use tracing::info;
use std::path::PathBuf;
use std::time::Duration;

use crate::prelude::MicrosoftGraphHelper;

pub async fn fetch_all_security_groups() -> Result<Vec<Group>> {
    info!("Fetching security groups");
    let query = MicrosoftGraphHelper::new(
        "https://graph.microsoft.com/v1.0/groups?$select=id,displayName,description,securityEnabled,isAssignableToRole&$filter=securityEnabled eq true",
        CacheBehaviour::Some {
            path: PathBuf::from("security_groups"),
            valid_for: Duration::from_hours(2),
        },
    );
    let groups: Vec<Group> = query.fetch_all().await?;
    Ok(groups)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::fetch_all_security_groups;
    use cloud_terrastodon_core_azure_types::prelude::Group;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let groups: Vec<Group> = fetch_all_security_groups().await?;
        println!("Found {} groups", groups.len());
        // ensure pagination working
        assert!(groups.len() > 300);
        Ok(())
    }
}
