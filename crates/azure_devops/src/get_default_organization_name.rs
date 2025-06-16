use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationName;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::bstr::ByteSlice;
use eyre::Context;
use eyre::OptionExt;
use eyre::bail;
use std::path::PathBuf;
use std::time::Duration;

pub async fn get_default_organization_url() -> eyre::Result<AzureDevOpsOrganizationUrl> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["devops", "configure", "--list"]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter(["az", "devops", "configure", "--list"]),
        valid_for: Duration::from_hours(8),
    });
    let resp = cmd.run_raw().await?;
    let resp = resp.stdout.to_str()?;
    let rtn: String = (|| {
        let org = resp
        .lines()
        .find(|line| line.contains("organization"))
        .ok_or_eyre("Expected organization to be configured using `az devops configure --defaults organization=https://dev.azure.com/myorg/`")?;
        let Some((_,org)) = org.rsplit_once('=') else {
            bail!("Expected org to have a slash before the name, found {org:?}");
        };
        Ok(org.trim().to_string())
    })()
    .wrap_err(format!("Failed to extract value from config:\n===\n{resp}\n==="))?;
    rtn.parse()
}

pub async fn get_default_organization_name() -> eyre::Result<AzureDevOpsOrganizationName> {
    Ok(get_default_organization_url().await?.organization_name)
}

#[cfg(test)]
mod test {
    use crate::prelude::get_default_organization_name;
    use crate::prelude::get_default_organization_url;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let x = get_default_organization_name().await?;
        println!("The default org is {x:?}");
        Ok(())
    }

    #[tokio::test]
    pub async fn it_works2() -> eyre::Result<()> {
        let x = get_default_organization_url().await?;
        println!("The default org is {x}");
        Ok(())
    }
}
