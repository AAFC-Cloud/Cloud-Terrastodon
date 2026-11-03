#[cfg(windows)]
use crate::windows_support;
use eyre::Result;
use std::sync::Arc;
use std::sync::OnceLock;

static PANIC_HOOK: OnceLock<Arc<color_eyre::config::PanicHook>> = OnceLock::new();

/// Prepare process output before parsing can print help or errors and exit.
///
/// Enable virtual terminal processing for Windows stdout/stderr consoles and
/// install the eyre error hook. Redirected streams do not need a console mode.
/// [`App::run`](crate::App::run) calls this automatically. Call it before
/// [`App::parse_from`](crate::App::parse_from) when
/// implementing a process entrypoint that handles its own parse outcomes.
pub fn prepare_process() -> Result<()> {
    #[cfg(windows)]
    windows_support::enable_virtual_terminal_processing();
    install_error_hook()
}

/// Install the process-global eyre error hook before parsing arguments.
///
/// The panic hook is retained for installation after parsing determines the
/// debug setting. `HookBuilder::try_into_hooks` also initializes color-spantrace
/// globally, so the pair must be built only once even though installing the
/// eyre hook itself is safe to retry. Installation conflicts are reported to
/// stderr without aborting startup.
pub fn install_error_hook() -> Result<()> {
    panic_hook()?;
    Ok(())
}

fn panic_hook() -> Result<Arc<color_eyre::config::PanicHook>> {
    if let Some(panic_hook) = PANIC_HOOK.get() {
        return Ok(Arc::clone(panic_hook));
    }

    let (panic_hook, error_hook) = color_eyre::config::HookBuilder::default().try_into_hooks()?;
    if let Err(error) = error_hook.install() {
        eprintln!("Failed to install the eyre error hook: {error}");
    }
    let panic_hook = Arc::new(panic_hook);
    if PANIC_HOOK.set(Arc::clone(&panic_hook)).is_err() {
        eprintln!("Failed to retain the panic hook: another hook has already been initialized");
    }
    Ok(panic_hook)
}

pub(crate) fn install_panic_hook(debug: bool) -> Result<()> {
    let panic_hook = panic_hook()?;
    // `set_hook` replaces the process hook, so no separate installed guard is
    // needed. Reinstalling captures the debug setting for this parsed CLI.
    std::panic::set_hook(Box::new(move |info| {
        use std::io::Write;

        let mut stderr = std::io::stderr().lock();
        // A broken stderr must not cause another panic inside a panic hook.
        let _ = writeln!(stderr, "{}", panic_hook.panic_report(info));
        if debug {
            // Environment mutation is unsafe once any host thread exists.
            let _ = writeln!(
                stderr,
                "\nDebug backtrace:\n{}",
                std::backtrace::Backtrace::force_capture()
            );
        }
    }));
    Ok(())
}
