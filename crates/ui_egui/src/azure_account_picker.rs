use cloud_terrastodon_auth::CloudTerrastodonAccountCatalogue;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountPrincipal;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eframe::egui::Align2;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::CursorIcon;
use eframe::egui::FontId;
use eframe::egui::Popup;
use eframe::egui::PopupCloseBehavior;
use eframe::egui::Rect;
use eframe::egui::Response;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::Sense;
use eframe::egui::Stroke;
use eframe::egui::Style;
use eframe::egui::TextStyle;
use eframe::egui::TextWrapMode;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::Visuals;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetText;
use eframe::egui::WidgetType;
use teamy_tracing::DiagnosticReport;

/// Selects an explicitly registered Azure account and credential mechanism.
///
/// The catalogue contains inspectable account metadata only. Opening or refreshing
/// the picker never reads credentials, acquires a token, or starts a login flow.
/// Selection stays local to this GUI and does not change configured CLI defaults.
pub struct AzureAccountPicker {
    pub catalogue: Result<CloudTerrastodonAccountCatalogue, DiagnosticReport>,
    pub selection_error: Option<DiagnosticReport>,
}

impl Default for AzureAccountPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl AzureAccountPicker {
    pub fn new() -> Self {
        Self {
            catalogue: CloudTerrastodonAccountStore::configured()
                .load()
                .map_err(DiagnosticReport::new),
            selection_error: None,
        }
    }

    /// Draw the header button and account menu, returning an explicit selection.
    /// The caller owns resetting account-specific UI data before starting new work.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        authentication: AzureBearerAuthenticationSelection,
        executor: &CloudTerrastodonExecutor,
    ) -> Option<AzureBearerAuthenticationSelection> {
        let alias = self
            .catalogue
            .as_ref()
            .ok()
            .and_then(|catalogue| {
                catalogue
                    .accounts
                    .iter()
                    .find(|account| account.id == authentication.account_id)
            })
            .map(|account| account.alias.to_string())
            .unwrap_or_else(|| authentication.account_id.to_string());
        let mut requested = None;
        let mut refresh = false;
        let response = Self::header_button(ui, &alias, authentication);
        Popup::menu(&response)
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .style(|style: &mut Style| {
                style.visuals = Visuals::light();
                style.spacing.item_spacing = Vec2::new(8.0, 6.0);
                style.spacing.interact_size.y = 28.0;
                style.spacing.button_padding = Vec2::new(10.0, 6.0);
            })
            .show(|ui| {
            ui.set_width(340.0);
            ui.heading("Cloud Terrastodon accounts");
            ui.small("Select an account and authentication source for this window.");
            ui.separator();
            if let Some(error) = &self.selection_error {
                ui.colored_label(Color32::DARK_RED, error.to_string());
                ui.separator();
            }
            match &self.catalogue {
                Err(error) => {
                    ui.colored_label(Color32::DARK_RED, error.to_string());
                }
                Ok(catalogue) if catalogue.accounts.is_empty() => {
                    ui.label("No Cloud Terrastodon accounts are registered.");
                    ui.small("Use ct account add to register an account, then refresh this list.");
                }
                Ok(catalogue) => {
                    ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
                        for account in &catalogue.accounts {
                            ui.push_id(account.id, |ui| {
                                ui.strong(account.alias.as_str());
                                match &account.namespace {
                                    CloudTerrastodonAccountNamespace::Entra { tenant_id } => {
                                        ui.small(format!("Tenant: {tenant_id}"));
                                    }
                                    CloudTerrastodonAccountNamespace::AzureDevOps {
                                        organization,
                                    } => {
                                        ui.small(format!("Azure DevOps: {organization}"));
                                    }
                                    CloudTerrastodonAccountNamespace::Gitea { instance } => {
                                        ui.small(format!("Gitea: {instance}"));
                                    }
                                }
                                if catalogue
                                    .defaults
                                    .iter()
                                    .any(|default| default.account_id == account.id)
                                {
                                    ui.small("Configured default account");
                                }
                                match account.principal {
                                    Some(CloudTerrastodonAccountPrincipal::EntraUser(id)) => {
                                        ui.small(format!("User ID: {id}"));
                                    }
                                    Some(CloudTerrastodonAccountPrincipal::EntraApplication(id)) => {
                                        ui.small(format!("Application ID: {id}"));
                                    }
                                    None => {}
                                }
                                if let CloudTerrastodonAccountNamespace::Entra { tenant_id } =
                                    account.namespace
                                {
                                    for registration in &account.credentials {
                                        let source = registration.credential.source();
                                        let Some(auth_source) = source.auth_source() else {
                                            continue;
                                        };
                                        let current = account.id == authentication.account_id
                                            && auth_source == authentication.auth_source.into();
                                        let active = matches!(
                                            registration.session,
                                            CloudTerrastodonAccountSession::Active { .. }
                                        );
                                        let mut label = source.to_string();
                                        if source == account.preferred_source {
                                            label.push_str(" (preferred)");
                                        }
                                        if current {
                                            label.push_str(" · selected");
                                        }
                                        if ui
                                            .add_enabled(active, Button::selectable(current, label))
                                            .on_hover_text(if active {
                                                "Use this account and authentication source"
                                            } else {
                                                "This source is not signed in; use ct account login first"
                                            })
                                            .clicked()
                                        {
                                            requested = Some((tenant_id, account.id, auth_source));
                                        }
                                    }
                                } else {
                                    ui.small("This service account cannot browse Azure resources.");
                                }
                                ui.separator();
                            });
                        }
                    });
                }
            }
            refresh = ui.button("Refresh accounts").clicked();
            ui.small("After signing in again from the CLI, restart this window to use the new session.");
            });
        let response = response.on_hover_text(format!(
            "Account: {}\nTenant: {}\nSource: {}",
            authentication.account_id, authentication.tenant_id, authentication.auth_source
        ));
        if refresh {
            self.catalogue = CloudTerrastodonAccountStore::configured()
                .load()
                .map_err(DiagnosticReport::new);
            self.selection_error = None;
            ui.ctx().request_repaint();
        }
        let (tenant_id, account_id, source) = requested?;
        match executor.resolve_azure_authentication(
            tenant_id,
            Some(&account_id.to_string()),
            source,
        ) {
            Ok(selected) => {
                self.selection_error = None;
                Popup::close_id(ui.ctx(), Popup::default_response_id(&response));
                (selected != authentication).then_some(selected)
            }
            Err(error) => {
                self.selection_error = Some(DiagnosticReport::new(error));
                ui.ctx().request_repaint();
                None
            }
        }
    }

    /// Keep the two text rows and vector affordances in fixed, independent regions.
    /// A single truncated multi-line button label can discard its second row.
    fn header_button(
        ui: &mut Ui,
        alias: &str,
        authentication: AzureBearerAuthenticationSelection,
    ) -> Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::new(236.0, 40.0), Sense::click());
        response.widget_info(|| {
            WidgetInfo::labeled(
                WidgetType::Button,
                ui.is_enabled(),
                format!("Choose account: {alias}"),
            )
        });
        if response.hovered()
            || response.has_focus()
            || Popup::is_id_open(ui.ctx(), Popup::default_response_id(&response))
        {
            ui.painter()
                .rect_filled(rect, 0.0, Color32::from_rgb(0x10, 0x6e, 0xbe));
        }

        let text_rect = Rect::from_min_max(
            rect.min + Vec2::new(12.0, 0.0),
            rect.max - Vec2::new(64.0, 0.0),
        );
        let title = WidgetText::from(
            RichText::new(alias)
                .size(14.0)
                .strong()
                .color(Color32::WHITE),
        )
        .into_galley(
            ui,
            Some(TextWrapMode::Truncate),
            text_rect.width(),
            TextStyle::Button,
        );
        let subtitle = WidgetText::from(
            RichText::new(format!(
                "{} | {}",
                authentication.auth_source,
                authentication
                    .tenant_id
                    .to_string()
                    .chars()
                    .take(8)
                    .collect::<String>()
            ))
            .size(11.0)
            .color(Color32::WHITE),
        )
        .into_galley(
            ui,
            Some(TextWrapMode::Truncate),
            text_rect.width(),
            TextStyle::Small,
        );
        let painter = ui
            .painter()
            .with_clip_rect(text_rect.intersect(ui.clip_rect()));
        painter.galley(text_rect.min + Vec2::new(0.0, 4.0), title, Color32::WHITE);
        painter.galley(
            text_rect.min + Vec2::new(0.0, 23.0),
            subtitle,
            Color32::WHITE,
        );

        // The badge represents the configured alias, not a fetched profile photo.
        let avatar_center = rect.right_center() - Vec2::new(42.0, 0.0);
        ui.painter()
            .circle_filled(avatar_center, 14.0, Color32::from_rgb(0x00, 0x5a, 0x9e));
        let mut initials = alias
            .split(|character: char| !character.is_alphanumeric())
            .filter_map(|word| word.chars().next())
            .take(2)
            .flat_map(char::to_uppercase)
            .collect::<String>();
        if initials.is_empty() {
            initials.push_str("CT");
        }
        ui.painter().text(
            avatar_center,
            Align2::CENTER_CENTER,
            initials,
            FontId::proportional(12.0),
            Color32::WHITE,
        );
        let arrow_center = rect.right_center() - Vec2::new(12.0, 0.0);
        ui.painter().line_segment(
            [
                arrow_center + Vec2::new(-4.0, -2.0),
                arrow_center + Vec2::new(0.0, 2.0),
            ],
            Stroke::new(1.5_f32, Color32::WHITE),
        );
        ui.painter().line_segment(
            [
                arrow_center + Vec2::new(0.0, 2.0),
                arrow_center + Vec2::new(4.0, -2.0),
            ],
            Stroke::new(1.5_f32, Color32::WHITE),
        );
        response.on_hover_cursor(CursorIcon::PointingHand)
    }
}
