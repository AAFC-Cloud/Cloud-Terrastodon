use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Console::{
    ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, GetStdHandle, STD_ERROR_HANDLE,
    STD_OUTPUT_HANDLE, SetConsoleMode,
};

pub(crate) fn initialize() {
    // Enable colour on both human-output streams when attached to consoles;
    // redirected handles legitimately have no console mode.
    unsafe {
        for stream in [STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
            if let Ok(handle) = GetStdHandle(stream)
                && handle != HANDLE::default()
            {
                let mut mode = Default::default();
                if GetConsoleMode(handle, &mut mode).is_ok() {
                    let _ = SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
                }
            }
        }
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
