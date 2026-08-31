use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::VirtualMachine;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::info;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct VirtualMachineListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

pub fn fetch_all_virtual_machines<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> VirtualMachineListRequest<'a> {
    VirtualMachineListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for VirtualMachineListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for VirtualMachineListRequest<'a> {
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

        let virtual_machines = ResourceGraphHelper::new(
            self.tenant_id,
            query,
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        )
        .collect_all::<VirtualMachine>()
        .await?;
        info!(count = virtual_machines.len(), "Found virtual machines");
        Ok(virtual_machines)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(VirtualMachineListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result =
            fetch_all_virtual_machines(get_test_tenant_id().await?, &AuthContext::default())
                .await?;
        assert!(!result.is_empty());
        assert!(result.iter().all(|vm| !vm.name.is_empty()));
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(VirtualMachineListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(VirtualMachineListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(VirtualMachineListRequest<'static> => Vec<VirtualMachine>);
