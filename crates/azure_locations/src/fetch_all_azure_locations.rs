use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use serde::Deserialize;

// pub async fn fetch_all_azure_locations() -> eyre::Result<Vec<String>> {
//     let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
//     cmd.args(["account", "list-locations"]);

//     #[derive(Deserialize)]
//     struct Location {
//         name: String,
//     }

//     #[derive(Deserialize)]
//     struct Resp {
//         value: Vec<Location>,
//     }

//     let resp = cmd.run::<Resp>().await?;
//     let locations = resp.value.into_iter().map(|loc| loc.name).collect();

//     Ok(locations)
// }
