use cloud_terrastodon_azure_types::prelude::LocationName;
use cloud_terrastodon_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_azure_types::prelude::VirtualMachineSize;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use std::path::PathBuf;

pub async fn fetch_virtual_machine_sizes(
    subscription_id: &SubscriptionId,
    location: &LocationName,
) -> eyre::Result<Vec<VirtualMachineSize>> {
    let url = format!(
        "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/{location}/vmSizes?api-version=2022-11-01"
    );
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", &url]);
    cmd.cache(CacheKey::new(PathBuf::from_iter([
        "az",
        "vm",
        "list-sizes",
        subscription_id.to_string().as_str(),
        location.to_string().as_str(),
    ])));
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Response {
        value: Vec<VirtualMachineSize>,
    }
    let rtn = cmd.run::<Response>().await?.value;
    Ok(rtn)
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_subscriptions;
    use crate::prelude::get_test_tenant_id;
    use cloud_terrastodon_azure_types::prelude::LocationName;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let subs = fetch_all_subscriptions(tenant_id).await?;
        let sub = subs.first().unwrap();
        let sizes =
            crate::prelude::fetch_virtual_machine_sizes(&sub.id, &LocationName::CanadaCentral)
                .await?;
        assert!(!sizes.is_empty());
        println!("Found {} VM sizes", sizes.len());
        println!("{sizes:#?}");
        Ok(())
    }
}
