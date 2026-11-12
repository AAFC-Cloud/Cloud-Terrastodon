use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;

/// Synthetic request identity for policy/URL tests; this never resolves a credential.
pub(crate) fn test_authentication() -> AzureBearerAuthenticationSelection {
    AzureBearerAuthenticationSelection {
        account_id: "11111111-1111-1111-1111-111111111111".parse().unwrap(),
        tenant_id: "22222222-2222-2222-2222-222222222222".parse().unwrap(),
        auth_source: AzureBearerAuthSource::AzureCli,
    }
}
