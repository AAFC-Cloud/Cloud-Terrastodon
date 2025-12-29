use cloud_terrastodon_azure::prelude::ResourceGraphHelper;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_pathing::Existy;
use cloud_terrastodon_user_input::PickerTui;
use serde_json::Value;
use tracing::debug;
use tracing::info;

pub async fn run_query_menu() -> eyre::Result<()> {
    let mut query = r#"
resources 
| union resourcecontainers
| project
    id,
    ['kind'] = type,
    name,
    display_name=properties.displayName,
    tags
| limit 25
    "#
    .trim()
    .to_owned();
    loop {
        debug!("Getting temp dir");
        let temp_dir = AppDir::Temp.as_path_buf();
        temp_dir.ensure_dir_exists().await?;

        debug!("Creating temp file");
        let query_path = tempfile::Builder::new()
            .suffix(".kql")
            .tempfile_in(&temp_dir)?
            .into_temp_path();

        debug!("Writing prompt for query into temp file");
        tokio::fs::write(&query_path, query).await?;

        info!("Getting query from user");
        let mut cmd = CommandBuilder::new(CommandKind::VSCode);
        cmd.arg(query_path.as_os_str());
        cmd.arg("--wait");
        cmd.run_raw().await?;
        query = tokio::fs::read_to_string(&query_path).await?;
        debug!("Received query:\n{}", query);

        info!("Running query");
        let rows: Vec<Value> = ResourceGraphHelper::new(query.clone(), CacheBehaviour::None)
            .collect_all()
            .await?;
        let rows_json = serde_json::to_string_pretty(&rows)?;

        debug!("Writing results to temp file");
        let results_path = tempfile::Builder::new()
            .suffix(".jsonc")
            .tempfile_in(temp_dir)?
            .into_temp_path();
        let results_string = format!("/*\n{query}\n*/\n{rows_json}");
        tokio::fs::write(&results_path, results_string).await?;

        info!("Displaying results");
        let mut cmd = CommandBuilder::new(CommandKind::VSCode);
        cmd.arg(results_path.as_os_str());
        cmd.run_raw().await?;

        let run_another_query = "run another query";
        let return_to_menu = "return to menu";
        let next = PickerTui::new().pick_one(vec![run_another_query, return_to_menu])?;
        match next {
            x if x == run_another_query => continue,
            x if x == return_to_menu => return Ok(()),
            _ => unreachable!(),
        }
    }
}
