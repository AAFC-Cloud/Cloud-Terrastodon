use crate::AZURE_DEVOPS_RESOURCE_ID;
use crate::AzureClaims;
use crate::AzureRestResource;
use crate::fetch_azure_access_token;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;

pub async fn get_azure_access_token_jwt() -> eyre::Result<()> {
    let access_token =
        fetch_azure_access_token::<String>(None, AzureRestResource::AzureResourceManager).await?;
    let mut validation = Validation::default();
    #[expect(deprecated)]
    validation.insecure_disable_signature_validation();
    validation.set_audience(&[
        "https://management.core.windows.net/",
        AZURE_DEVOPS_RESOURCE_ID,
    ]);
    let decoding_key = DecodingKey::from_rsa_raw_components(&[], &[]);
    let _token_data = jsonwebtoken::decode::<AzureClaims>(
        &access_token.access_token,
        &decoding_key,
        &validation,
    )?;
    Ok(())
}

#[cfg(test)]
mod test {
    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        super::get_azure_access_token_jwt().await?;
        Ok(())
    }
}
