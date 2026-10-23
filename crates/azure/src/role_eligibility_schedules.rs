use cloud_terrastodon_azure_types::RoleEligibilitySchedule;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use std::path::PathBuf;

#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct MyEntraRoleEligibilityScheduleListRequest {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
}

pub fn fetch_my_role_eligibility_schedules(
    cancellation: &CancellationScope,
) -> MyEntraRoleEligibilityScheduleListRequest {
    MyEntraRoleEligibilityScheduleListRequest {
        cancellation: cancellation.clone(),
    }
}

#[async_trait]
impl cloud_terrastodon_command::CacheableCommand for MyEntraRoleEligibilityScheduleListRequest {
    type Output = Vec<RoleEligibilitySchedule>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "roleEligibilitySchedules",
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let url = "https://management.azure.com/providers/Microsoft.Authorization/roleEligibilitySchedules?api-version=2020-10-01&$filter=asTarget()";

        #[derive(facet::Facet)]
        struct Response {
            value: Vec<RoleEligibilitySchedule>,
        }

        let request =
            RestRequest::new(http::Method::GET, url, &self.cancellation)?.cache(self.cache_key());
        let mut result: Result<Response, _> = request.clone().receive().await;
        if result.is_err() {
            self.cancellation.bail_if_cancelled()?;
            // single retry - sometimes this returns a gateway error
            result = request.receive().await;
        }
        Ok(result?.value)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(MyEntraRoleEligibilityScheduleListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::expect_aad_premium_p2_license;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let Some(found) = expect_aad_premium_p2_license(
                    fetch_my_role_eligibility_schedules(&cancellation).await,
                )
                .await?
                else {
                    return Ok(());
                };
                assert!(!found.is_empty());
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(MyEntraRoleEligibilityScheduleListRequest);
cloud_terrastodon_registry::register_arbitrary!(MyEntraRoleEligibilityScheduleListRequest);
cloud_terrastodon_registry::register_into_future!(MyEntraRoleEligibilityScheduleListRequest => Vec<RoleEligibilitySchedule>, effects = [Read]);
