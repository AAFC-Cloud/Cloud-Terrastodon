use eyre::Result;
use eyre::WrapErr;
use std::io;
#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
#[cfg(windows)]
use std::os::windows::io::FromRawHandle;
#[cfg(windows)]
use std::os::windows::io::OwnedHandle;
use std::process::ExitStatus;
use tokio::process::Child;
use tokio::process::ChildStderr;
use tokio::process::Command;
#[cfg(windows)]
use windows::Win32::Foundation::HANDLE;
#[cfg(windows)]
use windows::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot;
#[cfg(windows)]
use windows::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPTHREAD;
#[cfg(windows)]
use windows::Win32::System::Diagnostics::ToolHelp::THREADENTRY32;
#[cfg(windows)]
use windows::Win32::System::Diagnostics::ToolHelp::Thread32First;
#[cfg(windows)]
use windows::Win32::System::Diagnostics::ToolHelp::Thread32Next;
#[cfg(windows)]
use windows::Win32::System::JobObjects::AssignProcessToJobObject;
#[cfg(windows)]
use windows::Win32::System::JobObjects::CreateJobObjectW;
#[cfg(windows)]
use windows::Win32::System::JobObjects::JOB_OBJECT_LIMIT;
#[cfg(windows)]
use windows::Win32::System::JobObjects::JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
#[cfg(windows)]
use windows::Win32::System::JobObjects::JOBOBJECT_BASIC_ACCOUNTING_INFORMATION;
#[cfg(windows)]
use windows::Win32::System::JobObjects::JOBOBJECT_EXTENDED_LIMIT_INFORMATION;
#[cfg(windows)]
use windows::Win32::System::JobObjects::JobObjectBasicAccountingInformation;
#[cfg(windows)]
use windows::Win32::System::JobObjects::JobObjectExtendedLimitInformation;
#[cfg(windows)]
use windows::Win32::System::JobObjects::QueryInformationJobObject;
#[cfg(windows)]
use windows::Win32::System::JobObjects::SetInformationJobObject;
#[cfg(windows)]
use windows::Win32::System::JobObjects::TerminateJobObject;
#[cfg(windows)]
use windows::Win32::System::Threading::CREATE_SUSPENDED;
#[cfg(windows)]
use windows::Win32::System::Threading::OpenThread;
#[cfg(windows)]
use windows::Win32::System::Threading::ResumeThread;
#[cfg(windows)]
use windows::Win32::System::Threading::THREAD_SUSPEND_RESUME;
#[cfg(windows)]
use windows::core::PCWSTR;

/// Owns the private Azure CLI sign-in launcher and its credential-writing descendants.
pub(crate) struct AzureCliSignInProcess {
    child: Child,
    #[cfg(windows)]
    job: OwnedHandle,
    #[cfg(unix)]
    process_group: Option<libc::pid_t>,
}

impl AzureCliSignInProcess {
    pub(crate) fn spawn(command: &mut Command) -> Result<Self> {
        command.kill_on_drop(true);
        #[cfg(windows)]
        {
            // Start suspended so az.cmd cannot launch Python until both processes
            // are guaranteed to belong to our job through inheritance.
            let job = Self::windows_job()?;
            let child = command
                .creation_flags(CREATE_SUSPENDED.0)
                .spawn()
                .wrap_err("starting isolated Azure CLI sign-in")?;
            let process = HANDLE(
                child
                    .raw_handle()
                    .ok_or_else(|| eyre::eyre!("the Azure CLI sign-in launcher has no handle"))?,
            );
            // SAFETY: both handles are live. The child cannot execute before the resume call.
            unsafe { AssignProcessToJobObject(HANDLE(job.as_raw_handle()), process) }
                .wrap_err("isolating the Azure CLI sign-in process tree")?;
            Self::resume_primary_thread(
                child.id().ok_or_else(|| {
                    eyre::eyre!("the Azure CLI sign-in launcher has no process ID")
                })?,
            )?;
            Ok(Self { child, job })
        }
        #[cfg(unix)]
        {
            let child = command
                .process_group(0)
                .spawn()
                .wrap_err("starting isolated Azure CLI sign-in")?;
            let process_group =
                libc::pid_t::try_from(child.id().ok_or_else(|| {
                    eyre::eyre!("the Azure CLI sign-in launcher has no process ID")
                })?)?;
            Ok(Self {
                child,
                process_group: Some(process_group),
            })
        }
    }

    pub(crate) fn take_stderr(&mut self) -> Result<ChildStderr> {
        self.child.stderr.take().ok_or_else(|| {
            eyre::eyre!("the Azure CLI sign-in process did not expose its prompt pipe")
        })
    }

    pub(crate) async fn wait(&mut self) -> Result<ExitStatus> {
        let status = self
            .child
            .wait()
            .await
            .wrap_err("waiting for isolated Azure CLI sign-in")?;
        #[cfg(unix)]
        {
            self.process_group = None;
        }
        #[cfg(windows)]
        // If the CLI opened the browser in our job, allow it to remain open
        // after normal completion. Cancellation terminates the whole job.
        Self::set_job_limits(&self.job, JOB_OBJECT_LIMIT(0))?;
        Ok(status)
    }

    pub(crate) async fn terminate(&mut self) -> Result<()> {
        #[cfg(windows)]
        {
            // SAFETY: this job is owned by us and contains only the managed tree.
            unsafe { TerminateJobObject(HANDLE(self.job.as_raw_handle()), 1) }
                .wrap_err("terminating the Azure CLI sign-in process tree")?;
            // Termination is asynchronous. Preserve the generation lease until
            // every process in the job has stopped accessing the directory.
            loop {
                let mut accounting = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
                // SAFETY: the live job handle and output structure match the class and length.
                unsafe {
                    QueryInformationJobObject(
                        Some(HANDLE(self.job.as_raw_handle())),
                        JobObjectBasicAccountingInformation,
                        (&raw mut accounting).cast(),
                        size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                        None,
                    )
                }
                .wrap_err("waiting for the Azure CLI sign-in process tree to stop")?;
                if accounting.ActiveProcesses == 0 {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
        #[cfg(unix)]
        self.kill_process_group()?;
        self.child
            .wait()
            .await
            .wrap_err("reaping the cancelled Azure CLI sign-in launcher")?;
        #[cfg(unix)]
        {
            self.process_group = None;
        }
        Ok(())
    }

    #[cfg(windows)]
    fn windows_job() -> Result<OwnedHandle> {
        // SAFETY: no attributes or name are needed for this private job object.
        let handle = unsafe { CreateJobObjectW(None, PCWSTR::null()) }
            .wrap_err("creating a job for isolated Azure CLI sign-in")?;
        // SAFETY: CreateJobObjectW returned a handle owned by this call.
        let handle = unsafe { OwnedHandle::from_raw_handle(handle.0) };
        Self::set_job_limits(&handle, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE)?;
        Ok(handle)
    }

    #[cfg(windows)]
    fn set_job_limits(handle: &OwnedHandle, flags: JOB_OBJECT_LIMIT) -> Result<()> {
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = flags;
        // SAFETY: the job and structure match the specified information class.
        unsafe {
            SetInformationJobObject(
                HANDLE(handle.as_raw_handle()),
                JobObjectExtendedLimitInformation,
                (&raw const limits).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        }
        .wrap_err("configuring the Azure CLI sign-in job")
    }

    #[cfg(windows)]
    fn resume_primary_thread(pid: u32) -> Result<()> {
        // A process started suspended has only its primary thread. Stable Rust
        // child handles expose the process, so recover that thread by process ID.
        // SAFETY: a thread snapshot does not dereference caller-owned pointers.
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) }
            .wrap_err("locating the suspended Azure CLI sign-in thread")?;
        // SAFETY: CreateToolhelp32Snapshot returned an owned handle.
        let snapshot = unsafe { OwnedHandle::from_raw_handle(snapshot.0) };
        let snapshot_handle = HANDLE(snapshot.as_raw_handle());
        let mut entry = THREADENTRY32 {
            dwSize: size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };
        // SAFETY: the entry is initialized to the documented structure size.
        unsafe { Thread32First(snapshot_handle, &raw mut entry) }
            .wrap_err("reading the Azure CLI sign-in thread snapshot")?;
        while entry.th32OwnerProcessID != pid {
            entry.dwSize = size_of::<THREADENTRY32>() as u32;
            // SAFETY: snapshot is open and the entry is correctly sized and writable.
            unsafe { Thread32Next(snapshot_handle, &raw mut entry) }
                .wrap_err("finding the primary thread of the suspended Azure CLI sign-in")?;
        }
        // SAFETY: use only the suspension right for the thread found in this process.
        let thread = unsafe { OpenThread(THREAD_SUSPEND_RESUME, false, entry.th32ThreadID) }
            .wrap_err("opening the suspended Azure CLI sign-in thread")?;
        // SAFETY: OpenThread returned an owned handle.
        let thread = unsafe { OwnedHandle::from_raw_handle(thread.0) };
        // SAFETY: the isolated child is already in the job and the thread handle is live.
        if unsafe { ResumeThread(HANDLE(thread.as_raw_handle())) } == u32::MAX {
            return Err(io::Error::last_os_error()).wrap_err("resuming Azure CLI sign-in");
        }
        Ok(())
    }

    #[cfg(unix)]
    fn kill_process_group(&self) -> Result<()> {
        let Some(process_group) = self.process_group else {
            return Ok(());
        };
        // SAFETY: we created this private group and have not yet reaped its
        // leader. A positive ID to killpg targets only this group.
        if unsafe { libc::killpg(process_group, libc::SIGKILL) } == -1 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(error).wrap_err("terminating the Azure CLI sign-in process group");
            }
        }
        Ok(())
    }
}

#[cfg(unix)]
impl Drop for AzureCliSignInProcess {
    fn drop(&mut self) {
        if let Err(error) = self.kill_process_group() {
            tracing::warn!(%error, "Could not terminate the dropped Azure CLI sign-in process group");
        }
    }
}
