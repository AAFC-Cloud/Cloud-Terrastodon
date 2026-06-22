use crate::KeyVaultName;
use crate::KeyVaultSecretName;
use crate::scopes::strip_prefix_case_insensitive;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use eyre::Context;
use eyre::ContextCompat;
use eyre::eyre;
use std::str::FromStr;

/// Something interchangeable with the format of `https://keyvaultname.vault.azure.net/secrets/SECRETNAME/SECRETVERSION`
///
/// Does not contain subscription or resource group info.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct KeyVaultSecretVersionId {
    pub key_vault_name: KeyVaultName,
    pub secret_name: KeyVaultSecretName,
    pub secret_version: CompactString,
}
crate::impl_facet_string_proxy!(KeyVaultSecretVersionId, value => value.to_string());

impl KeyVaultSecretVersionId {
    pub fn new(
        key_vault_name: impl Into<KeyVaultName>,
        secret_name: impl Into<KeyVaultSecretName>,
        secret_version: impl Into<CompactString>,
    ) -> KeyVaultSecretVersionId {
        KeyVaultSecretVersionId {
            key_vault_name: key_vault_name.into(),
            secret_name: secret_name.into(),
            secret_version: secret_version.into(),
        }
    }

    pub fn try_new<V, S, SV>(
        key_vault_name: V,
        secret_name: S,
        secret_version: SV,
    ) -> eyre::Result<Self>
    where
        V: TryInto<KeyVaultName>,
        V::Error: Into<eyre::Error>,
        S: TryInto<KeyVaultSecretName>,
        S::Error: Into<eyre::Error>,
        SV: TryInto<CompactString>,
        SV::Error: Into<eyre::Error>,
    {
        let key_vault_name = key_vault_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert key_vault_name")?;
        let secret_name = secret_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert secret_name")?;
        let secret_version = secret_version
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert secret_version")?;
        Ok(KeyVaultSecretVersionId {
            key_vault_name,
            secret_name,
            secret_version,
        })
    }
}

impl std::fmt::Display for KeyVaultSecretVersionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "https://{}.vault.azure.net/secrets/{}/{}",
            self.key_vault_name, self.secret_name, self.secret_version
        ))
    }
}

impl FromStr for KeyVaultSecretVersionId {
    type Err = eyre::Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // https://keyvaultname.vault.azure.net/secrets/SECRETNAME/SECRETVERSION
        let remaining = strip_prefix_case_insensitive(input, "https://")?;
        let (key_vault_name, remaining) = remaining.split_once('.').wrap_err_with(|| {
            eyre!("Failed to parse KeyVaultSecretVersionId: missing '.' after key vault name in {input:?}")
        })?;
        let key_vault_name = KeyVaultName::from_str(key_vault_name).wrap_err_with(|| {
            eyre!("Failed to parse KeyVaultSecretVersionId: invalid key vault name {key_vault_name:?} in {input:?}")
        })?;
        let remaining = strip_prefix_case_insensitive(remaining, "vault.azure.net/secrets/")
            .wrap_err_with(|| eyre!("Expected 'vault.azure.net/secrets/' in {input:?}"))?;
        let (secret_name, secret_version) = remaining.split_once('/').wrap_err_with(|| {
            eyre!("Failed to parse KeyVaultSecretVersionId: missing '/' after secret name in {input:?}")
        })?;
        let secret_name = KeyVaultSecretName::from_str(secret_name).wrap_err_with(|| {
            eyre!("Failed to parse KeyVaultSecretVersionId: invalid secret name {secret_name:?} in {input:?}")
        })?;
        let secret_version = CompactString::from_str(secret_version).wrap_err_with(|| {
            eyre!("Failed to parse KeyVaultSecretVersionId: invalid secret version {secret_version:?} in {input:?}")
        })?;
        Ok(KeyVaultSecretVersionId {
            key_vault_name,
            secret_name,
            secret_version,
        })
    }
}

#[cfg(test)]
mod test {
    use super::KeyVaultSecretVersionId;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let s = "https://mykv.vault.azure.net/secrets/SECRETNAME/SECRETVERSION";
        let parsed: KeyVaultSecretVersionId = s.parse()?;
        let expected = KeyVaultSecretVersionId::try_new("mykv", "SECRETNAME", "SECRETVERSION")?;
        assert_eq!(parsed, expected);
        let json = facet_json::to_string(s)?;
        assert_eq!(facet_json::to_string(&parsed)?, json);
        assert_eq!(
            facet_json::from_str::<KeyVaultSecretVersionId>(&json)?,
            parsed
        );
        Ok(())
    }

    #[test]
    pub fn it_fails() -> eyre::Result<()> {
        let s = "https://my@@@kv.vault.azure.net/secrets/SECRETNAME/SECRETVERSION";
        let parsed: Result<KeyVaultSecretVersionId, _> = s.parse();
        assert!(parsed.is_err());
        Ok(())
    }
}
