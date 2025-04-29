use cloud_terrastodon_user_input::pick_many;

pub fn main() -> eyre::Result<()> {
    let choices = vec![
        "First\nSecond\nThird",
        "A\nB\nC",
        "IMPORT BRUH\nDO THING\nWOOHOO!",
        "single item",
        "another single item",
    ];
    let chosen = pick_many(choices)?;
    println!("You chose: {chosen:#?}");
    Ok(())
}
