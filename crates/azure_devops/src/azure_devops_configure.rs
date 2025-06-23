use std::{path::PathBuf, time::Duration};

use cloud_terrastodon_command::{CacheBehaviour, CommandBuilder, CommandKind};

pub fn get_azure_devops_configuration_command() -> CommandBuilder {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["devops", "configure", "--list"]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter(["az", "devops", "configure", "--list"]),
        valid_for: Duration::from_hours(8),
    });
    cmd
}
