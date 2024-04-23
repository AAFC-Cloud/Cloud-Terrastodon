use std::path::PathBuf;

use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use tf::prelude::process_to_files;
use tokio::fs::OpenOptions;
use tokio::fs::{self};
use tokio::io::AsyncWriteExt;
pub async fn process_generated() -> Result<()> {
    // Determine output directory
    let out_dir = PathBuf::from_iter(["ignore", "processed"]);

    // Cleanup
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).await?;
        fs::create_dir(&out_dir).await?;
    }

    // Read generated code
    let generated = fs::read(PathBuf::from_iter(["ignore", "imports", "generated.tf"])).await?;
    let generated_str = String::from_utf8(generated)?;
    let files = process_to_files(&generated_str, &out_dir)?;

    let mut error_count = 0;
    // Write the files
    for (path, content) in files {
        // Progress indicator
        // println!("Writing {} bytes to {:?}", content.len(), path);

        // Ensure parent dir exists
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent).await?;
        }

        // Open the file
        let mut file = match OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)
            .await
        {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Failed to open {:?}: {:?}", path, e);
                error_count += 1;
                continue;
            }
        };

        // Write the content
        file.write_all(content.as_bytes()).await?;
    }

    // Format the files
    CommandBuilder::new(CommandKind::TF)
        .should_announce(true)
        .use_run_dir(out_dir)
        .args(["fmt", "-recursive"])
        .run_raw()
        .await?;

    println!("Processing finished with {} problems.", error_count);

    Ok(())
}
