use crate::prelude::get_azure_devops_configuration_command;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::bstr::ByteSlice;
use eyre::Context;
use eyre::OptionExt;
use eyre::bail;

pub async fn get_default_organization_url() -> eyre::Result<AzureDevOpsOrganizationUrl> {
    let cmd = get_azure_devops_configuration_command();
    let resp = cmd.run_raw().await?;
    let resp = match resp.stdout.to_str() {
        Ok(s) => s,
        Err(e) => {
            cmd.bust_cache().await?;
            bail!("Failed to convert stdout to string: {e}");
        }
    };
    match (|| {
        let org = resp
            .lines()
            .find(|line| line.contains("organization"))
            .ok_or_eyre("Expected organization to be configured using `az devops configure --defaults organization=https://dev.azure.com/myorg/`")?;
        let Some((_,org)) = org.rsplit_once('=') else {
            bail!("Expected org to have a slash before the name, found {org:?}");
        };
        Ok(org.trim().to_string().parse()?)
    })()
    .wrap_err(format!("Failed to extract value from config:\n===\n{resp}\n===")) {
        Ok(s) => Ok(s),
        Err(e) => {
            _ = cmd.bust_cache().await;
            bail!(
                "Expected organization to be configured using `az devops configure --defaults organization=https://dev.azure.com/myorg/`\nError: {e}"
            );
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::get_default_organization_url;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let x = get_default_organization_url().await?;
        println!("The default org is {x}");
        Ok(())
    }
}
