use std::path::PathBuf;

use anyhow::Result;
use tofu::prelude::TofuImporter;
use tracing::{info, warn};
pub async fn perform_import() -> Result<()> {
    // not necessary if capturing terraform output
    // // Double check that we are logged in before running tf command
    // // Previous commands may have used cached results
    // // Capturing tf output while also sending to console to detect
    // // login failures for auto-retry is not yet implemented
    // if !is_logged_in().await {
    //     info!("You aren't logged in! Running login command...");
    //     login().await?;
    // }

    // Run tf import
    info!("Beginning tofu import...");
    let imports_dir = PathBuf::from("ignore").join("imports");
    match TofuImporter::default().using_dir(imports_dir).run().await {
        Ok(_) => info!("Import success!"),
        Err(_) => warn!("Import finished with problems, generated code will be fixed in processing step.")
    };

    Ok(())
}
