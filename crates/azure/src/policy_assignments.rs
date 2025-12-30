use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::PolicyAssignment;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
pub struct PolicyAssignmentListRequest;

pub fn fetch_all_policy_assignments() -> PolicyAssignmentListRequest {
    PolicyAssignmentListRequest
}

#[async_trait]
impl CacheableCommand for PolicyAssignmentListRequest {
    type Output = Vec<PolicyAssignment>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "policy_assignments",
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let mut qb = ResourceGraphHelper::new(
            r#"
policyresources
| where type =~ "microsoft.authorization/policyassignments"
| project 
 id,
 name,
 location,
 identity,
 properties
    "#,
            Some(self.cache_key()),
        );
        qb.collect_all().await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(PolicyAssignmentListRequest);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_policy_assignments().await?;
        println!("Found {} policy assignments:", result.len());
        for v in result {
            println!("- {}", v);
        }
        Ok(())
    }
}
