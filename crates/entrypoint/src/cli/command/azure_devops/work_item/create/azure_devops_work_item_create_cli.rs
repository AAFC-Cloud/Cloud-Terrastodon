use cloud_terrastodon_azure::ArbitraryJson;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsJsonPatch;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemCreateRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemPatchFields;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemRelationInput;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemType;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use eyre::ensure;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemCreateArgs {
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
    /// Work item type name, including custom project types.
    #[facet(figue::named, rename = "type")]
    pub work_item_type: AzureDevOpsWorkItemType,
    /// Title, or supply System.Title in --fields.
    #[facet(figue::named)]
    pub title: Option<String>,
    /// JSON field object, or @file. Values retain their JSON types.
    #[facet(figue::named)]
    pub fields: Option<String>,
    /// JSON array of {rel,url,attributes} inputs, or @file.
    #[facet(figue::named)]
    pub relations: Option<String>,
    /// Link to this existing parent in the create request.
    #[facet(figue::named)]
    pub parent: Option<AzureDevOpsWorkItemId>,
    /// Ask the server to validate without saving changes.
    #[facet(figue::named, default)]
    pub validate_only: bool,
    /// Suppress notifications for this change.
    #[facet(figue::named, default)]
    pub suppress_notifications: bool,
}

impl AzureDevOpsWorkItemCreateArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        let mut fields: AzureDevOpsWorkItemPatchFields = match self.fields {
            Some(input) => {
                let body = cloud_terrastodon_rest::read_optional_body(Some(input))
                    .await?
                    .ok_or_else(|| eyre::eyre!("Missing JSON input"))?;
                facet_json::from_str(&body)
                    .map_err(|error| eyre::eyre!("Invalid JSON input: {error}"))?
            }
            None => Default::default(),
        };
        if let Some(title) = self.title {
            ensure!(
                !fields.contains_key("System.Title"),
                "Supply the title once, through --title or --fields"
            );
            fields.insert(
                "System.Title".parse()?,
                ArbitraryJson::try_from_facet(&title)?,
            );
        }
        let mut relations: Vec<AzureDevOpsWorkItemRelationInput> = match self.relations {
            Some(input) => {
                let body = cloud_terrastodon_rest::read_optional_body(Some(input))
                    .await?
                    .ok_or_else(|| eyre::eyre!("Missing JSON input"))?;
                facet_json::from_str(&body)
                    .map_err(|error| eyre::eyre!("Invalid JSON input: {error}"))?
            }
            None => Vec::new(),
        };
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let auth_context = self.tenant.bind_auth_context(auth).await?;
        if let Some(parent) = self.parent {
            relations.push(AzureDevOpsWorkItemRelationInput::parent(&org_url, parent)?);
        }
        let patch = AzureDevOpsJsonPatch::work_item_create_patch(fields, &relations)?;
        let item = AzureDevOpsWorkItemCreateRequest {
            org_url: Cow::Owned(org_url),
            project: self.project,
            auth_context: Cow::Owned(auth_context),
            work_item_type: self.work_item_type,
            patch,
            validate_only: self.validate_only,
            suppress_notifications: self.suppress_notifications,
        }
        .await?;
        cloud_terrastodon_command::to_writer_pretty(std::io::stdout(), &item)?;
        Ok(())
    }
}
