use crate::app::MyApp;
use crate::portal_filter_chip::portal_filter_chip;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_tab_target::PortalTabTarget;
use crate::scope_activity_log::ScopeActivityLog;
use crate::scope_activity_period::ScopeActivityPeriod;
use crate::scope_activity_results::show_activity_log_results;
use cloud_terrastodon_azure::AzureActivityLogScope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use eframe::egui::Button;
use eframe::egui::Ui;
use std::sync::Arc;
use std::time::Instant;
use teamy_tracing::gui::DiagnosticReportView;

pub(crate) fn scope_activity_ui(app: &mut MyApp, ui: &mut Ui, origin: &PortalResourceScope) {
    if !app.azure.scope_activity.contains_key(origin) {
        load_scope_activity(app, origin, false);
    }
    let Some(activity) = app.azure.scope_activity.get(origin) else {
        return;
    };
    let request_id =
        ui.make_persistent_id(("activity-log-request", app.azure.account_generation, origin));
    let scope_id = request_id.with("scope");
    let period_id = request_id.with("period");
    let mut scope = ui
        .data(|data| data.get_temp::<AzureActivityLogScope>(scope_id))
        .unwrap_or_else(|| activity.scope.clone());
    let mut period = ui
        .data(|data| data.get_temp::<ScopeActivityPeriod>(period_id))
        .unwrap_or(activity.period);
    ui.horizontal_wrapped(|ui| {
        ui.strong("Azure request");
        ui.hyperlink_to(
            "Learn more about Activity Log",
            "https://aka.ms/ActivityLogOverviewDoc",
        );
    });
    let apply = ui
        .horizontal_wrapped(|ui| {
            request_scope_picker(app, ui, origin, &mut scope);
            portal_filter_chip(ui, format!("Timespan · {}", period.label()), |ui| {
                for choice in ScopeActivityPeriod::ALL {
                    if ui
                        .selectable_value(&mut period, choice, choice.label())
                        .clicked()
                    {
                        ui.close();
                    }
                }
            });
            let changed = scope != activity.scope || period != activity.period;
            ui.add_enabled(
                changed && !activity.is_loading(),
                Button::new("Apply request"),
            )
            .clicked()
        })
        .inner;
    if apply {
        ui.data_mut(|data| {
            data.remove::<AzureActivityLogScope>(scope_id);
            data.remove::<ScopeActivityPeriod>(period_id);
        });
        submit_scope_activity(app, origin, scope, period, false);
    } else {
        ui.data_mut(|data| {
            if scope != activity.scope {
                data.insert_temp(scope_id, scope);
            } else {
                data.remove::<AzureActivityLogScope>(scope_id);
            }
            if period != activity.period {
                data.insert_temp(period_id, period);
            } else {
                data.remove::<ScopeActivityPeriod>(period_id);
            }
        });
    }

    let Some(activity) = app.azure.scope_activity.get(origin) else {
        return;
    };
    if let Some(window) = &activity.utc_window {
        ui.weak(format!(
            "Submitted request · {} · {window}",
            request_scope_label(app, &activity.scope)
        ));
    }
    ui.separator();
    match &activity.response {
        Loadable::Loaded { value, .. } => {
            let events = Arc::clone(value);
            let requested_scope = activity.scope.clone();
            show_activity_log_results(app, ui, origin, &requested_scope, &events);
        }
        Loadable::Failed { error, .. } => {
            ui.label("Could not load activity for the requested scope.");
            let retry = ui.button("Retry request").clicked();
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui);
            });
            if retry {
                load_scope_activity(app, origin, true);
            }
        }
        Loadable::NotLoaded | Loadable::Loading { .. } => {
            ui.spinner();
        }
    }
}

pub(crate) fn load_scope_activity(app: &mut MyApp, origin: &PortalResourceScope, refresh: bool) {
    let (scope, period) = app.azure.scope_activity.get(origin).map_or_else(
        || (request_scope(origin), ScopeActivityPeriod::default()),
        |activity| (activity.scope.clone(), activity.period),
    );
    submit_scope_activity(app, origin, scope, period, refresh);
}

fn submit_scope_activity(
    app: &mut MyApp,
    origin: &PortalResourceScope,
    request_scope: AzureActivityLogScope,
    period: ScopeActivityPeriod,
    refresh: bool,
) {
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    if app
        .azure
        .scope_activity
        .get(origin)
        .is_some_and(ScopeActivityLog::is_loading)
    {
        return;
    }

    let request = period.request(authentication, request_scope.clone());
    let utc_window = request.as_ref().ok().map(|request| {
        format!(
            "{} to {} UTC",
            request.from.strftime("%F %H:%M"),
            request.until.strftime("%F %H:%M")
        )
    });
    app.azure.scope_activity.insert(
        origin.clone(),
        ScopeActivityLog {
            scope: request_scope,
            period,
            utc_window,
            response: Loadable::NotLoaded,
        },
    );
    let account_generation = app.azure.account_generation;
    let origin_id = origin.clone();
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let work_cache = app.work_cache.clone();
    let started_at = Instant::now();
    let enqueue_result = LoadableWorkBuilder::<MyApp, _>::new()
        .description("Load activity log")
        .setter(move |app, response| {
            if let Some(azure) = app.azure_content_mut(account_generation)
                && let Some(activity) = azure.scope_activity.get_mut(&origin_id)
            {
                activity.response = response;
            }
        })
        .work(async move |scope| {
            let request = request?;
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            request
                .execute(&executor)
                .with_invalidation(refresh)
                .await
                .map(Arc::new)
        })
        .build()
        .enqueue(
            &app.work_tracker.clone(),
            app.tx.clone(),
            app,
            &cancellation,
        );
    if let Err(error) = enqueue_result
        && let Some(activity) = app.azure.scope_activity.get_mut(origin)
    {
        activity.response = Loadable::Failed {
            error: error.into(),
            started_at,
            finished_at: Instant::now(),
        };
    }
}

fn request_scope(origin: &PortalResourceScope) -> AzureActivityLogScope {
    match origin.arm_scope() {
        ScopeImpl::ResourceGroup(id) => AzureActivityLogScope::ResourceGroup(id),
        ScopeImpl::Subscription(id) => AzureActivityLogScope::Subscription(id),
        ScopeImpl::ManagementGroup(id) => AzureActivityLogScope::ManagementGroup(id),
        id => AzureActivityLogScope::Resource(Box::new(id)),
    }
}

fn request_scope_picker(
    app: &MyApp,
    ui: &mut Ui,
    origin: &PortalResourceScope,
    selected: &mut AzureActivityLogScope,
) {
    portal_filter_chip(
        ui,
        format!("Request scope · {}", request_scope_label(app, selected)),
        |ui| {
            let initial = request_scope(origin);
            let ancestors = app.portal_hierarchy(&origin.overview());
            let choices = ancestors.into_iter().rev().filter_map(|ancestor| {
                let scope = match ancestor {
                    PortalTabTarget::ManagementGroup(id) => {
                        AzureActivityLogScope::ManagementGroup(id)
                    }
                    PortalTabTarget::Subscription(id) => AzureActivityLogScope::Subscription(id),
                    PortalTabTarget::ResourceGroup(id) => AzureActivityLogScope::ResourceGroup(id),
                    _ => return None,
                };
                (scope != initial).then_some(scope)
            });
            for scope in std::iter::once(initial.clone()).chain(choices) {
                let label = request_scope_label(app, &scope);
                let response = ui.selectable_label(*selected == scope, label);
                let response = if matches!(&scope, AzureActivityLogScope::ManagementGroup(_)) {
                    response.on_hover_text("Queries management group activity events")
                } else {
                    response
                };
                if response.clicked() {
                    *selected = scope;
                    ui.close();
                }
            }
        },
    );
}

fn request_scope_label(app: &MyApp, scope: &AzureActivityLogScope) -> String {
    let (kind, target) = match scope {
        AzureActivityLogScope::Subscription(id) => {
            ("Subscription", PortalTabTarget::Subscription(*id))
        }
        AzureActivityLogScope::ResourceGroup(id) => {
            ("Resource group", PortalTabTarget::ResourceGroup(id.clone()))
        }
        AzureActivityLogScope::ManagementGroup(id) => (
            "Management group",
            PortalTabTarget::ManagementGroup(id.clone()),
        ),
        AzureActivityLogScope::Resource(id) => ("Resource", PortalTabTarget::from_arm_scope(id)),
    };
    format!("{kind} {}", app.portal_tab_title(&target))
}
