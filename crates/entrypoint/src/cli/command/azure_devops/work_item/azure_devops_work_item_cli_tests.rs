use super::*;

fn parse(args: &[&str]) -> AzureDevOpsWorkItemArgs {
    figue::from_slice(args)
        .into_result()
        .expect("synthetic CLI arguments should parse")
        .get()
}

fn id() -> String {
    // Generated parser fixture, not an identifier from any service.
    ((uuid::Uuid::new_v4().as_u128() % u16::MAX as u128) + 1).to_string()
}

#[test]
fn parses_item_reads_and_tenant_context() {
    let id = id();
    let suffix = uuid::Uuid::new_v4();
    let tenant = format!("fixture-{suffix}");
    let org = format!("fixture-{suffix}");
    let project = format!("fixture {suffix}");
    let parsed = parse(&[
        "show",
        &id,
        "--tenant",
        &tenant,
        "--org",
        &org,
        "--project",
        &project,
        "--expand",
        "relations",
    ]);
    let AzureDevOpsWorkItemCommand::Show(args) = parsed.command else {
        panic!("expected show");
    };
    assert!(args.id.to_string() == id);
    assert!(
        args.tenant
            .as_ref()
            .is_some_and(|value| value.to_string() == tenant)
    );
    assert!(
        args.project
            .as_ref()
            .is_some_and(|value| value.to_string() == project)
    );
    assert!(
        args.org
            .as_ref()
            .is_some_and(|value| value.organization_name.as_ref() == org)
    );
    let cli: crate::cli::Cli = figue::from_slice(&[
        "--auth-source",
        "browser",
        "az",
        "devops",
        "work-item",
        "show",
        &id,
        "--tenant",
        &tenant,
        "--org",
        &org,
        "--project",
        &project,
    ])
    .into_result()
    .expect("full command with named scope parses")
    .get();
    assert!(cli.global_args.auth_source == cloud_terrastodon_credentials::AuthSource::Browser);
    let parsed = parse(&["list", "--ids", &id]);
    assert!(matches!(
        parsed.command,
        AzureDevOpsWorkItemCommand::List(_)
    ));
}

#[test]
fn parses_field_values_and_definitions_as_separate_commands() {
    let id = id();
    let project = "Synthetic project";
    for args in [
        vec!["field", "list", &id],
        vec!["field", "remove", &id, "Custom.Flag"],
        vec!["field", "definition", "list"],
    ] {
        assert!(matches!(
            parse(&args).command,
            AzureDevOpsWorkItemCommand::Field(_)
        ));
    }
    for args in [
        vec!["type", "list", "--project", project],
        vec!["type", "show", "Synthetic type", "--project", project],
        vec![
            "type",
            "field",
            "show",
            "System.Title",
            "--type",
            "Synthetic type",
            "--project",
            project,
        ],
    ] {
        assert!(matches!(
            parse(&args).command,
            AzureDevOpsWorkItemCommand::Type(_)
        ));
    }
    assert!(matches!(
        parse(&["field", "show", &id, "Custom.Flag"]).command,
        AzureDevOpsWorkItemCommand::Field(_)
    ));
    assert!(matches!(
        parse(&[
            "field",
            "set",
            &id,
            "Custom.Flag",
            "--value",
            "false",
            "--if-rev",
            "1"
        ])
        .command,
        AzureDevOpsWorkItemCommand::Field(_)
    ));
    assert!(matches!(
        parse(&["field", "definition", "show", "System.Title"]).command,
        AzureDevOpsWorkItemCommand::Field(_)
    ));
    assert!(matches!(
        parse(&[
            "type",
            "field",
            "list",
            "--type",
            "Synthetic type",
            "--project",
            project,
        ])
        .command,
        AzureDevOpsWorkItemCommand::Type(_)
    ));
}

#[test]
fn parses_relation_operations_and_create_inputs() {
    let id = id();
    let project = "Synthetic project";
    for args in [
        vec!["relation", "list", &id],
        vec!["relation", "type", "show", "System.LinkTypes.Related"],
    ] {
        assert!(matches!(
            parse(&args).command,
            AzureDevOpsWorkItemCommand::Relation(_)
        ));
    }
    assert!(matches!(
        parse(&["update", &id, "--patch", "[]"]).command,
        AzureDevOpsWorkItemCommand::Update(_)
    ));
    let target = id
        .parse::<i32>()
        .unwrap()
        .checked_add(1)
        .unwrap()
        .to_string();
    for verb in ["show", "create", "update", "remove"] {
        assert!(matches!(
            parse(&[
                "relation",
                verb,
                &id,
                "--type",
                "System.LinkTypes.Related",
                "--target",
                &target
            ])
            .command,
            AzureDevOpsWorkItemCommand::Relation(_)
        ));
    }
    assert!(matches!(
        parse(&["relation", "type", "list"]).command,
        AzureDevOpsWorkItemCommand::Relation(_)
    ));
    let parsed = parse(&[
        "create",
        "--type",
        "Synthetic type",
        "--title",
        "Fixture",
        "--parent",
        &id,
        "--project",
        project,
        "--validate-only",
    ]);
    let AzureDevOpsWorkItemCommand::Create(args) = parsed.command else {
        panic!("expected create");
    };
    assert!(args.parent.is_some() && args.validate_only);
}

#[test]
fn parses_moved_queries_and_copy_dry_run() {
    let query_id = uuid::Uuid::new_v4().to_string();
    let project = "Synthetic project";
    for args in [
        vec!["query", "list", "--project", project],
        vec!["query", "show", &query_id, "--project", project],
        vec![
            "query",
            "create",
            "--folder",
            &query_id,
            "--name",
            "Synthetic query",
            "--wiql",
            "SELECT [System.Id] FROM WorkItems",
            "--project",
            project,
        ],
    ] {
        assert!(matches!(
            parse(&args).command,
            AzureDevOpsWorkItemCommand::Query(_)
        ));
    }
    assert!(matches!(
        parse(&["query", "invoke", &query_id]).command,
        AzureDevOpsWorkItemCommand::Query(_)
    ));
    assert!(matches!(
        parse(&[
            "query",
            "invoke",
            "--wiql",
            "SELECT [System.Id] FROM WorkItems"
        ])
        .command,
        AzureDevOpsWorkItemCommand::Query(_)
    ));
    let id = id();
    assert!(figue::from_slice::<AzureDevOpsWorkItemArgs>(&["copy", &id, "--dry-run"]).is_err());
    let parsed = parse(&[
        "copy",
        &id,
        "--project",
        project,
        "--deep",
        "--allowed-ids",
        &id,
        "--dry-run",
    ]);
    let AzureDevOpsWorkItemCommand::Copy(args) = parsed.command else {
        panic!("expected copy");
    };
    assert!(args.deep && args.dry_run);
}

#[test]
fn azure_devops_dispatch_exposes_queries_only_under_work_item() {
    use crate::cli::azure_devops::AzureDevOpsArgs;
    use crate::cli::azure_devops::azure_devops_command_cli::AzureDevOpsCommand;
    let parsed: AzureDevOpsArgs = figue::from_slice(&[
        "work-item",
        "query",
        "list",
        "--project",
        "Synthetic project",
    ])
    .into_result()
    .expect("new path parses")
    .get();
    assert!(matches!(parsed.command, AzureDevOpsCommand::WorkItem(_)));
    assert!(figue::from_slice::<AzureDevOpsArgs>(&["query", "list"]).is_err());
}
