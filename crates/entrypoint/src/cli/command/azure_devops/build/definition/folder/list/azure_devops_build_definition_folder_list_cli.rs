use super::AzureDevOpsBuildDefinitionFolderListEntry;
use crate::cli::azure_devops::build::output;
use crate::cli::azure_devops::resolve_azure_devops_organization_url;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionReference;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildFolder;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildFolderPathArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_build_definitions;
use cloud_terrastodon_azure_devops::fetch_azure_devops_build_folders;
use cloud_terrastodon_azure_devops::get_default_project_name;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_credentials::AuthContext;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::owo_colors::Stream;
use color_eyre::owo_colors::Style;
use eyre::Result;
use std::collections::BTreeMap;
use std::fmt::Write;

/// List build definition folders and their definitions, including empty folders.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsBuildDefinitionFolderListArgs {
    /// Organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    /// Project ID or name. Defaults to the configured project.
    #[facet(figue::named)]
    pub project: Option<AzureDevOpsProjectArgument<'static>>,
    /// Tenant ID or tracked alias for delegated authentication.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
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
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        let path = self.path.map(AzureDevOpsBuildFolderPathArgument::into_path);
        let auth_context = self.tenant.bind_auth_context(auth_context).await?;
        let org = resolve_azure_devops_organization_url(self.org).await?;
        let project = match self.project {
            Some(project) => project,
            None => get_default_project_name().await?.into(),
        };
        let folders = fetch_azure_devops_build_folders(&org, project.clone(), &auth_context, path)
            .with_invalidation(self.no_cache)
            .await?;
        // Join the complete project inventory locally, rather than relying on
        // a definition path filter to include every returned child folder.
        let definitions =
            fetch_azure_devops_build_definitions(&org, project, &auth_context, None, None)
                .with_invalidation(self.no_cache)
                .await?;
        let folders = group_folders(folders, definitions);
        Ok(CliOutput::facet_with_text(folders, |folders| {
            render_folders(folders)
        }))
    }
}

fn group_folders(
    folders: Vec<AzureDevOpsBuildFolder>,
    definitions: Vec<AzureDevOpsBuildDefinitionReference>,
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

fn render_folders(folders: &[AzureDevOpsBuildDefinitionFolderListEntry]) -> Result<String> {
    if folders.is_empty() {
        return Ok("No build definition folders found.\n".to_owned());
    }
    let mut rendered = String::new();
    writeln!(
        rendered,
        "{} ({})",
        "Build definition folders".if_supports_color(Stream::Stdout, |text| text
            .style(Style::new().cyan().bold())),
        folders.len()
    )?;
    for folder in folders {
        let count = folder.definitions.len();
        let noun = if count == 1 {
            "definition"
        } else {
            "definitions"
        };
        writeln!(
            rendered,
            "  {} ({count} {noun})",
            output::terminal_text(folder.folder.path.as_str())
                .if_supports_color(Stream::Stdout, |text| text.bold()),
        )?;
        for definition in &folder.definitions {
            writeln!(
                rendered,
                "    {}  {}",
                definition
                    .id
                    .if_supports_color(Stream::Stdout, |id| id.green()),
                output::terminal_text(definition.name.as_str())
                    .if_supports_color(Stream::Stdout, |text| text.bold()),
            )?;
        }
    }
    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionId;
    use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionUri;

    fn folder(path: &str) -> AzureDevOpsBuildFolder {
        AzureDevOpsBuildFolder {
            path: path.parse().unwrap(),
            description: None,
            project: None,
            created_by: None,
            created_on: None,
            last_changed_by: None,
            last_changed_date: None,
        }
    }

    fn definition(id: i32, path: &str) -> AzureDevOpsBuildDefinitionReference {
        let id = AzureDevOpsBuildDefinitionId::new(id).unwrap();
        AzureDevOpsBuildDefinitionReference {
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
        }
    }

    #[test]
    fn folder_listing_preserves_empty_folders_and_counts_direct_definitions() -> Result<()> {
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
        let rendered = render_folders(&folders)?;
        assert!(rendered.contains("(0 definitions)"));
        assert!(rendered.contains("(1 definition)"));
        assert!(rendered.contains("(2 definitions)"));
        assert!(rendered.contains("Synthetic pipeline 3"));
        assert!(!rendered.contains("Synthetic pipeline 5"));
        Ok(())
    }
}
