use chrono::Local;
use clap::Args;
use cloud_terrastodon_azure::prelude::ResourceTagsId;
use cloud_terrastodon_azure::prelude::fetch_all_resources;
use cloud_terrastodon_azure::prelude::fetch_current_user;
use cloud_terrastodon_azure::prelude::merge_tags_for_resources;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::prompt_line;
use eyre::Result;
use std::collections::HashMap;
use tracing::info;

/// Arguments for tagging resources that are slated for cleanup.
#[derive(Args, Debug, Clone)]
pub struct AzureTagForCleanupArgs {}

impl AzureTagForCleanupArgs {
    pub async fn invoke(self) -> Result<()> {
        let cleanup_tagged_date = Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let cleanup_tagged_by = fetch_current_user().await?.user_principal_name;

        let cleanup_policy = {
            match PickerTui::new().pick_one([
                "CandidateForDeletion",
                "CandidateForReduction",
                "Other",
            ])? {
                "Other" => prompt_line("Please specify the cleanup policy: ").await?,
                policy => policy.to_string(),
            }
        };

        let resources = fetch_all_resources().await?;

        let chosen_resources = PickerTui::new().pick_many(resources)?;

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
        merge_tags_for_resources(updates).await?;

        Ok(())
    }
}
