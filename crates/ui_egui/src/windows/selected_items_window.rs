use crate::app::MyApp;
use crate::icons::DEVOPS_PROJECT_ICON;
use crate::icons::RESOURCE_GROUP_ICON;
use crate::icons::SUBSCRIPTION_ICON;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::Window;
use std::collections::HashSet;

pub fn draw_selected_items_window(app: &mut MyApp, ctx: &Context) {
    // Kept for backwards compatibility; still render as a floating window when needed.
    Window::new("Selected Items").show(ctx, |ui| {
        selected_items_ui(app, ui);
    });
}

/// Tile-friendly UI for the Selected Items pane
pub fn selected_items_ui(app: &mut MyApp, ui: &mut egui::Ui) {
    use eframe::egui::ScrollArea;
    use tracing::debug;

    let account = app.ready_azure_authentication();
    let active_pats: HashSet<_> = app
        .account_picker
        .catalogue
        .as_ref()
        .ok()
        .into_iter()
        .flat_map(|catalogue| &catalogue.accounts)
        .filter(|account| {
            account
                .registration(CloudTerrastodonCredentialSource::PersonalAccessToken)
                .is_ok_and(|registration| {
                    matches!(
                        registration.session,
                        CloudTerrastodonAccountSession::Active { .. }
                    )
                })
        })
        .map(|account| account.id)
        .collect();
    ScrollArea::both().show(ui, |ui| {
        for content in app
            .open_azure_contents_mut()
            .filter(|content| content.authentication == account)
        {
            if let Loadable::Loaded {
                value: subscriptions,
                ..
            } = &content.subscriptions
            {
                for subscription in subscriptions.iter() {
                    let checked = content
                        .checkboxes
                        .entry(Id::new(subscription.id))
                        .or_default();
                    if !*checked {
                        continue;
                    }
                    ui.horizontal(|ui| {
                        if ui.image(SUBSCRIPTION_ICON).clicked() {
                            debug!("Clicked on subscription icon");
                            *checked ^= true;
                        }
                        ui.checkbox(checked, subscription.to_string());
                    });
                }
            }
            if let Loadable::Loaded {
                value: resource_groups,
                ..
            } = &content.resource_groups
            {
                for resource_group in resource_groups.values() {
                    let checked = content
                        .checkboxes
                        .entry(Id::new(&resource_group.id))
                        .or_default();
                    if !*checked {
                        continue;
                    }
                    ui.horizontal(|ui| {
                        if ui.image(RESOURCE_GROUP_ICON).clicked() {
                            *checked ^= true;
                        }
                        ui.checkbox(checked, resource_group.name.as_str());
                    });
                }
            }
            for (organization, authentication, projects) in content.azure_devops_projects.loaded() {
                let pat = content.devops_pat_accounts.get(organization);
                let visible = match authentication {
                    AzureDevOpsAuthSelection::Bearer(selection) => {
                        pat.is_none() && Some(selection) == account
                    }
                    AzureDevOpsAuthSelection::PersonalAccessToken { account_id } => {
                        pat == Some(&account_id) && active_pats.contains(&account_id)
                    }
                };
                if !visible {
                    continue;
                }
                for project in projects.iter() {
                    let checked = content
                        .checkboxes
                        .entry(Id::new((organization, authentication, project.id)))
                        .or_default();
                    if !*checked {
                        continue;
                    }
                    ui.horizontal(|ui| {
                        if ui.image(DEVOPS_PROJECT_ICON).clicked() {
                            debug!("Clicked on Azure DevOps Project icon");
                            *checked ^= true;
                        }
                        ui.checkbox(checked, project.name.to_string());
                    });
                }
            }
        }
    });
}
