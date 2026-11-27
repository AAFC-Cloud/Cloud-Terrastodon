use cloud_terrastodon_azure::AzureTenantAlias;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::TrackedAzureDevOpsOrganization;
use eframe::egui::ComboBox;
use eframe::egui::Id;
use eframe::egui::Modal;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use eyre::WrapErr;

/// In-progress local registration of an organization and its tenant association.
#[derive(Clone, Default)]
pub(crate) struct AzureDevOpsOrganizationTrackingForm {
    input: String,
    tenant: Option<AzureTenantId>,
    error: Option<String>,
}

impl AzureDevOpsOrganizationTrackingForm {
    pub fn new(tenant: Option<AzureTenantId>) -> Self {
        Self {
            tenant,
            ..Self::default()
        }
    }

    pub fn show(
        &mut self,
        ui: &Ui,
        id: Id,
        title: &str,
        organization: Option<&AzureDevOpsOrganizationUrl>,
        tenants: &[(AzureTenantId, Vec<AzureTenantAlias>)],
    ) -> (Option<TrackedAzureDevOpsOrganization>, bool) {
        // A menu and its nested combo box would compete for egui's popup slot.
        let modal = Modal::new(id).show(ui.ctx(), |ui| {
            ui.set_min_width(350.0);
            ui.heading(title);
            if let Some(organization) = organization {
                ui.label(organization.to_string());
            } else {
                let label = ui.label("Organization name or URL");
                if ui
                    .add(
                        TextEdit::singleline(&mut self.input)
                            .id(id.with("organization-url"))
                            .desired_width(320.0),
                    )
                    .labelled_by(label.id)
                    .changed()
                {
                    self.error = None;
                }
            }
            let label = ui.label("Tenant");
            let selected = self.tenant.map_or_else(
                || "None".to_owned(),
                |tenant| {
                    let aliases = tenants
                        .iter()
                        .find(|(id, _)| *id == tenant)
                        .map_or(&[][..], |(_, aliases)| aliases.as_slice());
                    tenant_label(tenant, aliases)
                },
            );
            ComboBox::from_id_salt("tracked-tenant")
                .width(320.0)
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.tenant, None, "None");
                    for (tenant, aliases) in tenants {
                        ui.selectable_value(
                            &mut self.tenant,
                            Some(*tenant),
                            tenant_label(*tenant, aliases),
                        );
                    }
                })
                .response
                .labelled_by(label.id);

            if let Some(error) = &self.error {
                ui.label(error);
            }
            let save = ui
                .horizontal(|ui| {
                    let save = ui.button("Save").clicked();
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    save
                })
                .inner;
            if !save {
                return None;
            }
            let url = organization.cloned().map_or_else(
                || {
                    self.input
                        .trim()
                        .parse()
                        .wrap_err("Enter a valid organization name or URL")
                },
                Ok,
            );
            match url {
                Ok(url) => Some(TrackedAzureDevOpsOrganization {
                    url,
                    tenant_id: self.tenant,
                }),
                Err(error) => {
                    self.error = Some(format!("{error:#}"));
                    None
                }
            }
        });
        let dismissed = modal.should_close();
        (modal.inner, dismissed)
    }
}

fn tenant_label(tenant: AzureTenantId, aliases: &[AzureTenantAlias]) -> String {
    aliases
        .first()
        .map_or_else(|| tenant.to_string(), |alias| format!("{alias} · {tenant}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;
    use eframe::egui::Event;
    use eframe::egui::FullOutput;
    use eframe::egui::Modifiers;
    use eframe::egui::PointerButton;
    use eframe::egui::Pos2;
    use eframe::egui::RawInput;
    use eframe::egui::Rect;
    use eframe::egui::Shape;
    use eframe::egui::Vec2;

    #[test]
    fn organization_tracking_form_keeps_url_and_tenant_picks_open() -> eyre::Result<()> {
        let context = Context::default();
        let id = Id::new("track-org-test");
        let tenant: AzureTenantId = "00000000-0000-0000-0000-000000000010".parse()?;
        let tenants = [(tenant, vec!["example-tenant".parse()?])];
        let frame = |form: &mut AzureDevOpsOrganizationTrackingForm, events| {
            let mut output = context.run_ui(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(920.0, 520.0))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    let (registration, dismissed) =
                        form.show(ui, id, "Track organization", None, &tenants);
                    assert!(!dismissed);
                    assert!(registration.is_none());
                },
            );
            output.textures_delta.clear();
            output
        };
        let click = |form: &mut AzureDevOpsOrganizationTrackingForm, pos| {
            frame(form, vec![Event::PointerMoved(pos)]);
            for pressed in [true, false] {
                frame(
                    form,
                    vec![
                        Event::PointerMoved(pos),
                        Event::PointerButton {
                            pos,
                            button: PointerButton::Primary,
                            pressed,
                            modifiers: Modifiers::NONE,
                        },
                    ],
                );
            }
        };
        let text_position = |output: FullOutput, label: &str| {
            output
                .shapes
                .iter()
                .find_map(|clipped| match &clipped.shape {
                    Shape::Text(shape) if shape.galley.text() == label => {
                        Some(shape.visual_bounding_rect().center())
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("Missing {label:?}"))
        };
        let mut form = AzureDevOpsOrganizationTrackingForm::new(None);
        // The first pass measures the modal. Let egui publish its final
        // position to the interaction snapshot before locating the field.
        for _ in 0..3 {
            frame(&mut form, Vec::new());
        }
        let url_edit = context.read_response(id.with("organization-url")).unwrap();
        assert!(url_edit.interact_rect.is_positive());
        click(&mut form, url_edit.interact_rect.center());
        assert!(context.memory(|memory| memory.has_focus(url_edit.id)));
        let output = frame(&mut form, vec![Event::Text("example-org".into())]);
        assert_eq!(form.input, "example-org");

        click(&mut form, text_position(output, "None"));
        let label = format!("example-tenant · {tenant}");
        let output = frame(&mut form, Vec::new());
        click(&mut form, text_position(output, &label));
        assert_eq!(form.tenant, Some(tenant));

        let output = frame(&mut form, Vec::new());
        click(&mut form, text_position(output, &label));
        let output = frame(&mut form, Vec::new());
        click(&mut form, text_position(output, "None"));
        assert_eq!(form.tenant, None);
        assert_eq!(form.input, "example-org");
        Ok(())
    }
}
