use chrono::Local;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureResourceListRequest;
use cloud_terrastodon_azure::AzureResourceTagsPatchOperation;
use cloud_terrastodon_azure::AzureResourceTagsUpdateRequest;
use cloud_terrastodon_azure::EntraCurrentUserRequest;
use cloud_terrastodon_azure::ResourceTagsId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::prompt_line;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use tracing::info;

/// Arguments for tagging resources that are slated for cleanup.
#[derive(Facet, Debug, Clone)]
pub struct AzureTagForCleanupArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureTagForCleanupArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let cleanup_tagged_date = Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let cleanup_tagged_by = EntraCurrentUserRequest { authentication }
            .execute(executor)
            .await?
            .user_principal_name
            .clone();

        let cleanup_policy = {
            match PickerTui::<_>::new()
                .pick_one(
                    ["CandidateForDeletion", "CandidateForReduction", "Other"],
                    executor.cancellation(),
                )
                .await?
            {
                "Other" => prompt_line("Please specify the cleanup policy: ").await?,
                policy => policy.to_string(),
            }
        };

        let resources = AzureResourceListRequest { authentication }
            .execute(executor)
            .await?;

        let chosen_resources = PickerTui::<_>::new()
            .pick_many(resources, executor.cancellation())
            .await?;

        let cleanup_comments = prompt_line("CleanupComments: ").await?;

        let tags_to_merge: HashMap<String, String> = [
            ("CleanupTaggedBy".to_string(), cleanup_tagged_by.to_string()),
            ("CleanupComments".to_string(), cleanup_comments.to_string()),
            ("CleanupPolicy".to_string(), cleanup_policy.to_string()),
            (
                "CleanupTaggedDate".to_string(),
                cleanup_tagged_date.to_string(),
            ),
        ]
        .into();

        let mut updates: HashMap<ResourceTagsId, HashMap<String, String>> = HashMap::new();
        for resource in chosen_resources {
            updates.insert(
                ResourceTagsId::from_scope(&resource.id),
                tags_to_merge.clone(),
            );
            info!(
                resource_id=%resource.id,
                "Prepared to add cleanup tags to resource"
            )
        }

        info!(?tags_to_merge, "Executing tag modifications");
        updates
            .into_iter()
            .map_concurrent(
                executor,
                NonZeroUsize::try_from(20)?,
                async |child, (id, tags)| {
                    AzureResourceTagsUpdateRequest {
                        authentication,
                        id: Cow::Owned(id),
                        operation: AzureResourceTagsPatchOperation::Merge,
                        tags,
                    }
                    .execute(&child)
                    .await
                },
            )
            .await?;

        Ok(CliOutput::none())
    }
}
