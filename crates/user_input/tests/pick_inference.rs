use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;

#[ignore = "interactive test"]
#[tokio::test]
pub async fn it_works() -> eyre::Result<()> {
    let choices = [1, 2, 3];
    let _chosen = PickerTui::<i32>::new()
        .pick_one(choices.into_iter().map(|x| Choice {
            key: x.to_string(),
            value: x,
        }))
        .await?;
    Ok(())
}
