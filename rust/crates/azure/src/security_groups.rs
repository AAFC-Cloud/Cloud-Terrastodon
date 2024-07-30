use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use azure_types::prelude::SecurityGroup;
use command::prelude::CacheBehaviour;

use crate::prelude::MicrosoftGraphHelper;

pub async fn fetch_all_security_groups() -> Result<Vec<SecurityGroup>> {
    let query = MicrosoftGraphHelper::new(
        "https://graph.microsoft.com/v1.0/groups?$select=id,displayName&$filter=securityEnabled eq true", 
            CacheBehaviour::Some {
                path: PathBuf::from("security_groups"),
                valid_for: Duration::from_hours(2),
            }
        );
    let groups: Vec<SecurityGroup> = query.fetch_all().await?;
    Ok(groups)
}

#[cfg(test)]
mod tests {
    use crate::prelude::fetch_all_security_groups;
    use super::*;
    use azure_types::prelude::SecurityGroup;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let groups: Vec<SecurityGroup> = fetch_all_security_groups().await?;
        println!("Found {} groups", groups.len());
        assert!(groups.len() > 300);
        Ok(())
    }
}
