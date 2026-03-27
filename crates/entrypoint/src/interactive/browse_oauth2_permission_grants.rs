use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::pick_oauth2_permission_grants;

pub async fn browse_oauth2_permission_grants(tenant_id: AzureTenantId) -> eyre::Result<()> {
    let chosen = pick_oauth2_permission_grants(tenant_id).await?;
    // todo!("fix sorting by service principal clientid, add id in parens");
    // todo!("commit changes");
    println!("You chose {} items", chosen.len());
    for item in chosen {
        println!("{item:#?}");
    }
    Ok(())
}
