use cloud_terrastodon_azure::prelude::fetch_all_key_vaults;
use cloud_terrastodon_azure::prelude::fetch_all_role_definitions_and_assignments;
use cloud_terrastodon_azure::prelude::fetch_current_user;
use itertools::Itertools;
use tokio::try_join;

#[tokio::test]
#[ignore]
pub async fn predict_would_secret_list_succeed() -> eyre::Result<()> {
    let current_user = fetch_current_user().await?;
    let (key_vaults, rbac) = try_join!(
        fetch_all_key_vaults(),
        fetch_all_role_definitions_and_assignments(),
    )?;

    let key_vaults = key_vaults
        .into_iter()
        .map(|kv| (kv.can_list_secrets(&current_user.id, &rbac), kv))
        .sorted_by(|(a_can, a_kv), (b_can, b_kv)| a_can.cmp(b_can).then(a_kv.name.cmp(&b_kv.name)));
    for (can_list, kv) in key_vaults {
        println!("Key Vault: {} - {}", kv.name, can_list);
    }
    Ok(())
}
