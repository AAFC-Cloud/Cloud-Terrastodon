use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Context;
use eyre::bail;
use std::collections::HashSet;
use std::path::Path;
use tracing::debug;
use tracing::info;

/// Kill any processes whose exe path is in the given dirs or a child of one.
/// Fails if any of the given dirs do not exist.
pub fn prompt_kill_processes_using_dirs(
    dirs: impl IntoIterator<Item = impl AsRef<Path>>,
    header: String,
) -> eyre::Result<()> {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();

    let mut parents = HashSet::new();
    for dir in dirs {
        let dir = dir.as_ref().to_path_buf();
        let dir = dir
            .canonicalize()
            .wrap_err(format!("Failed to canonicalize given dir {dir:?}"))?;
        parents.insert(dir);
    }

    // Iterate over all processes and find those matching the pattern
    let mut to_kill = Vec::new();
    for (pid, process) in system.processes() {
        let Some(exe_path) = process.exe() else {
            continue;
        };
        let Some(exe_dir) = exe_path.parent() else {
            continue;
        };
        let exe_dir = exe_dir
            .canonicalize()
            .wrap_err(format!("Failed to canonicalize exe dir {exe_dir:?}"))?;
        let is_in_dir = exe_dir
            .ancestors()
            .any(|ancestor| parents.contains(ancestor));
        if is_in_dir {
            to_kill.push(Choice {
                key: format!(
                    "ID: {}, Name: {}, Path: {:?}",
                    pid,
                    process.name().to_string_lossy(),
                    process.exe()
                ),
                value: (pid, process),
            });
        }
    }
    if to_kill.is_empty() {
        return Ok(());
    }
    let to_kill = PickerTui::new(to_kill).set_header(header).pick_many()?;
    for Choice {
        key,
        value: (_pid, process),
    } in to_kill
    {
        debug!("Killing: {key}");
        let signal_sent_success = process.kill();
        if !signal_sent_success {
            bail!("Failed to send kill signal to {key}");
        }
        let exit_status = process.wait();
        info!("Killed  {key} with exit status: {exit_status:#?}");
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::prompt_kill_processes_using_dirs;
    use cloud_terrastodon_pathing::AppDir;

    #[test]
    fn main() {
        // Creating a System instance
        let mut system = sysinfo::System::new_all();

        // Refresh system processes information
        system.refresh_all();

        // Define the pattern to search for
        let pattern = "terraform-provider";

        // Iterate over all processes and find those matching the pattern
        for (pid, process) in system.processes() {
            let name = process.name().to_string_lossy();
            if !name.contains(pattern) {
                continue;
            }
            println!("ID: {}, Name: {}, Path: {:?}", pid, name, process.exe());
        }
    }
    #[test]
    #[ignore]
    fn kill() {
        prompt_kill_processes_using_dirs(
            [
                AppDir::Imports.as_path_buf(),
                AppDir::Processed.as_path_buf(),
            ],
            "Found the following processes, select the ones you want to kill".to_string(),
        )
        .unwrap();
    }
}
