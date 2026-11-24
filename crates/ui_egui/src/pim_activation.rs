use crate::app::MyApp;
use crate::pim_activation_assignment::PimActivationAssignment;
use crate::portal_page::PortalPage;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::uuid::Uuid;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use eframe::egui::Button;
use eframe::egui::DragValue;
use eframe::egui::Frame;
use eframe::egui::Margin;
use eframe::egui::Panel;
use eframe::egui::ScrollArea;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use std::time::Duration;
use std::time::Instant;
use teamy_tracing::gui::DiagnosticReportView;
use teamy_tracing::gui::escape_display;

/// An activation draft pinned to the account and role shown when it was opened.
pub(crate) struct PimActivation {
    id: Uuid,
    assignment: PimActivationAssignment,
    authentication: AzureBearerAuthenticationSelection,
    policy: Loadable<Duration>,
    minutes: u64,
    reason: String,
    submission: Loadable<()>,
}

impl PimActivation {
    pub fn is_loading(&self) -> bool {
        self.is_submitting() || matches!(self.policy, Loadable::Loading { .. })
    }

    fn is_submitting(&self) -> bool {
        matches!(self.submission, Loadable::Loading { .. })
    }

    fn duration(&self) -> Option<Duration> {
        let maximum = self.policy.as_loaded()?;
        (self.minutes > 0 && self.minutes <= maximum.as_secs() / 60)
            .then(|| Duration::from_secs(self.minutes * 60))
    }
}

pub(crate) fn open_pim_activation(app: &mut MyApp, assignment: PimActivationAssignment) {
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    if !matches!(
        authentication.auth_source,
        AzureBearerAuthSource::AzureCli | AzureBearerAuthSource::Browser
    ) || !assignment.can_activate()
        || app
            .azure
            .pim_activation
            .as_ref()
            .is_some_and(PimActivation::is_submitting)
    {
        return;
    }

    let id = Uuid::new_v4();
    let account_generation = app.azure.account_generation;
    let executor = app.executor.clone();
    let work_cache = app.work_cache.clone();
    let cancellation = executor.cancellation().clone();
    let started_at = Instant::now();
    app.azure.pim_activation = Some(PimActivation {
        id,
        assignment: assignment.clone(),
        authentication,
        policy: Loadable::NotLoaded,
        minutes: 60,
        reason: String::new(),
        submission: Loadable::NotLoaded,
    });
    let queued = LoadableWorkBuilder::<MyApp, Duration>::new()
        .description("Load PIM activation policy")
        .setter(move |app, policy| {
            if let Some(draft) = activation_mut(app, account_generation, id) {
                if let Some(maximum) = policy.as_loaded() {
                    draft.minutes = draft.minutes.min(maximum.as_secs() / 60);
                }
                draft.policy = policy;
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            assignment.maximum_duration(authentication, &executor).await
        })
        .build()
        .enqueue(
            &app.work_tracker.clone(),
            app.tx.clone(),
            app,
            &cancellation,
        );
    if let Err(error) = queued
        && let Some(draft) = activation_mut(app, account_generation, id)
    {
        draft.policy = Loadable::Failed {
            error: error.into(),
            started_at,
            finished_at: Instant::now(),
        };
    }
}

pub(crate) fn show_pim_activation(app: &mut MyApp, ui: &mut Ui, page: PortalPage) {
    let authentication = app.ready_azure_authentication();
    let Some(draft) = app.azure.pim_activation.as_mut() else {
        return;
    };
    if !matches!(
        (&draft.assignment, page),
        (
            PimActivationAssignment::Entra { .. },
            PortalPage::PimEntraRoles
        ) | (
            PimActivationAssignment::Azure(_),
            PortalPage::PimAzureResources
        )
    ) {
        return;
    }
    let mut submit = false;
    let mut close = false;
    Panel::right(ui.make_persistent_id("pim-activation"))
        .exact_size((ui.available_width() * 0.52).clamp(240.0, 420.0).min(ui.available_width()))
        .resizable(false)
        .frame(Frame::new().inner_margin(Margin::symmetric(16, 12)))
        .show(ui, |ui| {
            ScrollArea::vertical().id_salt(draft.id).show(ui, |ui| {
                ui.heading("Activate role");
                ui.strong(escape_display(draft.assignment.role_name()));
                ui.label(escape_display(&draft.assignment.scope_label(draft.authentication)));
                ui.add_space(12.0);

                match &draft.submission {
                    Loadable::NotLoaded => {
                        match &draft.policy {
                            Loadable::Loading { .. } | Loadable::NotLoaded => {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label("Checking activation policy…");
                                });
                            }
                            Loadable::Failed { error, .. } => {
                                ui.label("Could not load the activation policy.");
                                ui.collapsing("Error details", |ui| DiagnosticReportView::new(error).show(ui));
                            }
                            Loadable::Loaded { value, .. } => {
                                let maximum_minutes = value.as_secs() / 60;
                                if maximum_minutes == 0 {
                                    ui.label("The role policy allows less than one minute of activation.");
                                } else {
                                    ui.horizontal(|ui| {
                                        let label = ui.label("Duration (minutes)");
                                        ui.add(DragValue::new(&mut draft.minutes).range(1..=maximum_minutes))
                                            .labelled_by(label.id);
                                    });
                                    let label = ui.label("Reason");
                                    ui.add(TextEdit::multiline(&mut draft.reason).char_limit(500).desired_rows(4).desired_width(f32::INFINITY))
                                        .labelled_by(label.id);
                                }
                            }
                        }
                        let can_submit = authentication == Some(draft.authentication)
                            && draft.assignment.can_activate()
                            && draft.duration().is_some()
                            && !draft.reason.trim().is_empty();
                        submit = ui.add_enabled(can_submit, Button::new("Activate")).clicked();
                    }
                    Loadable::Loading { .. } => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Submitting activation request…");
                        });
                    }
                    Loadable::Loaded { .. } => {
                        ui.label("Activation request submitted. Azure may still require approval.");
                    }
                    Loadable::Failed { error, .. } => {
                        ui.label("Could not confirm the activation request.");
                        ui.collapsing("Error details", |ui| DiagnosticReportView::new(error).show(ui));
                    }
                }
                let label = if matches!(draft.submission, Loadable::NotLoaded) { "Cancel" } else { "Close" };
                close = ui.add_enabled(!draft.is_submitting(), Button::new(label)).clicked();
            });
        });

    if submit {
        submit_pim_activation(app);
    } else if close {
        app.azure.pim_activation = None;
    }
}

fn submit_pim_activation(app: &mut MyApp) {
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    let Some(draft) = app.azure.pim_activation.as_ref() else {
        return;
    };
    let Some(duration) = draft.duration() else {
        return;
    };
    let reason = draft.reason.trim();
    if !matches!(draft.submission, Loadable::NotLoaded)
        || draft.authentication != authentication
        || !draft.assignment.can_activate()
        || reason.is_empty()
        || reason.chars().count() > 500
    {
        return;
    }

    let reason = reason.to_owned();
    let id = draft.id;
    let assignment = draft.assignment.clone();
    let is_entra = matches!(assignment, PimActivationAssignment::Entra { .. });
    let account_generation = app.azure.account_generation;
    let executor = app.executor.clone();
    let work_cache = app.work_cache.clone();
    let cancellation = executor.cancellation().clone();
    let started_at = Instant::now();
    let queued = LoadableWorkBuilder::<MyApp, ()>::new()
        .description("Submit PIM activation")
        .setter(move |app, submission| {
            let Some(content) = app.azure_content_mut(account_generation) else {
                return;
            };
            let Some(draft) = content
                .pim_activation
                .as_mut()
                .filter(|draft| draft.id == id)
            else {
                return;
            };
            let finished = matches!(
                submission,
                Loadable::Loaded { .. } | Loadable::Failed { .. }
            );
            draft.submission = submission;
            if finished && is_entra {
                content.pim_roles.entra = Loadable::NotLoaded;
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            assignment
                .submit(authentication, duration, reason, &executor)
                .await
        })
        .build()
        .enqueue(
            &app.work_tracker.clone(),
            app.tx.clone(),
            app,
            &cancellation,
        );
    if let Err(error) = queued
        && let Some(draft) = activation_mut(app, account_generation, id)
    {
        draft.submission = Loadable::Failed {
            error: error.into(),
            started_at,
            finished_at: Instant::now(),
        };
    }
}

fn activation_mut(app: &mut MyApp, generation: u64, id: Uuid) -> Option<&mut PimActivation> {
    app.azure_content_mut(generation)?
        .pim_activation
        .as_mut()
        .filter(|draft| draft.id == id)
}
