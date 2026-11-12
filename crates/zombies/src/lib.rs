use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Context;
use eyre::bail;
use std::collections::HashSet;
use std::path::Path;
use teamy_cancellation::CancellationScope;
use tracing::debug;
use tracing::info;

/// Kill any processes whose exe path is in the given dirs or a child of one.
/// Fails if any of the given dirs do not exist.
pub async fn prompt_kill_processes_using_dirs(
    dirs: impl IntoIterator<Item = impl AsRef<Path>>,
    header: String,
    cancellation: &CancellationScope,
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
            let key = format!(
                "ID: {}, Name: {}, Path: {:?}",
                pid,
                process.name().to_string_lossy(),
                process.exe()
            );
            to_kill.push(Choice {
                key: key.clone(),
                value: (key, *pid, process),
            });
        }
    }
    if to_kill.is_empty() {
        return Ok(());
    }
    let selected_to_kill = PickerTui::<(String, sysinfo::Pid, &sysinfo::Process)>::new()
        .set_header(header)
        .pick_many(to_kill, cancellation)
        .await?;
    for (key, pid, process) in selected_to_kill {
        debug!(
            "Killing ID: {pid}, Name: {name}, Path: {path:?}",
            name = process.name().to_string_lossy(),
            path = process.exe()
        );
        let signal_sent_success = process.kill();
        if !signal_sent_success {
            bail!("Failed to send kill signal to {key}");
        }
        let exit_status = process.wait();
        info!("Killed  {key} with exit status: {exit_status:#?}");
    }

    Ok(())
}
