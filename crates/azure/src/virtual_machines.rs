use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::VirtualMachine;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use indoc::indoc;
use std::path::PathBuf;
use tracing::info;

#[must_use = "This is a future request, you must .await it"]
pub struct VirtualMachineListRequest;

pub fn fetch_all_virtual_machines() -> VirtualMachineListRequest {
    VirtualMachineListRequest
}

#[async_trait]
impl CacheableCommand for VirtualMachineListRequest {
    type Output = Vec<VirtualMachine>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "virtual_machines",
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        info!(fetching = "virtual machines");
        let query = indoc! {r#"
            Resources
            | where type == "microsoft.compute/virtualmachines"
            | project
                id,
                name,
                location,
                resource_group_name=resourceGroup,
                subscription_id=subscriptionId,
                tags,
                properties
        "#}
        .to_owned();

        let virtual_machines = ResourceGraphHelper::new(query, Some(self.cache_key()))
            .collect_all::<VirtualMachine>()
            .await?;
        info!(count = virtual_machines.len(), "Found virtual machines");
        Ok(virtual_machines)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(VirtualMachineListRequest);

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result = fetch_all_virtual_machines().await?;
        assert!(!result.is_empty());
        println!("Found {} virtual machines:", result.len());
        for vm in result.iter().take(5) {
            println!("{:#?}", vm);
        }
        Ok(())
    }
}
