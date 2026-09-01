#[cfg(target_os = "linux")]
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::process::Command;
#[cfg(target_os = "linux")]
use std::process::Stdio;

/// Open an HTTP URL in the user's default browser.
///
/// `webbrowser` normally bridges WSL to the Windows browser through executables
/// on `PATH`. When `[interop] appendWindowsPath = false`, discover the mounted
/// Windows system drive and invoke its built-in URL handler directly.
pub(crate) fn open_browser_url(url: &str) -> Result<(), String> {
    match webbrowser::open(url) {
        Ok(()) => Ok(()),
        Err(webbrowser_error) => {
            #[cfg(target_os = "linux")]
            if is_wsl() {
                return open_browser_with_windows_interop(url).map_err(|wsl_error| {
                    format!("{webbrowser_error}; Windows interop fallback failed: {wsl_error}")
                });
            }

            Err(webbrowser_error.to_string())
        }
    }
}

#[cfg(target_os = "linux")]
fn is_wsl() -> bool {
    std::env::var_os("WSL_INTEROP").is_some()
        || std::env::var_os("WSL_DISTRO_NAME").is_some()
        || std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .is_ok_and(|release| release.to_ascii_lowercase().contains("microsoft"))
}

#[cfg(target_os = "linux")]
fn open_browser_with_windows_interop(url: &str) -> Result<(), String> {
    let cmd_path = windows_cmd_path().ok_or_else(|| {
        "could not locate Windows System32/cmd.exe from the mounted C: drive".to_owned()
    })?;
    let escaped_url = escape_windows_cmd_argument(url);
    let status = Command::new(&cmd_path)
        .args(["/D", "/C", "start", "", escaped_url.as_str()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("failed to invoke {}: {error}", cmd_path.display()))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{} exited with status {status}",
            cmd_path.display()
        ))
    }
}

#[cfg(target_os = "linux")]
fn windows_cmd_path() -> Option<PathBuf> {
    let mounts = std::fs::read_to_string("/proc/mounts").ok()?;
    windows_cmd_path_from_mounts(&mounts).filter(|path| path.is_file())
}

#[cfg(target_os = "linux")]
fn windows_cmd_path_from_mounts(mounts: &str) -> Option<PathBuf> {
    mounts.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        let source = fields.next()?;
        let mount_point = fields.next()?;
        let filesystem = fields.next()?;
        if !source
            .get(..2)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("C:"))
            || !matches!(filesystem, "9p" | "drvfs")
        {
            return None;
        }
        Some(PathBuf::from(decode_mount_field(mount_point)).join("Windows/System32/cmd.exe"))
    })
}

#[cfg(target_os = "linux")]
fn decode_mount_field(value: &str) -> String {
    value
        .replace("\\040", " ")
        .replace("\\011", "\t")
        .replace("\\012", "\n")
        .replace("\\134", "\\")
}

#[cfg(target_os = "linux")]
fn escape_windows_cmd_argument(value: &str) -> String {
    value.replace('^', "^^").replace('&', "^&")
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn discovers_windows_cmd_when_windows_paths_are_not_in_linux_path() {
        let mounts = r#"
C:\134 /windrives/c 9p rw,noatime,aname=drvfs;path=C:\134 0 0
D:\134 /windrives/d 9p rw,noatime,aname=drvfs;path=D:\134 0 0
"#;
        assert_eq!(
            windows_cmd_path_from_mounts(mounts),
            Some(PathBuf::from("/windrives/c/Windows/System32/cmd.exe"))
        );
    }

    #[test]
    fn escapes_oauth_query_separators_for_windows_cmd() {
        assert_eq!(
            escape_windows_cmd_argument("https://example.test?a=one&b=two^three"),
            "https://example.test?a=one^&b=two^^three"
        );
    }
}
