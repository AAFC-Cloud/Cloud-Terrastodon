use crate::azure_account_picker_action::AzureAccountPickerAction;
use crate::azure_account_profile::AzureAccountProfile;
use crate::azure_account_profiles::AzureAccountProfiles;
use crate::azure_account_registration_form::source_label;
use cloud_terrastodon_auth::CloudTerrastodonAccountCatalogue;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountProfile;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialRegistration;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_azure::AzureTenantId;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::CursorIcon;
use eframe::egui::Frame;
use eframe::egui::Image;
use eframe::egui::Popup;
use eframe::egui::PopupCloseBehavior;
use eframe::egui::Rect;
use eframe::egui::Response;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::Sense;
use eframe::egui::Shape;
use eframe::egui::Stroke;
use eframe::egui::StrokeKind;
use eframe::egui::Style;
use eframe::egui::TextStyle;
use eframe::egui::TextWrapMode;
use eframe::egui::TextureHandle;
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
        profiles: &AzureAccountProfiles,
        busy: bool,
        demo_mode: &mut bool,
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
        let selected_profile = authentication.and_then(|selection| profiles.get(selection));
        let email = selected_profile.and_then(|profile| profile.display_email(*demo_mode));
        let tenant = authentication.map(|selection| selection.tenant_id.to_string());
        let response = Self::header_button(
            ui,
            Self::display_alias(alias, *demo_mode),
            tenant.as_deref(),
            email,
            selected_profile,
        );
        let popup_id = Popup::default_response_id(&response);
        let open_requested = std::mem::take(&mut self.open_requested);
        let opening =
            open_requested || (response.clicked() && !Popup::is_id_open(ui.ctx(), popup_id));
        if open_requested {
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
                ui.set_width(382.0);
                ui.heading("Accounts");
                if let Some(error) = &self.selection_error {
                    ui.collapsing("Account error", |ui| {
                        DiagnosticReportView::new(error).show(ui)
                    });
                }
                ui.add_enabled_ui(!busy, |ui| match &self.catalogue {
                    Err(error) => {
                        ui.label("Could not load accounts.");
                        ui.collapsing("Error details", |ui| {
                            DiagnosticReportView::new(error).show(ui)
                        });
                    }
                    Ok(catalogue) => {
                        let mut any_azure_accounts = false;
                        let accounts =
                            selected_account
                                .into_iter()
                                .chain(catalogue.accounts.iter().filter(|account| {
                                    authentication
                                        .is_none_or(|selected| account.id != selected.account_id)
                                }));
                        let mut cards = ScrollArea::vertical().max_height(390.0);
                        if opening {
                            cards = cards.vertical_scroll_offset(0.0);
                        }
                        cards.show(ui, |ui| {
                            for account in accounts {
                                let CloudTerrastodonAccountNamespace::Entra { tenant_id } =
                                    account.namespace
                                else {
                                    continue;
                                };
                                any_azure_accounts = true;
                                if let Some(next) = ui
                                    .push_id(account.id, |ui| {
                                        Self::account_card(
                                            ui,
                                            account,
                                            tenant_id,
                                            authentication,
                                            profiles,
                                            *demo_mode,
                                        )
                                    })
                                    .inner
                                {
                                    action = Some(next);
                                }
                            }
                        });
                        if !any_azure_accounts {
                            ui.label("No Azure accounts");
                        }
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(!busy, |ui| {
                        if ui.button("Manage accounts").clicked() {
                            action = Some(ManageAccounts);
                        }
                        if ui.button("Reload").clicked() {
                            action = Some(Reload);
                        }
                    });
                    if ui
                        .toggle_value(demo_mode, "Demo mode")
                        .on_hover_text("Redact email addresses in the account switcher")
                        .changed()
                    {
                        ui.ctx().request_discard("account demo mode changed");
                    }
                });
            });
        if let Some(selection) = authentication {
            let email = selected_profile
                .and_then(|profile| profile.display_email(*demo_mode))
                .map_or(String::new(), |email| format!("\n{email}"));
            let alias = Self::display_alias(alias, *demo_mode);
            response.on_hover_text(format!(
                "{alias}{email}\nTenant {}\n{}",
                selection.tenant_id,
                source_label(Self::selected_source(selection))
            ));
        }
        if !matches!(action, None | Some(Reload)) {
            Popup::close_id(ui.ctx(), popup_id);
        }
        action
    }

    fn account_card(
        ui: &mut Ui,
        account: &CloudTerrastodonAccountProfile,
        tenant_id: AzureTenantId,
        authentication: Option<AzureBearerAuthenticationSelection>,
        profiles: &AzureAccountProfiles,
        demo_mode: bool,
    ) -> Option<AzureAccountPickerAction> {
        let sources = account.credentials.iter().filter_map(|registration| {
            let source = registration.credential.source();
            let auth_source = source.auth_source()?.try_into().ok()?;
            Some((
                registration,
                AzureBearerAuthenticationSelection {
                    account_id: account.id,
                    auth_source,
                    tenant_id,
                },
            ))
        });
        let selected = authentication.filter(|selection| selection.account_id == account.id);
        let alias = Self::display_alias(account.alias.as_str(), demo_mode);
        let profile = selected
            .and_then(|selection| profiles.get(selection))
            .or_else(|| {
                sources
                    .clone()
                    .find_map(|(_, selection)| profiles.get(selection))
            });
        Frame::new()
            .fill(Color32::from_rgb(246, 249, 252))
            .stroke(Stroke::new(1.0, Color32::from_rgb(218, 228, 238)))
            .corner_radius(8_u8)
            .inner_margin(10.0)
            .show(ui, |ui| {
                let card_width = ui.available_width();
                ui.set_min_width(card_width);
                let identity = ui.horizontal(|ui| {
                    let (avatar, _) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::hover());
                    Self::paint_avatar(
                        ui,
                        avatar,
                        profile.and_then(|profile| profile.photo.as_ref()),
                    );
                    ui.vertical(|ui| {
                        ui.strong(alias);
                        if let Some(email) =
                            profile.and_then(|profile| profile.display_email(demo_mode))
                        {
                            ui.small(email);
                        }
                        ui.small(format!("Tenant {tenant_id}"));
                    });
                });
                if let Some((preferred, selection)) = sources.clone().find(|(registration, _)| {
                    registration.credential.source() == account.preferred_source
                }) {
                    let mut label = format!("Use {alias}");
                    if selected == Some(selection) {
                        label.push_str(" (selected)");
                    }
                    let rect = Rect::from_min_size(
                        identity.response.rect.min,
                        Vec2::new(card_width, identity.response.rect.height()),
                    );
                    let target = ui
                        .interact(rect, ui.id().with("account"), Sense::click())
                        .on_hover_cursor(CursorIcon::PointingHand);
                    target.widget_info(|| {
                        WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), label.as_str())
                    });
                    if target.clicked() {
                        return Some(Self::activate_source(selection, preferred));
                    }
                    if target.hovered() || target.has_focus() {
                        ui.painter().rect_stroke(
                            rect.expand(4.0),
                            6_u8,
                            Stroke::new(1.0, Color32::from_rgb(122, 173, 211)),
                            StrokeKind::Outside,
                        );
                    }
                }
                ui.add_space(2.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().button_padding = Vec2::new(9.0, 3.0);
                    ui.spacing_mut().interact_size.y = 24.0;
                    let mut next = None;
                    for (registration, selection) in sources {
                        let source = registration.credential.source();
                        let active = matches!(
                            registration.session,
                            CloudTerrastodonAccountSession::Active { .. }
                        );
                        let label = if active {
                            source_label(source).to_owned()
                        } else {
                            format!("{} · Sign in", source_label(source))
                        };
                        let response = ui.add(
                            Button::selectable(active && selected == Some(selection), label)
                                .corner_radius(12_u8),
                        );
                        if response.clicked() {
                            next = Some(Self::activate_source(selection, registration));
                        }
                        if source == account.preferred_source {
                            response.on_hover_text("Preferred source");
                        }
                        if !active
                            && source == CloudTerrastodonCredentialSource::AzureCli
                            && ui
                                .add(Button::new("Device code").corner_radius(12_u8))
                                .on_hover_text("Sign in to Azure CLI with a device code")
                                .clicked()
                        {
                            next = Some(AzureAccountPickerAction::SignInWithDeviceCode(account.id));
                        }
                    }
                    next
                })
                .inner
            })
            .inner
    }

    fn activate_source(
        selection: AzureBearerAuthenticationSelection,
        registration: &CloudTerrastodonCredentialRegistration,
    ) -> AzureAccountPickerAction {
        if matches!(
            registration.session,
            CloudTerrastodonAccountSession::Active { .. }
        ) {
            AzureAccountPickerAction::Select(selection)
        } else {
            AzureAccountPickerAction::SignIn(selection)
        }
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

    fn display_alias(alias: &str, demo_mode: bool) -> &str {
        if demo_mode && alias.contains('@') {
            AzureAccountProfile::REDACTED_EMAIL
        } else {
            alias
        }
    }

    /// Give each line its own clip region within the fixed-height portal header.
    fn header_button(
        ui: &mut Ui,
        alias: &str,
        tenant: Option<&str>,
        email: Option<&str>,
        profile: Option<&AzureAccountProfile>,
    ) -> Response {
        let width = ui.available_width().clamp(64.0, 280.0);
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 40.0), Sense::click());
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

        if width > 80.0 {
            let text_rect = Rect::from_min_max(
                rect.min + Vec2::new(10.0, 0.0),
                rect.max - Vec2::new(62.0, 0.0),
            );
            let painter = ui
                .painter()
                .with_clip_rect(text_rect.intersect(ui.clip_rect()));
            let text = |value: RichText, top| {
                let value = WidgetText::from(value.color(Color32::WHITE)).into_galley(
                    ui,
                    Some(TextWrapMode::Truncate),
                    text_rect.width(),
                    TextStyle::Small,
                );
                painter.galley(text_rect.min + Vec2::new(0.0, top), value, Color32::WHITE);
            };
            let title = RichText::new(alias).size(13.0).strong();
            match (email, tenant) {
                (Some(email), Some(tenant)) => {
                    text(title, 0.0);
                    text(RichText::new(email).size(11.0), 13.0);
                    text(RichText::new(tenant).size(10.0), 26.0);
                }
                (_, Some(tenant)) => {
                    text(title, 4.0);
                    text(RichText::new(tenant).size(10.0), 23.0);
                }
                _ => text(title, 11.0),
            }
        }
        Self::paint_avatar(
            ui,
            Rect::from_center_size(
                rect.right_center() - Vec2::new(40.0, 0.0),
                Vec2::splat(28.0),
            ),
            profile.and_then(|profile| profile.photo.as_ref()),
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

    fn paint_avatar(ui: &Ui, rect: Rect, photo: Option<&TextureHandle>) {
        if let Some(photo) = photo {
            Image::new(photo).corner_radius(16_u8).paint_at(ui, rect);
            return;
        }
        let center = rect.center();
        let width = rect.width();
        let foreground = Color32::from_rgb(35, 83, 119);
        let stroke = Stroke::new(1.35, foreground);
        let painter = ui.painter();
        painter.circle_filled(center, width * 0.5, Color32::from_rgb(224, 237, 248));
        painter.circle_stroke(center + Vec2::new(0.0, -width * 0.13), width * 0.10, stroke);
        painter.add(Shape::line(
            [
                (-0.23, 0.24),
                (-0.18, 0.14),
                (-0.09, 0.09),
                (0.09, 0.09),
                (0.18, 0.14),
                (0.23, 0.24),
            ]
            .into_iter()
            .map(|(x, y)| center + width * Vec2::new(x, y))
            .collect(),
            stroke,
        ));
    }
}
