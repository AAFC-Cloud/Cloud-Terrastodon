use crate::prelude::fetch_all_policy_assignments;
use anyhow::Result;
use azure_types::prelude::DistinctByScope;
use azure_types::prelude::PolicyAssignment;
use azure_types::prelude::Scope;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use fzf::pick;
use fzf::Choice;
use fzf::FzfArgs;
use indoc::formatdoc;
use pathing_types::Existy;
use pathing_types::IgnoreDir;
use std::ffi::OsString;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tracing::info;

pub async fn evaluate_policy_assignment_compliance() -> Result<()> {
    info!("Fetching policy assignments");
    let policy_assignments = fetch_all_policy_assignments().await?;

    info!("Building policy choice list");
    let choices = policy_assignments
        .into_values()
        .flatten()
        .distinct_by_scope()
        .map(|ass| Choice::<PolicyAssignment> {
            display: format!("{} {:?}", ass.name, ass.display_name),
            inner: ass,
        })
        .collect();

    info!("Prompting for user choice");
    let Choice {
        inner: policy_assignment,
        ..
    } = pick(FzfArgs {
        choices,
        prompt: None,
        header: Some("Choose policy to evaluate".to_string()),
    })?;

    info!("You chose: {:?}", policy_assignment.id);

    let query = formatdoc! {
        r#"
            policyResources 
                | where type =~ 'Microsoft.PolicyInsights/PolicyStates'
                | where properties.policyAssignmentId =~ "{}"
                | where properties.complianceState =~ "noncompliant"
                | summarize count() by tostring(properties.policyDefinitionReferenceId)
                | order by count_ desc
        "#,
        policy_assignment.id.expanded_form()
    };

    // Write the query to disk so we can use with az cli @<file> convention
    // https://github.com/Azure/azure-cli/blob/dev/doc/quoting-issues-with-powershell.md#best-practice-use-file-input-for-json
    // we could also use "@-" for stdin but I didn't think of that till now

    // // Acquire temp file path
    // let temp_dir = IgnoreDir::Temp.as_path_buf();
    // temp_dir.ensure_dir_exists().await?;
    // let query_file_path = tempfile::Builder::new()
    //     .prefix("policy_assignment_compliance_graph_query_")
    //     .suffix(".kql")
    //     .tempfile_in(temp_dir)?.into_temp_path();

    // // Write content async
    // let mut query_file = tokio::fs::OpenOptions::new().write(true).open(&query_file_path).await?;
    // query_file
    //     .write_all(query.as_bytes())
    //     .await?;

    // // Build command
    // let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    // cmd.args(["graph", "query", "--graph-query"]);
    // let mut arg = OsString::new();
    // arg.push("@");
    // arg.push(query_file_path.canonicalize()?);
    // cmd.arg(arg);

    // // Clean up temp file
    // drop(query_file_path);

    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["graph", "query", "--graph-query"]);
    cmd.azure_arg("query.kql", query);
    cmd.use_cache_dir(PathBuf::from_iter([
        "az graph query",
        format!(
            "--graph-query policy-compliance-for-{}",
            policy_assignment.name
        )
        .as_str(),
    ]));
    // Run command
    let results = cmd.run_raw().await?;

    Ok(())
}
