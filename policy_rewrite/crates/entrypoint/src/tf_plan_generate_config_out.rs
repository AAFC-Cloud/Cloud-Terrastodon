use std::path::PathBuf;

use anyhow::Result;
use tf::prelude::*;
pub async fn run_tf_import() -> Result<()> {
    // not necessary if capturing terraform output
    // // Double check that we are logged in before running tf command
    // // Previous commands may have used cached results
    // // Capturing tf output while also sending to console to detect
    // // login failures for auto-retry is not yet implemented
    // if !is_logged_in().await {
    //     println!("You aren't logged in! Running login command...");
    //     login().await?;
    // }

    // Run tf import
    println!("Beginning tf import...");
    let imports_dir = PathBuf::from("ignore").join("imports");
    TFImporter::default().using_dir(imports_dir).run().await?;

    Ok(())
}
