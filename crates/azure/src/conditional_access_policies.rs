use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use cloud_terrastodon_azure_types::prelude::ConditionalAccessPolicy;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
pub struct ConditionalAccessPolicyListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_conditional_access_policies(
    tenant_id: AzureTenantId,
) -> ConditionalAccessPolicyListRequest {
    ConditionalAccessPolicyListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for ConditionalAccessPolicyListRequest {
    type Output = Vec<ConditionalAccessPolicy>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "conditional_access_policies",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let query = MicrosoftGraphHelper::new(
            self.tenant_id,
            "https://graph.microsoft.com/beta/identity/conditionalAccess/policies",
            Some(self.cache_key()),
        );

        let policies = query.fetch_all().await?;
        Ok(policies)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ConditionalAccessPolicyListRequest);

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_conditional_access_named_locations;
    use crate::prelude::fetch_all_conditional_access_policies;
    use crate::prelude::get_test_tenant_id;
    use cloud_terrastodon_azure_types::prelude::AllOr;
    use cloud_terrastodon_azure_types::prelude::ConditionalAccessPolicyGrantControlBuiltInControl;
    use cloud_terrastodon_azure_types::prelude::ConditionalAccessPolicyState;
    use cloud_terrastodon_azure_types::prelude::ipnetwork::Ipv4Network;
    use std::net::Ipv4Addr;
    use tokio::try_join;
    use tracing::warn;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let found = fetch_all_conditional_access_policies(get_test_tenant_id().await?).await?;
        assert!(!found.is_empty());
        Ok(())
    }

    #[tokio::test]
    pub async fn disallowed_ips() -> eyre::Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let (locations, policies) = try_join!(
            fetch_all_conditional_access_named_locations(tenant_id),
            fetch_all_conditional_access_policies(tenant_id),
        )?;
        assert!(!locations.is_empty());
        assert!(!policies.is_empty());
        let locations_by_id = locations
            .into_iter()
            .map(|location| (*location.id(), location))
            .collect::<std::collections::HashMap<_, _>>();

        enum IncludeOrExclude {
            Include,
            Exclude,
        }
        for policy in policies {
            if policy.state != ConditionalAccessPolicyState::Enabled {
                continue;
            }
            if policy.grant_controls.is_none() {
                warn!(
                    "Policy {} has no grant controls, skipping",
                    policy.display_name
                );
                continue;
            }
            if !policy
                .grant_controls
                .as_ref()
                .unwrap()
                .built_in_controls
                .contains(&ConditionalAccessPolicyGrantControlBuiltInControl::Block)
            {
                warn!(
                    "Policy {} does not block access, skipping",
                    policy.display_name
                );
                continue;
            }
            if let Some(locations) = policy.conditions.locations {
                for (location, _mode) in locations
                    .include_locations
                    .iter()
                    .map(|location| (location, IncludeOrExclude::Include))
                    .chain(
                        locations
                            .exclude_locations
                            .iter()
                            .map(|location| (location, IncludeOrExclude::Exclude)),
                    )
                {
                    let _ips = match location {
                        AllOr::All => {
                            vec![(
                                "All".to_string(),
                                Ipv4Network::new(Ipv4Addr::new(0, 0, 0, 0), 0)?,
                            )]
                        }
                        AllOr::Some(location) => {
                            let Some(location) = locations_by_id.get(location) else {
                                warn!("Failed to find location {}", location);
                                continue;
                            };
                            location
                                .ips()
                                .into_iter()
                                .map(|ip| (location.display_name().to_string(), *ip))
                                .collect()
                        }
                        x => {
                            warn!("Unexpected include location type: {:?}", x);
                            vec![]
                        }
                    };
                    // todo: this should become a cli command to preview this info
                    // match mode {
                    //     IncludeOrExclude::Include => {
                    //         for (display, ip) in ips {
                    //             println!("\tInclude: {} | {}", display, ip);
                    //         }
                    //     }
                    //     IncludeOrExclude::Exclude => {
                    //         for (display, ip) in ips {
                    //             println!("\tExclude: {} | {}", display, ip);
                    //         }
                    //     }
                    // }
                }
            }
        }
        Ok(())
    }
}
