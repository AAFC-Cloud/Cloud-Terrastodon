use eframe::CreationContext;
use eyre::Result;
use eyre::WrapErr;
use eyre::bail;
use raw_window_handle::HasWindowHandle;
use raw_window_handle::RawWindowHandle;
use std::io;
use std::ptr;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::ICON_BIG;
use windows_sys::Win32::UI::WindowsAndMessaging::ICON_SMALL;
use windows_sys::Win32::UI::WindowsAndMessaging::IMAGE_ICON;
use windows_sys::Win32::UI::WindowsAndMessaging::LR_DEFAULTSIZE;
use windows_sys::Win32::UI::WindowsAndMessaging::LR_SHARED;
use windows_sys::Win32::UI::WindowsAndMessaging::LoadImageW;
use windows_sys::Win32::UI::WindowsAndMessaging::SendMessageW;
use windows_sys::Win32::UI::WindowsAndMessaging::WM_SETICON;
use windows_sys::w;

/// Use the hosting executable's `app_icon` resource for its native GUI window.
///
/// `resources/app.rc` already embeds the image for the Cloud Terrastodon binary;
/// the Windows resource compiler names it `APP_ICON`. Apply it after eframe
/// creates the window, replacing the framework icon without embedding or decoding
/// another copy. Other executable hosts must provide the same named resource.
///
/// The shared resource handle remains valid for the executable's lifetime and
/// must not be destroyed by this code. See [LoadImageW] and [window icons].
///
/// [LoadImageW]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-loadimagew
/// [window icons]: https://learn.microsoft.com/en-us/windows/win32/menurc/about-icons
pub(crate) fn apply(context: &CreationContext<'_>) -> Result<()> {
    let window = context
        .window_handle()
        .wrap_err("Obtaining the GUI window handle for its bundled icon")?;
    let RawWindowHandle::Win32(window) = window.as_raw() else {
        bail!("The GUI did not provide a Windows window handle");
    };

    // SAFETY: a null module name returns the current executable's module, which
    // remains loaded for the lifetime of its windows. This borrows that module;
    // GetModuleHandleW does not add a reference that callers should release.
    let module = unsafe { GetModuleHandleW(ptr::null()) };
    if module.is_null() {
        return Err(io::Error::last_os_error())
            .wrap_err("Obtaining the executable module for its bundled icon");
    }

    // SAFETY: the module is live and the resource name is static UTF-16 text.
    // LR_SHARED makes Windows own the icon handle, including after this call.
    let icon = unsafe {
        LoadImageW(
            module,
            w!("APP_ICON"),
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE | LR_SHARED,
        )
    };
    if icon.is_null() {
        return Err(io::Error::last_os_error())
            .wrap_err("Loading the executable's bundled APP_ICON resource");
    }

    // SAFETY: CreationContext lends this live HWND on the GUI thread. WM_SETICON
    // borrows the shared handle; Windows scales it for the caption and taskbar.
    // The returned previous icons remain owned by eframe/winit, so do not destroy
    // them. A null previous icon is a valid result, not an error.
    unsafe {
        SendMessageW(
            window.hwnd.get() as HWND,
            WM_SETICON,
            ICON_BIG as usize,
            icon as isize,
        );
        SendMessageW(
            window.hwnd.get() as HWND,
            WM_SETICON,
            ICON_SMALL as usize,
            icon as isize,
        );
    }
    Ok(())
}
