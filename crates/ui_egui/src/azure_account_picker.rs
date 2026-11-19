use crate::azure_account_picker_action::AzureAccountPickerAction;
use crate::azure_account_registration_form::source_label;
use cloud_terrastodon_auth::CloudTerrastodonAccountCatalogue;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use eframe::egui::Align2;
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
use teamy_tracing::gui::DiagnosticReportView;

/// Account selection over a local catalogue snapshot.
pub struct AzureAccountPicker {
    pub catalogue: Result<CloudTerrastodonAccountCatalogue, DiagnosticReport>,
    pub selection_error: Option<DiagnosticReport>,
    open_requested: bool,
}

impl AzureAccountPicker {
    pub fn new(store: &CloudTerrastodonAccountStore) -> Self {
        Self {
            catalogue: store.load().map_err(DiagnosticReport::new),
            selection_error: None,
            open_requested: false,
        }
    }

    pub fn open(&mut self) {
        self.open_requested = true;
    }

    pub(crate) fn show(
        &mut self,
        ui: &mut Ui,
        authentication: Option<AzureBearerAuthenticationSelection>,
        busy: bool,
    ) -> Option<AzureAccountPickerAction> {
        use AzureAccountPickerAction::*;

        let selected_account = authentication.and_then(|selection| {
            self.catalogue
                .as_ref()
                .ok()?
                .by_id(selection.account_id)
                .ok()
        });
        let alias = selected_account.map_or("Choose account", |account| account.alias.as_str());
        let detail = authentication.map_or_else(
            || "Not signed in".to_owned(),
            |selection| {
                let tenant = selection.tenant_id.to_string();
                format!(
                    "{} · {}",
                    source_label(Self::selected_source(selection)),
                    &tenant[..8]
                )
            },
        );
        let response = Self::header_button(ui, alias, &detail);
        let popup_id = Popup::default_response_id(&response);
        if std::mem::take(&mut self.open_requested) {
            Popup::open_id(ui.ctx(), popup_id);
        }
        let mut action = None;
        Popup::menu(&response)
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .style(|style: &mut Style| {
                style.visuals = Visuals::light();
                style.spacing.item_spacing = Vec2::new(8.0, 6.0);
                style.spacing.interact_size.y = 28.0;
                style.spacing.button_padding = Vec2::new(10.0, 6.0);
            })
            .show(|ui| {
                ui.set_width(360.0);
                ui.heading("Accounts");
                if let Some(error) = &self.selection_error {
                    ui.collapsing("Account error", |ui| {
                        DiagnosticReportView::new(error).show(ui)
                    });
                }
                ui.add_enabled_ui(!busy, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Add account").clicked() {
                            action = Some(Add);
                        }
                        if ui.button("Reload").clicked() {
                            action = Some(Reload);
                        }
                        if ui.button("Tenants").clicked() {
                            action = Some(ManageTenants);
                        }
                    });
                    ui.separator();
                    match &self.catalogue {
                        Err(error) => {
                            ui.label("Could not load accounts.");
                            ui.collapsing("Error details", |ui| {
                                DiagnosticReportView::new(error).show(ui)
                            });
                        }
                        Ok(catalogue) => {
                            let mut any_azure_accounts = false;
                            ScrollArea::vertical().max_height(390.0).show(ui, |ui| {
                                for account in &catalogue.accounts {
                                    let CloudTerrastodonAccountNamespace::Entra { tenant_id } =
                                        account.namespace
                                    else {
                                        continue;
                                    };
                                    any_azure_accounts = true;
                                    ui.push_id(account.id, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.strong(account.alias.as_str());
                                            if ui.small_button("Manage").clicked() {
                                                action = Some(Manage(account.id));
                                            }
                                        });
                                        ui.small(tenant_id.to_string());
                                        for registration in &account.credentials {
                                            let source = registration.credential.source();
                                            let Some(auth_source) = source
                                                .auth_source()
                                                .and_then(|source| source.try_into().ok())
                                            else {
                                                continue;
                                            };
                                            let selection = AzureBearerAuthenticationSelection {
                                                account_id: account.id,
                                                auth_source,
                                                tenant_id,
                                            };
                                            let current = authentication == Some(selection);
                                            let active = matches!(
                                                registration.session,
                                                CloudTerrastodonAccountSession::Active { .. }
                                            );
                                            let mut label = source_label(source).to_owned();
                                            if source == account.preferred_source {
                                                label.push_str(" (preferred)");
                                            }
                                            ui.horizontal(|ui| {
                                                if active {
                                                    if ui.selectable_label(current, label).clicked()
                                                    {
                                                        action = Some(Select(selection));
                                                    }
                                                    if ui.small_button("Sign in again").clicked() {
                                                        action = Some(Reauthenticate(selection));
                                                    }
                                                } else {
                                                    ui.label(label);
                                                    if ui.small_button("Sign in").clicked() {
                                                        action = Some(SignIn(selection));
                                                    }
                                                }
                                            });
                                        }
                                        ui.separator();
                                    });
                                }
                            });
                            if !any_azure_accounts {
                                ui.label("No Azure accounts");
                            }
                        }
                    }
                    if ui.button("Manage all accounts").clicked() {
                        action = Some(ManageAccounts);
                    }
                });
            });
        if let Some(selection) = authentication {
            response.on_hover_text(format!(
                "Account: {}
Tenant: {}
Source: {}",
                selection.account_id, selection.tenant_id, selection.auth_source
            ));
        }
        if !matches!(action, None | Some(Reload)) {
            Popup::close_id(ui.ctx(), popup_id);
        }
        action
    }

    pub(crate) fn selected_source(
        selection: AzureBearerAuthenticationSelection,
    ) -> CloudTerrastodonCredentialSource {
        match selection.auth_source {
            AzureBearerAuthSource::AzureCli => CloudTerrastodonCredentialSource::AzureCli,
            AzureBearerAuthSource::Browser => CloudTerrastodonCredentialSource::Browser,
            AzureBearerAuthSource::WorkloadIdentity => {
                CloudTerrastodonCredentialSource::WorkloadIdentity
            }
        }
    }

    /// Keep the two text rows and vector affordances in fixed, independent regions.
    /// A single truncated multi-line button label can discard its second row.
    fn header_button(ui: &mut Ui, alias: &str, detail: &str) -> Response {
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
        let subtitle = WidgetText::from(RichText::new(detail).size(11.0).color(Color32::WHITE))
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
