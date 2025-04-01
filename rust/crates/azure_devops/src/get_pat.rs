use eyre::bail;

pub struct SensitiveString {
    pub inner: String,
}

pub async fn get_personal_access_token() -> eyre::Result<SensitiveString> {
    match std::env::var("AZDO_PERSONAL_ACCESS_TOKEN") {
        Ok(token) => Ok(SensitiveString { inner: token }),
        Err(e) => match e {
            std::env::VarError::NotPresent => {
                bail!("AZDO_PERSONAL_ACCESS_TOKEN environment variable not set!\n```pwsh\n$env:AZDO_PERSONAL_ACCESS_TOKEN=Read-Host -MaskInput \"Enter PAT\"\n```\n")
            }
            std::env::VarError::NotUnicode(_) => {
                bail!("AZDO_PERSONAL_ACCESS_TOKEN contains invalid unicode",)
            }
        },
    }
}
