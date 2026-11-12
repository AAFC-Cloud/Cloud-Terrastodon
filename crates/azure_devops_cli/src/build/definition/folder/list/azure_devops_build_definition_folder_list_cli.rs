use super::AzureDevOpsBuildDefinitionFolderListEntry;
use crate::build::output;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionReference;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildFolder;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildFolderListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildFolderPathArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::owo_colors::Stream;
use color_eyre::owo_colors::Style;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io::Write;
use std::sync::Arc;

/// List build definition folders and their definitions, including empty folders.
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsBuildDefinitionFolderListArgs {
    /// Organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project ID or name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,
    /// Refresh cached folders and definitions before listing them.
    #[facet(
        figue::named,
        figue::alias = "skip-cache",
        figue::alias = "clean",
        default = false
    )]
    pub no_cache: bool,
    /// Starting folder path. Defaults to all project folders.
    #[facet(figue::named)]
    pub path: Option<AzureDevOpsBuildFolderPathArgument>,
}

impl AzureDevOpsBuildDefinitionFolderListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let path = self.path.map(AzureDevOpsBuildFolderPathArgument::into_path);
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let folders = AzureDevOpsBuildFolderListRequest {
            org_url: Cow::Borrowed(&self.org),
            project: (self.project.clone()),
            auth_selection,
            path: (path),
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;
        // Join the complete project inventory locally, rather than relying on
        // a definition path filter to include every returned child folder.
        let definitions = AzureDevOpsBuildDefinitionListRequest {
            org_url: Cow::Borrowed(&self.org),
            project: (self.project),
            auth_selection,
            name: (None),
            path: (None),
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;
        let folders = group_folders(folders, definitions);
        Ok(CliOutput::facet_with_text(
            folders,
            |folders, writer, cancellation| write_folders(folders, writer, cancellation),
        ))
    }
}

fn group_folders(
    folders: Vec<Arc<AzureDevOpsBuildFolder>>,
    definitions: Vec<Arc<AzureDevOpsBuildDefinitionReference>>,
) -> Vec<AzureDevOpsBuildDefinitionFolderListEntry> {
    let mut definitions_by_path = BTreeMap::<_, Vec<_>>::new();
    for definition in definitions {
        definitions_by_path
            .entry(definition.path.case_insensitive_key())
            .or_default()
            .push(definition);
    }
    folders
        .into_iter()
        .map(|folder| {
            let definitions = definitions_by_path
                .get(&folder.path.case_insensitive_key())
                .cloned()
                .unwrap_or_default();
            AzureDevOpsBuildDefinitionFolderListEntry {
                folder,
                definitions,
            }
        })
        .collect()
}

fn write_folders(
    folders: &[AzureDevOpsBuildDefinitionFolderListEntry],
    writer: &mut dyn Write,
    cancellation: &CancellationScope,
) -> Result<()> {
    if folders.is_empty() {
        writeln!(writer, "No build definition folders found.")?;
        return Ok(());
    }
    writeln!(
        writer,
        "{} ({})",
        "Build definition folders".if_supports_color(Stream::Stdout, |text| text
            .style(Style::new().cyan().bold())),
        folders.len()
    )?;
    for folder in folders {
        cancellation.bail_if_cancelled()?;
        let count = folder.definitions.len();
        let noun = if count == 1 {
            "definition"
        } else {
            "definitions"
        };
        writeln!(
            writer,
            "  {} ({count} {noun})",
            output::terminal_text(folder.folder.path.as_str())
                .if_supports_color(Stream::Stdout, |text| text.bold()),
        )?;
        for definition in &folder.definitions {
            cancellation.bail_if_cancelled()?;
            writeln!(
                writer,
                "    {}  {}",
                definition
                    .id
                    .if_supports_color(Stream::Stdout, |id| id.green()),
                output::terminal_text(definition.name.as_str())
                    .if_supports_color(Stream::Stdout, |text| text.bold()),
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionId;
    use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionUri;
    use cloud_terrastodon_command::CancellationToken;

    fn folder(path: &str) -> Arc<AzureDevOpsBuildFolder> {
        Arc::new(AzureDevOpsBuildFolder {
            path: path.parse().unwrap(),
            description: None,
            project: None,
            created_by: None,
            created_on: None,
            last_changed_by: None,
            last_changed_date: None,
        })
    }

    fn definition(id: i32, path: &str) -> Arc<AzureDevOpsBuildDefinitionReference> {
        let id = AzureDevOpsBuildDefinitionId::new(id).unwrap();
        Arc::new(AzureDevOpsBuildDefinitionReference {
            id,
            name: format!("Synthetic pipeline {id}").parse().unwrap(),
            path: path.parse().unwrap(),
            uri: AzureDevOpsBuildDefinitionUri::new(id),
            created_date: chrono::DateTime::UNIX_EPOCH,
            revision: None,
            quality: None,
            authored_by: None,
            drafts: None,
            draft_of: None,
            queue: None,
            queue_status: None,
            r#type: None,
            url: None,
            project: None,
            links: None,
        })
    }

    #[test]
    fn folder_listing_preserves_empty_folders_and_counts_direct_definitions() -> Result<()> {
        CancellationToken::new().with_scope(|cancellation| {
            let folders = group_folders(
                vec![
                    folder(r"\Team\CI"),
                    folder(r"\Empty"),
                    folder(r"\"),
                    folder(r"\Team"),
                ],
                vec![
                    definition(1, r"\Team"),
                    definition(2, r"\team"),
                    definition(3, r"\TEAM\CI"),
                    definition(4, r"\"),
                    definition(5, r"\TeamOther"),
                ],
            );
            assert_eq!(
                folders
                    .iter()
                    .map(|entry| entry.folder.path.as_str())
                    .collect::<Vec<_>>(),
                [r"\Team\CI", r"\Empty", r"\", r"\Team"]
            );
            assert_eq!(
                folders
                    .iter()
                    .map(|entry| entry
                        .definitions
                        .iter()
                        .map(|definition| definition.id.get())
                        .collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
                [vec![3], vec![], vec![4], vec![1, 2]]
            );
            let mut rendered = Vec::new();
            write_folders(&folders, &mut rendered, &cancellation)?;
            let rendered = String::from_utf8(rendered)?;
            assert!(rendered.contains("(0 definitions)"));
            assert!(rendered.contains("(1 definition)"));
            assert!(rendered.contains("(2 definitions)"));
            assert!(rendered.contains("Synthetic pipeline 3"));
            assert!(!rendered.contains("Synthetic pipeline 5"));
            Ok(())
        })
    }
}
