use super::*;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_credentials::AuthSource;

fn session() -> DumpPlanSession {
    let planner = DumpPlanner::new([], [], Vec::new());
    let targets = vec!["FirstInventory".into(), "SecondInventory".into()];
    let plan = CancellationToken::new()
        .with_scope(|scope| planner.plan(&targets, &scope))
        .unwrap();
    DumpPlanSession::new(
        planner,
        DumpPlanReport {
            organization: None,
            tenant: None,
            auth_source: AuthSource::None,
            plan,
            diagram: None,
        },
        targets,
    )
}

#[test]
fn replacing_targets_retains_unknown_choices_and_can_clear_the_entire_plan() -> Result<()> {
    CancellationToken::new().with_scope(|scope| {
        let mut session = session();
        let initial = session.refresh(None, &scope)?.unwrap();
        let expected = session.targets().to_vec();
        assert_eq!(session.target_choices(), expected);
        let cleared = session
            .replace_targets(&initial, &expected, &[], &scope)?
            .unwrap();
        assert!(session.targets().is_empty());
        assert!(session.report.plan.nodes.is_empty());
        assert!(!cleared.contains("Selected targets"));
        assert!(!cleared.contains("Target search"));
        assert!(
            session.refresh(Some(&cleared), &scope)?.is_none(),
            "accepted edits should settle without rewriting"
        );
        Ok(())
    })
}

#[test]
fn target_replacement_is_terminal_owned_and_stale_snapshots_are_rejected() -> Result<()> {
    CancellationToken::new().with_scope(|scope| {
        let mut session = session();
        let initial = session.refresh(None, &scope)?.unwrap();
        let expected = session.targets().to_vec();
        let latest = session
            .replace_targets(
                &initial,
                &expected,
                &["AppServiceSiteConfig".into()],
                &scope,
            )?
            .unwrap();
        assert_eq!(session.targets(), ["AppServiceSiteConfig"]);
        assert!(
            session
                .report
                .plan
                .nodes
                .iter()
                .any(|node| { node.target && node.label == "AppServiceSiteConfig" })
        );
        let error = session
            .replace_targets(&latest, &expected, &[], &scope)
            .unwrap_err();
        assert!(error.to_string().contains("Plan targets changed"));
        assert_eq!(session.targets(), ["AppServiceSiteConfig"]);
        assert!(session.refresh(Some(&latest), &scope)?.is_none());
        // The old file does not contain authoritative target selection and
        // cannot replace the accepted terminal state when it is saved again.
        session.refresh(Some(&initial), &scope)?;
        assert_eq!(session.targets(), ["AppServiceSiteConfig"]);
        Ok(())
    })
}

#[test]
fn app_service_site_config_is_selectable_and_updates_the_diagram_without_execution() -> Result<()> {
    use cloud_terrastodon_azure::AppServiceListRequest;

    CancellationToken::new().with_scope(|scope| {
        // Only the descriptor participates. No Azure request, authentication or
        // arbitrary constructor runs while deriving the nested output shape.
        let planner = DumpPlanner::new(
            [AppServiceListRequest::REGISTERED_INTO_FUTURE],
            [],
            Vec::new(),
        );
        let plan = planner.plan(&[], &scope)?;
        let mut session = DumpPlanSession::new(
            planner,
            DumpPlanReport {
                organization: None,
                tenant: None,
                auth_source: AuthSource::None,
                plan,
                diagram: None,
            },
            Vec::new(),
        );
        assert!(
            session
                .target_choices()
                .iter()
                .any(|name| name == "AzureAppServiceSiteConfig")
        );
        let initial = session.refresh(None, &scope)?.unwrap();
        let updated = session
            .replace_targets(&initial, &[], &["AzureAppServiceSiteConfig".into()], &scope)?
            .unwrap();
        assert_eq!(session.targets(), ["AzureAppServiceSiteConfig"]);
        assert!(
            session
                .report
                .plan
                .nodes
                .iter()
                .any(|node| node.target && node.label == "AzureAppServiceSiteConfig")
        );
        assert!(updated.contains("AzureAppServiceSiteConfig"));
        assert!(!updated.contains("Selected targets"));
        assert!(session.refresh(Some(&updated), &scope)?.is_none());
        Ok(())
    })
}
