use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::Builder;
pub fn dump_to_ignore_file(content: &str) -> Result<PathBuf> {
    let dir_path = "ignore/";

    // Ensure the directory exists
    fs::create_dir_all(dir_path)?;

    // Create a temporary file within the specified directory
    let mut temp_file = Builder::new()
        .prefix("temp_") // Optional: Set a prefix for the file name
        .suffix(".json") // Optional: Set a suffix for the file name
        .tempfile_in(dir_path)?;

    // Write some data to the file
    temp_file.write_all(content.as_bytes())?;

    // Get the path
    let (_file, path) = temp_file.keep()?;

    // Open in editor
    Command::new("code.cmd").arg(path.clone()).spawn()?;

    // Return the path
    Ok(path.to_owned())
}
