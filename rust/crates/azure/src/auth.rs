use anyhow::Result;
use azure_types::prelude::User;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use command::prelude::RetryBehaviour;
use tracing::warn;

pub async fn fetch_current_user() -> Result<User> {
    CommandBuilder::new(CommandKind::AzureCLI)
        .use_retry_behaviour(RetryBehaviour::Fail)
        .args(["ad", "signed-in-user", "show"])
        .run()
        .await
}

pub async fn ensure_logged_in() -> Result<()> {
    if !is_logged_in().await {
        login().await?;
    }
    Ok(())
}

pub async fn is_logged_in() -> bool {
    fetch_current_user().await.is_ok()
}

pub async fn login() -> Result<()> {
    warn!("Refreshing credential, user action required in a moment...");
    CommandBuilder::new(CommandKind::AzureCLI)
        .args(["login"])
        .run_raw()
        .await?;
    Ok(())
}
