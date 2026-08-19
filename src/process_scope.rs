use super::Result;
use std::io;
use std::process::{Child, ChildStderr, ChildStdout, Command, ExitStatus};
use std::thread;
use std::time::{Duration, Instant};

const MAX_CLEANUP_RESERVE: Duration = Duration::from_secs(2);
const POLL_INTERVAL: Duration = Duration::from_millis(10);

pub(super) fn operation_deadlines(started: Instant, total_timeout: Duration) -> (Instant, Instant) {
    let cleanup_reserve = std::cmp::min(MAX_CLEANUP_RESERVE, total_timeout / 4);
    (
        started + total_timeout.saturating_sub(cleanup_reserve),
        started + total_timeout,
    )
}

pub(super) struct OwnedProcess {
    child: Child,
    #[cfg(unix)]
    process_group_id: libc::pid_t,
    #[cfg(windows)]
    job: WindowsJob,
}

impl OwnedProcess {
    pub(super) fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.child.stdout.take()
    }

    pub(super) fn take_stderr(&mut self) -> Option<ChildStderr> {
        self.child.stderr.take()
    }

    pub(super) fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }

    pub(super) fn wait_for_scope_quiescence(
        &mut self,
        deadline: Instant,
        label: &str,
    ) -> Result<bool> {
        loop {
            if self.scope_is_quiescent(label)? {
                return Ok(true);
            }
            let now = Instant::now();
            if now >= deadline {
                return Ok(false);
            }
            thread::sleep(POLL_INTERVAL.min(deadline.saturating_duration_since(now)));
        }
    }

    pub(super) fn terminate_and_prove(&mut self, deadline: Instant, label: &str) -> Result<()> {
        self.terminate_scope(label)?;
        loop {
            let direct_exited = self
                .child
                .try_wait()
                .map_err(|error| {
                    format!("{label} failed while reaping its owned direct child: {error}")
                })?
                .is_some();
            let scope_quiescent = self.scope_is_quiescent(label)?;
            if direct_exited && scope_quiescent {
                return Ok(());
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(format!(
                    "{label} owned process scope could not be proven terminated inside the bounded cleanup window"
                )
                .into());
            }
            thread::sleep(POLL_INTERVAL.min(deadline.saturating_duration_since(now)));
        }
    }

    #[cfg(unix)]
    fn terminate_scope(&mut self, label: &str) -> Result<()> {
        let result = unsafe { libc::kill(-self.process_group_id, libc::SIGKILL) };
        if result == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ESRCH) {
            Ok(())
        } else {
            Err(format!("{label} failed to terminate its owned process group: {error}").into())
        }
    }

    #[cfg(windows)]
    fn terminate_scope(&mut self, label: &str) -> Result<()> {
        self.job.terminate(label)
    }

    #[cfg(not(any(unix, windows)))]
    fn terminate_scope(&mut self, label: &str) -> Result<()> {
        match self.child.kill() {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::InvalidInput => Ok(()),
            Err(error) => {
                Err(format!("{label} failed to terminate its owned child: {error}").into())
            }
        }
    }

    #[cfg(unix)]
    fn scope_is_quiescent(&self, label: &str) -> Result<bool> {
        let result = unsafe { libc::kill(-self.process_group_id, 0) };
        if result == 0 {
            return Ok(false);
        }
        let error = io::Error::last_os_error();
        match error.raw_os_error() {
            Some(libc::ESRCH) => Ok(true),
            Some(libc::EPERM) => Ok(false),
            _ => Err(format!("{label} could not inspect its owned process group: {error}").into()),
        }
    }

    #[cfg(windows)]
    fn scope_is_quiescent(&self, label: &str) -> Result<bool> {
        Ok(self.job.active_processes(label)? == 0)
    }

    #[cfg(not(any(unix, windows)))]
    fn scope_is_quiescent(&self, _label: &str) -> Result<bool> {
        Ok(false)
    }
}

impl Drop for OwnedProcess {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            // Always terminate the owned process group on fallback Drop. The direct
            // child may already be reaped while a descendant remains in the group.
            let _ = unsafe { libc::kill(-self.process_group_id, libc::SIGKILL) };
        }
        #[cfg(not(any(unix, windows)))]
        {
            if self.child.try_wait().ok().flatten().is_none() {
                let _ = self.child.kill();
            }
        }
    }
}

#[cfg(unix)]
pub(super) fn spawn_owned_process(command: &mut Command, label: &str) -> Result<OwnedProcess> {
    use std::os::unix::process::CommandExt;

    command.process_group(0);
    let child = command
        .spawn()
        .map_err(|error| format!("{label} could not start its owned subprocess: {error}"))?;
    let process_group_id = libc::pid_t::try_from(child.id())
        .map_err(|_| format!("{label} child process id does not fit a Unix process-group id"))?;
    Ok(OwnedProcess {
        child,
        process_group_id,
    })
}

#[cfg(windows)]
pub(super) fn spawn_owned_process(command: &mut Command, label: &str) -> Result<OwnedProcess> {
    use std::os::windows::io::AsRawHandle;
    use std::os::windows::process::CommandExt;

    let job = WindowsJob::new(label)?;
    command.creation_flags(CREATE_SUSPENDED);
    let mut child = command.spawn().map_err(|error| {
        format!("{label} could not start its suspended owned subprocess: {error}")
    })?;

    if let Err(assign_error) = job.assign(child.as_raw_handle().cast(), label) {
        let _ = child.kill();
        let cleanup_deadline = Instant::now() + MAX_CLEANUP_RESERVE;
        while Instant::now() < cleanup_deadline {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => thread::sleep(POLL_INTERVAL),
                Err(_) => break,
            }
        }
        return Err(assign_error);
    }

    let mut owned = OwnedProcess { child, job };
    if let Err(resume_error) = resume_suspended_primary_thread(owned.child.id(), label) {
        let cleanup = owned.terminate_and_prove(Instant::now() + MAX_CLEANUP_RESERVE, label);
        return match cleanup {
            Ok(()) => Err(resume_error),
            Err(cleanup_error) => Err(format!(
                "{resume_error}; suspended owned process cleanup also failed: {cleanup_error}"
            )
            .into()),
        };
    }
    Ok(owned)
}

#[cfg(not(any(unix, windows)))]
pub(super) fn spawn_owned_process(command: &mut Command, label: &str) -> Result<OwnedProcess> {
    let child = command
        .spawn()
        .map_err(|error| format!("{label} could not start its owned subprocess: {error}"))?;
    Ok(OwnedProcess { child })
}

#[cfg(windows)]
type WinHandle = *mut std::ffi::c_void;

#[cfg(windows)]
const CREATE_SUSPENDED: u32 = 0x0000_0004;
#[cfg(windows)]
const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x0000_2000;
#[cfg(windows)]
const JOB_OBJECT_BASIC_ACCOUNTING_INFORMATION_CLASS: i32 = 1;
#[cfg(windows)]
const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS: i32 = 9;
#[cfg(windows)]
const TH32CS_SNAPTHREAD: u32 = 0x0000_0004;
#[cfg(windows)]
const THREAD_SUSPEND_RESUME: u32 = 0x0000_0002;
#[cfg(windows)]
const ERROR_NO_MORE_FILES: u32 = 18;

#[cfg(windows)]
#[repr(C)]
struct JobObjectBasicLimitInformation {
    per_process_user_time_limit: i64,
    per_job_user_time_limit: i64,
    limit_flags: u32,
    minimum_working_set_size: usize,
    maximum_working_set_size: usize,
    active_process_limit: u32,
    affinity: usize,
    priority_class: u32,
    scheduling_class: u32,
}

#[cfg(windows)]
#[repr(C)]
struct IoCounters {
    read_operation_count: u64,
    write_operation_count: u64,
    other_operation_count: u64,
    read_transfer_count: u64,
    write_transfer_count: u64,
    other_transfer_count: u64,
}

#[cfg(windows)]
#[repr(C)]
struct JobObjectExtendedLimitInformation {
    basic_limit_information: JobObjectBasicLimitInformation,
    io_info: IoCounters,
    process_memory_limit: usize,
    job_memory_limit: usize,
    peak_process_memory_used: usize,
    peak_job_memory_used: usize,
}

#[cfg(windows)]
#[repr(C)]
struct JobObjectBasicAccountingInformation {
    total_user_time: i64,
    total_kernel_time: i64,
    this_period_total_user_time: i64,
    this_period_total_kernel_time: i64,
    total_page_fault_count: u32,
    total_processes: u32,
    active_processes: u32,
    total_terminated_processes: u32,
}

#[cfg(windows)]
#[repr(C)]
struct ThreadEntry32 {
    size: u32,
    usage_count: u32,
    thread_id: u32,
    owner_process_id: u32,
    base_priority: i32,
    delta_priority: i32,
    flags: u32,
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    #[link_name = "CreateJobObjectW"]
    fn create_job_object_w(attributes: *const std::ffi::c_void, name: *const u16) -> WinHandle;
    #[link_name = "SetInformationJobObject"]
    fn set_information_job_object(
        job: WinHandle,
        information_class: i32,
        information: *const std::ffi::c_void,
        information_length: u32,
    ) -> i32;
    #[link_name = "AssignProcessToJobObject"]
    fn assign_process_to_job_object(job: WinHandle, process: WinHandle) -> i32;
    #[link_name = "TerminateJobObject"]
    fn terminate_job_object(job: WinHandle, exit_code: u32) -> i32;
    #[link_name = "QueryInformationJobObject"]
    fn query_information_job_object(
        job: WinHandle,
        information_class: i32,
        information: *mut std::ffi::c_void,
        information_length: u32,
        return_length: *mut u32,
    ) -> i32;
    #[link_name = "CloseHandle"]
    fn close_handle(handle: WinHandle) -> i32;
    #[link_name = "CreateToolhelp32Snapshot"]
    fn create_toolhelp32_snapshot(flags: u32, process_id: u32) -> WinHandle;
    #[link_name = "Thread32First"]
    fn thread32_first(snapshot: WinHandle, entry: *mut ThreadEntry32) -> i32;
    #[link_name = "Thread32Next"]
    fn thread32_next(snapshot: WinHandle, entry: *mut ThreadEntry32) -> i32;
    #[link_name = "OpenThread"]
    fn open_thread(desired_access: u32, inherit_handle: i32, thread_id: u32) -> WinHandle;
    #[link_name = "ResumeThread"]
    fn resume_thread(thread: WinHandle) -> u32;
    #[link_name = "GetLastError"]
    fn get_last_error() -> u32;
}

#[cfg(windows)]
struct OwnedWinHandle(WinHandle);

#[cfg(windows)]
impl OwnedWinHandle {
    fn new(handle: WinHandle, label: &str) -> Result<Self> {
        if handle.is_null() || handle as isize == -1 {
            Err(format!("{label}: {}", io::Error::last_os_error()).into())
        } else {
            Ok(Self(handle))
        }
    }

    fn raw(&self) -> WinHandle {
        self.0
    }
}

#[cfg(windows)]
impl Drop for OwnedWinHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = close_handle(self.0);
        }
    }
}

#[cfg(windows)]
struct WindowsJob {
    handle: OwnedWinHandle,
}

#[cfg(windows)]
impl WindowsJob {
    fn new(label: &str) -> Result<Self> {
        let raw = unsafe { create_job_object_w(std::ptr::null(), std::ptr::null()) };
        let handle = OwnedWinHandle::new(
            raw,
            &format!("{label} could not create a Windows Job Object"),
        )?;
        let mut information: JobObjectExtendedLimitInformation = unsafe { std::mem::zeroed() };
        information.basic_limit_information.limit_flags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let result = unsafe {
            set_information_job_object(
                handle.raw(),
                JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS,
                (&information as *const JobObjectExtendedLimitInformation).cast(),
                std::mem::size_of::<JobObjectExtendedLimitInformation>() as u32,
            )
        };
        if result == 0 {
            return Err(format!(
                "{label} could not configure KILL_ON_JOB_CLOSE on its Windows Job Object: {}",
                io::Error::last_os_error()
            )
            .into());
        }
        Ok(Self { handle })
    }

    fn assign(&self, process: WinHandle, label: &str) -> Result<()> {
        let result = unsafe { assign_process_to_job_object(self.handle.raw(), process) };
        if result == 0 {
            Err(format!(
                "{label} could not assign its suspended child to the owned Windows Job Object: {}",
                io::Error::last_os_error()
            )
            .into())
        } else {
            Ok(())
        }
    }

    fn terminate(&self, label: &str) -> Result<()> {
        if self.active_processes(label)? == 0 {
            return Ok(());
        }
        let result = unsafe { terminate_job_object(self.handle.raw(), 1) };
        if result == 0 {
            Err(format!(
                "{label} could not terminate its owned Windows Job Object: {}",
                io::Error::last_os_error()
            )
            .into())
        } else {
            Ok(())
        }
    }

    fn active_processes(&self, label: &str) -> Result<u32> {
        let mut information: JobObjectBasicAccountingInformation = unsafe { std::mem::zeroed() };
        let result = unsafe {
            query_information_job_object(
                self.handle.raw(),
                JOB_OBJECT_BASIC_ACCOUNTING_INFORMATION_CLASS,
                (&mut information as *mut JobObjectBasicAccountingInformation).cast(),
                std::mem::size_of::<JobObjectBasicAccountingInformation>() as u32,
                std::ptr::null_mut(),
            )
        };
        if result == 0 {
            Err(format!(
                "{label} could not query its Windows Job Object accounting state: {}",
                io::Error::last_os_error()
            )
            .into())
        } else {
            Ok(information.active_processes)
        }
    }
}

#[cfg(windows)]
fn resume_suspended_primary_thread(process_id: u32, label: &str) -> Result<()> {
    let snapshot_raw = unsafe { create_toolhelp32_snapshot(TH32CS_SNAPTHREAD, 0) };
    let snapshot = OwnedWinHandle::new(
        snapshot_raw,
        &format!("{label} could not snapshot Windows threads for suspended-child resume"),
    )?;

    let mut entry: ThreadEntry32 = unsafe { std::mem::zeroed() };
    entry.size = std::mem::size_of::<ThreadEntry32>() as u32;
    if unsafe { thread32_first(snapshot.raw(), &mut entry) } == 0 {
        return Err(format!(
            "{label} could not enumerate Windows threads for suspended-child resume: {}",
            io::Error::last_os_error()
        )
        .into());
    }

    let mut owned_thread_id = None;
    loop {
        if entry.owner_process_id == process_id
            && owned_thread_id.replace(entry.thread_id).is_some()
        {
            return Err(format!(
                "{label} suspended child exposed multiple threads before resume; refusing ambiguous ownership"
            )
            .into());
        }
        entry.size = std::mem::size_of::<ThreadEntry32>() as u32;
        if unsafe { thread32_next(snapshot.raw(), &mut entry) } != 0 {
            continue;
        }
        let last_error = unsafe { get_last_error() };
        if last_error == ERROR_NO_MORE_FILES {
            break;
        }
        return Err(format!(
            "{label} failed while enumerating Windows threads for suspended-child resume: OS error {last_error}"
        )
        .into());
    }

    let thread_id = owned_thread_id
        .ok_or_else(|| format!("{label} suspended child primary thread could not be identified"))?;
    let thread_raw = unsafe { open_thread(THREAD_SUSPEND_RESUME, 0, thread_id) };
    let thread_handle = OwnedWinHandle::new(
        thread_raw,
        &format!("{label} could not open its suspended primary thread"),
    )?;
    let previous_count = unsafe { resume_thread(thread_handle.raw()) };
    if previous_count == u32::MAX {
        return Err(format!(
            "{label} could not resume its suspended primary thread: {}",
            io::Error::last_os_error()
        )
        .into());
    }
    if previous_count != 1 {
        return Err(format!(
            "{label} suspended primary thread had unexpected suspend count {previous_count}; refusing ambiguous resume state"
        )
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{operation_deadlines, spawn_owned_process};
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    fn wait_for_direct_exit(process: &mut super::OwnedProcess, deadline: Instant) -> bool {
        loop {
            match process.try_wait() {
                Ok(Some(_)) => return true,
                Ok(None) => {}
                Err(_) => return false,
            }
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn short_owned_process_quiesces() {
        let mut command = if cfg!(windows) {
            let mut command = Command::new("cmd.exe");
            command.args(["/d", "/s", "/c", "exit 0"]);
            command
        } else {
            let mut command = Command::new("/bin/sh");
            command.args(["-c", "exit 0"]);
            command
        };
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let started = Instant::now();
        let (command_deadline, cleanup_deadline) =
            operation_deadlines(started, Duration::from_secs(3));
        let mut process = spawn_owned_process(&mut command, "process-scope short fixture").unwrap();
        assert!(wait_for_direct_exit(&mut process, command_deadline));
        assert!(
            process
                .wait_for_scope_quiescence(cleanup_deadline, "process-scope short fixture")
                .unwrap()
        );
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn surviving_descendant_is_detected_and_terminated_as_owned_scope() {
        let mut command = if cfg!(windows) {
            let mut command = Command::new("powershell.exe");
            command.args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Process -FilePath \"$env:SystemRoot\\System32\\ping.exe\" -ArgumentList @('-n','30','127.0.0.1') -WindowStyle Hidden; exit 0",
            ]);
            command
        } else {
            let mut command = Command::new("/bin/sh");
            command.args(["-c", "sleep 30 &"]);
            command
        };
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let mut process =
            spawn_owned_process(&mut command, "process-scope descendant fixture").unwrap();
        assert!(wait_for_direct_exit(
            &mut process,
            Instant::now() + Duration::from_secs(5)
        ));
        assert!(
            !process
                .wait_for_scope_quiescence(
                    Instant::now() + Duration::from_millis(100),
                    "process-scope descendant fixture",
                )
                .unwrap(),
            "descendant fixture must keep the owned process scope live"
        );
        process
            .terminate_and_prove(
                Instant::now() + Duration::from_secs(2),
                "process-scope descendant fixture",
            )
            .unwrap();
        assert!(
            process
                .wait_for_scope_quiescence(
                    Instant::now() + Duration::from_millis(100),
                    "process-scope descendant fixture",
                )
                .unwrap()
        );
    }
}
