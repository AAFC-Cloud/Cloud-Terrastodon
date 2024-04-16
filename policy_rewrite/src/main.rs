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
    println!("Fetching management groups...");
    let management_groups = fetch_management_groups().await?;
    let options = PickOptions {
        choices: management_groups,
        many: true,
        prompt: Some("policy import > ".to_string()),
        header: Some("Management Groups".to_string()),
    };
    let management_groups = choose(options)?;

    let mut js = JoinSet::new();

    println!("Fetching policy definitions...");

    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("💤🎂");
    let m = MultiProgress::new();
    for mg in management_groups.iter() {
        let mg = mg.clone();
        let pb = m.add(ProgressBar::new(1));
        pb.set_style(spinner_style.clone());
        pb.set_message(mg.display_name.clone());
        js.spawn(async move {
            let result = fetch_policy_definitions(Some(mg.name.clone()), None).await;
            pb.inc(1);
            (mg, pb, result)
        });
    }
    while let Some(res) = js.join_next().await {
        let (mg, pb, policy_definitions) = res?;
        let policy_definitions = policy_definitions?;
        pb.finish_with_message(format!(
            "Found {} policy definitions for {}",
            policy_definitions.len(),
            mg.display_name
        ));
    }

    Ok(())
}
