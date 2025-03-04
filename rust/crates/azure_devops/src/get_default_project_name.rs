use cloud_terrastodon_core_azure_devops_types::prelude::AzureDevopsProjectName;
use cloud_terrastodon_core_command::prelude::bstr::ByteSlice;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use eyre::Context;
use eyre::OptionExt;
use eyre::bail;

pub async fn get_default_project_name() -> eyre::Result<AzureDevopsProjectName> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["devops", "configure", "--list"]);
    let resp = cmd.run_raw().await?.stdout;
    let resp = resp.to_str()?;
    let rtn: AzureDevopsProjectName = (|| {
        let project = resp
            .lines()
            .find(|line| line.contains("project"))
            .ok_or_eyre("Expected project to be configured using `az devops configure --defaults project=MyProject`")?;
        let Some((_, project)) = project.split_once('=') else {
            bail!("Expected project to have a slash before the name, found {project:?}");
        };
        let project = project.trim();
        eyre::Ok(AzureDevopsProjectName::new(project.to_string()))
    })()
    .wrap_err(format!("Failed to extract value from config:\n===\n{resp}\n==="))?;
    Ok(rtn)
}

#[cfg(test)]
mod test {
    use crate::prelude::get_default_project_name;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let x = get_default_project_name().await?;
        println!("The default project is {x:?}");
        Ok(())
    }
}
