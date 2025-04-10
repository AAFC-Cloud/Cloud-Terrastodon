use std::path::PathBuf;
use std::time::Duration;

use cloud_terrastodon_core_azure_types::prelude::Account;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;

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
