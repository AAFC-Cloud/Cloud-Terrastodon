use crate::app::MyApp;
use crate::azure_account_picker_action::AzureAccountPickerAction;
use crate::azure_account_profile::AzureAccountProfile;
use crate::azure_account_profiles::AzureAccountProfiles;
use crate::azure_account_registration_form::source_label;
use crate::capture_gui_failure::capture_gui_failure;
use crate::portal_tab_id::PortalTabId;
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
use eframe::egui::Align;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::CursorIcon;
use eframe::egui::Frame;
use eframe::egui::Id;
use eframe::egui::Image;
use eframe::egui::Layout;
use eframe::egui::Popup;
use eframe::egui::PopupCloseBehavior;
use eframe::egui::Rect;
use eframe::egui::Response;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::Sense;
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
use eframe::egui::include_image;
use std::collections::HashMap;
use std::collections::HashSet;
use teamy_tracing::DiagnosticReport;
use teamy_tracing::gui::DiagnosticReportView;

const ACCOUNT_CONTROL_HEIGHT: f32 = 36.0;

/// Account selection over a local catalogue snapshot.
pub struct AzureAccountPicker {
    pub catalogue: Result<CloudTerrastodonAccountCatalogue, DiagnosticReport>,
    selection_errors: HashMap<(PortalTabId, Id), DiagnosticReport>,
    open_requested: HashSet<(PortalTabId, Id)>,
}

impl AzureAccountPicker {
    pub fn new(store: &CloudTerrastodonAccountStore) -> Self {
        Self {
            catalogue: store
                .load()
                .map_err(|error| capture_gui_failure(error, "Could not load account catalogue")),
            selection_errors: HashMap::new(),
            open_requested: HashSet::new(),
        }
    }

    pub(crate) fn open(&mut self, tab: PortalTabId) {
        self.open_requested.insert(Self::page_instance(tab));
    }

    pub(crate) fn set_selection_error(
        &mut self,
        tab: PortalTabId,
        error: Option<DiagnosticReport>,
    ) {
        if let Some(error) = error {
            self.selection_errors
                .insert(Self::page_instance(tab), error);
        } else {
            self.selection_errors.remove(&Self::page_instance(tab));
        }
    }

    pub(crate) fn selection_error(&self, tab: PortalTabId) -> Option<&DiagnosticReport> {
        self.selection_errors.get(&Self::page_instance(tab))
    }

    pub(crate) fn retain_tabs(&mut self, is_open: impl Fn(PortalTabId) -> bool) {
        self.selection_errors.retain(|&(tab, _), _| is_open(tab));
        self.open_requested.retain(|&(tab, _)| is_open(tab));
    }

    fn page_instance(tab: PortalTabId) -> (PortalTabId, Id) {
        (tab, Id::new("page-azure-account"))
    }

    pub(crate) fn show(
        &mut self,
        ui: &mut Ui,
        instance: (PortalTabId, Id),
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
        let response = Self::account_button(
            ui,
            Self::display_alias(alias, *demo_mode),
            tenant.as_deref(),
            email,
            selected_profile,
        );
        let popup_id = Popup::default_response_id(&response);
        let open_requested = self.open_requested.remove(&instance);
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
                if let Some(error) = self.selection_errors.get(&instance) {
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

    pub(crate) fn display_alias(alias: &str, demo_mode: bool) -> &str {
        if demo_mode && alias.contains('@') {
            AzureAccountProfile::REDACTED_EMAIL
        } else {
            alias
        }
    }

    fn account_button(
        ui: &mut Ui,
        alias: &str,
        tenant: Option<&str>,
        email: Option<&str>,
        profile: Option<&AzureAccountProfile>,
    ) -> Response {
        const LEFT_INSET: f32 = 8.0;
        const RIGHT_INSET: f32 = 49.0;

        let secondary_color = ui.visuals().weak_text_color();
        let tenant_text =
            tenant.map(|tenant| RichText::new(tenant).size(9.0).color(secondary_color));
        let tenant_width = tenant_text.as_ref().map_or(0.0, |text| {
            WidgetText::from(text.clone())
                .into_galley(
                    ui,
                    Some(TextWrapMode::Extend),
                    f32::INFINITY,
                    TextStyle::Small,
                )
                .size()
                .x
                .ceil()
                + LEFT_INSET
                + RIGHT_INSET
                + 1.0
        });
        let available = ui.available_width();
        let width = (available * 0.42)
            .clamp(56.0, 236.0)
            .max(tenant_width.min(available * 0.58))
            .min(available);
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(width, ACCOUNT_CONTROL_HEIGHT), Sense::click());
        response.widget_info(|| {
            WidgetInfo::labeled(
                WidgetType::Button,
                ui.is_enabled(),
                format!("Choose account: {alias}"),
            )
        });
        let visuals = ui.style().interact(&response);
        if response.hovered()
            || response.has_focus()
            || Popup::is_id_open(ui.ctx(), Popup::default_response_id(&response))
        {
            ui.painter().rect_filled(rect, 6, visuals.weak_bg_fill);
        }
        if response.has_focus() {
            ui.painter()
                .rect_stroke(rect, 6, visuals.fg_stroke, StrokeKind::Inside);
        }

        if width > 80.0 {
            let text_color = ui.visuals().text_color();
            let text_rect = Rect::from_min_max(
                rect.min + Vec2::new(LEFT_INSET, 0.0),
                rect.max - Vec2::new(RIGHT_INSET, 0.0),
            );
            let painter = ui
                .painter()
                .with_clip_rect(text_rect.intersect(ui.clip_rect()));
            let lines = [
                (
                    Some(RichText::new(alias).size(12.0).strong().color(text_color)),
                    14.0,
                ),
                (
                    email.map(|email| RichText::new(email).size(10.0).color(secondary_color)),
                    11.0,
                ),
                (tenant_text, 10.0),
            ];
            let height: f32 = lines
                .iter()
                .filter_map(|(text, height)| text.as_ref().map(|_| *height))
                .sum();
            let mut top = rect.center().y - height * 0.5;
            for (text, height) in lines {
                let Some(text) = text else {
                    continue;
                };
                let value = WidgetText::from(text).into_galley(
                    ui,
                    Some(TextWrapMode::Truncate),
                    text_rect.width(),
                    TextStyle::Small,
                );
                let mut position = text_rect.min;
                position.y = top + (height - value.size().y) * 0.5;
                painter.galley(position, value, text_color);
                top += height;
            }
        }
        Self::paint_avatar(
            ui,
            Rect::from_center_size(
                rect.right_center() - Vec2::new(32.0, 0.0),
                Vec2::splat(24.0),
            ),
            profile.and_then(|profile| profile.photo.as_ref()),
        );
        let arrow_center = rect.right_center() - Vec2::new(12.0, 0.0);
        ui.painter().line_segment(
            [
                arrow_center + Vec2::new(-4.0, -2.0),
                arrow_center + Vec2::new(0.0, 2.0),
            ],
            Stroke::new(1.5, visuals.fg_stroke.color),
        );
        ui.painter().line_segment(
            [
                arrow_center + Vec2::new(0.0, 2.0),
                arrow_center + Vec2::new(4.0, -2.0),
            ],
            Stroke::new(1.5, visuals.fg_stroke.color),
        );
        response.on_hover_cursor(CursorIcon::PointingHand)
    }

    fn paint_avatar(ui: &Ui, rect: Rect, photo: Option<&TextureHandle>) {
        if let Some(photo) = photo {
            Image::new(photo).corner_radius(16_u8).paint_at(ui, rect);
            return;
        }
        Image::new(include_image!("../assets/iam-user.svg")).paint_at(ui, rect);
    }
}

impl MyApp {
    /// Draw the account control for pages whose data depends on an Azure account.
    pub(crate) fn show_page_account_picker(&mut self, ui: &mut Ui) {
        self.update_account_profile(ui.ctx());
        let instance = AzureAccountPicker::page_instance(self.tabs.active_id());
        let action = ui
            .push_id(instance, |ui| {
                self.account_picker.show(
                    ui,
                    instance,
                    self.azure.authentication,
                    &self.account_profiles,
                    self.account_operation.is_some(),
                    &mut self.egui_config.demo_mode,
                )
            })
            .inner;
        if let Some(action) = action {
            self.handle_account_picker_action(action, ui.ctx());
        }
    }

    pub(crate) fn account_page_header<R>(
        &mut self,
        ui: &mut Ui,
        heading: impl FnOnce(&mut Self, &mut Ui) -> R,
    ) -> R {
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), ACCOUNT_CONTROL_HEIGHT),
            Layout::right_to_left(Align::Center),
            |ui| {
                self.show_page_account_picker(ui);
                ui.add_space(12.0);
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| heading(self, ui))
                    .inner
            },
        )
        .inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portal_typography;
    use eframe::egui::Context;
    use eframe::egui::Pos2;
    use eframe::egui::RawInput;
    use eframe::egui::Shape;

    #[test]
    fn page_account_control_shows_the_full_tenant_id() {
        const TENANT: &str = "bdbdbdbd-bdbd-4bdb-abdb-bdbdbdbdbdbd";
        let context = Context::default();
        portal_typography::install(&context);
        let mut output = context.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(480.0, 60.0))),
                ..Default::default()
            },
            |ui| {
                AzureAccountPicker::account_button(ui, "admin", Some(TENANT), None, None);
            },
        );
        output.textures_delta.clear();
        assert!(output.shapes.iter().any(|clipped| {
            matches!(&clipped.shape, Shape::Text(text)
                if text.galley.text() == TENANT && !text.galley.elided
                    && clipped.clip_rect.contains_rect(text.visual_bounding_rect()))
        }));
    }

    #[test]
    fn account_popup_request_waits_for_the_originating_tab() {
        let context = Context::default();
        let mut picker = AzureAccountPicker {
            catalogue: Ok(CloudTerrastodonAccountCatalogue::default()),
            selection_errors: HashMap::new(),
            open_requested: HashSet::new(),
        };
        let profiles = AzureAccountProfiles::default();
        let requested = PortalTabId(1);
        picker.open(requested);

        let mut demo_mode = false;
        let expected_instance = AzureAccountPicker::page_instance(requested);
        for instance in [
            AzureAccountPicker::page_instance(PortalTabId(2)),
            (requested, Id::new("comparison-account")),
            expected_instance,
        ] {
            let mut output = context.run_ui(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(700.0, 480.0))),
                    ..Default::default()
                },
                |ui| {
                    ui.push_id(instance, |ui| {
                        picker.show(ui, instance, None, &profiles, false, &mut demo_mode);
                    });
                },
            );
            output.textures_delta.clear();
            assert_eq!(Popup::is_any_open(&context), instance == expected_instance);
        }
    }
}
