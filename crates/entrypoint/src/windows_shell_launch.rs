use std::os::windows::io::AsRawHandle;
use std::os::windows::io::FromRawHandle;
use std::os::windows::io::OwnedHandle;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot;
use windows::Win32::System::Diagnostics::ToolHelp::PROCESSENTRY32W;
use windows::Win32::System::Diagnostics::ToolHelp::Process32FirstW;
use windows::Win32::System::Diagnostics::ToolHelp::Process32NextW;
use windows::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPPROCESS;

/// Explorer starts console executables with a console of their own, so terminal
/// detection alone cannot tell a double-click apart from a shell invocation.
pub(crate) fn launched_from_explorer() -> bool {
    // SAFETY: TH32CS_SNAPPROCESS takes a system-wide snapshot and ignores the PID.
    let Ok(snapshot) = (unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }) else {
        return false;
    };
    // SAFETY: The successful call returned a newly owned handle closed by OwnedHandle.
    let snapshot = unsafe { OwnedHandle::from_raw_handle(snapshot.0) };
    let Some(current) = process_entry(&snapshot, std::process::id()) else {
        return false;
    };
    let Some(parent) = process_entry(&snapshot, current.th32ParentProcessID) else {
        return false;
    };
    let name = &parent.szExeFile;
    let end = name
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(name.len());
    String::from_utf16_lossy(&name[..end]).eq_ignore_ascii_case("explorer.exe")
}

fn process_entry(snapshot: &OwnedHandle, process_id: u32) -> Option<PROCESSENTRY32W> {
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let snapshot = HANDLE(snapshot.as_raw_handle());
    // SAFETY: The snapshot stays open while borrowed, and entry is a writable,
    // initialized PROCESSENTRY32W with its size set for Tool Help.
    unsafe {
        Process32FirstW(snapshot, &mut entry).ok()?;
        while entry.th32ProcessID != process_id {
            Process32NextW(snapshot, &mut entry).ok()?;
        }
    }
    Some(entry)
}
