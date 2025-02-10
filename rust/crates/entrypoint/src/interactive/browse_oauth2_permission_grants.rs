use cloud_terrastodon_core_azure::prelude::fetch_all_service_principals;
use cloud_terrastodon_core_azure::prelude::fetch_all_users;
use cloud_terrastodon_core_azure::prelude::fetch_oauth2_permission_grants;
use cloud_terrastodon_core_azure::prelude::ConsentType;
use cloud_terrastodon_core_azure::prelude::OAuth2PermissionGrant;
use cloud_terrastodon_core_azure::prelude::ServicePrincipal;
use cloud_terrastodon_core_azure::prelude::User;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use eyre::bail;
use eyre::Result;
use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::HashMap;
use tokio::try_join;

pub async fn browse_oauth2_permission_grants() -> Result<()> {
    let grants = fetch_oauth2_permission_grants();
    let service_principals = fetch_all_service_principals();
    let users = fetch_all_users();
    let (grants, service_principals, users) = try_join!(grants, service_principals, users)?;
    let service_principals_map = service_principals
        .iter()
        .map(|x| (&x.id, x))
        .collect::<HashMap<_, _>>();
    let users_map = users.iter().map(|x| (&x.id, x)).collect::<HashMap<_, _>>();
    #[derive(Debug)]
    struct Grant<'a> {
        grant: OAuth2PermissionGrant,
        service_principal: &'a ServicePrincipal,
        target: Target<'a>,
    }
    #[derive(Debug)]
    enum Target<'a> {
        AllPrincipals,
        User(&'a User),
    }
    let grants = grants
        .into_iter()
        .map(|grant| {
            let Some(service_principal) = service_principals_map.get(&grant.client_id) else {
                bail!(
                    "Failed to find service principal with id {:?} for grant {:?}",
                    &grant.client_id,
                    grant
                );
            };
            Ok(match (&grant.consent_type, &grant.principal_id) {
                (ConsentType::AllPrincipals, None) => Grant {
                    grant,
                    service_principal,
                    target: Target::AllPrincipals,
                },
                (ConsentType::Principal, Some(user_id)) => {
                    let Some(user) = users_map.get(&user_id.clone()) else {
                        bail!("User not found with id {} for grant {:?}", user_id, grant);
                    };
                    Grant {
                        grant,
                        service_principal,
                        target: Target::User(user),
                    }
                }
                _ => bail!("Invalid state: consent type inconsistent with principal id for {:?}", grant)
            })
        })
        .collect::<eyre::Result<Vec<Grant>>>()?;
    let chosen = pick_many(FzfArgs {
        choices: grants
            .into_iter()
            .map(|g| Choice {
                key: format!(
                    "{}\n| {}\n| {}",
                    g.service_principal.display_name,
                    match &g.target {
                        Target::User(user) => format!("User ({})", user.user_principal_name),
                        x => format!("{x:?}")
                    },
                    g.grant.scope.trim()
                ),
                value: g,
            })
            .sorted_unstable_by(|a, b| {
                a.grant.client_id
                    .cmp(&b.grant.client_id)
                    .then_with(|| match (a.grant.principal_id, b.grant.principal_id) {
                        (Some(a), Some(b)) => a.cmp(&b),
                        (None, None) => Ordering::Equal,
                        (a, b) => a.is_some().cmp(&b.is_some()),
                    })
            })
            .collect_vec(),
        prompt: None,
        header: Some("Pick the items to browse".to_string()),
    })?;
    // todo!("fix sorting by service principal clientid, add id in parens");
    // todo!("commit changes");
    println!("You chose {} items", chosen.len());
    for item in chosen {
        println!("{:#?}", item);
    }
    Ok(())
}
