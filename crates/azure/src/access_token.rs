use cloud_terrastodon_azure_types::prelude::AccessToken;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use serde::de::DeserializeOwned;

pub async fn fetch_access_token<T: DeserializeOwned>() -> eyre::Result<AccessToken<T>> {
    CommandBuilder::new(CommandKind::AzureCLI)
        .args(["account", "get-access-token", "--output", "json"])
        .run::<AccessToken<T>>()
        .await
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_access_token;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let token = fetch_access_token::<String>().await?;
        // The debug representation of AccessToken<T> will redact the inner token.
        println!("{:?}", token);
        Ok(())
    }
}
