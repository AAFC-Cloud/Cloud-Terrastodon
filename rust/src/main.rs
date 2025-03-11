#![feature(let_chains)]
use cloud_terrastodon_core_entrypoint::prelude::Version;
use cloud_terrastodon_core_entrypoint::prelude::main as entrypoint_main;
use eyre::Result;
use itertools::Itertools;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[cfg(windows)]
mod windows_ansi {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Console::ENABLE_VIRTUAL_TERMINAL_PROCESSING;
    use windows::Win32::System::Console::GetConsoleMode;
    use windows::Win32::System::Console::GetStdHandle;
    use windows::Win32::System::Console::STD_OUTPUT_HANDLE;
    use windows::Win32::System::Console::SetConsoleMode;
    use windows::core::Result;

    pub fn enable_ansi_support() -> Result<()> {
        unsafe {
            let handle = GetStdHandle(STD_OUTPUT_HANDLE)?;
            if handle == HANDLE::default() {
                return Err(windows::core::Error::from_win32());
            }

            let mut mode = std::mem::zeroed();
            GetConsoleMode(handle, &mut mode)?;
            SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING)?;
            Ok(())
        }
    }
}

#[cfg(windows)]
mod windows_utf8 {
    use windows::Win32::Globalization::GetACP;
    pub fn is_system_utf8() -> bool {
        unsafe { GetACP() == 65001 }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    // start logging

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        .add_directive(
            format!(
                "
                {}=info
                ",
                env!("CARGO_PKG_NAME")
            )
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.starts_with("//"))
            .filter(|line| !line.is_empty())
            .join(",")
            .trim()
            .parse()
            .unwrap(),
        );
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .without_time()
        .init();

    // fix colours in the default exe terminal
    // show no errors when colours unavailable (piping situations)
    #[cfg(windows)]
    let _ = windows_ansi::enable_ansi_support();

    #[cfg(windows)]
    if !windows_utf8::is_system_utf8() {
        tracing::warn!("The current system codepage is not UTF-8. This may cause '�' problems.");
        tracing::warn!("See https://github.com/Azure/azure-cli/issues/22616#issuecomment-1147061949");
        tracing::warn!("Control panel -> Clock and Region -> Region -> Administrative -> Change system locale -> Check Beta: Use Unicode UTF-8 for worldwide language support.");
    }

    // go to menu
    entrypoint_main(Version::new(env!("CARGO_PKG_VERSION").to_string())).await?;
    Ok(())
}
