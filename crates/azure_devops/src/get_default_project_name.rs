use crate::prelude::get_azure_devops_configuration_command;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectName;
use cloud_terrastodon_command::bstr::ByteSlice;
use eyre::Context;
use eyre::OptionExt;
use eyre::bail;

pub async fn get_default_project_name() -> eyre::Result<AzureDevOpsProjectName> {
    let cmd = get_azure_devops_configuration_command();
    let resp = cmd.run_raw().await?;
    let resp = match resp.stdout.to_str() {
        Ok(s) => s,
        Err(e) => {
            cmd.bust_cache().await?;
            bail!("Failed to convert stdout to string: {e}");
        }
    };
    let rtn: AzureDevOpsProjectName = (|| {
        let project = resp
            .lines()
            .find(|line| line.contains("project"))
            .ok_or_eyre("Expected project to be configured using `az devops configure --defaults project=MyProject`")?;
        let Some((_, project)) = project.split_once('=') else {
            bail!("Expected project to have a slash before the name, found {project:?}");
        };
        let project = project.trim();
        eyre::Ok(AzureDevOpsProjectName::new(project.to_string()))
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
