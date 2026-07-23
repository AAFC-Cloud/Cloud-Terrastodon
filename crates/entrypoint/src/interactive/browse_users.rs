use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::EntraUser;
use cloud_terrastodon_azure::fetch_all_entra_users;
use cloud_terrastodon_azure::search_entra_users;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerEvent;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use tracing::info;

pub async fn browse_users(tenant_id: AzureTenantId) -> Result<()> {
    let users = PickerTui::<EntraUser>::new()
        .set_header("Users")
        .add_named_event_handler("fetching all users", {
            move |event, sink| async move {
                if matches!(event.as_ref(), PickerEvent::InitialLoad) {
                    let users = fetch_all_entra_users(tenant_id).await?;
                    sink.push(users.into_iter().map(user_choice))?;
                }
                Ok(())
            }
        })
        .add_named_event_handler("fetching query", {
            move |event, sink| async move {
                let query = match event.as_ref() {
                    PickerEvent::QueryChanged(query) | PickerEvent::ReloadRequested(query) => query,
                    PickerEvent::InitialLoad | PickerEvent::QueryCleared => return Ok(()),
                };
                let users = search_entra_users(tenant_id, query.as_ref()).await?;
                sink.push(users.into_iter().map(user_choice))?;
                Ok(())
            }
        })
        .pick_many_events()
        .await?;
    info!("You chose:");
    for user in users {
        println!(
            "- {} {:64} {}",
            user.id, user.display_name, user.user_principal_name
        );
    }
    Ok(())
}

fn user_choice(user: EntraUser) -> Choice<EntraUser> {
    Choice {
        key: format!(
            "{} {:64} {}",
            user.id, user.display_name, user.user_principal_name
        ),
        value: user,
    }
}
