use super::Result;
use super::shell_profiles::{ShellProfile, validate_shell_profile_for_launch};
use portable_pty::{CommandBuilder, ExitStatus, MasterPty, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerminalSessionId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    pub rows: u16,
    pub cols: u16,
}

impl TerminalSize {
    fn to_pty_size(self) -> Result<PtySize> {
        if self.rows == 0 || self.cols == 0 {
            return Err("terminal size rows and columns must both be non-zero".into());
        }
        Ok(PtySize {
            rows: self.rows,
            cols: self.cols,
            pixel_width: 0,
            pixel_height: 0,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalExit {
    pub exit_code: u32,
    pub signal: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TerminalDropCleanupOutcome {
    ExitedBeforeCleanup(TerminalExit),
    Terminated(TerminalExit),
    Unproven,
}

pub struct TerminalSession {
    session_id: TerminalSessionId,
    profile_id: String,
    start_cwd: PathBuf,
    master: Option<Box<dyn MasterPty + Send>>,
    output_reader: Option<Box<dyn Read + Send>>,
    writer: Option<Box<dyn Write + Send>>,
    child: Option<Box<dyn portable_pty::Child + Send + Sync>>,
    final_exit: Option<TerminalExit>,
    drop_cleanup_attempted: bool,
}

impl TerminalSession {
    pub fn start(profile: &ShellProfile, cwd: &Path, size: TerminalSize) -> Result<Self> {
        validate_shell_profile_for_launch(profile)?;
        Self::start_command(
            &profile.profile_id,
            Path::new(&profile.executable),
            &profile.arguments,
            cwd,
            size,
        )
    }

    #[cfg(windows)]
    pub(super) fn start_exact_launch(
        profile_id: &str,
        executable: &Path,
        arguments: &[String],
        cwd: &Path,
        size: TerminalSize,
    ) -> Result<Self> {
        if profile_id.is_empty() {
            return Err("terminal launch profile identity cannot be empty".into());
        }
        if !executable.is_absolute() {
            return Err("terminal launch executable must be an absolute path".into());
        }
        let metadata = std::fs::metadata(executable).map_err(|error| {
            format!(
                "terminal launch executable cannot be inspected ({}): {error}",
                executable.display()
            )
        })?;
        if !metadata.is_file() {
            return Err(format!(
                "terminal launch executable is not a file: {}",
                executable.display()
            )
            .into());
        }
        Self::start_command(profile_id, executable, arguments, cwd, size)
    }

    fn start_command(
        profile_id: &str,
        executable: &Path,
        arguments: &[String],
        cwd: &Path,
        size: TerminalSize,
    ) -> Result<Self> {
        let start_cwd = canonical_start_cwd(cwd)?;
        let spawn_cwd = terminal_spawn_cwd(&start_cwd)?;
        let pty_size = size.to_pty_size()?;
        let session_id = next_session_id()?;

        let pty_system = native_pty_system();
        let pair = pty_system.openpty(pty_size)?;
        let output_reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let mut command = CommandBuilder::new(executable.as_os_str());
        command.args(arguments);
        command.cwd(spawn_cwd.as_os_str());
        let child = pair.slave.spawn_command(command)?;
        drop(pair.slave);

        Ok(Self {
            session_id,
            profile_id: profile_id.to_owned(),
            start_cwd,
            master: Some(pair.master),
            output_reader: Some(output_reader),
            writer: Some(writer),
            child: Some(child),
            final_exit: None,
            drop_cleanup_attempted: false,
        })
    }

    pub fn session_id(&self) -> TerminalSessionId {
        self.session_id
    }

    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    pub fn start_cwd(&self) -> &Path {
        &self.start_cwd
    }

    pub fn take_output_reader(&mut self) -> Result<Box<dyn Read + Send>> {
        self.output_reader
            .take()
            .ok_or_else(|| "terminal session output reader has already been taken".into())
    }

    pub fn send_input(&mut self, bytes: &[u8]) -> Result<()> {
        self.require_active()?;
        let writer = self
            .writer
            .as_mut()
            .ok_or("terminal session input is already closed")?;
        writer.write_all(bytes)?;
        writer.flush()?;
        Ok(())
    }

    pub fn resize(&mut self, size: TerminalSize) -> Result<()> {
        self.require_active()?;
        self.master
            .as_ref()
            .ok_or("terminal PTY master is already closed")?
            .resize(size.to_pty_size()?)?;
        Ok(())
    }

    pub fn current_size(&self) -> Result<TerminalSize> {
        let size = self
            .master
            .as_ref()
            .ok_or("terminal PTY master is already closed")?
            .get_size()?;
        Ok(TerminalSize {
            rows: size.rows,
            cols: size.cols,
        })
    }

    pub fn interrupt(&mut self) -> Result<()> {
        self.require_active()?;
        self.interrupt_platform()
    }

    #[cfg(unix)]
    fn interrupt_platform(&mut self) -> Result<()> {
        let child_pid = self
            .child
            .as_ref()
            .and_then(|child| child.process_id())
            .ok_or("terminal interrupt unavailable: owned child process id is unknown")?;
        let child_pid = libc::pid_t::try_from(child_pid)
            .map_err(|_| "terminal interrupt unavailable: child process id is out of range")?;
        let process_group = self
            .master
            .as_ref()
            .ok_or("terminal interrupt unavailable: PTY master is already closed")?
            .process_group_leader()
            .ok_or("terminal interrupt unavailable: foreground PTY process group is unknown")?;

        // portable-pty 0.9.0 establishes the spawned child as a session leader
        // before attaching this PTY as its controlling terminal. A foreground
        // process group is therefore signalable only while it still belongs to
        // the live session led by the exact child handle retained above.
        let session_leader = unsafe { libc::getsid(process_group) };
        if session_leader == -1 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ESRCH) && self.try_wait()?.is_some() {
                return Ok(());
            }
            return Err(format!(
                "failed to validate terminal foreground process-group ownership: {error}"
            )
            .into());
        }
        if session_leader != child_pid {
            return Err("terminal foreground process group is not owned by this session".into());
        }

        if unsafe { libc::killpg(process_group, libc::SIGINT) } == 0 {
            return Ok(());
        }

        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ESRCH) && self.try_wait()?.is_some() {
            return Ok(());
        }
        Err(format!("failed to interrupt owned terminal foreground process group: {error}").into())
    }

    #[cfg(windows)]
    fn interrupt_platform(&mut self) -> Result<()> {
        Err(
            "terminal interrupt is unsupported on native Windows in Spec 003 T051; use terminate for owned process termination"
                .into(),
        )
    }

    pub fn try_wait(&mut self) -> Result<Option<TerminalExit>> {
        if let Some(exit) = &self.final_exit {
            return Ok(Some(exit.clone()));
        }

        let status = match self.child.as_mut() {
            Some(child) => child.try_wait()?,
            None => return Err("terminal session lost its owned child handle".into()),
        };
        Ok(status.map(|status| self.finish_exit(status)))
    }

    pub fn wait(&mut self) -> Result<TerminalExit> {
        if let Some(exit) = &self.final_exit {
            return Ok(exit.clone());
        }
        let status = self
            .child
            .as_mut()
            .ok_or("terminal session lost its owned child handle")?
            .wait()?;
        Ok(self.finish_exit(status))
    }

    pub fn terminate(&mut self) -> Result<TerminalExit> {
        match self.cleanup_for_drop(Duration::from_millis(500))? {
            TerminalDropCleanupOutcome::ExitedBeforeCleanup(exit)
            | TerminalDropCleanupOutcome::Terminated(exit) => Ok(exit),
            TerminalDropCleanupOutcome::Unproven => Err(
                "terminal terminate could not prove owned child exit inside bounded cleanup window"
                    .into(),
            ),
        }
    }

    pub fn close(&mut self) -> Result<TerminalExit> {
        match self.cleanup_for_drop(Duration::from_millis(500))? {
            TerminalDropCleanupOutcome::ExitedBeforeCleanup(exit)
            | TerminalDropCleanupOutcome::Terminated(exit) => Ok(exit),
            TerminalDropCleanupOutcome::Unproven => Err(
                "terminal close could not prove owned child exit inside bounded cleanup window"
                    .into(),
            ),
        }
    }

    pub(crate) fn cleanup_for_drop(
        &mut self,
        timeout: Duration,
    ) -> Result<TerminalDropCleanupOutcome> {
        if let Some(exit) = &self.final_exit {
            self.drop_cleanup_attempted = true;
            return Ok(TerminalDropCleanupOutcome::ExitedBeforeCleanup(
                exit.clone(),
            ));
        }
        if self.drop_cleanup_attempted {
            return Ok(TerminalDropCleanupOutcome::Unproven);
        }
        self.drop_cleanup_attempted = true;

        let result = (|| {
            self.writer.take();
            if let Some(exit) = self.try_wait()? {
                return Ok(TerminalDropCleanupOutcome::ExitedBeforeCleanup(exit));
            }

            let kill_result = self
                .child
                .as_mut()
                .ok_or("terminal session lost its owned child handle")?
                .kill();
            if let Err(kill_error) = kill_result {
                if let Some(exit) = self.try_wait()? {
                    return Ok(TerminalDropCleanupOutcome::ExitedBeforeCleanup(exit));
                }
                return Err(format!(
                    "failed to request bounded cleanup of owned terminal child: {kill_error}"
                )
                .into());
            }

            let started = Instant::now();
            loop {
                if let Some(exit) = self.try_wait()? {
                    return Ok(TerminalDropCleanupOutcome::Terminated(exit));
                }
                if started.elapsed() >= timeout {
                    return Ok(TerminalDropCleanupOutcome::Unproven);
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        })();

        if matches!(result, Err(_) | Ok(TerminalDropCleanupOutcome::Unproven)) {
            self.drop_cleanup_attempted = false;
        }
        result
    }

    pub(crate) fn suppress_drop_cleanup_after_ownership_loss(&mut self) {
        self.drop_cleanup_attempted = true;
    }

    fn require_active(&mut self) -> Result<()> {
        if self.try_wait()?.is_some() {
            return Err("terminal session has already exited".into());
        }
        Ok(())
    }

    fn finish_exit(&mut self, status: ExitStatus) -> TerminalExit {
        let exit = TerminalExit {
            exit_code: status.exit_code(),
            signal: status.signal().map(str::to_owned),
        };
        self.child.take();
        self.writer.take();
        self.output_reader.take();
        self.master.take();
        self.final_exit = Some(exit.clone());
        exit
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        if self.final_exit.is_none() && !self.drop_cleanup_attempted {
            let _ = self.cleanup_for_drop(Duration::from_millis(500));
        }
    }
}

fn canonical_start_cwd(cwd: &Path) -> Result<PathBuf> {
    if !cwd.is_absolute() {
        return Err("terminal start cwd must be an absolute path".into());
    }
    let canonical = cwd
        .canonicalize()
        .map_err(|error| format!("terminal start cwd cannot be canonicalized: {error}"))?;
    if !canonical.is_dir() {
        return Err("terminal start cwd must be a directory".into());
    }
    Ok(canonical)
}

#[cfg(not(windows))]
fn terminal_spawn_cwd(canonical_cwd: &Path) -> Result<PathBuf> {
    Ok(canonical_cwd.to_path_buf())
}

#[cfg(windows)]
fn terminal_spawn_cwd(canonical_cwd: &Path) -> Result<PathBuf> {
    let value = canonical_cwd
        .to_str()
        .ok_or("native Windows terminal cwd is not valid UTF-8")?;
    if value.starts_with(r"\\?\UNC\") {
        return Err(
            "native Windows terminal cwd cannot use a UNC path in Spec 003 T051; refusing to let the shell silently fall back to another directory"
                .into(),
        );
    }
    if let Some(rest) = value.strip_prefix(r"\\?\") {
        let bytes = rest.as_bytes();
        let ordinary_drive_path = bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'\\' | b'/');
        if !ordinary_drive_path {
            return Err(
                "native Windows terminal cwd cannot be represented safely for the PTY child".into(),
            );
        }
        return Ok(PathBuf::from(rest));
    }
    if value.starts_with(r"\\") {
        return Err(
            "native Windows terminal cwd cannot use a UNC path in Spec 003 T051; refusing to let the shell silently fall back to another directory"
                .into(),
        );
    }
    Ok(canonical_cwd.to_path_buf())
}

fn next_session_id() -> Result<TerminalSessionId> {
    let previous = NEXT_SESSION_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| "terminal session identity space exhausted")?;
    Ok(TerminalSessionId(previous + 1))
}

#[cfg(all(test, unix))]
mod tests {
    use super::{TerminalSession, TerminalSize};
    use crate::git::shell_profiles::discover_native_shell_profiles;
    use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
    use std::ffi::OsStr;
    use std::fs;
    use std::io::Read;
    use std::os::unix::ffi::OsStrExt;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
    use std::thread;
    use std::time::{Duration, Instant};

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);
    const OUTPUT_LIMIT: usize = 128 * 1024;

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new(name: &str) -> Self {
            let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "winds-t050-{name}-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let canonical_temp = match std::env::temp_dir().canonicalize() {
                Ok(value) => value,
                Err(_) => return,
            };
            let canonical_root = match self.0.canonicalize() {
                Ok(value) => value,
                Err(_) => return,
            };
            let owned_name = canonical_root
                .file_name()
                .and_then(OsStr::to_str)
                .is_some_and(|name| name.starts_with("winds-t050-"));
            if owned_name && canonical_root.starts_with(&canonical_temp) {
                let _ = fs::remove_dir_all(canonical_root);
            }
        }
    }

    enum OutputEvent {
        Chunk(Vec<u8>),
        Error(String),
        Eof,
    }

    fn native_sh_profile() -> crate::git::shell_profiles::ShellProfile {
        let inventory = WorkspaceEnvironmentInventory {
            host_os: std::env::consts::OS.to_owned(),
            host_arch: std::env::consts::ARCH.to_owned(),
            canonical_worktree_root: "/unused/worktree".to_owned(),
            git_common_dir: "/unused/git-common".to_owned(),
            shell_candidates: vec!["/bin/sh".to_owned()],
            detected_manifests: Vec::new(),
        };
        discover_native_shell_profiles(&inventory)
            .unwrap()
            .into_iter()
            .find(|profile| profile.executable == "/bin/sh")
            .expect("/bin/sh must be discoverable on supported Unix CI hosts")
    }

    fn start_output_reader(mut reader: Box<dyn Read + Send>) -> Receiver<OutputEvent> {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let mut buffer = [0_u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        let _ = sender.send(OutputEvent::Eof);
                        return;
                    }
                    Ok(count) => {
                        if sender
                            .send(OutputEvent::Chunk(buffer[..count].to_vec()))
                            .is_err()
                        {
                            return;
                        }
                    }
                    Err(error) if error.raw_os_error() == Some(libc::EIO) => {
                        let _ = sender.send(OutputEvent::Eof);
                        return;
                    }
                    Err(error) => {
                        let _ = sender.send(OutputEvent::Error(error.to_string()));
                        return;
                    }
                }
            }
        });
        receiver
    }

    fn wait_for_output(receiver: &Receiver<OutputEvent>, needle: &[u8]) -> Vec<u8> {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut output = Vec::new();
        while Instant::now() < deadline {
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(OutputEvent::Chunk(chunk)) => {
                    output.extend_from_slice(&chunk);
                    assert!(
                        output.len() <= OUTPUT_LIMIT,
                        "PTY test output exceeded bound"
                    );
                    if output.windows(needle.len()).any(|window| window == needle) {
                        return output;
                    }
                }
                Ok(OutputEvent::Error(error)) => panic!("PTY output reader failed: {error}"),
                Ok(OutputEvent::Eof) => break,
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        panic!(
            "timed out waiting for PTY output marker {:?}; observed {:?}",
            String::from_utf8_lossy(needle),
            String::from_utf8_lossy(&output)
        );
    }

    fn shell_quote_path(path: &Path) -> Vec<u8> {
        let mut quoted = Vec::with_capacity(path.as_os_str().as_bytes().len() + 2);
        quoted.push(b'\'');
        for byte in path.as_os_str().as_bytes() {
            if *byte == b'\'' {
                quoted.extend_from_slice(b"'\\''");
            } else {
                quoted.push(*byte);
            }
        }
        quoted.push(b'\'');
        quoted
    }

    fn default_size() -> TerminalSize {
        TerminalSize { rows: 24, cols: 80 }
    }

    #[test]
    fn streams_input_output_from_exact_start_cwd_and_observes_exit() {
        let root = TestRoot::new("stream");
        let canonical_root = root.path().canonicalize().unwrap();
        let profile = native_sh_profile();
        let mut session = TerminalSession::start(&profile, root.path(), default_size()).unwrap();
        let session_id = session.session_id();
        assert_eq!(session.profile_id(), profile.profile_id);
        assert_eq!(session.start_cwd(), canonical_root);

        let output = start_output_reader(session.take_output_reader().unwrap());
        assert!(session.take_output_reader().is_err());
        session
            .send_input(
                b"pwd\nprintf '\\127\\111\\116\\104\\123\\137\\122\\105\\101\\104\\131\\012'\nexit\n",
            )
            .unwrap();
        let observed = wait_for_output(&output, b"WINDS_READY");
        assert!(
            observed
                .windows(canonical_root.as_os_str().as_encoded_bytes().len())
                .any(|window| window == canonical_root.as_os_str().as_encoded_bytes())
        );

        let exit = session.wait().unwrap();
        assert_eq!(exit.exit_code, 0);
        assert_eq!(session.session_id(), session_id);
        assert_eq!(session.try_wait().unwrap(), Some(exit));
    }

    #[test]
    fn resize_updates_owned_pty_dimensions() {
        let root = TestRoot::new("resize");
        let profile = native_sh_profile();
        let mut session = TerminalSession::start(&profile, root.path(), default_size()).unwrap();

        let resized = TerminalSize {
            rows: 40,
            cols: 120,
        };
        session.resize(resized).unwrap();
        assert_eq!(session.current_size().unwrap(), resized);
        session.close().unwrap();
    }

    #[test]
    fn interrupt_signals_foreground_job_and_keeps_terminal_shell_usable() {
        let root = TestRoot::new("interrupt");
        let script = root.path().join("interrupt.sh");
        fs::write(
            &script,
            concat!(
                "trap 'printf \"\\127\\111\\116\\104\\123\\137\\111\\116\\124\\105\\122\\122\\125\\120\\124\\105\\104\\012\"; exit 130' INT\n",
                "printf '\\127\\111\\116\\104\\123\\137\\122\\105\\101\\104\\131\\012'\n",
                "while :; do sleep 1; done\n"
            ),
        )
        .unwrap();

        let profile = native_sh_profile();
        let mut session = TerminalSession::start(&profile, root.path(), default_size()).unwrap();
        let output = start_output_reader(session.take_output_reader().unwrap());

        let mut command = b"sh ".to_vec();
        command.extend_from_slice(&shell_quote_path(&script));
        command.push(b'\n');
        session.send_input(&command).unwrap();
        wait_for_output(&output, b"WINDS_READY");

        session.interrupt().unwrap();
        wait_for_output(&output, b"WINDS_INTERRUPTED");

        session
            .send_input(
                b"printf '\\127\\111\\116\\104\\123\\137\\101\\106\\124\\105\\122\\012'\nexit\n",
            )
            .unwrap();
        wait_for_output(&output, b"WINDS_AFTER");
        let exit = session.wait().unwrap();
        assert_eq!(exit.exit_code, 0);
    }

    #[test]
    fn terminate_reaps_only_the_owned_child_and_is_idempotently_observed() {
        let root = TestRoot::new("terminate");
        let profile = native_sh_profile();
        let mut session = TerminalSession::start(&profile, root.path(), default_size()).unwrap();
        let output = start_output_reader(session.take_output_reader().unwrap());

        session
            .send_input(
                b"printf '\\127\\111\\116\\104\\123\\137\\122\\105\\101\\104\\131\\012'; exec sleep 30\n",
            )
            .unwrap();
        wait_for_output(&output, b"WINDS_READY");
        let exit = session.terminate().unwrap();
        assert!(!exit.signal.as_deref().unwrap_or_default().is_empty() || exit.exit_code != 0);
        assert_eq!(session.try_wait().unwrap(), Some(exit.clone()));
        assert_eq!(session.close().unwrap(), exit);
    }

    #[test]
    fn dropping_live_terminal_session_is_bounded() {
        let root = TestRoot::new("bounded-drop");
        let profile = native_sh_profile();
        let session = TerminalSession::start(&profile, root.path(), default_size()).unwrap();
        let started = Instant::now();
        drop(session);
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "dropping a directly owned live terminal must not block indefinitely"
        );
    }

    #[test]
    fn start_rejects_relative_cwd_and_zero_dimensions_before_spawning() {
        let profile = native_sh_profile();
        let relative = Path::new("relative");
        let error = TerminalSession::start(&profile, relative, default_size())
            .err()
            .expect("relative cwd must fail");
        assert!(error.to_string().contains("absolute path"));

        let root = TestRoot::new("invalid-size");
        let error =
            TerminalSession::start(&profile, root.path(), TerminalSize { rows: 0, cols: 80 })
                .err()
                .expect("zero rows must fail");
        assert!(error.to_string().contains("non-zero"));
    }
}

#[cfg(all(test, windows))]
#[path = "terminal_windows_tests.rs"]
mod windows_tests;
