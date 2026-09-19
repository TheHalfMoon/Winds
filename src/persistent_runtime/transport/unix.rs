use crate::persistent_runtime::domain::{OwnerGenerationId, RuntimeNamespaceId};
use crate::persistent_runtime::peer::{current_effective_uid, require_same_user_peer};
use crate::persistent_runtime::transport::PosixTransportError;
use std::env;
use std::fs::{self, DirBuilder, File, FileType, Metadata, OpenOptions, Permissions};
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

const RUNTIME_DIRECTORY_NAME: &str = "winds-runtime-v1";
const ENDPOINT_FILE_NAME: &str = "owner.sock";
const BIND_LOCK_FILE_NAME: &str = ".bind.lock";
const RUNTIME_DIRECTORY_MODE: u32 = 0o700;
const ENDPOINT_MODE: u32 = 0o600;
const BIND_LOCK_MODE: u32 = 0o600;
const MAX_PORTABLE_UNIX_SOCKET_PATH_BYTES: usize = 100;
const IDENTITY_ENTROPY_BYTES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EndpointIdentity {
    device: u64,
    inode: u64,
}

impl EndpointIdentity {
    fn from_metadata(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }
}

struct PendingEndpointGuard {
    path: PathBuf,
    identity: EndpointIdentity,
    armed: bool,
}

impl PendingEndpointGuard {
    fn new(path: PathBuf, identity: EndpointIdentity) -> Self {
        Self {
            path,
            identity,
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for PendingEndpointGuard {
    fn drop(&mut self) {
        if self.armed {
            remove_endpoint_if_same(&self.path, self.identity);
        }
    }
}

#[derive(Debug)]
pub(crate) struct BoundUnixListener {
    listener: UnixListener,
    endpoint_path: PathBuf,
    endpoint_identity: EndpointIdentity,
}

impl BoundUnixListener {
    pub(crate) fn bind_default() -> Result<Self, PosixTransportError> {
        let directory = resolve_runtime_directory()?;
        Self::bind(&directory)
    }

    pub(crate) fn bind(runtime_directory: &Path) -> Result<Self, PosixTransportError> {
        prepare_runtime_directory(runtime_directory)?;
        let _bind_lock = acquire_bind_lock(runtime_directory)?;
        let endpoint_path = endpoint_path(runtime_directory)?;
        prepare_endpoint_for_bind(&endpoint_path)?;

        let listener = UnixListener::bind(&endpoint_path).map_err(PosixTransportError::from)?;
        let initial_metadata =
            fs::symlink_metadata(&endpoint_path).map_err(PosixTransportError::from)?;
        if !initial_metadata.file_type().is_socket()
            || initial_metadata.uid() != current_effective_uid()
        {
            return Err(PosixTransportError::EndpointIdentityChanged);
        }
        let endpoint_identity = EndpointIdentity::from_metadata(&initial_metadata);
        let mut cleanup = PendingEndpointGuard::new(endpoint_path.clone(), endpoint_identity);

        fs::set_permissions(&endpoint_path, Permissions::from_mode(ENDPOINT_MODE))
            .map_err(PosixTransportError::from)?;
        let metadata = validate_endpoint_metadata(&endpoint_path, current_effective_uid())?;
        if EndpointIdentity::from_metadata(&metadata) != endpoint_identity {
            return Err(PosixTransportError::EndpointIdentityChanged);
        }
        cleanup.disarm();

        Ok(Self {
            listener,
            endpoint_path,
            endpoint_identity,
        })
    }

    pub(crate) fn endpoint_path(&self) -> &Path {
        &self.endpoint_path
    }

    pub(crate) fn accept_same_user(&self) -> Result<UnixStream, PosixTransportError> {
        let (stream, _) = self.listener.accept().map_err(PosixTransportError::from)?;
        require_same_user_peer(&stream)?;
        Ok(stream)
    }
}

impl Drop for BoundUnixListener {
    fn drop(&mut self) {
        remove_endpoint_if_same(&self.endpoint_path, self.endpoint_identity);
    }
}

fn remove_endpoint_if_same(path: &Path, identity: EndpointIdentity) {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return;
    };
    if !metadata.file_type().is_socket() {
        return;
    }
    if EndpointIdentity::from_metadata(&metadata) != identity {
        return;
    }
    let _ = fs::remove_file(path);
}

pub(crate) fn connect_default_same_user() -> Result<UnixStream, PosixTransportError> {
    let directory = resolve_runtime_directory()?;
    connect_same_user(&directory)
}

pub(crate) fn connect_same_user(
    runtime_directory: &Path,
) -> Result<UnixStream, PosixTransportError> {
    validate_runtime_directory(runtime_directory, current_effective_uid())?;
    let endpoint = endpoint_path(runtime_directory)?;
    validate_endpoint_metadata(&endpoint, current_effective_uid())?;
    let stream = UnixStream::connect(endpoint).map_err(PosixTransportError::from)?;
    require_same_user_peer(&stream)?;
    Ok(stream)
}

pub(crate) fn resolve_runtime_directory() -> Result<PathBuf, PosixTransportError> {
    let uid = current_effective_uid();
    let preferred_base = preferred_runtime_base();
    resolve_runtime_directory_from(preferred_base.as_deref(), Path::new("/tmp"), uid)
}

fn preferred_runtime_base() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        if let Some(value) = env::var_os("XDG_RUNTIME_DIR") {
            let path = PathBuf::from(value);
            if path.is_absolute() {
                return Some(path);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(value) = env::var_os("TMPDIR") {
            let path = PathBuf::from(value);
            if path.is_absolute() {
                return Some(path);
            }
        }
    }

    None
}

fn resolve_runtime_directory_from(
    preferred_base: Option<&Path>,
    fallback_base: &Path,
    uid: u32,
) -> Result<PathBuf, PosixTransportError> {
    if let Some(preferred_base) = preferred_base
        && let Ok(canonical_preferred_base) = preferred_base.canonicalize()
        && preferred_base_is_private(&canonical_preferred_base, uid)
    {
        let preferred = canonical_preferred_base.join(RUNTIME_DIRECTORY_NAME);
        if preferred.is_absolute() && socket_path_fits(&preferred.join(ENDPOINT_FILE_NAME)) {
            return Ok(preferred);
        }
    }

    let canonical_fallback_base = fallback_base
        .canonicalize()
        .map_err(PosixTransportError::from)?;
    let fallback = canonical_fallback_base.join(format!("winds-runtime-{uid}"));
    if !fallback.is_absolute() {
        return Err(PosixTransportError::RuntimeDirectoryNotAbsolute);
    }
    if !socket_path_fits(&fallback.join(ENDPOINT_FILE_NAME)) {
        return Err(PosixTransportError::EndpointPathTooLong);
    }
    Ok(fallback)
}

fn preferred_base_is_private(path: &Path, expected_uid: u32) -> bool {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return false;
    };
    metadata.file_type().is_dir()
        && !metadata.file_type().is_symlink()
        && metadata.uid() == expected_uid
        && metadata.mode() & 0o300 == 0o300
        && metadata.mode() & 0o022 == 0
}

pub(crate) fn prepare_runtime_directory(path: &Path) -> Result<(), PosixTransportError> {
    validate_canonical_runtime_path(path)?;
    match fs::symlink_metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut builder = DirBuilder::new();
            builder.mode(RUNTIME_DIRECTORY_MODE);
            match builder.create(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(PosixTransportError::from(error)),
            }
        }
        Err(error) => return Err(PosixTransportError::from(error)),
    }

    validate_runtime_directory(path, current_effective_uid())
}

fn validate_runtime_directory(path: &Path, expected_uid: u32) -> Result<(), PosixTransportError> {
    validate_canonical_runtime_path(path)?;
    let metadata = fs::symlink_metadata(path).map_err(PosixTransportError::from)?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return Err(PosixTransportError::RuntimeDirectorySymlink);
    }
    if !file_type.is_dir() {
        return Err(PosixTransportError::RuntimeDirectoryNotDirectory);
    }
    validate_directory_facts(metadata.uid(), metadata.mode(), expected_uid)
}

fn validate_canonical_runtime_path(path: &Path) -> Result<(), PosixTransportError> {
    if !path.is_absolute() {
        return Err(PosixTransportError::RuntimeDirectoryNotAbsolute);
    }
    let parent = path
        .parent()
        .ok_or(PosixTransportError::RuntimeDirectoryMissingParent)?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|_| PosixTransportError::RuntimeDirectoryMissingParent)?;
    if canonical_parent != parent {
        return Err(PosixTransportError::RuntimeDirectoryPathAlias);
    }
    Ok(())
}

fn validate_directory_facts(
    observed_uid: u32,
    observed_mode: u32,
    expected_uid: u32,
) -> Result<(), PosixTransportError> {
    if observed_uid != expected_uid {
        return Err(PosixTransportError::RuntimeDirectoryOwnershipMismatch);
    }
    if observed_mode & 0o777 != RUNTIME_DIRECTORY_MODE {
        return Err(PosixTransportError::RuntimeDirectoryModeMismatch);
    }
    Ok(())
}

fn acquire_bind_lock(runtime_directory: &Path) -> Result<File, PosixTransportError> {
    let path = runtime_directory.join(BIND_LOCK_FILE_NAME);
    let mut options = OpenOptions::new();
    options
        .read(true)
        .write(true)
        .create(true)
        .mode(BIND_LOCK_MODE)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let file = options
        .open(&path)
        .map_err(|_| PosixTransportError::BindLockUnavailable)?;
    validate_bind_lock_file(&file, &path)?;

    // SAFETY: file owns a valid descriptor for the private regular lock file. flock only uses the
    // descriptor and releases the advisory lock automatically when this File is dropped.
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
        return Err(PosixTransportError::BindLockUnavailable);
    }
    validate_bind_lock_file(&file, &path)?;
    Ok(file)
}

fn validate_bind_lock_file(file: &File, path: &Path) -> Result<(), PosixTransportError> {
    let file_metadata = file
        .metadata()
        .map_err(|_| PosixTransportError::BindLockUnavailable)?;
    if !file_metadata.is_file()
        || file_metadata.uid() != current_effective_uid()
        || file_metadata.mode() & 0o777 != BIND_LOCK_MODE
    {
        return Err(PosixTransportError::BindLockUnavailable);
    }
    let path_metadata =
        fs::symlink_metadata(path).map_err(|_| PosixTransportError::BindLockUnavailable)?;
    if path_metadata.file_type().is_symlink()
        || !path_metadata.is_file()
        || EndpointIdentity::from_metadata(&path_metadata)
            != EndpointIdentity::from_metadata(&file_metadata)
    {
        return Err(PosixTransportError::BindLockUnavailable);
    }
    Ok(())
}

pub(crate) fn endpoint_path(runtime_directory: &Path) -> Result<PathBuf, PosixTransportError> {
    if !runtime_directory.is_absolute() {
        return Err(PosixTransportError::RuntimeDirectoryNotAbsolute);
    }
    let endpoint = runtime_directory.join(ENDPOINT_FILE_NAME);
    if !socket_path_fits(&endpoint) {
        return Err(PosixTransportError::EndpointPathTooLong);
    }
    Ok(endpoint)
}

fn socket_path_fits(path: &Path) -> bool {
    path.as_os_str().as_bytes().len() <= MAX_PORTABLE_UNIX_SOCKET_PATH_BYTES
}

fn prepare_endpoint_for_bind(path: &Path) -> Result<(), PosixTransportError> {
    let expected_uid = current_effective_uid();
    let existing = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(PosixTransportError::from(error)),
    };
    validate_endpoint_file_type(&existing.file_type())?;
    validate_endpoint_facts(existing.uid(), existing.mode(), expected_uid)?;
    let existing_identity = EndpointIdentity::from_metadata(&existing);

    match UnixStream::connect(path) {
        Ok(stream) => {
            let _ = require_same_user_peer(&stream);
            Err(PosixTransportError::LiveEndpointCollision)
        }
        Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => {
            let current = fs::symlink_metadata(path).map_err(PosixTransportError::from)?;
            validate_endpoint_file_type(&current.file_type())?;
            validate_endpoint_facts(current.uid(), current.mode(), expected_uid)?;
            if EndpointIdentity::from_metadata(&current) != existing_identity {
                return Err(PosixTransportError::EndpointIdentityChanged);
            }
            fs::remove_file(path).map_err(PosixTransportError::from)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(PosixTransportError::StaleEndpointUnproven),
    }
}

fn validate_endpoint_metadata(
    path: &Path,
    expected_uid: u32,
) -> Result<Metadata, PosixTransportError> {
    let metadata = fs::symlink_metadata(path).map_err(PosixTransportError::from)?;
    validate_endpoint_file_type(&metadata.file_type())?;
    validate_endpoint_facts(metadata.uid(), metadata.mode(), expected_uid)?;
    Ok(metadata)
}

fn validate_endpoint_file_type(file_type: &FileType) -> Result<(), PosixTransportError> {
    if file_type.is_symlink() {
        return Err(PosixTransportError::EndpointSymlink);
    }
    if !file_type.is_socket() {
        return Err(PosixTransportError::EndpointCollision);
    }
    Ok(())
}

fn validate_endpoint_facts(
    observed_uid: u32,
    observed_mode: u32,
    expected_uid: u32,
) -> Result<(), PosixTransportError> {
    if observed_uid != expected_uid {
        return Err(PosixTransportError::EndpointOwnershipMismatch);
    }
    if observed_mode & 0o777 != ENDPOINT_MODE {
        return Err(PosixTransportError::EndpointModeMismatch);
    }
    Ok(())
}

pub(crate) fn generate_runtime_namespace_id() -> Result<RuntimeNamespaceId, PosixTransportError> {
    let bytes = os_entropy_128()?;
    RuntimeNamespaceId::from_entropy_bytes(bytes).map_err(|_| PosixTransportError::EntropyInvalid)
}

pub(crate) fn generate_owner_generation_id() -> Result<OwnerGenerationId, PosixTransportError> {
    let bytes = os_entropy_128()?;
    OwnerGenerationId::from_entropy_bytes(bytes).map_err(|_| PosixTransportError::EntropyInvalid)
}

fn entropy_128_with<F>(mut fill: F) -> Result<[u8; IDENTITY_ENTROPY_BYTES], PosixTransportError>
where
    F: FnMut(&mut [u8]) -> Result<usize, PosixTransportError>,
{
    let mut bytes = [0_u8; IDENTITY_ENTROPY_BYTES];
    let written = fill(&mut bytes)?;
    if written != IDENTITY_ENTROPY_BYTES {
        return Err(PosixTransportError::EntropyShortRead);
    }
    if bytes.iter().all(|byte| *byte == 0) {
        return Err(PosixTransportError::EntropyInvalid);
    }
    Ok(bytes)
}

#[cfg(target_os = "linux")]
fn os_entropy_128() -> Result<[u8; IDENTITY_ENTROPY_BYTES], PosixTransportError> {
    entropy_128_with(|bytes| {
        // SAFETY: bytes points to writable storage of exactly bytes.len() bytes. getrandom writes
        // at most that length and does not retain the pointer.
        let result = unsafe { libc::getrandom(bytes.as_mut_ptr().cast(), bytes.len(), 0) };
        if result < 0 {
            Err(PosixTransportError::EntropyUnavailable)
        } else {
            Ok(result as usize)
        }
    })
}

#[cfg(target_os = "macos")]
fn os_entropy_128() -> Result<[u8; IDENTITY_ENTROPY_BYTES], PosixTransportError> {
    entropy_128_with(|bytes| {
        // SAFETY: bytes points to writable storage of exactly bytes.len() bytes. getentropy fills
        // the requested buffer on success and does not retain the pointer.
        let result = unsafe { libc::getentropy(bytes.as_mut_ptr().cast(), bytes.len()) };
        if result != 0 {
            Err(PosixTransportError::EntropyUnavailable)
        } else {
            Ok(bytes.len())
        }
    })
}

#[cfg(test)]
#[path = "../../t149_posix_private_transport_tests.rs"]
mod t149_posix_private_transport_tests;
