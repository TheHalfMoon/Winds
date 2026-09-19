use crate::git::shell_profiles::ShellProfile;
use crate::git::terminal::TerminalSize;
use crate::persistent_runtime::domain::{OwnerGenerationId, RuntimeAlias, RuntimeNamespaceId};
use crate::persistent_runtime::runtime::{
    PersistentTerminalAttachment, PersistentTerminalRegistry, PersistentTerminalSnapshot,
};
use crate::store::Store;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub(crate) const INTERNAL_OWNER_COMMAND: &str = "__winds-internal-owner-v1";
pub(crate) const OWNER_IDLE_GRACE_MS: u64 = 300_000;
const OWNER_POLL_INTERVAL_MS: u64 = 250;

pub(crate) type OwnerResult<T> = Result<T, OwnerError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OwnerError {
    HomeMustBeAbsolute,
    HomeUnavailable,
    HomeNotCanonical,
    SingletonAlreadyActive,
    SingletonSecurityMismatch,
    SingletonIo(String),
    Entropy(String),
    Store(String),
    Endpoint(String),
    ClockUnavailable,
    ActivityUnderflow,
    Runtime(String),
}

impl fmt::Display for OwnerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HomeMustBeAbsolute => {
                formatter.write_str("persistent owner home must be absolute")
            }
            Self::HomeUnavailable => formatter.write_str("persistent owner home is unavailable"),
            Self::HomeNotCanonical => {
                formatter.write_str("persistent owner home could not be canonicalized")
            }
            Self::SingletonAlreadyActive => {
                formatter.write_str("persistent owner singleton is already active")
            }
            Self::SingletonSecurityMismatch => formatter.write_str(
                "persistent owner singleton security facts do not match the accepted principal",
            ),
            Self::SingletonIo(message) => write!(
                formatter,
                "persistent owner singleton I/O failed: {message}"
            ),
            Self::Entropy(message) => write!(
                formatter,
                "persistent owner generation entropy failed: {message}"
            ),
            Self::Store(message) => write!(
                formatter,
                "persistent owner store startup failed: {message}"
            ),
            Self::Endpoint(message) => write!(
                formatter,
                "persistent owner endpoint startup failed: {message}"
            ),
            Self::ClockUnavailable => formatter.write_str("persistent owner clock is unavailable"),
            Self::ActivityUnderflow => {
                formatter.write_str("persistent owner client activity underflow")
            }
            Self::Runtime(message) => {
                write!(formatter, "persistent owner runtime failed: {message}")
            }
        }
    }
}

impl Error for OwnerError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OwnerStartupPhase {
    SingletonAcquired,
    GenerationCreated,
    Reconciled,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OwnerActivity {
    connected_clients: usize,
    live_runtimes: usize,
    idle_since_monotonic_ms: Option<u64>,
}

impl OwnerActivity {
    pub(crate) fn new(now_monotonic_ms: u64) -> Self {
        Self {
            connected_clients: 0,
            live_runtimes: 0,
            idle_since_monotonic_ms: Some(now_monotonic_ms),
        }
    }

    pub(crate) fn client_connected(&mut self) {
        self.connected_clients = self.connected_clients.saturating_add(1);
        self.idle_since_monotonic_ms = None;
    }

    pub(crate) fn client_disconnected(&mut self, now_monotonic_ms: u64) -> OwnerResult<()> {
        if self.connected_clients == 0 {
            return Err(OwnerError::ActivityUnderflow);
        }
        self.connected_clients -= 1;
        self.refresh_idle_start(now_monotonic_ms);
        Ok(())
    }

    pub(crate) fn set_live_runtime_count(&mut self, count: usize, now_monotonic_ms: u64) {
        self.live_runtimes = count;
        if count > 0 {
            self.idle_since_monotonic_ms = None;
        } else {
            self.refresh_idle_start(now_monotonic_ms);
        }
    }

    pub(crate) fn should_exit(&self, now_monotonic_ms: u64) -> bool {
        if self.connected_clients != 0 || self.live_runtimes != 0 {
            return false;
        }
        self.idle_since_monotonic_ms.is_some_and(|idle_since| {
            now_monotonic_ms.saturating_sub(idle_since) >= OWNER_IDLE_GRACE_MS
        })
    }

    fn refresh_idle_start(&mut self, now_monotonic_ms: u64) {
        if self.connected_clients == 0 && self.live_runtimes == 0 {
            self.idle_since_monotonic_ms.get_or_insert(now_monotonic_ms);
        }
    }
}

pub(crate) struct PersistentOwner {
    _singleton: OwnerSingleton,
    generation_id: OwnerGenerationId,
    store: Store,
    _endpoint: OwnerEndpoint,
    activity: OwnerActivity,
    reconciled_runtime_count: usize,
    ready_unix_ms: i64,
    startup_phase: OwnerStartupPhase,
    runtime_registry: PersistentTerminalRegistry,
}

impl PersistentOwner {
    pub(crate) fn start(home: &Path, now_unix_ms: i64) -> OwnerResult<Self> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            Self::start_with_posix_endpoint(home, now_unix_ms, None)
        }
        #[cfg(windows)]
        {
            Self::start_with_windows_endpoint(home, now_unix_ms)
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
        {
            let _ = (home, now_unix_ms);
            Err(OwnerError::Endpoint(
                "unsupported owner platform".to_owned(),
            ))
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn start_with_posix_endpoint(
        home: &Path,
        now_unix_ms: i64,
        runtime_directory: Option<&Path>,
    ) -> OwnerResult<Self> {
        use crate::persistent_runtime::transport::unix::{
            BoundUnixListener, generate_owner_generation_id,
        };

        let canonical_home = prepare_owner_home(home)?;
        let singleton = OwnerSingleton::acquire(&canonical_home)?;
        let _phase = OwnerStartupPhase::SingletonAcquired;
        let generation_id = generate_owner_generation_id()
            .map_err(|error| OwnerError::Entropy(format!("{error:?}")))?;
        let _phase = OwnerStartupPhase::GenerationCreated;
        let (store, reconciled_runtime_count) =
            reconcile_store(&canonical_home, generation_id, now_unix_ms)?;
        let _phase = OwnerStartupPhase::Reconciled;
        let listener = match runtime_directory {
            Some(path) => BoundUnixListener::bind(path),
            None => BoundUnixListener::bind_default(),
        }
        .map_err(|error| OwnerError::Endpoint(format!("{error:?}")))?;

        Ok(Self {
            _singleton: singleton,
            generation_id,
            store,
            _endpoint: OwnerEndpoint::Posix(listener),
            activity: OwnerActivity::new(0),
            reconciled_runtime_count,
            ready_unix_ms: now_unix_ms,
            startup_phase: OwnerStartupPhase::Ready,
            runtime_registry: PersistentTerminalRegistry::new(),
        })
    }

    #[cfg(windows)]
    fn start_with_windows_endpoint(home: &Path, now_unix_ms: i64) -> OwnerResult<Self> {
        use crate::persistent_runtime::transport::windows::{
            WindowsNamedPipeServer, generate_owner_generation_id,
        };

        let canonical_home = prepare_owner_home(home)?;
        let singleton = OwnerSingleton::acquire(&canonical_home)?;
        let _phase = OwnerStartupPhase::SingletonAcquired;
        let generation_id = generate_owner_generation_id()
            .map_err(|error| OwnerError::Entropy(format!("{error:?}")))?;
        let _phase = OwnerStartupPhase::GenerationCreated;
        let (store, reconciled_runtime_count) =
            reconcile_store(&canonical_home, generation_id, now_unix_ms)?;
        let _phase = OwnerStartupPhase::Reconciled;
        let server = WindowsNamedPipeServer::bind(generation_id)
            .map_err(|error| OwnerError::Endpoint(format!("{error:?}")))?;

        Ok(Self {
            _singleton: singleton,
            generation_id,
            store,
            _endpoint: OwnerEndpoint::Windows(server),
            activity: OwnerActivity::new(0),
            reconciled_runtime_count,
            ready_unix_ms: now_unix_ms,
            startup_phase: OwnerStartupPhase::Ready,
            runtime_registry: PersistentTerminalRegistry::new(),
        })
    }

    #[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
    pub(crate) fn start_for_test(
        home: &Path,
        runtime_directory: &Path,
        now_unix_ms: i64,
    ) -> OwnerResult<Self> {
        Self::start_with_posix_endpoint(home, now_unix_ms, Some(runtime_directory))
    }

    pub(crate) fn generation_id(&self) -> OwnerGenerationId {
        self.generation_id
    }

    pub(crate) fn is_ready(&self) -> bool {
        self.startup_phase == OwnerStartupPhase::Ready
    }

    pub(crate) fn reconciled_runtime_count(&self) -> usize {
        self.reconciled_runtime_count
    }

    pub(crate) fn ready_unix_ms(&self) -> i64 {
        self.ready_unix_ms
    }

    pub(crate) fn activity_mut(&mut self) -> &mut OwnerActivity {
        &mut self.activity
    }

    pub(crate) fn start_terminal_runtime(
        &mut self,
        runtime_alias: RuntimeAlias,
        profile: &ShellProfile,
        cwd: &Path,
        terminal_size: TerminalSize,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<PersistentTerminalAttachment> {
        let attachment = self
            .runtime_registry
            .start_shell(
                &self.store,
                self.generation_id,
                runtime_alias,
                profile,
                cwd,
                terminal_size,
                now_unix_ms,
            )
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        self.sync_runtime_activity(now_monotonic_ms);
        Ok(attachment)
    }

    pub(crate) fn reattach_terminal_runtime(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
        expected_owner_generation_id: OwnerGenerationId,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<PersistentTerminalAttachment> {
        let attachment = self
            .runtime_registry
            .reattach(
                &self.store,
                runtime_namespace_id,
                expected_owner_generation_id,
                self.generation_id,
                now_unix_ms,
            )
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        self.sync_runtime_activity(now_monotonic_ms);
        Ok(attachment)
    }

    pub(crate) fn terminal_runtime_snapshot(
        &mut self,
        attachment: &PersistentTerminalAttachment,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<PersistentTerminalSnapshot> {
        let snapshot = self
            .runtime_registry
            .snapshot(&self.store, attachment, self.generation_id, now_unix_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        self.sync_runtime_activity(now_monotonic_ms);
        Ok(snapshot)
    }

    pub(crate) fn send_terminal_runtime_input(
        &mut self,
        attachment: &PersistentTerminalAttachment,
        bytes: &[u8],
    ) -> OwnerResult<()> {
        self.runtime_registry
            .send_input(attachment, self.generation_id, bytes)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn read_terminal_runtime_output(
        &mut self,
        attachment: &PersistentTerminalAttachment,
        buffer: &mut [u8],
    ) -> OwnerResult<usize> {
        self.runtime_registry
            .read_output(attachment, self.generation_id, buffer)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn resize_terminal_runtime(
        &mut self,
        attachment: &PersistentTerminalAttachment,
        terminal_size: TerminalSize,
    ) -> OwnerResult<()> {
        self.runtime_registry
            .resize(attachment, self.generation_id, terminal_size)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn terminal_runtime_size(
        &mut self,
        attachment: &PersistentTerminalAttachment,
    ) -> OwnerResult<TerminalSize> {
        self.runtime_registry
            .current_size(attachment, self.generation_id)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn interrupt_terminal_runtime(
        &mut self,
        attachment: &PersistentTerminalAttachment,
    ) -> OwnerResult<()> {
        self.runtime_registry
            .interrupt(attachment, self.generation_id)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn terminate_terminal_runtime(
        &mut self,
        attachment: &PersistentTerminalAttachment,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<PersistentTerminalSnapshot> {
        let snapshot = self
            .runtime_registry
            .terminate(&self.store, attachment, self.generation_id, now_unix_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        self.sync_runtime_activity(now_monotonic_ms);
        Ok(snapshot)
    }

    pub(crate) fn close_terminal_runtime(
        &mut self,
        attachment: &PersistentTerminalAttachment,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<PersistentTerminalSnapshot> {
        let snapshot = self
            .runtime_registry
            .close(&self.store, attachment, self.generation_id, now_unix_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        self.sync_runtime_activity(now_monotonic_ms);
        Ok(snapshot)
    }

    pub(crate) fn poll_terminal_runtimes(
        &mut self,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<usize> {
        let observed = self
            .runtime_registry
            .poll_exits(&self.store, self.generation_id, now_unix_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        self.sync_runtime_activity(now_monotonic_ms);
        Ok(observed)
    }

    pub(crate) fn live_terminal_runtime_count(&self) -> usize {
        self.runtime_registry.live_count()
    }

    fn sync_runtime_activity(&mut self, now_monotonic_ms: u64) {
        self.activity
            .set_live_runtime_count(self.runtime_registry.live_count(), now_monotonic_ms);
    }

    pub(crate) fn should_exit(&self, now_monotonic_ms: u64) -> bool {
        self.activity.should_exit(now_monotonic_ms)
    }
}

fn reconcile_store(
    canonical_home: &Path,
    generation_id: OwnerGenerationId,
    now_unix_ms: i64,
) -> OwnerResult<(Store, usize)> {
    let store =
        Store::open(canonical_home).map_err(|error| OwnerError::Store(error.to_string()))?;
    store
        .record_persistent_runtime_owner_generation(generation_id, now_unix_ms)
        .map_err(|error| OwnerError::Store(error.to_string()))?;
    let reconciled = store
        .reconcile_persistent_runtime_records(Some(generation_id), now_unix_ms)
        .map_err(|error| OwnerError::Store(error.to_string()))?;
    Ok((store, reconciled))
}

fn prepare_owner_home(home: &Path) -> OwnerResult<PathBuf> {
    if !home.is_absolute() {
        return Err(OwnerError::HomeMustBeAbsolute);
    }
    fs::create_dir_all(home).map_err(|_| OwnerError::HomeUnavailable)?;
    let canonical = home
        .canonicalize()
        .map_err(|_| OwnerError::HomeNotCanonical)?;
    if !canonical.is_absolute() {
        return Err(OwnerError::HomeNotCanonical);
    }
    Ok(canonical)
}

pub(crate) fn run_internal_owner(home: &Path) -> OwnerResult<()> {
    let mut owner = PersistentOwner::start(home, system_unix_ms()?)?;
    let monotonic_origin = Instant::now();
    loop {
        let now_monotonic_ms = monotonic_elapsed_ms(monotonic_origin);
        owner.poll_terminal_runtimes(system_unix_ms()?, now_monotonic_ms)?;
        if owner.should_exit(now_monotonic_ms) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(OWNER_POLL_INTERVAL_MS));
    }
}

fn monotonic_elapsed_ms(origin: Instant) -> u64 {
    u64::try_from(origin.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn system_unix_ms() -> OwnerResult<i64> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| OwnerError::ClockUnavailable)?;
    i64::try_from(elapsed.as_millis()).map_err(|_| OwnerError::ClockUnavailable)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
enum OwnerEndpoint {
    Posix(crate::persistent_runtime::transport::unix::BoundUnixListener),
}

#[cfg(windows)]
enum OwnerEndpoint {
    Windows(crate::persistent_runtime::transport::windows::WindowsNamedPipeServer),
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
enum OwnerEndpoint {}

#[cfg(any(target_os = "linux", target_os = "macos"))]
struct OwnerSingleton {
    _file: std::fs::File,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
impl OwnerSingleton {
    fn acquire(home: &Path) -> OwnerResult<Self> {
        use crate::persistent_runtime::peer::current_effective_uid;
        use std::fs::{DirBuilder, OpenOptions, Permissions};
        use std::os::fd::AsRawFd;
        use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};

        const DIRECTORY_MODE: u32 = 0o700;
        const LOCK_MODE: u32 = 0o600;
        let directory = home.join("persistent-runtime");
        match fs::symlink_metadata(&directory) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink()
                    || !metadata.is_dir()
                    || metadata.uid() != current_effective_uid()
                {
                    return Err(OwnerError::SingletonSecurityMismatch);
                }
                if metadata.mode() & 0o777 != DIRECTORY_MODE {
                    fs::set_permissions(&directory, Permissions::from_mode(DIRECTORY_MODE))
                        .map_err(|error| OwnerError::SingletonIo(error.to_string()))?;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let mut builder = DirBuilder::new();
                builder.mode(DIRECTORY_MODE);
                match builder.create(&directory) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(error) => return Err(OwnerError::SingletonIo(error.to_string())),
                }
                let metadata = fs::symlink_metadata(&directory)
                    .map_err(|error| OwnerError::SingletonIo(error.to_string()))?;
                if metadata.file_type().is_symlink()
                    || !metadata.is_dir()
                    || metadata.uid() != current_effective_uid()
                    || metadata.mode() & 0o777 != DIRECTORY_MODE
                {
                    return Err(OwnerError::SingletonSecurityMismatch);
                }
            }
            Err(error) => return Err(OwnerError::SingletonIo(error.to_string())),
        }

        let lock_path = directory.join("owner.lock");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .mode(LOCK_MODE)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&lock_path)
            .map_err(|error| OwnerError::SingletonIo(error.to_string()))?;
        let file_metadata = file
            .metadata()
            .map_err(|error| OwnerError::SingletonIo(error.to_string()))?;
        let path_metadata = fs::symlink_metadata(&lock_path)
            .map_err(|error| OwnerError::SingletonIo(error.to_string()))?;
        if !file_metadata.is_file()
            || path_metadata.file_type().is_symlink()
            || !path_metadata.is_file()
            || file_metadata.uid() != current_effective_uid()
            || file_metadata.mode() & 0o777 != LOCK_MODE
            || file_metadata.dev() != path_metadata.dev()
            || file_metadata.ino() != path_metadata.ino()
        {
            return Err(OwnerError::SingletonSecurityMismatch);
        }

        // SAFETY: file owns a valid descriptor for the private regular lock file. The kernel
        // releases this advisory lock automatically when the File is dropped.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            let error = std::io::Error::last_os_error();
            if matches!(
                error.raw_os_error(),
                Some(code) if code == libc::EWOULDBLOCK || code == libc::EAGAIN
            ) {
                return Err(OwnerError::SingletonAlreadyActive);
            }
            return Err(OwnerError::SingletonIo(error.to_string()));
        }

        Ok(Self { _file: file })
    }
}

#[cfg(windows)]
struct OwnerSingleton {
    handle: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl OwnerSingleton {
    fn acquire(home: &Path) -> OwnerResult<Self> {
        use crate::persistent_runtime::peer::{
            current_process_user_sid, validate_pipe_owner_and_dacl,
        };
        use std::ffi::c_void;
        use std::mem::size_of;
        use std::os::windows::ffi::OsStrExt;
        use std::ptr::null_mut;
        use windows_sys::Win32::Foundation::{
            CloseHandle, ERROR_SHARING_VIOLATION, GENERIC_READ, GENERIC_WRITE, GetLastError,
            INVALID_HANDLE_VALUE,
        };
        use windows_sys::Win32::Security::Authorization::{
            ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
        };
        use windows_sys::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
        use windows_sys::Win32::Storage::FileSystem::{
            CreateFileW, FILE_ATTRIBUTE_NORMAL, OPEN_ALWAYS, READ_CONTROL,
        };

        struct Descriptor(PSECURITY_DESCRIPTOR);
        impl Drop for Descriptor {
            fn drop(&mut self) {
                if !self.0.is_null() {
                    // SAFETY: the descriptor was allocated by the SDDL conversion API via LocalAlloc.
                    let _ = unsafe { windows_sys::Win32::Foundation::LocalFree(self.0) };
                }
            }
        }

        let user_sid = current_process_user_sid()
            .map_err(|error| OwnerError::SingletonIo(format!("{error:?}")))?;
        let sid = user_sid
            .to_sddl_string()
            .map_err(|error| OwnerError::SingletonIo(format!("{error:?}")))?;
        let sddl = format!("O:{sid}D:P(A;;FA;;;{sid})");
        let mut sddl_wide: Vec<u16> = sddl.encode_utf16().collect();
        sddl_wide.push(0);
        let mut raw_descriptor: PSECURITY_DESCRIPTOR = null_mut();
        // SAFETY: sddl_wide is NUL-terminated and raw_descriptor is a valid writable output pointer.
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl_wide.as_ptr(),
                SDDL_REVISION_1,
                &mut raw_descriptor,
                null_mut(),
            )
        } == 0
            || raw_descriptor.is_null()
        {
            return Err(OwnerError::SingletonSecurityMismatch);
        }
        let descriptor = Descriptor(raw_descriptor);
        let security = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor.0.cast::<c_void>(),
            bInheritHandle: 0,
        };
        let lock_path = home.join("persistent-owner.lock");
        let mut wide: Vec<u16> = lock_path.as_os_str().encode_wide().collect();
        if wide.contains(&0) {
            return Err(OwnerError::SingletonIo(
                "singleton path contains NUL".to_owned(),
            ));
        }
        wide.push(0);
        // SAFETY: wide is NUL-terminated, security points to a live descriptor, share mode zero
        // gives the singleton kernel ownership property, and template handle is null.
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                GENERIC_READ | GENERIC_WRITE | READ_CONTROL,
                0,
                &security,
                OPEN_ALWAYS,
                FILE_ATTRIBUTE_NORMAL,
                null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE || handle.is_null() {
            // SAFETY: GetLastError has no preconditions.
            let code = unsafe { GetLastError() };
            if code == ERROR_SHARING_VIOLATION {
                return Err(OwnerError::SingletonAlreadyActive);
            }
            return Err(OwnerError::SingletonIo(format!("CreateFileW error {code}")));
        }
        if validate_pipe_owner_and_dacl(handle, &user_sid).is_err() {
            // SAFETY: handle is owned by this scope and has not been transferred.
            let _ = unsafe { CloseHandle(handle) };
            return Err(OwnerError::SingletonSecurityMismatch);
        }
        Ok(Self { handle })
    }
}

#[cfg(windows)]
impl Drop for OwnerSingleton {
    fn drop(&mut self) {
        if !self.handle.is_null()
            && self.handle != windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE
        {
            // SAFETY: this guard owns the singleton file handle and closes it exactly once.
            let _ = unsafe { windows_sys::Win32::Foundation::CloseHandle(self.handle) };
        }
    }
}

#[cfg(test)]
#[path = "../t151_persistent_owner_shell_tests.rs"]
mod t151_persistent_owner_shell_tests;

#[cfg(test)]
#[path = "../t152_persistent_terminal_tests.rs"]
mod t152_persistent_terminal_tests;
