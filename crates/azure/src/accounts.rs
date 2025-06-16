use cloud_terrastodon_azure_types::prelude::Account;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use std::path::PathBuf;
use std::time::Duration;

pub async fn az_account_list() -> eyre::Result<Vec<Account>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["account", "list", "--output", "json"]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        valid_for: Duration::from_secs(30),
        path: PathBuf::from_iter(["az", "account", "list"]),
    });
    let rtn = cmd.run().await?;
    Ok(rtn)
}

#[cfg(test)]
mod test {
    use crate::prelude::az_account_list;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let accounts = az_account_list().await?;
        dbg!(&accounts);
        assert_ne!(accounts.as_slice(), []);
        Ok(())
    }
}
