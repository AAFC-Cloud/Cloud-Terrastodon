use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Console::ENABLE_PROCESSED_OUTPUT;
use windows::Win32::System::Console::ENABLE_VIRTUAL_TERMINAL_PROCESSING;
use windows::Win32::System::Console::GetConsoleMode;
use windows::Win32::System::Console::GetStdHandle;
use windows::Win32::System::Console::STD_ERROR_HANDLE;
use windows::Win32::System::Console::STD_OUTPUT_HANDLE;
use windows::Win32::System::Console::SetConsoleMode;

/// Enable [virtual terminal sequences](https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences)
/// before help, version output, or parse errors can exit the process.
pub(crate) fn enable_virtual_terminal_processing() {
    // Enable colour on both human-output streams when attached to consoles;
    // redirected handles legitimately have no console mode.
    // SAFETY: Standard handles are borrowed, never closed; failed lookups or
    // console-mode queries are ignored. SetConsoleMode retains existing flags.
    unsafe {
        for stream in [STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
            if let Ok(handle) = GetStdHandle(stream)
                && handle != HANDLE::default()
            {
                let mut mode = Default::default();
                if GetConsoleMode(handle, &mut mode).is_ok() {
                    let _ = SetConsoleMode(
                        handle,
                        mode | ENABLE_PROCESSED_OUTPUT | ENABLE_VIRTUAL_TERMINAL_PROCESSING,
                    );
                }
            }
        }
    }
}

/// Emit the existing codepage warning once tracing is available.
pub(crate) fn warn_if_not_utf8() {
    // SAFETY: GetACP takes no arguments and returns the process ANSI codepage.
    unsafe {
        if windows::Win32::Globalization::GetACP() != 65001 {
            tracing::warn!(
                "The current system codepage is not UTF-8. This may cause '�' problems."
            );
            tracing::warn!(
                "See https://github.com/Azure/azure-cli/issues/22616#issuecomment-1147061949"
            );
            tracing::warn!(
                "Control panel -> Clock and Region -> Region and Language -> Administrative -> Change system locale -> Check Beta: Use Unicode UTF-8 for worldwide language support."
            );
        }
    }
}
