use cloud_terrastodon_user_input::PickerTui;

pub fn main() -> eyre::Result<()> {
    let choices = vec![
        "First\nSecond\nThird",
        "A\nB\nC",
        "IMPORT BRUH\nDO THING\nWOOHOO!",
        "single item",
        "another single item",
    ];
    let chosen = PickerTui::new(choices).pick_many()?;
    println!("You chose: {chosen:#?}");
    Ok(())
}
