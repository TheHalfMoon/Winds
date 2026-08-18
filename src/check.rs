use crate::domain::CheckStatus;
#[cfg(unix)]
use std::io::{Read, Result as IoResult};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::Path;
#[cfg(unix)]
use std::process::{Command, Stdio};
#[cfg(unix)]
use std::sync::mpsc;
#[cfg(unix)]
use std::thread;
use std::time::Duration;
#[cfg(unix)]
use std::time::Instant;

#[cfg(unix)]
const OUTPUT_CAP_BYTES: usize = 1_048_576;
#[cfg(unix)]
const STREAM_CLOSE_GRACE: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub struct CapturedStream {
    pub bytes: Vec<u8>,
    pub truncated: bool,
}

#[derive(Debug)]
pub struct CheckRun {
    pub status: CheckStatus,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
    pub stdout: CapturedStream,
    pub stderr: CapturedStream,
}

#[cfg(unix)]
pub fn run_check(cwd: &Path, command: &str, timeout: Duration) -> Result<CheckRun, String> {
    let started = Instant::now();
    let mut child = Command::new("/bin/sh")
        .args(["-c", command])
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .map_err(|error| format!("failed to start required check: {error}"))?;
    let group_pid = child.id();

    let stdout = child
        .stdout
        .take()
        .ok_or("failed to capture check stdout")?;
    let stderr = child
        .stderr
        .take()
        .ok_or("failed to capture check stderr")?;

    let (stdout_tx, stdout_rx) = mpsc::sync_channel(1);
    let (stderr_tx, stderr_rx) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let _ = stdout_tx.send(read_capped(stdout));
    });
    thread::spawn(move || {
        let _ = stderr_tx.send(read_capped(stderr));
    });

    let mut timed_out = false;
    let exit = loop {
        let exited = match child_exited_without_reaping(group_pid) {
            Ok(exited) => exited,
            Err(error) => {
                terminate_process_group(group_pid);
                let _ = child.kill();
                let _ = child.wait();
                return Err(error);
            }
        };
        if exited {
            // Keep the exited group leader unreaped until descendants have been terminated so the
            // process-group id cannot be recycled to an unrelated process group.
            terminate_process_group(group_pid);
            break child
                .wait()
                .map_err(|error| format!("failed waiting for required check: {error}"))?;
        }
        if started.elapsed() < timeout {
            thread::sleep(Duration::from_millis(25));
            continue;
        }

        timed_out = true;
        terminate_process_group(group_pid);
        let _ = child.kill();
        break child
            .wait()
            .map_err(|error| format!("failed waiting after check timeout: {error}"))?;
    };

    let stdout = receive_stream(stdout_rx, "stdout")?;
    let stderr = receive_stream(stderr_rx, "stderr")?;

    let status = if timed_out {
        CheckStatus::Timeout
    } else if exit.success() {
        CheckStatus::Pass
    } else {
        CheckStatus::Fail
    };

    Ok(CheckRun {
        status,
        exit_code: exit.code(),
        duration_ms: started.elapsed().as_millis(),
        stdout,
        stderr,
    })
}

#[cfg(not(unix))]
pub fn run_check(_cwd: &Path, _command: &str, _timeout: Duration) -> Result<CheckRun, String> {
    Err(
        "required checks are unsupported on native Windows in Spec 003 T051; Winds verification remains Unix-only"
            .to_owned(),
    )
}

#[cfg(unix)]
fn child_exited_without_reaping(pid: u32) -> Result<bool, String> {
    // waitid + WNOWAIT observes child exit without releasing its pid, which keeps the process-group
    // identity stable until Winds has signalled any surviving descendants.
    let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
    let result = unsafe {
        libc::waitid(
            libc::P_PID,
            pid as libc::id_t,
            &mut info,
            libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
        )
    };
    if result == -1 {
        return Err(format!(
            "failed while waiting for required check: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(unsafe { info.si_pid() } != 0)
}

#[cfg(unix)]
fn receive_stream(
    receiver: mpsc::Receiver<IoResult<CapturedStream>>,
    name: &str,
) -> Result<CapturedStream, String> {
    receiver
        .recv_timeout(STREAM_CLOSE_GRACE)
        .map_err(|_| {
            format!(
                "check {name} did not close after process termination; a descendant may have escaped the check process group"
            )
        })?
        .map_err(|error| format!("failed reading check {name}: {error}"))
}

#[cfg(unix)]
fn read_capped<R: Read>(mut reader: R) -> IoResult<CapturedStream> {
    let mut captured = Vec::new();
    let mut truncated = false;
    let mut buffer = [0_u8; 8192];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let remaining = OUTPUT_CAP_BYTES.saturating_sub(captured.len());
        let keep = remaining.min(count);
        captured.extend_from_slice(&buffer[..keep]);
        if keep < count {
            truncated = true;
        }
    }

    Ok(CapturedStream {
        bytes: captured,
        truncated,
    })
}

#[cfg(unix)]
fn terminate_process_group(pid: u32) {
    let group = format!("-{pid}");
    let term = Command::new("/bin/kill")
        .args(["-TERM", "--", group.as_str()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if !matches!(term, Ok(status) if status.success()) {
        return;
    }

    thread::sleep(Duration::from_millis(100));
    let _ = Command::new("/bin/kill")
        .args(["-KILL", "--", group.as_str()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
