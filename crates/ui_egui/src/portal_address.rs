use crate::app::MyApp;
use crate::portal_page::PortalPage;
use crate::portal_resource_output_target::PortalResourceOutputTarget;
use crate::portal_tab_target::PortalTabTarget;
use crate::resource_group_list_command::resource_group_list_command;
use crate::storage_account_show_command::storage_account_show_command;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::PercentEncodeExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use eframe::egui::Align;
use eframe::egui::Frame;
use eframe::egui::Label;
use eframe::egui::Layout;
use eframe::egui::Margin;
use eframe::egui::OpenUrl;
use eframe::egui::RichText;
use eframe::egui::Ui;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetType;
use eyre::Result;
use figue::ToArgs;

/// The display address derived from a typed destination and execution identity.
pub(crate) struct PortalAddress {
    target: PortalTabTarget,
    authentication: Option<AzureBearerAuthenticationSelection>,
    local_address: Result<Option<(&'static str, String)>>,
    azure_portal_url: Option<String>,
}

impl PortalAddress {
    fn resolve(
        target: &PortalTabTarget,
        authentication: Option<AzureBearerAuthenticationSelection>,
    ) -> Result<Option<(&'static str, String)>> {
        let cli = |command: String| Some(("CLI", command));
        let resource_id = |id: String| Some(("Resource ID", id));
        let address = match target {
            PortalTabTarget::Page(PortalPage::AzureTenants) => cli("ct az tenant list".into()),
            PortalTabTarget::Page(PortalPage::ResourceGroups) => authentication
                .map(resource_group_list_command)
                .transpose()?
                .and_then(cli),
            PortalTabTarget::Page(
                page @ (PortalPage::Resources
                | PortalPage::Subscriptions
                | PortalPage::StorageAccounts),
            ) => authentication
                .map(|selection| {
                    let operation = match page {
                        PortalPage::Resources => "resource list",
                        PortalPage::Subscriptions => "subscription list",
                        _ => "storage account list",
                    };
                    let args = AzureAuthenticationArgs {
                        tenant: Some(selection.tenant_id.into()),
                        account: Some(CloudTerrastodonAccountArgument::Id(selection.account_id)),
                        auth_source: selection.auth_source.into(),
                    };
                    eyre::Ok(format!(
                        "ct az {operation} {}",
                        args.to_args_string()?.to_string_lossy()
                    ))
                })
                .transpose()?
                .and_then(cli),
            PortalTabTarget::StorageAccount(id) => {
                if let Some(selection) = authentication {
                    cli(storage_account_show_command(selection, id, false, None)?)
                } else {
                    resource_id(id.expanded_form())
                }
            }
            PortalTabTarget::ResourceOutput { resource, format } => {
                if let (PortalResourceOutputTarget::StorageAccount(id), Some(selection)) =
                    (resource, authentication)
                {
                    cli(storage_account_show_command(
                        selection,
                        id,
                        false,
                        Some(*format),
                    )?)
                } else {
                    resource_id(resource.resource_id())
                }
            }
            PortalTabTarget::ManagementGroup(id) => resource_id(id.expanded_form()),
            PortalTabTarget::Subscription(id) => resource_id(id.expanded_form()),
            PortalTabTarget::ResourceGroup(id) => resource_id(id.expanded_form()),
            PortalTabTarget::ScopeView { scope, .. } => {
                resource_id(scope.arm_scope().expanded_form())
            }
            PortalTabTarget::ResourcesOfType(kind) => Some(("Resource type", kind.to_string())),
            PortalTabTarget::LocalPath(path) => Some(("Path", path.display().to_string())),
            PortalTabTarget::Page(_) => None,
        };
        Ok(address)
    }

    fn azure_portal_url(target: &PortalTabTarget, tenant_id: AzureTenantId) -> Option<String> {
        let id = match target {
            PortalTabTarget::Subscription(id) => id.expanded_form(),
            PortalTabTarget::ResourceGroup(id) => id.expanded_form(),
            PortalTabTarget::ScopeView { scope, .. } => scope.arm_scope().expanded_form(),
            PortalTabTarget::StorageAccount(id) => id.expanded_form(),
            PortalTabTarget::ResourceOutput { resource, .. } => resource.resource_id(),
            _ => return None,
        };
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

    fn show(
        ui: &mut Ui,
        target: &PortalTabTarget,
        local_address: &Result<Option<(&'static str, String)>>,
        azure_portal_url: Option<&str>,
    ) {
        let output_view = matches!(
            target,
            PortalTabTarget::ResourceOutput { .. } | PortalTabTarget::ScopeView { .. }
        );
        let local = local_address.as_ref().ok().and_then(Option::as_ref);
        let title = target.title();
        let text = match (output_view, azure_portal_url, local) {
            (false, Some(url), _) => RichText::new(url),
            (_, _, Some((_, address))) => RichText::new(address).monospace(),
            _ => RichText::new(title.as_ref()),
        };
        let hover_text = local_address.as_ref().err().map_or_else(
            || text.text().to_owned(),
            |error| format!("Could not format the local address · {error}"),
        );

        Frame::new()
            .fill(ui.visuals().faint_bg_color)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .corner_radius(16_u8)
            .inner_margin(Margin::symmetric(12, 2))
            .show(ui, |ui| {
                ui.set_min_height(26.0);
                ui.spacing_mut().interact_size.y = 26.0;
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if azure_portal_url.is_some() || local.is_some() {
                        let menu = ui.menu_button("…", |ui| {
                            if let Some(url) = azure_portal_url {
                                let open_label = if output_view {
                                    "Open resource overview in Azure Portal"
                                } else {
                                    "Open in Azure Portal"
                                };
                                if ui.button(open_label).clicked() {
                                    ui.ctx().open_url(OpenUrl::new_tab(url));
                                    ui.close();
                                }
                                if ui.button("Copy Azure Portal URL").clicked() {
                                    ui.ctx().copy_text(url.to_owned());
                                    ui.close();
                                }
                            }
                            if let Some((kind, address)) = local {
                                let copy_label = match *kind {
                                    "CLI" => "Copy CLI command",
                                    "Resource ID" => "Copy resource ID",
                                    "Resource type" => "Copy resource type",
                                    _ => "Copy path",
                                };
                                if ui.button(copy_label).on_hover_text(address).clicked() {
                                    ui.ctx().copy_text(address.to_owned());
                                    ui.close();
                                }
                            }
                        });
                        menu.response.widget_info(|| {
                            WidgetInfo::labeled(
                                WidgetType::Button,
                                ui.is_enabled(),
                                "Address actions",
                            )
                        });
                        menu.response.on_hover_text("Address actions");
                    }
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.add(Label::new(text).truncate().selectable(true))
                            .on_hover_text(hover_text);
                    });
                });
            });
    }
}

impl MyApp {
    pub(crate) fn show_portal_address(&mut self, ui: &mut Ui, target: &PortalTabTarget) {
        if *target == PortalTabTarget::Page(PortalPage::Accounts) {
            let address = Ok(self
                .account_manager
                .cli_command()
                .map(|command| ("CLI", command)));
            PortalAddress::show(ui, target, &address, None);
            return;
        }
        let authentication = self.ready_azure_authentication();
        if self.portal_address.as_ref().is_none_or(|address| {
            &address.target != target || address.authentication != authentication
        }) {
            self.portal_address = Some(PortalAddress {
                target: target.clone(),
                authentication,
                local_address: PortalAddress::resolve(target, authentication),
                azure_portal_url: authentication.and_then(|selection| {
                    PortalAddress::azure_portal_url(target, selection.tenant_id)
                }),
            });
        }
        if let Some(address) = &self.portal_address {
            PortalAddress::show(
                ui,
                target,
                &address.local_address,
                address.azure_portal_url.as_deref(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portal_resource_scope::PortalResourceScope;
    use cloud_terrastodon_app::OutputFormat;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use cloud_terrastodon_azure::AzureResourceGroupId;
    use cloud_terrastodon_azure::AzureStorageAccountId;
    use cloud_terrastodon_azure::SubscriptionId;

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
        let url = PortalAddress::azure_portal_url(&target, tenant_id).unwrap();
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
            let (kind, address) = PortalAddress::resolve(&target, authentication)?.unwrap();
            assert_eq!(kind, "CLI");
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
        let authentication = AzureBearerAuthenticationSelection {
            account_id: "00000000-0000-0000-0000-000000000003".parse()?,
            tenant_id,
            auth_source: AzureBearerAuthSource::AzureCli,
        };
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
            assert_eq!(
                PortalAddress::resolve(&output, Some(authentication))?,
                Some(("Resource ID", id))
            );
            assert_eq!(
                PortalAddress::azure_portal_url(&output, tenant_id),
                PortalAddress::azure_portal_url(&overview, tenant_id)
            );
        }
        Ok(())
    }
}
