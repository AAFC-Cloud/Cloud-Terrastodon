use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;

#[test]
#[ignore = "interactive test"]
pub fn it_works() -> eyre::Result<()> {
    let choices = [1, 2, 3];
    let _chosen = PickerTui::from(choices.into_iter().map(|x| Choice {
        key: x.to_string(),
        value: x,
    }))
    .pick_one()?;
    Ok(())
}
