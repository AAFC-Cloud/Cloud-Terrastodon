use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use eyre::Context;
use eyre::OptionExt;
use eyre::bail;
use cloud_terrastodon_core_command::prelude::bstr::ByteSlice;

pub async fn get_default_organization_name() -> eyre::Result<String> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["devops", "configure", "--list"]);
    let resp = cmd.run_raw().await?;
    let resp = resp.stdout.to_str()?;
    let rtn: String = (|| {
        let org = resp
        .lines()
        .find(|line| line.contains("organization"))
        .ok_or_eyre("Expected organization to be configured using `az devops configure --defaults organization=https://dev.azure.com/myorg/`")?;
        let Some(org) = org.strip_suffix('/') else {
            bail!("Expected org to end with a slash, found {org:?}");
        };
        let Some((_,org)) = org.rsplit_once('/') else {
            bail!("Expected org to have a slash before the name, found {org:?}");
        };
        Ok(org.to_string())
    })()
    .wrap_err(format!("Failed to extract value from config:\n===\n{resp}\n==="))?;
    Ok(rtn)
}

#[cfg(test)]
mod test {
    use crate::prelude::get_default_organization_name;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let x = get_default_organization_name().await?;
        println!("The default org is {x:?}");
        Ok(())
    }
}
