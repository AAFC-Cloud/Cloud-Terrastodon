use anyhow::Result;
use azure::prelude::fetch_my_role_eligibility_schedules;
use fzf::pick_many;
use fzf::Choice;
use fzf::FzfArgs;
use itertools::Itertools;
use tracing::info;
pub async fn pim_activate() -> Result<()> {
    let eligibile = fetch_my_role_eligibility_schedules().await?;
    let chosen = pick_many(FzfArgs {
        choices: eligibile
            .into_iter()
            .map(|x| Choice {
                display: x.to_string(),
                inner: x,
            })
            .collect_vec(),
        prompt: None,
        header: Some("Choose roles to activate".to_string()),
    })?;

    for x in chosen {
        info!("Activating {x}");
    }
    Ok(())
}
