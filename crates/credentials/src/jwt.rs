use cloud_terrastodon_azure_types::prelude::AccessToken;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;

use crate::azure_access_token::AZURE_DEVOPS_RESOURCE_ID;
use crate::AzureClaims;

pub async fn get_azure_access_token_jwt() -> eyre::Result<()> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["account", "get-access-token"]);
    let access_token: AccessToken<String> = cmd.run().await?;
    let mut validation = Validation::default();
    validation.insecure_disable_signature_validation();
    validation.set_audience(&[
        "https://management.core.windows.net/",
        AZURE_DEVOPS_RESOURCE_ID
    ]);
    let decoding_key = DecodingKey::from_rsa_raw_components(&[],&[]);
    let token_data = jsonwebtoken::decode::<AzureClaims>(&access_token.access_token, &decoding_key, &validation)?;
    println!("{:#?}", token_data);
    Ok(())
}

#[cfg(test)]
mod test {
    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let _ = super::get_azure_access_token_jwt().await?;
        Ok(())
    }
}
