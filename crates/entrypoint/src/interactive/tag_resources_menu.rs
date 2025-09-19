use cloud_terrastodon_azure::prelude::Resource;
use cloud_terrastodon_azure::prelude::ResourceGroup;
use cloud_terrastodon_azure::prelude::ResourceTagsId;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::fetch_all_resource_groups;
use cloud_terrastodon_azure::prelude::fetch_all_resources;
use cloud_terrastodon_azure::prelude::get_tags_for_resources;
use cloud_terrastodon_azure::prelude::replace_tags_for_resources;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::prompt_line;
use itertools::Itertools;
use tracing::info;

pub async fn tag_resources_menu() -> eyre::Result<()> {
    let resource_groups = fetch_all_resource_groups().await?;
    let resource_group: ResourceGroup =
        PickerTui::new(resource_groups.into_iter().map(|rg| Choice {
            key: rg.id.expanded_form().to_string(),
            value: rg,
        }))
        .set_header("Choose a resource group")
        .pick_one()?;
    let resources = fetch_all_resources()
        .await?
        .into_iter()
        .filter(|res| {
            res.id
                .expanded_form()
                .starts_with(&resource_group.id.expanded_form())
        })
        .map(|r| Choice {
            key: r.id.expanded_form().to_string(),
            value: r,
        });
    let resources = PickerTui::<Resource>::new(resources)
        .set_header("Choose resources to tag")
        .pick_many()?;
    let resource_tags = get_tags_for_resources(
        resources
            .into_iter()
            .map(|r: Resource| ResourceTagsId::from_scope(&r))
            .collect_vec(),
    )
    .await?;
    let tag_key = prompt_line("Enter tag key: ").await?;
    let tag_value = prompt_line("Enter tag value: ").await?;
    let result = replace_tags_for_resources(
        resource_tags
            .into_iter()
            .map(|(id, mut tags)| {
                tags.insert(tag_key.to_owned(), tag_value.to_owned());
                (id, tags)
            })
            .collect(),
    )
    .await?;
    for tags in result.values() {
        assert_eq!(tags.get(&tag_key), Some(&tag_value));
    }
    info!("Successfully added tag for {} resources", result.len());
    Ok(())
}
