use crate::prelude::fetch_all_service_principals;
use crate::prelude::fetch_all_users;
use crate::prelude::fetch_oauth2_permission_grants;
use cloud_terrastodon_azure_types::prelude::ConsentType;
use cloud_terrastodon_azure_types::prelude::EntraServicePrincipal;
use cloud_terrastodon_azure_types::prelude::EntraUser;
use cloud_terrastodon_azure_types::prelude::OAuth2PermissionGrant;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::bail;
use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::future::IntoFuture;
use tokio::try_join;

#[derive(Debug)]
pub struct Grant {
    pub grant: OAuth2PermissionGrant,
    pub service_principal: EntraServicePrincipal,
    pub target: Target,
}
#[derive(Debug)]
pub enum Target {
    AllPrincipals,
    User(Box<EntraUser>),
}

impl PartialEq for Grant {
    fn eq(&self, other: &Self) -> bool {
        self.grant.client_id == other.grant.client_id
            && self.grant.principal_id == other.grant.principal_id
    }
}

impl Eq for Grant {}

impl PartialOrd for Grant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Grant {
    fn cmp(&self, other: &Self) -> Ordering {
        self.grant
            .client_id
            .cmp(&other.grant.client_id)
            .then_with(
                || match (&self.grant.principal_id, &other.grant.principal_id) {
                    (Some(a), Some(b)) => a.cmp(b),
                    (None, None) => Ordering::Equal,
                    (Some(_), None) => Ordering::Greater,
                    (None, Some(_)) => Ordering::Less,
                },
            )
    }
}

impl fmt::Display for Grant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\n| {}\n| {}",
            self.service_principal.display_name,
            match &self.target {
                Target::User(user) => format!("User ({})", user.user_principal_name),
                x => format!("{x:?}"),
            },
            self.grant.scope.trim()
        )
    }
}

pub async fn pick_oauth2_permission_grants() -> eyre::Result<Vec<Grant>> {
    let grants = fetch_oauth2_permission_grants();
    let service_principals = fetch_all_service_principals();
    let users = fetch_all_users().into_future();
    let (grants, service_principals, users) = try_join!(grants, service_principals, users)?;
    let service_principals_map = service_principals
        .iter()
        .map(|x| (&x.id, x))
        .collect::<HashMap<_, _>>();
    let users_map = users.iter().map(|x| (&x.id, x)).collect::<HashMap<_, _>>();

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
                    service_principal: (*service_principal).clone(),
                    target: Target::AllPrincipals,
                },
                (ConsentType::Principal, Some(user_id)) => {
                    let Some(user) = users_map.get(&user_id.clone()) else {
                        bail!("User not found with id {} for grant {:?}", user_id, grant);
                    };
                    Grant {
                        grant,
                        service_principal: (*service_principal).clone(),
                        target: Target::User(Box::new((*user).clone())),
                    }
                }
                _ => bail!(
                    "Invalid state: consent type inconsistent with principal id for {:?}",
                    grant
                ),
            })
        })
        .collect::<eyre::Result<Vec<Grant>>>()?;
    let choices = grants.into_iter().sorted_unstable().map(|g| Choice {
        key: g.to_string(),
        value: g,
    });
    let chosen = PickerTui::new()
        .set_header("Pick the items to browse")
        .pick_many(choices)?;
    Ok(chosen)
}
