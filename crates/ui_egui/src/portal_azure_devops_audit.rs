use crate::app::MyApp;
use crate::portal_audit_output::show_audit_output;
use crate::portal_page::PortalPage;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_azure_devops_cli::audit_azure_devops;
use cloud_terrastodon_azure_devops_cli::audit_cli::DEFAULT_PAID_LICENSE_INACTIVITY_THRESHOLD;
use cloud_terrastodon_azure_devops_cli::audit_cli::DEFAULT_TEST_LICENSE_INACTIVITY_THRESHOLD;
use cloud_terrastodon_cli_types::HumantimeDurationCli;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use std::time::Duration;

pub(crate) fn show_azure_devops_audit(app: &mut MyApp, ui: &mut Ui) {
    app.show_azure_devops_organization_scope_picker(ui);
    let Some(organization) = app.azure.azure_devops_organization.clone() else {
        return;
    };
    let Some(authentication) = app.ready_azure_authentication() else {
        app.show_azure_authentication_required(ui);
        return;
    };
    if PortalTabOpen::from_response(&ui.link("View audit warnings in Logs"), || {
        PortalTabTarget::Page(PortalPage::Logs)
    })
    .is_some_and(|open| open.apply(app))
    {
        return;
    }

    let paid = show_inactivity_threshold(
        ui,
        "Paid license inactivity",
        DEFAULT_PAID_LICENSE_INACTIVITY_THRESHOLD,
    );
    let test_plans = show_inactivity_threshold(
        ui,
        "Test Plans license inactivity",
        DEFAULT_TEST_LICENSE_INACTIVITY_THRESHOLD,
    );
    let (Some(paid), Some(test_plans)) = (paid, test_plans) else {
        return;
    };

    ui.push_id(
        (app.azure.devops_organization_generation, paid, test_plans),
        |ui| {
            show_audit_output(
                app,
                ui,
                "Azure DevOps audit",
                async move |executor, clean| {
                    audit_azure_devops(
                        organization,
                        test_plans,
                        paid,
                        authentication,
                        clean,
                        &executor,
                    )
                    .await
                },
            );
        },
    );
}

fn show_inactivity_threshold(ui: &mut Ui, label: &str, default: Duration) -> Option<Duration> {
    let id = ui.make_persistent_id(label);
    let mut text = ui
        .data(|data| data.get_temp::<String>(id))
        .unwrap_or_else(|| String::from(&HumantimeDurationCli(default.into())));
    ui.horizontal_wrapped(|ui| {
        let label = ui.label(label);
        let response = ui
            .add(TextEdit::singleline(&mut text).desired_width(128.0))
            .labelled_by(label.id)
            .on_hover_text("Enter a duration such as 60d or 2months");
        if response.changed() {
            ui.data_mut(|data| data.insert_temp(id, text.clone()));
        }
        match HumantimeDurationCli::try_from(text) {
            Ok(duration) => Some(duration.0.into()),
            Err(error) => {
                ui.colored_label(ui.visuals().error_fg_color, error.to_string());
                None
            }
        }
    })
    .inner
}
