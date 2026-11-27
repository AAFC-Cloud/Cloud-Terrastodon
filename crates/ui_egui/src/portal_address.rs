use crate::app::MyApp;
use crate::portal_address_format::PortalAddressFormat;
use crate::portal_azure_devops_organization_view::PortalAzureDevOpsOrganizationView;
use crate::portal_omnibox::OMNIBOX_CONTENT_HEIGHT;
use crate::portal_omnibox::portal_omnibox;
use crate::portal_page::PortalPage;
use crate::portal_pim::PIM_PORTAL_URL;
use crate::portal_resource_output_target::PortalResourceOutputTarget;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_scope_view::PortalScopeView;
use crate::portal_tab_target::PortalTabTarget;
use crate::resource_group_list_command::resource_group_list_command;
use crate::storage_account_show_command::storage_account_show_command;
use cloud_terrastodon_app::OutputFormat;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountAlias;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::PercentEncodeExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure_cli::audit_cli::AzureAuditArgs;
use cloud_terrastodon_azure_cli::entra::application_registration::AzureEntraApplicationRegistrationListArgs;
use cloud_terrastodon_azure_cli::entra::application_registration::AzureEntraApplicationRegistrationShowArgs;
use cloud_terrastodon_azure_cli::entra::group::AzureEntraGroupListArgs;
use cloud_terrastodon_azure_cli::entra::group::AzureEntraGroupSearchArgs;
use cloud_terrastodon_azure_cli::entra::user::AzureEntraUserListArgs;
use cloud_terrastodon_azure_cli::entra::user::AzureEntraUserShowArgs;
use cloud_terrastodon_azure_cli::resource::AzureResourceShowArgs;
use cloud_terrastodon_azure_cli::storage::account::AzureStorageAccountAuditArgs;
use cloud_terrastodon_azure_cli::storage::blob::container::AzureStorageBlobContainerListArgs;
use cloud_terrastodon_azure_cli::storage::blob::container::AzureStorageBlobContainerShowArgs;
use cloud_terrastodon_azure_cli::tenant::AzureTenantShowArgs;
use cloud_terrastodon_azure_devops_cli::AzureDevOpsAuditArgs;
use cloud_terrastodon_azure_devops_cli::project::list::AzureDevOpsProjectListArgs;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use eframe::egui::Align;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::CursorIcon;
use eframe::egui::Id;
use eframe::egui::Layout;
use eframe::egui::OpenUrl;
use eframe::egui::Response;
use eframe::egui::RichText;
use eframe::egui::Sense;
use eframe::egui::Shape;
use eframe::egui::Stroke;
use eframe::egui::StrokeKind;
use eframe::egui::Ui;
use eframe::egui::UiBuilder;
use eframe::egui::Vec2;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetType;
use eframe::egui::containers::menu::MenuButton;
use eyre::Result;
use figue::ToArgs;
use std::sync::Arc;

/// The display address derived from a typed destination and execution identity.
pub(crate) struct PortalAddress {
    target: PortalTabTarget,
    authentication: Option<AzureBearerAuthenticationSelection>,
    devops_authentication: Option<AzureDevOpsAuthSelection>,
    account_alias: Option<CloudTerrastodonAccountAlias>,
    devops_project_name: Option<String>,
    cli_command: Result<Option<String>>,
    identifier: Option<String>,
    azure_portal_url: Option<String>,
}

impl PortalAddress {
    fn devops_cli_command(
        target: &PortalTabTarget,
        authentication: AzureDevOpsAuthSelection,
        account_alias: Option<&CloudTerrastodonAccountAlias>,
    ) -> Result<Option<String>> {
        let authentication = match authentication {
            AzureDevOpsAuthSelection::Bearer(selection) => AzureDevOpsAuthenticationArgs {
                tenant: Some(selection.tenant_id.into()),
                account: Some(account_argument(selection.account_id, account_alias)),
                auth_source: selection.auth_source.into(),
            },
            AzureDevOpsAuthSelection::PersonalAccessToken { account_id } => {
                AzureDevOpsAuthenticationArgs {
                    tenant: None,
                    account: Some(account_argument(account_id, account_alias)),
                    auth_source: AuthSource::PersonalAccessToken,
                }
            }
        };
        match target {
            PortalTabTarget::AzureDevOpsOrganization {
                organization,
                view: PortalAzureDevOpsOrganizationView::Projects,
            } => format_shell_command(
                "ct az devops project list".to_owned(),
                &AzureDevOpsProjectListArgs {
                    no_cache: false,
                    org: organization.as_ref().clone(),
                    authentication,
                },
            )
            .map(Some),
            PortalTabTarget::AzureDevOpsProject {
                organization,
                project,
                view,
            } => view.cli_command(organization, *project, authentication),
            _ => Ok(None),
        }
    }

    fn cli_command(
        target: &PortalTabTarget,
        authentication: Option<AzureBearerAuthenticationSelection>,
        account_alias: Option<&CloudTerrastodonAccountAlias>,
    ) -> Result<Option<String>> {
        match target {
            PortalTabTarget::Page(PortalPage::AzureTenants) => {
                return Ok(Some("ct az tenant list".into()));
            }
            PortalTabTarget::Page(PortalPage::AzureDevOpsOrganizations) => {
                return Ok(Some("ct az devops org list".into()));
            }
            PortalTabTarget::AzureTenant(tenant) => {
                let authentication = authentication
                    .filter(|selection| selection.tenant_id == *tenant)
                    .map(|selection| authentication_args(selection, account_alias))
                    .unwrap_or_else(|| AzureAuthenticationArgs {
                        tenant: Some((*tenant).into()),
                        ..Default::default()
                    });
                return Ok(Some(format_cli_command(
                    "tenant show",
                    &AzureTenantShowArgs { authentication },
                )?));
            }
            _ => {}
        }
        let Some(selection) = authentication else {
            return Ok(None);
        };
        let authentication = authentication_args(selection, account_alias);
        let command = match target {
            PortalTabTarget::Page(PortalPage::ResourceGroups) => {
                resource_group_list_command(authentication)?
            }
            PortalTabTarget::Page(PortalPage::Auditing) => format_cli_command(
                "audit",
                &AzureAuditArgs {
                    authentication,
                    clean: false,
                },
            )?,
            PortalTabTarget::Page(PortalPage::StorageAccountAudit) => format_cli_command(
                "storage account audit",
                &AzureStorageAccountAuditArgs {
                    authentication,
                    clean: false,
                },
            )?,
            PortalTabTarget::Page(
                page @ (PortalPage::Resources
                | PortalPage::Subscriptions
                | PortalPage::StorageAccounts),
            ) => {
                let operation = match page {
                    PortalPage::Resources => "resource list",
                    PortalPage::Subscriptions => "subscription list",
                    _ => "storage account list",
                };
                format_cli_command(operation, &authentication)?
            }
            PortalTabTarget::Page(PortalPage::EntraUsers) => format_cli_command(
                "entra user list",
                &AzureEntraUserListArgs { authentication },
            )?,
            PortalTabTarget::Page(PortalPage::EntraGroups) => format_cli_command(
                "entra group list",
                &AzureEntraGroupListArgs {
                    authentication,
                    for_member: None,
                    no_cache: false,
                },
            )?,
            PortalTabTarget::Page(PortalPage::EntraAppRegistrations) => format_cli_command(
                "entra application-registration list",
                &AzureEntraApplicationRegistrationListArgs {
                    authentication,
                    no_cache: false,
                },
            )?,
            PortalTabTarget::EntraUser(id) => format_cli_command(
                "entra user show",
                &AzureEntraUserShowArgs {
                    authentication,
                    user: (*id).into(),
                },
            )?,
            PortalTabTarget::EntraGroup(id) => format_cli_command(
                "entra group search",
                &AzureEntraGroupSearchArgs {
                    authentication,
                    search_term: id.to_string(),
                    no_cache: false,
                },
            )?,
            PortalTabTarget::EntraAppRegistration(id) => format_cli_command(
                "entra application-registration show",
                &AzureEntraApplicationRegistrationShowArgs {
                    authentication,
                    application_registration: id.to_string(),
                    no_cache: false,
                },
            )?,
            PortalTabTarget::ScopeView {
                scope: PortalResourceScope::StorageAccount(id),
                view: PortalScopeView::StorageExplorer,
            } => format_cli_command(
                "storage blob container list",
                &AzureStorageBlobContainerListArgs {
                    authentication,
                    storage_account: id.clone(),
                    no_cache: false,
                },
            )?,
            PortalTabTarget::StorageAccount(id)
            | PortalTabTarget::ScopeView {
                scope: PortalResourceScope::StorageAccount(id),
                view: PortalScopeView::Networking,
            } => storage_account_show_command(authentication, id, false, None)?,
            PortalTabTarget::StorageContainer(id) => format_shell_command(
                "ct az storage blob container show".to_owned(),
                &AzureStorageBlobContainerShowArgs {
                    authentication,
                    storage_account: id.storage_account_id.clone(),
                    container: id.container_name.clone(),
                    no_cache: false,
                },
            )?,
            PortalTabTarget::Resource(id) => {
                resource_show_command(authentication, id.expanded_form(), None)?
            }
            PortalTabTarget::ResourceOutput { resource, format } => match resource {
                PortalResourceOutputTarget::StorageAccount(id) => {
                    storage_account_show_command(authentication, id, false, Some(*format))?
                }
                PortalResourceOutputTarget::Scope(PortalResourceScope::Resource(id)) => {
                    resource_show_command(authentication, id.expanded_form(), Some(*format))?
                }
                _ => return Ok(None),
            },
            _ => return Ok(None),
        };
        Ok(Some(command))
    }

    fn identifier(target: &PortalTabTarget) -> Option<String> {
        Some(match target {
            PortalTabTarget::ManagementGroup(id) => id.expanded_form(),
            PortalTabTarget::Subscription(id) => id.expanded_form(),
            PortalTabTarget::ResourceGroup(id) => id.expanded_form(),
            PortalTabTarget::StorageAccount(id) => id.expanded_form(),
            PortalTabTarget::StorageContainer(id) => id.to_string(),
            PortalTabTarget::Resource(id) => id.expanded_form(),
            PortalTabTarget::ResourceOutput { resource, .. } => resource.resource_id(),
            PortalTabTarget::ScopeView { scope, .. } => scope.arm_scope().expanded_form(),
            PortalTabTarget::EntraUser(id) => id.to_string(),
            PortalTabTarget::EntraGroup(id) => id.to_string(),
            PortalTabTarget::EntraAppRegistration(id) => id.to_string(),
            PortalTabTarget::AzureTenant(id) => id.to_string(),
            PortalTabTarget::ResourcesOfType(kind) => kind.to_string(),
            PortalTabTarget::LocalPath(path) => path.display().to_string(),
            PortalTabTarget::AzureDevOpsOrganization { organization, view } => match view {
                PortalAzureDevOpsOrganizationView::Project(id) => id.to_string(),
                _ => organization.expanded_form(),
            },
            PortalTabTarget::AzureDevOpsProject { project, .. } => project.to_string(),
            PortalTabTarget::Page(_) => return None,
        })
    }

    fn azure_portal_url(
        target: &PortalTabTarget,
        tenant_id: Option<AzureTenantId>,
    ) -> Option<String> {
        let id = match target {
            // Microsoft documents this shortcut for opening My roles directly.
            // https://learn.microsoft.com/en-us/entra/id-governance/privileged-identity-management/groups-activate-roles#activate-a-role
            PortalTabTarget::Page(page) if page.is_pim_section() => {
                return Some(PIM_PORTAL_URL.into());
            }
            PortalTabTarget::AzureDevOpsOrganization {
                organization,
                view: PortalAzureDevOpsOrganizationView::Overview,
            } => return Some(organization.expanded_form()),
            PortalTabTarget::Page(PortalPage::Subscriptions) => {
                return Some(
                    "https://portal.azure.com/#view/Microsoft_Azure_Billing/SubscriptionsBladeV2"
                        .into(),
                );
            }
            PortalTabTarget::Subscription(id) => id.expanded_form(),
            PortalTabTarget::ResourceGroup(id) => id.expanded_form(),
            PortalTabTarget::ScopeView { scope, .. } => scope.arm_scope().expanded_form(),
            PortalTabTarget::StorageAccount(id) => id.expanded_form(),
            PortalTabTarget::StorageContainer(id) => id.storage_account_id.expanded_form(),
            PortalTabTarget::Resource(id) => id.expanded_form(),
            PortalTabTarget::ResourceOutput { resource, .. } => resource.resource_id(),
            _ => return None,
        };
        let tenant_id = tenant_id?;
        // Escape the ARM ID's components while keeping the portal fragment's separators.
        let path = id
            .split('/')
            .map(PercentEncodeExt::percent_encode)
            .collect::<Vec<_>>()
            .join("/");
        Some(format!(
            "https://portal.azure.com/#@{tenant_id}/resource{path}/overview"
        ))
    }

    fn value(&self, format: PortalAddressFormat) -> Option<&str> {
        match format {
            PortalAddressFormat::PortalUrl => self.azure_portal_url.as_deref(),
            PortalAddressFormat::Cli => self.cli_command.as_ref().ok()?.as_deref(),
            PortalAddressFormat::Id => self.identifier.as_deref(),
        }
    }

    fn default_format(&self) -> Option<PortalAddressFormat> {
        let order = if matches!(
            self.target,
            PortalTabTarget::ResourceOutput { .. }
                | PortalTabTarget::ScopeView { .. }
                | PortalTabTarget::StorageContainer(_)
        ) {
            [
                PortalAddressFormat::Cli,
                PortalAddressFormat::Id,
                PortalAddressFormat::PortalUrl,
            ]
        } else {
            PortalAddressFormat::ALL
        };
        order
            .into_iter()
            .find(|&format| self.value(format).is_some())
    }

    fn show(&self, ui: &mut Ui, search: &mut String, focus_search: bool) -> bool {
        let format_id = Id::new("portal-address-format");
        let selected = ui.data(|data| data.get_temp::<PortalAddressFormat>(format_id));
        let mut format = selected
            .filter(|&format| self.value(format).is_some())
            .or_else(|| self.default_format());
        let border = ui.visuals().widgets.noninteractive.bg_stroke;
        let height = OMNIBOX_CONTENT_HEIGHT + 2.0 * border.width;
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
        let radius = (rect.height() * 0.5).round() as u8;
        ui.painter()
            .rect_filled(rect, radius, ui.visuals().faint_bg_color);

        let mut content = ui.new_child(
            UiBuilder::new()
                .id_salt("portal-address-content")
                .max_rect(rect.shrink(border.width))
                .layout(Layout::right_to_left(Align::Center)),
        );
        content.spacing_mut().item_spacing.x = 0.0;
        content.spacing_mut().interact_size.y = OMNIBOX_CONTENT_HEIGHT;
        content.add_space(8.0);
        if let Some(url) = &self.azure_portal_url {
            open_in_browser(&mut content, url);
            content.add_space(4.0);
        }

        let mut leading_segment = None;
        let changed = content
            .with_layout(Layout::left_to_right(Align::Center), |ui| {
                if let Some(current) = format {
                    let background = ui.painter().add(Shape::Noop);
                    let (choice, picker) = self.show_format_picker(ui, current);
                    let fill = ui.style().interact(&picker).weak_bg_fill;
                    leading_segment = Some((background, picker.rect.right(), fill));
                    if choice != current {
                        format = Some(choice);
                        ui.data_mut(|data| data.insert_temp(format_id, choice));
                    }
                }
                ui.add_space(10.0);
                let title = self.target.title();
                let address = format
                    .and_then(|format| self.value(format))
                    .unwrap_or(&title);
                let error_tooltip = self
                    .cli_command
                    .as_ref()
                    .err()
                    .map(|error| format!("{address}\nCould not format CLI command · {error}"));
                portal_omnibox(
                    ui,
                    address,
                    error_tooltip.as_deref().unwrap_or(address),
                    matches!(
                        format,
                        Some(PortalAddressFormat::Cli | PortalAddressFormat::Id)
                    ),
                    search,
                    focus_search,
                )
            })
            .inner;
        if let Some((background, divider, fill)) = leading_segment {
            // Paint the same rounded shape as the bar, clipped at the join.
            let mut segment = rect;
            segment.set_right(divider);
            ui.painter()
                .with_clip_rect(segment)
                .set(background, Shape::rect_filled(rect, radius, fill));
            ui.painter().vline(divider, rect.y_range(), border);
        }
        ui.painter()
            .rect_stroke(rect, radius, border, StrokeKind::Inside);
        changed
    }

    fn show_format_picker(
        &self,
        ui: &mut Ui,
        current: PortalAddressFormat,
    ) -> (PortalAddressFormat, Response) {
        let menu_style = Arc::clone(ui.style());
        let mut format = current;
        let picker = ui
            .scope(|ui| {
                ui.spacing_mut().button_padding.x = 12.0;
                let widgets = &mut ui.visuals_mut().widgets;
                for visuals in [
                    &mut widgets.inactive,
                    &mut widgets.hovered,
                    &mut widgets.active,
                    &mut widgets.open,
                ] {
                    visuals.expansion = 0.0;
                    visuals.bg_stroke = Stroke::NONE;
                }
                let button = Button::new(current.label())
                    .right_text(RichText::new("▼").size(8.0))
                    .min_size(Vec2::new(96.0, OMNIBOX_CONTENT_HEIGHT))
                    .fill(Color32::TRANSPARENT);
                MenuButton::from_button(button)
                    .ui(ui, |ui| {
                        ui.set_style(menu_style);
                        for choice in PortalAddressFormat::ALL {
                            if self.value(choice).is_some()
                                && ui
                                    .selectable_label(current == choice, choice.label())
                                    .clicked()
                            {
                                format = choice;
                                ui.close();
                            }
                        }
                        if let Some(value) = self.value(current) {
                            ui.separator();
                            if ui.button(format!("Copy {}", current.label())).clicked() {
                                ui.ctx().copy_text(value.to_owned());
                                ui.close();
                            }
                        }
                    })
                    .0
                    .on_hover_text("Address format")
            })
            .inner;
        (format, picker)
    }
}

fn open_in_browser(ui: &mut Ui, url: &str) {
    let label = "Open in browser";
    let (rect, response) =
        ui.allocate_exact_size(Vec2::splat(OMNIBOX_CONTENT_HEIGHT), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, response.enabled(), label));
    let painter = ui.painter();
    let visuals = ui.style().interact(&response);
    let center = rect.center();
    if response.hovered() || response.is_pointer_button_down_on() {
        painter.circle_filled(center, 12.0, visuals.weak_bg_fill);
    }
    if response.has_focus() {
        painter.circle_stroke(center, 12.0, visuals.fg_stroke);
    }
    let stroke = Stroke::new(1.4, visuals.fg_stroke.color);
    let point = |x, y| center + Vec2::new(x, y);
    painter.add(Shape::line(
        vec![
            point(0.0, -5.0),
            point(-5.5, -5.0),
            point(-5.5, 5.5),
            point(5.0, 5.5),
            point(5.0, 0.0),
        ],
        stroke,
    ));
    painter.line_segment([point(-0.5, 0.5), point(6.5, -6.5)], stroke);
    painter.add(Shape::line(
        vec![point(2.0, -6.5), point(6.5, -6.5), point(6.5, -2.0)],
        stroke,
    ));
    if response
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(label)
        .clicked()
    {
        ui.ctx().open_url(OpenUrl::new_tab(url));
    }
}

fn account_argument(
    account_id: CloudTerrastodonAccountId,
    alias: Option<&CloudTerrastodonAccountAlias>,
) -> CloudTerrastodonAccountArgument {
    alias.cloned().map_or(
        CloudTerrastodonAccountArgument::Id(account_id),
        CloudTerrastodonAccountArgument::Alias,
    )
}

fn authentication_args(
    selection: AzureBearerAuthenticationSelection,
    account_alias: Option<&CloudTerrastodonAccountAlias>,
) -> AzureAuthenticationArgs {
    AzureAuthenticationArgs {
        tenant: Some(selection.tenant_id.into()),
        account: Some(account_argument(selection.account_id, account_alias)),
        auth_source: selection.auth_source.into(),
    }
}

fn format_cli_command(operation: &str, args: &impl ToArgs) -> Result<String> {
    format_shell_command(format!("ct az {operation}"), args)
}

fn resource_show_command(
    authentication: AzureAuthenticationArgs,
    resource: String,
    format: Option<OutputFormat>,
) -> Result<String> {
    let args = AzureResourceShowArgs {
        authentication,
        resource,
    };
    let format_flag = match format {
        None => "",
        Some(OutputFormat::Json) => " --output-format json",
        Some(OutputFormat::FacetPretty) => " --output-format facet-pretty",
        Some(OutputFormat::Text) => " --output-format text",
        Some(OutputFormat::Auto) => " --output-format auto",
    };
    format_shell_command(format!("ct{format_flag} az resource show"), &args)
}

pub(crate) fn format_shell_command(mut command: String, args: &impl ToArgs) -> Result<String> {
    for arg in args.to_args()? {
        let arg = arg.to_string_lossy();
        command.push(' ');
        if arg
            .chars()
            .all(|c| c.is_alphanumeric() || "/-_.".contains(c))
        {
            command.push_str(&arg);
        } else {
            // The display command is copied into PowerShell on Windows and a POSIX shell elsewhere.
            command.push('\'');
            #[cfg(windows)]
            for c in arg.chars() {
                // PowerShell treats curly single quotes as string delimiters too.
                if matches!(c, '\'' | '‘' | '’' | '‚' | '‛') {
                    command.push(c);
                }
                command.push(c);
            }
            #[cfg(not(windows))]
            command.push_str(&arg.replace('\'', "'\\''"));
            command.push('\'');
        }
    }
    Ok(command)
}

impl MyApp {
    fn portal_account_alias(
        &self,
        account_id: CloudTerrastodonAccountId,
    ) -> Option<&CloudTerrastodonAccountAlias> {
        let catalogue = self.account_picker.catalogue.as_ref().ok()?;
        Some(&catalogue.by_id(account_id).ok()?.alias)
    }

    pub(crate) fn show_portal_address(
        &mut self,
        ui: &mut Ui,
        target: &PortalTabTarget,
        focus_search: bool,
    ) -> bool {
        if *target == PortalTabTarget::Page(PortalPage::Accounts) {
            return PortalAddress {
                target: target.clone(),
                authentication: None,
                devops_authentication: None,
                account_alias: None,
                devops_project_name: None,
                cli_command: Ok(self.account_manager.cli_command()),
                identifier: None,
                azure_portal_url: None,
            }
            .show(ui, &mut self.azure.resource_search, focus_search);
        }
        let authentication = self.ready_azure_authentication();
        if *target == PortalTabTarget::Page(PortalPage::AzureDevOpsAudit) {
            let cli_command = self
                .azure
                .azure_devops_organization
                .as_ref()
                .zip(authentication)
                .map(|(org, selection)| {
                    format_cli_command(
                        "dev-ops audit",
                        &AzureDevOpsAuditArgs::new(
                            org.clone(),
                            authentication_args(
                                selection,
                                self.portal_account_alias(selection.account_id),
                            ),
                        ),
                    )
                })
                .transpose();
            return PortalAddress {
                target: target.clone(),
                authentication,
                devops_authentication: None,
                account_alias: None,
                devops_project_name: None,
                cli_command,
                identifier: None,
                azure_portal_url: None,
            }
            .show(ui, &mut self.azure.resource_search, focus_search);
        }
        let (devops_authentication, devops_project_name) = match target {
            PortalTabTarget::AzureDevOpsOrganization { organization, .. } => (
                self.ready_azure_devops_authentication_for(organization),
                None,
            ),
            PortalTabTarget::AzureDevOpsProject {
                organization,
                project,
                ..
            } => (
                self.ready_azure_devops_authentication_for(organization),
                self.azure_devops_project_name_in(&self.azure, organization, *project),
            ),
            _ => (None, None),
        };
        let account_alias = devops_authentication
            .and_then(|selection| AuthenticationSelection::from(selection).account_id())
            .or_else(|| authentication.map(|selection| selection.account_id))
            .and_then(|id| self.portal_account_alias(id));
        if self.portal_address.as_ref().is_none_or(|address| {
            &address.target != target
                || address.authentication != authentication
                || address.devops_authentication != devops_authentication
                || address.account_alias.as_ref() != account_alias
                || address.devops_project_name.as_deref() != devops_project_name
        }) {
            self.portal_address = Some(PortalAddress {
                target: target.clone(),
                authentication,
                devops_authentication,
                account_alias: account_alias.cloned(),
                devops_project_name: devops_project_name.map(str::to_owned),
                cli_command: match devops_authentication {
                    Some(selection) => {
                        PortalAddress::devops_cli_command(target, selection, account_alias)
                    }
                    None => PortalAddress::cli_command(target, authentication, account_alias),
                },
                identifier: PortalAddress::identifier(target),
                azure_portal_url: match target {
                    PortalTabTarget::AzureDevOpsProject {
                        organization, view, ..
                    } => devops_project_name.map(|name| view.portal_url(organization, name)),
                    _ => PortalAddress::azure_portal_url(
                        target,
                        authentication.map(|selection| selection.tenant_id),
                    ),
                },
            });
        }
        self.portal_address
            .as_ref()
            .is_some_and(|address| address.show(ui, &mut self.azure.resource_search, focus_search))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portal_browser_control::PortalBrowserControl;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use cloud_terrastodon_azure::AzureResourceGroupId;
    use cloud_terrastodon_azure::AzureStorageAccountBlobContainerId;
    use cloud_terrastodon_azure::AzureStorageAccountId;
    use cloud_terrastodon_azure::ScopeImpl;
    use cloud_terrastodon_azure::SubscriptionId;
    use eframe::egui::Context;
    use eframe::egui::Pos2;
    use eframe::egui::RawInput;
    use eframe::egui::Rect;

    #[test]
    fn compact_omnibox_shares_the_controls_center_and_the_dropdown_outline() {
        let context = Context::default();
        let address = PortalAddress {
            target: PortalTabTarget::Page(PortalPage::Resources),
            authentication: None,
            devops_authentication: None,
            account_alias: None,
            devops_project_name: None,
            cli_command: Ok(Some("ct az resource list".to_owned())),
            identifier: None,
            azure_portal_url: Some("https://portal.azure.com/".to_owned()),
        };
        let mut centers = [0.0; 2];
        let mut output = context.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(620.0, 80.0))),
                ..Default::default()
            },
            |ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), PortalBrowserControl::SIZE),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        centers[0] = PortalBrowserControl::Back.show(ui, true).rect.center().y;
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            centers[1] =
                                PortalBrowserControl::History.show(ui, true).rect.center().y;
                            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                address.show(ui, &mut String::new(), false);
                            });
                        });
                    },
                );
            },
        );
        output.textures_delta.clear();
        let outline = output
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                Shape::Rect(shape) if shape.corner_radius.nw > 0 && !shape.stroke.is_empty() => {
                    Some(shape)
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(outline.rect.height(), 28.0);
        assert_eq!(outline.corner_radius.nw, 14);
        assert!(
            centers
                .iter()
                .all(|y| (y - outline.rect.center().y).abs() < 0.01)
        );
        assert!(output.shapes.iter().any(|clipped| {
            matches!(&clipped.shape, Shape::Rect(shape)
                if shape.rect == outline.rect && shape.corner_radius == outline.corner_radius
                    && shape.fill != Color32::TRANSPARENT
                    && clipped.clip_rect.min == outline.rect.min
                    && clipped.clip_rect.bottom() == outline.rect.bottom()
                    && clipped.clip_rect.right() < outline.rect.right())
        }));
    }

    #[test]
    fn subscriptions_list_has_a_portal_url_without_a_selected_tenant() {
        let target = PortalTabTarget::Page(PortalPage::Subscriptions);
        assert_eq!(
            PortalAddress::azure_portal_url(&target, None).as_deref(),
            Some("https://portal.azure.com/#view/Microsoft_Azure_Billing/SubscriptionsBladeV2")
        );
    }

    #[test]
    fn project_agent_pools_address_keeps_the_collection_and_project_scope() -> Result<()> {
        use crate::portal_azure_devops_project_view::PortalAzureDevOpsProjectView;

        let organization = "https://dev.example.test/tfs/Synthetic%20collection".parse()?;
        let project_id = "00000000-0000-0000-0000-000000000001";
        let account_id = "00000000-0000-0000-0000-000000000002";
        let alias = "automation’s pat".parse()?;
        let view = PortalAzureDevOpsProjectView::AgentPools;
        let target = view.target(&organization, project_id.parse()?);
        let command = PortalAddress::devops_cli_command(
            &target,
            AzureDevOpsAuthSelection::PersonalAccessToken {
                account_id: account_id.parse()?,
            },
            Some(&alias),
        )?
        .unwrap();
        assert!(command.starts_with("ct az devops agent pool entitlement list "));
        assert!(command.contains(&organization.expanded_form()));
        assert!(command.contains(&format!("--project {project_id}")));
        #[cfg(windows)]
        assert!(command.contains("--account 'automation’’s pat'"));
        #[cfg(not(windows))]
        assert!(command.contains("--account 'automation’s pat'"));
        assert!(command.contains("--auth-source pat"));
        assert_eq!(
            PortalAddress::identifier(&target).as_deref(),
            Some(project_id)
        );
        assert_eq!(
            view.portal_url(&organization, "Synthetic project"),
            "https://dev.example.test/tfs/Synthetic%20collection/Synthetic%20project/_settings/agentqueues"
        );
        Ok(())
    }

    #[test]
    fn resource_overview_link_uses_the_selected_tenant_and_escapes_the_id() -> Result<()> {
        let target = PortalTabTarget::ResourceGroup(
            concat!(
                "/subscriptions/00000000-0000-0000-0000-000000000001",
                "/resourceGroups/équipe(test)"
            )
            .parse::<AzureResourceGroupId>()?,
        );
        let tenant_id = "00000000-0000-0000-0000-000000000002".parse()?;
        assert!(PortalAddress::azure_portal_url(&target, None).is_none());
        let url = PortalAddress::azure_portal_url(&target, Some(tenant_id)).unwrap();
        assert_eq!(
            url,
            concat!(
                "https://portal.azure.com/#@00000000-0000-0000-0000-000000000002/resource",
                "/subscriptions/00000000-0000-0000-0000-000000000001",
                "/resourceGroups/%C3%A9quipe%28test%29/overview"
            )
        );
        Ok(())
    }

    #[test]
    fn nested_resource_address_uses_the_full_id_and_output_format() -> Result<()> {
        let id = concat!(
            "/subscriptions/00000000-0000-0000-0000-000000000001",
            "/resourceGroups/synthetic(group)",
            "/providers/Microsoft.Compute/galleries/example/images/child/versions/1.0.0",
        );
        let authentication = AzureBearerAuthenticationSelection {
            account_id: "00000000-0000-0000-0000-000000000002".parse()?,
            tenant_id: "00000000-0000-0000-0000-000000000003".parse()?,
            auth_source: AzureBearerAuthSource::AzureCli,
        };
        let target = PortalTabTarget::ResourceOutput {
            resource: PortalResourceScope::Resource(Box::new(ScopeImpl::from(id))).output_target(),
            format: OutputFormat::Json,
        };
        let command = PortalAddress::cli_command(&target, Some(authentication), None)?
            .ok_or_else(|| eyre::eyre!("expected a copyable resource command"))?;
        assert!(command.starts_with("ct --output-format json az resource show "));
        assert!(command.contains(&format!("'{id}'")));
        assert_eq!(PortalAddress::identifier(&target).as_deref(), Some(id));
        Ok(())
    }

    #[test]
    fn storage_scope_views_address_the_selected_account() -> Result<()> {
        let alias = "personal".parse()?;
        let id = concat!(
            "/subscriptions/00000000-0000-0000-0000-000000000001",
            "/resourceGroups/synthetic-group",
            "/providers/Microsoft.Storage/storageAccounts/syntheticstore",
        )
        .parse::<AzureStorageAccountId>()?;
        let authentication = AzureBearerAuthenticationSelection {
            account_id: "00000000-0000-0000-0000-000000000002".parse()?,
            tenant_id: "00000000-0000-0000-0000-000000000003".parse()?,
            auth_source: AzureBearerAuthSource::AzureCli,
        };
        for (view, command) in [
            (
                PortalScopeView::StorageExplorer,
                "ct az storage blob container list",
            ),
            (PortalScopeView::Networking, "ct az storage account show"),
        ] {
            let target = PortalTabTarget::ScopeView {
                scope: PortalResourceScope::StorageAccount(id.clone()),
                view,
            };
            let address =
                PortalAddress::cli_command(&target, Some(authentication), Some(&alias))?.unwrap();
            assert!(address.starts_with(command));
            assert!(address.contains(&id.expanded_form()));
            assert!(address.contains("--account personal"));
            assert!(address.contains(&format!("--tenant {}", authentication.tenant_id)));
        }
        Ok(())
    }

    #[test]
    fn storage_container_address_quotes_special_names_and_uses_the_full_identity() -> Result<()> {
        let storage_account = AzureStorageAccountId::try_new(
            AzureResourceGroupId::try_new(
                "00000000-0000-0000-0000-000000000001",
                "synthetic(group)",
            )?,
            "syntheticstore",
        )?;
        let container = AzureStorageAccountBlobContainerId {
            storage_account_id: storage_account.clone(),
            container_name: "$web".parse()?,
        };
        let resource_id = container.to_string();
        let target = PortalTabTarget::StorageContainer(container);
        let authentication = AzureBearerAuthenticationSelection {
            account_id: "00000000-0000-0000-0000-000000000002".parse()?,
            tenant_id: "00000000-0000-0000-0000-000000000003".parse()?,
            auth_source: AzureBearerAuthSource::AzureCli,
        };

        let command = PortalAddress::cli_command(&target, Some(authentication), None)?.unwrap();
        assert!(command.starts_with("ct az storage blob container show "));
        assert!(command.contains("--container '$web'"));
        assert!(command.contains(&format!("'{}'", storage_account.expanded_form())));
        assert!(command.contains(&authentication.account_id.to_string()));
        assert_eq!(PortalAddress::identifier(&target), Some(resource_id));
        assert_eq!(
            PortalAddress::azure_portal_url(&target, Some(authentication.tenant_id)),
            PortalAddress::azure_portal_url(
                &PortalTabTarget::StorageAccount(storage_account),
                Some(authentication.tenant_id),
            )
        );
        Ok(())
    }

    #[test]
    fn output_format_is_part_of_the_copyable_command_address() -> Result<()> {
        let id = concat!(
            "/subscriptions/00000000-0000-0000-0000-000000000001",
            "/resourceGroups/synthetic-group",
            "/providers/Microsoft.Storage/storageAccounts/syntheticstore",
        )
        .parse::<AzureStorageAccountId>()?;
        let authentication = Some(AzureBearerAuthenticationSelection {
            account_id: "00000000-0000-0000-0000-000000000002".parse()?,
            tenant_id: "00000000-0000-0000-0000-000000000003".parse()?,
            auth_source: AzureBearerAuthSource::AzureCli,
        });
        for (format, flag) in [
            (OutputFormat::Json, "json"),
            (OutputFormat::FacetPretty, "facet-pretty"),
            (OutputFormat::Text, "text"),
        ] {
            let target = PortalTabTarget::ResourceOutput {
                resource: PortalResourceOutputTarget::StorageAccount(id.clone()),
                format,
            };
            let address = PortalAddress::cli_command(&target, authentication, None)?.unwrap();
            assert!(address.starts_with(&format!(
                "ct --output-format {flag} az storage account show "
            )));
            assert!(address.contains(&id.expanded_form()));
        }
        Ok(())
    }

    #[test]
    fn scoped_output_keeps_the_resource_id_and_overview_url() -> Result<()> {
        let subscription: SubscriptionId = "00000000-0000-0000-0000-000000000001".parse()?;
        let group = AzureResourceGroupId::try_new(subscription, "synthetic-group")?;
        let tenant_id = "00000000-0000-0000-0000-000000000002".parse()?;
        for (scope, id) in [
            (
                PortalResourceScope::Subscription(subscription),
                subscription.expanded_form(),
            ),
            (
                PortalResourceScope::ResourceGroup(group.clone()),
                group.expanded_form(),
            ),
        ] {
            let overview = scope.overview();
            let output = PortalTabTarget::ResourceOutput {
                resource: PortalResourceOutputTarget::Scope(scope),
                format: OutputFormat::Text,
            };
            assert_eq!(PortalAddress::identifier(&output), Some(id));
            assert_eq!(
                PortalAddress::azure_portal_url(&output, Some(tenant_id)),
                PortalAddress::azure_portal_url(&overview, Some(tenant_id))
            );
        }
        Ok(())
    }

    #[test]
    fn directory_object_address_keeps_the_object_and_account_ids() -> Result<()> {
        let object_id = "00000000-0000-0000-0000-000000000001";
        let account_id = "00000000-0000-0000-0000-000000000002";
        let authentication = AzureBearerAuthenticationSelection {
            account_id: account_id.parse()?,
            tenant_id: "00000000-0000-0000-0000-000000000003".parse()?,
            auth_source: AzureBearerAuthSource::AzureCli,
        };
        for (target, operation) in [
            (PortalTabTarget::EntraUser(object_id.parse()?), "user show"),
            (
                PortalTabTarget::EntraGroup(object_id.parse()?),
                "group search",
            ),
            (
                PortalTabTarget::EntraAppRegistration(object_id.parse()?),
                "application-registration show",
            ),
        ] {
            let command = PortalAddress::cli_command(&target, Some(authentication), None)?.unwrap();
            assert!(command.starts_with(&format!("ct az entra {operation} ")));
            assert!(command.contains(object_id));
            assert!(command.contains(account_id));
            assert_eq!(
                PortalAddress::identifier(&target).as_deref(),
                Some(object_id)
            );
            assert!(
                PortalAddress::azure_portal_url(&target, Some(authentication.tenant_id)).is_none()
            );
        }
        Ok(())
    }
}
