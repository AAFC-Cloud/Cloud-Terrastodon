use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::VirtualMachine;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use indoc::indoc;
use std::path::PathBuf;
use tracing::info;

#[must_use = "This is a future request, you must .await it"]
pub struct VirtualMachineListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_virtual_machines(tenant_id: AzureTenantId) -> VirtualMachineListRequest {
    VirtualMachineListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for VirtualMachineListRequest {
    type Output = Vec<VirtualMachine>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "virtual_machines",
            self.tenant_id.to_string().as_str(),
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

        let virtual_machines =
            ResourceGraphHelper::new(self.tenant_id, query, Some(self.cache_key()))
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
    use crate::get_test_tenant_id;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result = fetch_all_virtual_machines(get_test_tenant_id().await?).await?;
        assert!(!result.is_empty());
        assert!(result.iter().all(|vm| !vm.name.is_empty()));
        Ok(())
    }
}
