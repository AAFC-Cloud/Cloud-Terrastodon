#![feature(let_chains)]
use anyhow::Result;
use cloud_terrastodon_core_entrypoint::prelude::main as entrypoint_main;
use cloud_terrastodon_core_entrypoint::prelude::Version;

#[cfg(windows)]
mod windows_ansi {
    use windows::core::Result;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Console::GetConsoleMode;
    use windows::Win32::System::Console::GetStdHandle;
    use windows::Win32::System::Console::SetConsoleMode;
    use windows::Win32::System::Console::ENABLE_VIRTUAL_TERMINAL_PROCESSING;
    use windows::Win32::System::Console::STD_OUTPUT_HANDLE;

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

#[tokio::main]
async fn main() -> Result<()> {
    // start logging
    tracing_subscriber::fmt::init();

    // fix colours in the default exe terminal
    // show no errors when colours unavailable (piping situations)
    #[cfg(windows)]
    let _ = windows_ansi::enable_ansi_support();

    // go to menu
    entrypoint_main(Version::new(env!("CARGO_PKG_VERSION").to_string())).await?;
    Ok(())
}
