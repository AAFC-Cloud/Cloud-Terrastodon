#![feature(let_chains)]
use anyhow::Result;
use entrypoint::prelude::Version;
use tracing::info;

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
    // fix colours in the default exe terminal
    #[cfg(windows)]
    if let Err(e) = windows_ansi::enable_ansi_support() {
        eprintln!("Failed to enable terminal colours: {e:?}");
    };

    // start logging
    tracing_subscriber::fmt::init();
    info!("Ahoy!");

    // go to menu
    entrypoint::prelude::main(Version::new(env!("CARGO_PKG_VERSION").to_string())).await?;
    Ok(())
}
