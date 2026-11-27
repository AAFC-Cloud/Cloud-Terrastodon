use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_bmc_helix::BmcHelixServerStore;
use cloud_terrastodon_bmc_helix_types::BmcHelixServer;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use eyre::Result;

pub(super) fn resolve_bmc_helix_account(
    server: Option<&BmcHelixServerName>,
    account: Option<&CloudTerrastodonAccountArgument>,
) -> Result<(BmcHelixServer, CloudTerrastodonAccountId)> {
    let server = BmcHelixServerStore::configured().get(server)?;
    let profile =
        CloudTerrastodonAccountStore::configured().resolve_bmc_helix(&server.url, account)?;
    Ok((server, profile.id))
}
