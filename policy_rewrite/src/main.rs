use anyhow::Result;
use azure::prelude::fetch_management_groups;
use azure::prelude::fetch_policy_definitions;
use fzf::choose;
use fzf::PickOptions;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> Result<()> {
    // Fetch management groups
    println!("Fetching management groups...");
    let management_groups = fetch_management_groups().await?;

    // Pick management groups to import from
    let options = PickOptions {
        choices: management_groups,
        many: true,
        prompt: Some("policy import > ".to_string()),
        header: Some("Management Groups".to_string()),
    };
    let management_groups = choose(options)?;

    // Prepare progress indicators
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("💤🎂");
    let m = MultiProgress::new();

    // Prepare work pool
    let mut work_pool = JoinSet::new();

    // Fetch info from each management group
    println!("Fetching info from management groups...");
    for mg in management_groups.iter() {
        // Prepare progress indicator
        let mg = mg.clone();
        let pb = m.add(ProgressBar::new(1));
        pb.set_style(spinner_style.clone());
        pb.set_message(mg.display_name.clone());

        // Launch background worker
        work_pool.spawn(async move {
            // Fetch policy definitions
            let result = fetch_policy_definitions(Some(mg.name.clone()), None).await;

            // Update progress indicator
            pb.inc(1);

            // Return results
            (mg, pb, result)
        });
    }

    // Collect worker results
    while let Some(res) = work_pool.join_next().await {
        // Get result if worker success
        let (mg, pb, policy_definitions) = res?;

        // Get policy definitions if success
        let policy_definitions = policy_definitions?;

        // Update progress indicator
        pb.finish_with_message(format!(
            "Found {} policy definitions for {}",
            policy_definitions.len(),
            mg.display_name
        ));
    }

    Ok(())
}
