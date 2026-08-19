use super::workspace::{WorkspaceInspection, inspect_existing_workspace};
use super::{Result, git_command};
use crate::store::{NewWorkspace, Store};
use serde::Serialize;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::ffi::CString;
use std::ffi::OsString;
use std::fs;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;
#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(windows)]
use std::{ffi::c_void, mem::MaybeUninit};

static NEXT_CLONE_STAGING_ID: AtomicU64 = AtomicU64::new(0);
const MAX_STAGING_ATTEMPTS: usize = 128;

#[cfg(unix)]
type ClonePathIdentity = (u64, u64);
#[cfg(windows)]
type ClonePathIdentity = (u64, [u8; 16]);
#[cfg(not(any(unix, windows)))]
type ClonePathIdentity = ();

#[derive(Debug)]
struct OwnedCloneStaging {
    path: PathBuf,
    identity: ClonePathIdentity,
    // Holding the original Unix directory open pins its inode until this
    // staging owner is dropped. That prevents delete+recreate from being
    // accepted through immediate inode-number reuse.
    #[cfg(unix)]
    _identity_handle: fs::File,
}

#[allow(
    dead_code,
    reason = "Spec 003 T046 backend API; the user-facing CLI caller lands in T057"
)]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ClonedWorkspace {
    pub workspace: WorkspaceInspection,
    pub remote_identity: String,
}

#[allow(
    dead_code,
    reason = "Spec 003 T046 backend API; the user-facing CLI caller lands in T057"
)]
pub fn clone_and_register_workspace(
    remote: &str,
    destination: &Path,
    canonical_state_root: &Path,
    now_ms: i64,
) -> Result<ClonedWorkspace> {
    clone_and_register_workspace_impl(remote, destination, canonical_state_root, now_ms, |_, _| {
        Ok(())
    })
}

fn clone_and_register_workspace_impl<F>(
    remote: &str,
    destination: &Path,
    canonical_state_root: &Path,
    now_ms: i64,
    after_staging_created: F,
) -> Result<ClonedWorkspace>
where
    F: FnOnce(&Path, &Path) -> Result<()>,
{
    let remote_identity = sanitize_remote_identity(remote)?;
    let planned_destination = plan_clone_destination(destination, canonical_state_root)?;
    let parent = planned_destination
        .parent()
        .ok_or("clone destination has no parent directory")?;
    require_no_retained_clone_payload(parent)?;
    let git_remote = git_remote_argument(remote, &remote_identity)?;
    let staging = create_private_clone_staging(parent)?;
    let staged_checkout = staging.path.join("checkout");
    let git_destination = match git_cli_local_path(&staged_checkout) {
        Ok(destination) => destination,
        Err(error) => {
            return fail_with_owned_staging_cleanup(
                format!("clone destination could not be prepared for system Git: {error}"),
                &staging,
            );
        }
    };

    if let Err(error) = after_staging_created(&staged_checkout, &planned_destination) {
        return fail_with_owned_staging_cleanup(
            format!("clone staging callback failed before Git clone: {error}"),
            &staging,
        );
    }

    if let Err(error) =
        require_clone_directory_identity(&staging.path, &staging.identity, "private clone staging")
    {
        return fail_with_owned_staging_cleanup(
            format!("private clone staging ownership changed before Git clone: {error}"),
            &staging,
        );
    }

    let status = match git_command(&staging.path)
        .arg("-c")
        .arg("core.askPass=")
        .arg("clone")
        .arg("--")
        .arg(&git_remote)
        .arg(&git_destination)
        .env_remove("GIT_ASKPASS")
        .env_remove("SSH_ASKPASS")
        .env_remove("GIT_SSH")
        .env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes")
        .env("GIT_SSH_VARIANT", "ssh")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        Ok(status) => status,
        Err(error) => {
            return fail_with_owned_staging_cleanup(
                format!(
                    "system Git clone could not be started or observed: {error}; requested destination was not published or registered"
                ),
                &staging,
            );
        }
    };
    if !status.success() {
        let status = status
            .code()
            .map_or_else(|| "signal".to_owned(), |code| code.to_string());
        return fail_with_owned_staging_cleanup(
            format!(
                "system Git clone failed with status {status}; requested destination was not published or registered"
            ),
            &staging,
        );
    }

    let checkout_identity = match require_owned_staged_checkout(&staging, &staged_checkout) {
        Ok(identity) => identity,
        Err(error) => {
            return fail_with_owned_staging_cleanup(
                format!(
                    "cloned checkout failed private staging validation; requested destination was not published or registered: {error}"
                ),
                &staging,
            );
        }
    };

    if let Err(error) =
        require_clone_directory_identity(&staging.path, &staging.identity, "private clone staging")
    {
        return fail_with_owned_staging_cleanup(
            format!("private clone staging ownership changed before publication: {error}"),
            &staging,
        );
    }
    if let Err(error) = require_clone_directory_identity(
        &staged_checkout,
        &checkout_identity,
        "staged clone checkout",
    ) {
        return fail_with_owned_staging_cleanup(
            format!("staged clone checkout ownership changed before publication: {error}"),
            &staging,
        );
    }

    if let Err(error) = atomic_publish_no_replace(&staged_checkout, &planned_destination) {
        return fail_with_owned_staging_cleanup(
            format!(
                "cloned checkout could not be atomically published without replacing the requested destination; requested destination was not registered: {error}"
            ),
            &staging,
        );
    }

    let published_identity = match clone_directory_identity(
        &planned_destination,
        "published clone destination",
    ) {
        Ok(identity) => identity,
        Err(error) => {
            return fail_after_publication(
                format!(
                    "published clone destination identity could not be proven after atomic publication; destination was not registered and was retained for recovery: {error}"
                ),
                &staging,
            );
        }
    };
    if published_identity != checkout_identity {
        return fail_after_publication(
            "published clone destination filesystem identity does not match the approved staged checkout; destination was not registered and was retained for recovery".to_owned(),
            &staging,
        );
    }

    if let Err(error) = retain_empty_owned_clone_staging(&staging) {
        return Err(format!(
            "atomically published clone staging retention could not be proven safely; destination was not registered and was retained for recovery: {error}"
        )
        .into());
    }

    let workspace = inspect_existing_workspace(&planned_destination, canonical_state_root)?;
    if Path::new(&workspace.canonical_worktree_root) != planned_destination {
        return Err(
            "cloned workspace canonical root does not match the atomically published clone destination"
                .into(),
        );
    }
    let mut store = Store::open(canonical_state_root)?;
    require_clone_directory_identity(
        &planned_destination,
        &checkout_identity,
        "published clone destination",
    )
    .map_err(|error| {
        format!(
            "published clone destination changed filesystem identity before registration; destination was not registered and was retained for recovery: {error}"
        )
    })?;
    store.register_cloned_workspace(
        NewWorkspace {
            workspace_id: &workspace.workspace_id,
            canonical_worktree_root: &workspace.canonical_worktree_root,
            git_common_dir: &workspace.git_common_dir,
        },
        &remote_identity,
        now_ms,
    )?;

    Ok(ClonedWorkspace {
        workspace,
        remote_identity,
    })
}

fn plan_clone_destination(destination: &Path, canonical_state_root: &Path) -> Result<PathBuf> {
    if !destination.is_absolute() {
        return Err("clone destination must be an absolute path".into());
    }

    let state_root = canonical_state_root
        .canonicalize()
        .map_err(|error| format!("Winds state root cannot be canonicalized: {error}"))?;
    if state_root != canonical_state_root {
        return Err("Winds state root must be supplied in canonical form".into());
    }
    if !state_root.is_dir() {
        return Err("Winds state root is not a directory".into());
    }

    let parent = destination
        .parent()
        .ok_or("clone destination has no parent directory")?;
    let file_name = destination
        .file_name()
        .ok_or("clone destination must name a directory")?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|error| format!("clone destination parent cannot be canonicalized: {error}"))?;
    if !canonical_parent.is_dir() {
        return Err("clone destination parent is not a directory".into());
    }

    let planned = canonical_parent.join(file_name);
    match fs::symlink_metadata(&planned) {
        Ok(_) => {
            return Err(format!("clone destination already exists: {}", planned.display()).into());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "clone destination cannot be inspected before clone: {}: {error}",
                planned.display()
            )
            .into());
        }
    }

    if planned.starts_with(&state_root) || state_root.starts_with(&planned) {
        return Err("clone destination and Winds state root must not overlap".into());
    }

    Ok(planned)
}

fn require_no_retained_clone_payload(parent: &Path) -> Result<()> {
    let entries = fs::read_dir(parent).map_err(|error| {
        format!(
            "clone destination parent cannot be inspected for retained private staging: {error}"
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            format!("clone destination parent contains an unreadable entry: {error}")
        })?;
        let name = entry.file_name();
        if !name.as_encoded_bytes().starts_with(b".winds-clone-stage-") {
            continue;
        }

        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            format!(
                "retained private clone staging candidate {} cannot be inspected: {error}",
                path.display()
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(format!(
                "clone destination parent contains an ambiguous Winds staging entry {}; refusing a new clone until it is inspected and recovered manually",
                path.display()
            )
            .into());
        }

        let mut contents = fs::read_dir(&path).map_err(|error| {
            format!(
                "retained private clone staging {} cannot be inspected safely: {error}",
                path.display()
            )
        })?;
        if contents.next().is_some() {
            return Err(format!(
                "retained private clone staging {} contains clone payload from an earlier failed operation; refusing to allocate another staging payload under the same parent until manual recovery prevents unbounded disk growth",
                path.display()
            )
            .into());
        }
    }

    Ok(())
}

fn create_private_clone_staging(parent: &Path) -> Result<OwnedCloneStaging> {
    for _ in 0..MAX_STAGING_ATTEMPTS {
        let sequence = NEXT_CLONE_STAGING_ID.fetch_add(1, Ordering::Relaxed);
        let staging = parent.join(format!(
            ".winds-clone-stage-{}-{sequence}",
            std::process::id()
        ));
        let mut builder = fs::DirBuilder::new();
        builder.recursive(false);
        #[cfg(unix)]
        builder.mode(0o700);
        match builder.create(&staging) {
            Ok(()) => {
                let canonical = staging.canonicalize().map_err(|error| {
                    format!("private clone staging cannot be canonicalized: {error}")
                })?;
                if canonical != staging {
                    return Err("private clone staging changed identity during creation".into());
                }
                #[cfg(unix)]
                let identity_handle = fs::File::open(&staging).map_err(|error| {
                    format!(
                        "private clone staging could not be pinned by an open directory handle after creation; staging was retained at {}: {error}",
                        staging.display()
                    )
                })?;
                #[cfg(unix)]
                let identity =
                    clone_directory_identity_from_handle(&identity_handle, "private clone staging")
                        .map_err(|error| {
                            format!(
                                "private clone staging filesystem identity could not be captured from its pinned handle after creation; staging was retained at {}: {error}",
                                staging.display()
                            )
                        })?;
                #[cfg(unix)]
                require_clone_directory_identity(&staging, &identity, "private clone staging")
                    .map_err(|error| {
                        format!(
                            "private clone staging path no longer matches its pinned creation handle; staging was retained at {}: {error}",
                            staging.display()
                        )
                    })?;

                #[cfg(not(unix))]
                let identity = clone_directory_identity(&staging, "private clone staging")
                    .map_err(|error| {
                        format!(
                            "private clone staging filesystem identity could not be captured after creation; staging was retained at {}: {error}",
                            staging.display()
                        )
                    })?;

                return Ok(OwnedCloneStaging {
                    path: staging,
                    identity,
                    #[cfg(unix)]
                    _identity_handle: identity_handle,
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "failed to create private clone staging under {}: {error}",
                    parent.display()
                )
                .into());
            }
        }
    }
    Err("could not allocate a unique private clone staging directory".into())
}

#[cfg(unix)]
fn clone_directory_identity_from_handle(
    handle: &fs::File,
    label: &str,
) -> Result<ClonePathIdentity> {
    let metadata = handle
        .metadata()
        .map_err(|error| format!("{label} pinned handle cannot be inspected: {error}"))?;
    if !metadata.is_dir() {
        return Err(format!("{label} pinned handle is not a directory").into());
    }
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(unix)]
fn clone_directory_identity(path: &Path, label: &str) -> Result<ClonePathIdentity> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("{label} cannot be inspected: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("{label} is not a real directory").into());
    }
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
fn clone_directory_identity(path: &Path, label: &str) -> Result<ClonePathIdentity> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("{label} cannot be inspected: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("{label} is not a real directory").into());
    }
    windows_directory_identity(path, label)
}

#[cfg(not(any(unix, windows)))]
fn clone_directory_identity(_path: &Path, label: &str) -> Result<ClonePathIdentity> {
    Err(format!("{label} filesystem identity is unsupported on this platform").into())
}

fn require_clone_directory_identity(
    path: &Path,
    expected: &ClonePathIdentity,
    label: &str,
) -> Result<()> {
    let current = clone_directory_identity(path, label)?;
    if current != *expected {
        return Err(format!("{label} filesystem identity changed").into());
    }
    Ok(())
}

fn require_owned_staged_checkout(
    staging: &OwnedCloneStaging,
    staged_checkout: &Path,
) -> Result<ClonePathIdentity> {
    require_clone_directory_identity(&staging.path, &staging.identity, "private clone staging")?;
    let canonical_staging = staging
        .path
        .canonicalize()
        .map_err(|error| format!("private clone staging cannot be canonicalized: {error}"))?;
    if canonical_staging != staging.path {
        return Err("private clone staging path is no longer canonical".into());
    }

    let checkout_metadata = fs::symlink_metadata(staged_checkout)
        .map_err(|error| format!("staged clone checkout cannot be inspected: {error}"))?;
    if checkout_metadata.file_type().is_symlink() || !checkout_metadata.is_dir() {
        return Err("staged clone checkout is not a real directory".into());
    }
    let canonical_checkout = staged_checkout
        .canonicalize()
        .map_err(|error| format!("staged clone checkout cannot be canonicalized: {error}"))?;
    if canonical_checkout != staged_checkout
        || canonical_checkout.parent() != Some(canonical_staging.as_path())
    {
        return Err("staged clone checkout escaped its private staging parent".into());
    }
    clone_directory_identity(staged_checkout, "staged clone checkout")
}

fn retain_owned_clone_staging(staging: &OwnedCloneStaging) -> Result<()> {
    retain_owned_clone_staging_impl(staging, || Ok(()))
}

fn retain_owned_clone_staging_impl<F>(
    staging: &OwnedCloneStaging,
    after_identity_proven: F,
) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    require_clone_directory_identity(
        &staging.path,
        &staging.identity,
        "private clone staging",
    )
    .map_err(|error| {
        format!(
            "private clone staging ownership is ambiguous; refusing recursive cleanup and retaining {}: {error}",
            staging.path.display()
        )
    })?;

    // No destructive operation follows this proof. Production passes a
    // no-op; the regression swaps the pathname here to prove that even a
    // post-proof replacement is retained untouched.
    after_identity_proven()?;
    Ok(())
}

fn retain_empty_owned_clone_staging(staging: &OwnedCloneStaging) -> Result<()> {
    require_clone_directory_identity(
        &staging.path,
        &staging.identity,
        "private clone staging",
    )
    .map_err(|error| {
        format!(
            "empty private clone staging ownership is ambiguous; retaining {} without unlink: {error}",
            staging.path.display()
        )
        .into()
    })
}

fn fail_with_owned_staging_cleanup<T>(primary: String, staging: &OwnedCloneStaging) -> Result<T> {
    match retain_owned_clone_staging(staging) {
        Ok(()) => Err(format!(
            "{primary}; private clone staging was retained for recovery at {} because recursive deletion cannot be bound safely to stable filesystem objects on every supported platform",
            staging.path.display()
        )
        .into()),
        Err(identity_error) => Err(format!(
            "{primary}; private clone staging ownership is ambiguous, so Winds refused recursive cleanup and retained the staging path without mutation: {identity_error}"
        )
        .into()),
    }
}

fn fail_after_publication<T>(primary: String, staging: &OwnedCloneStaging) -> Result<T> {
    match retain_empty_owned_clone_staging(staging) {
        Ok(()) => Err(format!(
            "{primary}; empty private clone staging shell was retained at {} because Winds does not unlink the root through a mutable parent pathname",
            staging.path.display()
        )
        .into()),
        Err(retention_error) => Err(format!(
            "{primary}; private staging retention proof also failed and the staging path was left untouched: {retention_error}"
        )
        .into()),
    }
}

#[cfg(target_os = "linux")]
fn atomic_publish_no_replace(source: &Path, destination: &Path) -> Result<()> {
    let source = unix_path_cstring(source, "staged clone source")?;
    let destination = unix_path_cstring(destination, "clone destination")?;
    let result = unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            source.as_ptr(),
            libc::AT_FDCWD,
            destination.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(format!(
            "atomic no-replace clone publish failed: {}",
            std::io::Error::last_os_error()
        )
        .into())
    }
}

#[cfg(target_os = "macos")]
fn atomic_publish_no_replace(source: &Path, destination: &Path) -> Result<()> {
    let source = unix_path_cstring(source, "staged clone source")?;
    let destination = unix_path_cstring(destination, "clone destination")?;
    let result =
        unsafe { libc::renamex_np(source.as_ptr(), destination.as_ptr(), libc::RENAME_EXCL) };
    if result == 0 {
        Ok(())
    } else {
        Err(format!(
            "atomic no-replace clone publish failed: {}",
            std::io::Error::last_os_error()
        )
        .into())
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn unix_path_cstring(path: &Path, label: &str) -> Result<CString> {
    CString::new(path.as_os_str().as_bytes())
        .map_err(|_| format!("{label} contains an embedded NUL byte").into())
}

#[cfg(windows)]
const WINDOWS_FILE_ATTRIBUTE_DIRECTORY: u32 = 0x0000_0010;
#[cfg(windows)]
const WINDOWS_FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
#[cfg(windows)]
const WINDOWS_FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
#[cfg(windows)]
const WINDOWS_FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
#[cfg(windows)]
const WINDOWS_FILE_ATTRIBUTE_TAG_INFO_CLASS: i32 = 9;
#[cfg(windows)]
const WINDOWS_FILE_ID_INFO_CLASS: i32 = 18;

#[cfg(windows)]
#[repr(C)]
struct WindowsFileAttributeTagInfo {
    file_attributes: u32,
    _reparse_tag: u32,
}

#[cfg(windows)]
#[repr(C)]
struct WindowsFileIdInfo {
    volume_serial_number: u64,
    file_id: [u8; 16],
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn MoveFileExW(existing_file_name: *const u16, new_file_name: *const u16, flags: u32) -> i32;
    fn GetFileInformationByHandleEx(
        file_handle: *mut c_void,
        file_information_class: i32,
        file_information: *mut c_void,
        buffer_size: u32,
    ) -> i32;
}

#[cfg(windows)]
fn windows_directory_identity(path: &Path, label: &str) -> Result<ClonePathIdentity> {
    let handle = fs::OpenOptions::new()
        .access_mode(0)
        .custom_flags(WINDOWS_FILE_FLAG_OPEN_REPARSE_POINT | WINDOWS_FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)
        .map_err(|error| format!("{label} cannot be opened for identity inspection: {error}"))?;

    let mut attribute_info = MaybeUninit::<WindowsFileAttributeTagInfo>::uninit();
    let attribute_result = unsafe {
        GetFileInformationByHandleEx(
            handle.as_raw_handle(),
            WINDOWS_FILE_ATTRIBUTE_TAG_INFO_CLASS,
            attribute_info.as_mut_ptr().cast::<c_void>(),
            std::mem::size_of::<WindowsFileAttributeTagInfo>() as u32,
        )
    };
    if attribute_result == 0 {
        return Err(format!(
            "{label} handle attributes cannot be inspected: {}",
            std::io::Error::last_os_error()
        )
        .into());
    }
    let attribute_info = unsafe { attribute_info.assume_init() };
    if attribute_info.file_attributes & WINDOWS_FILE_ATTRIBUTE_REPARSE_POINT != 0
        || attribute_info.file_attributes & WINDOWS_FILE_ATTRIBUTE_DIRECTORY == 0
    {
        return Err(format!("{label} handle is a reparse point or not a real directory").into());
    }

    let mut identity_info = MaybeUninit::<WindowsFileIdInfo>::uninit();
    let identity_result = unsafe {
        GetFileInformationByHandleEx(
            handle.as_raw_handle(),
            WINDOWS_FILE_ID_INFO_CLASS,
            identity_info.as_mut_ptr().cast::<c_void>(),
            std::mem::size_of::<WindowsFileIdInfo>() as u32,
        )
    };
    if identity_result == 0 {
        return Err(format!(
            "{label} filesystem identity cannot be inspected: {}",
            std::io::Error::last_os_error()
        )
        .into());
    }
    let identity_info = unsafe { identity_info.assume_init() };
    Ok((identity_info.volume_serial_number, identity_info.file_id))
}

#[cfg(windows)]
fn atomic_publish_no_replace(source: &Path, destination: &Path) -> Result<()> {
    let source = windows_path_wide(source, "staged clone source")?;
    let destination = windows_path_wide(destination, "clone destination")?;
    let result = unsafe { MoveFileExW(source.as_ptr(), destination.as_ptr(), 0) };
    if result != 0 {
        Ok(())
    } else {
        Err(format!(
            "atomic no-replace clone publish failed: {}",
            std::io::Error::last_os_error()
        )
        .into())
    }
}

#[cfg(windows)]
fn windows_path_wide(path: &Path, label: &str) -> Result<Vec<u16>> {
    let mut encoded = path.as_os_str().encode_wide().collect::<Vec<_>>();
    if encoded.contains(&0) {
        return Err(format!("{label} contains an embedded NUL code unit").into());
    }
    encoded.push(0);
    Ok(encoded)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
fn atomic_publish_no_replace(_source: &Path, _destination: &Path) -> Result<()> {
    Err("atomic no-replace clone publish is unsupported on this platform".into())
}

fn git_remote_argument(remote: &str, remote_identity: &str) -> Result<OsString> {
    if Path::new(remote).is_absolute() {
        return Ok(git_cli_local_path(Path::new(remote_identity))?.into_os_string());
    }
    Ok(OsString::from(remote))
}

#[cfg(not(windows))]
fn git_cli_local_path(path: &Path) -> Result<PathBuf> {
    Ok(path.to_path_buf())
}

#[cfg(windows)]
fn git_cli_local_path(path: &Path) -> Result<PathBuf> {
    let value = path.to_str().ok_or("local Git path is not valid UTF-8")?;
    if let Some(rest) = value.strip_prefix(r"\\?\UNC\") {
        let mut components = rest.split('\\');
        let server = components.next().unwrap_or_default();
        let share = components.next().unwrap_or_default();
        if server.is_empty() || share.is_empty() {
            return Err(
                "Windows verbatim UNC path must include non-empty server and share components"
                    .into(),
            );
        }
        return Ok(PathBuf::from(format!(r"\\{rest}")));
    }
    if let Some(rest) = value.strip_prefix(r"\\?\") {
        let bytes = rest.as_bytes();
        let ordinary_drive_path = bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'\\' | b'/');
        if !ordinary_drive_path {
            return Err(
                "Windows verbatim local Git path cannot be represented safely for Git CLI".into(),
            );
        }
        return Ok(PathBuf::from(rest));
    }
    Ok(path.to_path_buf())
}

fn sanitize_remote_identity(remote: &str) -> Result<String> {
    if remote.is_empty() {
        return Err("clone remote must not be empty".into());
    }
    if remote.chars().any(char::is_control) {
        return Err("clone remote contains control characters".into());
    }

    let local_path = Path::new(remote);
    if local_path.is_absolute() {
        let canonical = local_path
            .canonicalize()
            .map_err(|error| format!("local clone remote cannot be canonicalized: {error}"))?;
        return canonical
            .to_str()
            .map(str::to_owned)
            .ok_or_else(|| "local clone remote is not valid UTF-8".into());
    }

    if let Some((scheme, rest)) = remote.split_once("://") {
        return sanitize_url_remote(scheme, rest);
    }

    if remote.contains("::") {
        return Err("Git remote-helper transport syntax is not supported by Spec 003 T046".into());
    }

    if let Some(sanitized) = sanitize_scp_like_remote(remote) {
        return Ok(sanitized);
    }

    Err(
        "relative local clone remotes are ambiguous; use an absolute path or explicit Git URL"
            .into(),
    )
}

fn sanitize_url_remote(scheme: &str, rest: &str) -> Result<String> {
    let valid_scheme = !scheme.is_empty()
        && scheme.chars().enumerate().all(|(index, ch)| {
            if index == 0 {
                ch.is_ascii_alphabetic()
            } else {
                ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.')
            }
        });
    if !valid_scheme {
        return Err("clone remote has an invalid URL scheme".into());
    }

    let scheme = scheme.to_ascii_lowercase();
    if !matches!(scheme.as_str(), "http" | "https" | "ssh" | "git" | "file") {
        return Err("clone remote URL scheme is not supported by Spec 003 T046".into());
    }

    let tail_index = rest
        .char_indices()
        .find_map(|(index, ch)| matches!(ch, '?' | '#').then_some(index))
        .unwrap_or(rest.len());
    let without_query_or_fragment = &rest[..tail_index];
    let authority_end = without_query_or_fragment
        .find('/')
        .unwrap_or(without_query_or_fragment.len());
    let raw_authority = &without_query_or_fragment[..authority_end];
    let authority = raw_authority
        .rsplit_once('@')
        .map_or(raw_authority, |(_, host)| host);
    if authority.is_empty() && scheme != "file" {
        return Err("clone remote URL has no host after credential removal".into());
    }

    let path = &without_query_or_fragment[authority_end..];
    Ok(format!("{scheme}://{authority}{path}"))
}

fn sanitize_scp_like_remote(remote: &str) -> Option<String> {
    let colon = remote.find(':')?;
    let authority = &remote[..colon];
    let raw_path = &remote[colon + 1..];
    if authority.is_empty()
        || raw_path.is_empty()
        || authority.contains('/')
        || authority.contains('\\')
    {
        return None;
    }
    let tail_index = raw_path
        .char_indices()
        .find_map(|(index, ch)| matches!(ch, '?' | '#').then_some(index))
        .unwrap_or(raw_path.len());
    let path = &raw_path[..tail_index];
    if path.is_empty() {
        return None;
    }
    let host = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    if host.is_empty() {
        return None;
    }
    Some(format!("{host}:{path}"))
}

#[cfg(test)]
mod tests {
    use super::{
        clone_and_register_workspace, clone_and_register_workspace_impl, clone_directory_identity,
        create_private_clone_staging, require_clone_directory_identity,
        retain_owned_clone_staging_impl, sanitize_remote_identity,
    };
    use crate::store::Store;
    use rusqlite::{Connection, params};
    use std::ffi::OsStr;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    fn test_root(name: &str) -> PathBuf {
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "winds-t046-{name}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        root
    }

    fn run_git<I, S>(cwd: &Path, args: I) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = Command::new("git")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_COMMON_DIR")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .arg("-c")
            .arg("core.hooksPath=/dev/null")
            .arg("-C")
            .arg(cwd)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }

    fn initialize_remote(root: &Path, marker: &Path) -> (PathBuf, String) {
        let source = root.join("source");
        fs::create_dir(&source).unwrap();
        run_git(&source, ["init", "--initial-branch=main"]);
        run_git(&source, ["config", "user.name", "Winds Test"]);
        run_git(&source, ["config", "user.email", "winds@example.invalid"]);
        fs::write(source.join("tracked.txt"), b"tracked\n").unwrap();
        fs::write(
            source.join(".envrc"),
            format!("touch '{}'\n", marker.display()),
        )
        .unwrap();
        fs::write(source.join(".mise.toml"), b"[tools]\nnode = '22'\n").unwrap();
        run_git(
            &source,
            ["add", "--", "tracked.txt", ".envrc", ".mise.toml"],
        );
        run_git(&source, ["commit", "--no-gpg-sign", "-m", "fixture"]);
        let head = run_git(&source, ["rev-parse", "HEAD"]);

        let remote = root.join("remote.git");
        run_git(
            root,
            [
                "clone",
                "--bare",
                "--",
                source.to_str().unwrap(),
                remote.to_str().unwrap(),
            ],
        );
        (remote, head)
    }

    fn create_state_root(root: &Path) -> PathBuf {
        let home = root.join("winds-home");
        fs::create_dir(&home).unwrap();
        home.canonicalize().unwrap()
    }

    fn cleanup_owned_root(root: &Path) {
        let canonical_root = root.canonicalize().unwrap();
        let canonical_temp = std::env::temp_dir().canonicalize().unwrap();
        let owned_name = canonical_root
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| name.starts_with("winds-t046-"));
        assert!(canonical_root.starts_with(&canonical_temp));
        assert!(owned_name);
        fs::remove_dir_all(&canonical_root).unwrap();
    }

    fn private_clone_staging_paths(root: &Path) -> Vec<PathBuf> {
        fs::read_dir(root)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.file_name()
                    .and_then(OsStr::to_str)
                    .is_some_and(|name| name.starts_with(".winds-clone-stage-"))
            })
            .collect()
    }
    fn assert_private_clone_staging_failure_state_is_safe(root: &Path) {
        let staging_paths = private_clone_staging_paths(root);
        assert!(
            !staging_paths.is_empty(),
            "fail-closed clone cleanup must retain private staging for recovery"
        );
        for staging in staging_paths {
            let metadata = fs::symlink_metadata(&staging).unwrap();
            assert!(
                metadata.is_dir() && !metadata.file_type().is_symlink(),
                "retained private staging must remain a real directory"
            );
        }
    }

    #[test]
    fn clone_registers_workspace_and_persists_only_sanitized_remote_identity() {
        let root = test_root("clone");
        let marker = root.join("bootstrap-ran");
        let (remote, source_head) = initialize_remote(&root, &marker);
        let state_root = create_state_root(&root);
        let destination = root.join("cloned workspace");

        let cloned =
            clone_and_register_workspace(remote.to_str().unwrap(), &destination, &state_root, 100)
                .unwrap();

        assert_eq!(
            cloned.workspace.head_oid.as_deref(),
            Some(source_head.as_str())
        );
        assert_eq!(cloned.workspace.branch.as_deref(), Some("main"));
        assert!(!cloned.workspace.detached);
        assert!(!cloned.workspace.dirty);
        assert_eq!(
            cloned.workspace.canonical_worktree_root,
            destination.canonicalize().unwrap().to_str().unwrap()
        );
        assert_eq!(
            cloned.remote_identity,
            remote.canonicalize().unwrap().to_str().unwrap()
        );
        assert!(destination.join(".envrc").is_file());
        assert!(destination.join(".mise.toml").is_file());
        assert!(!marker.exists());

        let connection = Connection::open(state_root.join("winds.db")).unwrap();
        let (remote_identity, recorded_unix_ms): (String, i64) = connection
            .query_row(
                "SELECT remote_identity, recorded_unix_ms
                 FROM workspace_clone_origins WHERE workspace_id = ?1",
                params![cloned.workspace.workspace_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(remote_identity, cloned.remote_identity);
        assert_eq!(recorded_unix_ms, 100);
        drop(connection);

        cleanup_owned_root(&root);
    }

    #[test]
    fn clone_failure_happens_before_workspace_registration_and_allows_retry() {
        let root = test_root("failure");
        let state_root = create_state_root(&root);
        let not_a_repo = root.join("not-a-repo");
        fs::write(&not_a_repo, b"not git\n").unwrap();
        let destination = root.join("failed-clone");

        let error = clone_and_register_workspace(
            not_a_repo.to_str().unwrap(),
            &destination,
            &state_root,
            200,
        )
        .unwrap_err();
        assert!(error.to_string().contains("system Git clone failed"));
        assert!(!destination.exists());
        assert!(!state_root.join("winds.db").exists());
        assert_private_clone_staging_failure_state_is_safe(&root);

        let marker = root.join("retry-bootstrap-ran");
        let retry_root = root.join("retry-source");
        fs::create_dir(&retry_root).unwrap();
        let (remote, _) = initialize_remote(&retry_root, &marker);
        clone_and_register_workspace(remote.to_str().unwrap(), &destination, &state_root, 201)
            .unwrap();
        assert!(destination.is_dir());
        assert!(!marker.exists());

        cleanup_owned_root(&root);
    }

    #[test]
    fn destination_validation_is_fail_closed_before_clone() {
        let root = test_root("destination");
        let marker = root.join("bootstrap-ran");
        let (remote, _) = initialize_remote(&root, &marker);
        let state_root = create_state_root(&root);

        let existing = root.join("existing");
        fs::create_dir(&existing).unwrap();
        let error =
            clone_and_register_workspace(remote.to_str().unwrap(), &existing, &state_root, 300)
                .unwrap_err();
        assert!(error.to_string().contains("already exists"));

        let inside_state = state_root.join("source-inside-state");
        let error =
            clone_and_register_workspace(remote.to_str().unwrap(), &inside_state, &state_root, 301)
                .unwrap_err();
        assert!(error.to_string().contains("must not overlap"));
        assert!(!inside_state.exists());
        assert!(!state_root.join("winds.db").exists());

        cleanup_owned_root(&root);
    }

    #[test]
    fn origin_persistence_failure_rolls_back_workspace_registration() {
        let root = test_root("atomic-origin");
        let marker = root.join("bootstrap-ran");
        let (remote, _) = initialize_remote(&root, &marker);
        let state_root = create_state_root(&root);
        let destination = root.join("atomic-clone");

        let store = Store::open(&state_root).unwrap();
        drop(store);
        let connection = Connection::open(state_root.join("winds.db")).unwrap();
        connection
            .execute_batch(
                "CREATE TRIGGER fail_clone_origin
                 BEFORE INSERT ON workspace_clone_origins
                 BEGIN
                     SELECT RAISE(ABORT, 'forced clone-origin failure');
                 END;",
            )
            .unwrap();
        drop(connection);

        let error =
            clone_and_register_workspace(remote.to_str().unwrap(), &destination, &state_root, 350)
                .unwrap_err();
        assert!(error.to_string().contains("forced clone-origin failure"));
        assert!(destination.is_dir());

        let connection = Connection::open(state_root.join("winds.db")).unwrap();
        let workspace_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM workspaces", [], |row| row.get(0))
            .unwrap();
        let origin_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM workspace_clone_origins", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(workspace_count, 0);
        assert_eq!(origin_count, 0);
        drop(connection);

        cleanup_owned_root(&root);
    }

    #[test]
    fn concurrent_destination_creation_blocks_atomic_publish_without_replacement() {
        let root = test_root("publish-race");
        let marker = root.join("bootstrap-ran");
        let (remote, _) = initialize_remote(&root, &marker);
        let state_root = create_state_root(&root);
        let destination = root.join("raced-destination");
        let replacement_marker = destination.join("replacement-marker");
        let mut staged_checkout = None;

        let error = clone_and_register_workspace_impl(
            remote.to_str().unwrap(),
            &destination,
            &state_root,
            360,
            |staged, requested| {
                staged_checkout = Some(staged.to_path_buf());
                let expected_requested = destination
                    .parent()
                    .unwrap()
                    .canonicalize()
                    .unwrap()
                    .join(destination.file_name().unwrap());
                assert_eq!(requested, expected_requested);
                fs::create_dir(requested)?;
                fs::write(requested.join("replacement-marker"), b"replacement\n")?;
                Ok(())
            },
        )
        .unwrap_err();

        assert!(error.to_string().contains("atomically published"));
        assert_eq!(fs::read(&replacement_marker).unwrap(), b"replacement\n");
        let staged_checkout = staged_checkout.unwrap();
        assert!(staged_checkout.is_dir());
        assert!(
            fs::read_dir(&staged_checkout).unwrap().next().is_some(),
            "failed publication must retain clone payload rather than recursively delete through mutable pathnames"
        );
        assert_private_clone_staging_failure_state_is_safe(&root);
        assert!(!state_root.join("winds.db").exists());

        cleanup_owned_root(&root);
    }

    #[test]
    fn retained_failed_clone_payload_blocks_additional_staging_allocation() {
        let root = test_root("retained-staging-bound");
        let marker = root.join("bootstrap-ran");
        let (remote, _) = initialize_remote(&root, &marker);
        let state_root = create_state_root(&root);
        let first_destination = root.join("first-raced-destination");

        let first_error = clone_and_register_workspace_impl(
            remote.to_str().unwrap(),
            &first_destination,
            &state_root,
            363,
            |_, requested| {
                fs::create_dir(requested)?;
                fs::write(requested.join("foreign-marker"), b"foreign\n")?;
                Ok(())
            },
        )
        .unwrap_err();

        assert!(first_error.to_string().contains("atomically published"));
        let staging_before_retry = private_clone_staging_paths(&root);
        assert_eq!(
            staging_before_retry.len(),
            1,
            "the failed publication must retain exactly one private staging payload fixture"
        );
        assert!(
            fs::read_dir(&staging_before_retry[0])
                .unwrap()
                .next()
                .is_some(),
            "the retained staging fixture must contain payload so the bounded-retention gate is exercised"
        );

        let second_destination = root.join("second-clone-destination");
        let second_error = clone_and_register_workspace(
            remote.to_str().unwrap(),
            &second_destination,
            &state_root,
            364,
        )
        .unwrap_err();

        let second_error = second_error.to_string();
        assert!(second_error.contains("retained private clone staging"));
        assert!(second_error.contains("unbounded disk growth"));
        assert!(!second_destination.exists());
        assert_eq!(
            private_clone_staging_paths(&root),
            staging_before_retry,
            "a blocked retry must not allocate another private staging directory"
        );
        assert!(!state_root.join("winds.db").exists());

        cleanup_owned_root(&root);
    }

    #[test]
    fn failed_clone_never_recursively_cleans_a_concurrent_destination() {
        let root = test_root("failure-race");
        let state_root = create_state_root(&root);
        let not_a_repo = root.join("not-a-repo");
        fs::write(&not_a_repo, b"not git\n").unwrap();
        let destination = root.join("raced-destination");
        let replacement_marker = destination.join("replacement-marker");

        let error = clone_and_register_workspace_impl(
            not_a_repo.to_str().unwrap(),
            &destination,
            &state_root,
            361,
            |_, requested| {
                fs::create_dir(requested)?;
                fs::write(requested.join("replacement-marker"), b"replacement\n")?;
                Ok(())
            },
        )
        .unwrap_err();

        assert!(error.to_string().contains("system Git clone failed"));
        assert_eq!(fs::read(&replacement_marker).unwrap(), b"replacement\n");
        assert!(!state_root.join("winds.db").exists());

        cleanup_owned_root(&root);
    }

    #[cfg(any(target_os = "linux", target_os = "macos", windows))]
    #[test]
    fn cleanup_swap_after_identity_proof_never_deletes_foreign_replacement() {
        let root = test_root("cleanup-final-identity-swap")
            .canonicalize()
            .unwrap();
        let staging = create_private_clone_staging(&root).unwrap();
        let original_staging_path = staging.path.clone();
        let moved_owned_staging = root.join("moved-owned-staging");
        let checkout = original_staging_path.join("checkout");
        fs::create_dir(&checkout).unwrap();
        fs::write(checkout.join("owned-payload"), b"owned\n").unwrap();

        let foreign_marker = original_staging_path.join("foreign-replacement-marker");
        let retention = retain_owned_clone_staging_impl(&staging, || {
            fs::rename(&original_staging_path, &moved_owned_staging)?;
            fs::create_dir(&original_staging_path)?;
            fs::write(&foreign_marker, b"foreign\n")?;
            Ok(())
        });

        retention.unwrap();
        assert_eq!(
            fs::read(&foreign_marker).unwrap(),
            b"foreign\n",
            "post-proof pathname replacement must remain untouched"
        );
        assert_eq!(
            fs::read(moved_owned_staging.join("checkout").join("owned-payload")).unwrap(),
            b"owned\n",
            "post-proof pathname replacement must retain the original owned payload as well as the foreign replacement"
        );

        cleanup_owned_root(&root);
    }

    #[test]
    fn staging_path_replacement_is_not_cleaned_or_registered() {
        let root = test_root("staging-replacement");
        let marker = root.join("bootstrap-ran");
        let (remote, _) = initialize_remote(&root, &marker);
        let state_root = create_state_root(&root);
        let destination = root.join("clone-destination");
        let mut replacement_marker = None;

        let error = clone_and_register_workspace_impl(
            remote.to_str().unwrap(),
            &destination,
            &state_root,
            362,
            |staged, _| {
                let staging_root = staged.parent().unwrap();
                fs::remove_dir(staging_root)?;
                fs::create_dir(staging_root)?;
                let marker = staging_root.join("foreign-replacement-marker");
                fs::write(&marker, b"foreign\n")?;
                replacement_marker = Some(marker);
                Ok(())
            },
        )
        .unwrap_err();

        let error = error.to_string();
        assert!(error.contains("filesystem identity changed"));
        assert!(error.contains("refusing recursive cleanup"));
        assert_eq!(fs::read(replacement_marker.unwrap()).unwrap(), b"foreign\n");
        assert!(!destination.exists());
        assert!(!state_root.join("winds.db").exists());

        cleanup_owned_root(&root);
    }

    #[test]
    fn clone_directory_identity_rejects_same_path_replacement() {
        let root = test_root("directory-identity");
        let checkout = root.join("checkout");
        let original = root.join("checkout-original");
        fs::create_dir(&checkout).unwrap();
        let identity = clone_directory_identity(&checkout, "test checkout").unwrap();
        assert_eq!(
            clone_directory_identity(&checkout, "test checkout").unwrap(),
            identity
        );

        fs::rename(&checkout, &original).unwrap();
        fs::create_dir(&checkout).unwrap();
        let replacement = clone_directory_identity(&checkout, "test checkout").unwrap();
        assert_ne!(replacement, identity);
        let error =
            require_clone_directory_identity(&checkout, &identity, "test checkout").unwrap_err();
        assert!(error.to_string().contains("filesystem identity changed"));

        cleanup_owned_root(&root);
    }

    #[cfg(windows)]
    #[test]
    fn windows_clone_directory_identity_is_stable_and_detects_replacement() {
        let root = test_root("windows-directory-identity");
        let checkout = root.join("checkout");
        let original = root.join("checkout-original");
        fs::create_dir(&checkout).unwrap();
        let first = clone_directory_identity(&checkout, "Windows checkout").unwrap();
        let same = clone_directory_identity(&checkout, "Windows checkout").unwrap();
        assert_eq!(first, same);

        fs::rename(&checkout, &original).unwrap();
        fs::create_dir(&checkout).unwrap();
        let replacement = clone_directory_identity(&checkout, "Windows checkout").unwrap();
        assert_ne!(first, replacement);

        cleanup_owned_root(&root);
    }

    #[test]
    fn remote_sanitization_removes_credentials_and_url_secret_components() {
        let sanitized = sanitize_remote_identity(
            "https://alice:super-secret@example.test/org/repo.git?token=also-secret#private",
        )
        .unwrap();
        assert_eq!(sanitized, "https://example.test/org/repo.git");
        assert!(!sanitized.contains("alice"));
        assert!(!sanitized.contains("super-secret"));
        assert!(!sanitized.contains("also-secret"));
        assert!(!sanitized.contains("private"));

        assert_eq!(
            sanitize_remote_identity("git@example.test:org/repo.git").unwrap(),
            "example.test:org/repo.git"
        );
        assert_eq!(
            sanitize_remote_identity("git@example.test:org/repo.git?token=secret").unwrap(),
            "example.test:org/repo.git"
        );
        assert_eq!(
            sanitize_remote_identity("git@example.test:org/repo.git#private").unwrap(),
            "example.test:org/repo.git"
        );
        assert!(sanitize_remote_identity("ext::sh -c 'echo unsafe'").is_err());
        assert!(sanitize_remote_identity("custom://example.test/org/repo.git").is_err());
        assert!(sanitize_remote_identity("../relative/repo.git").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn absolute_local_symlink_remote_uses_one_canonical_identity_for_git_and_persistence() {
        let root = test_root("remote-symlink");
        let first_root = root.join("first");
        let second_root = root.join("second");
        fs::create_dir(&first_root).unwrap();
        fs::create_dir(&second_root).unwrap();
        let (first_remote, _) = initialize_remote(&first_root, &root.join("first-marker"));
        let (second_remote, _) = initialize_remote(&second_root, &root.join("second-marker"));
        let link = root.join("remote-link");
        symlink(&first_remote, &link).unwrap();

        let identity = sanitize_remote_identity(link.to_str().unwrap()).unwrap();
        assert_eq!(
            identity,
            first_remote.canonicalize().unwrap().to_str().unwrap()
        );

        fs::remove_file(&link).unwrap();
        symlink(&second_remote, &link).unwrap();
        let git_argument = super::git_remote_argument(link.to_str().unwrap(), &identity).unwrap();
        assert_eq!(PathBuf::from(git_argument), PathBuf::from(&identity));
        assert_ne!(
            identity,
            second_remote.canonicalize().unwrap().to_str().unwrap()
        );

        cleanup_owned_root(&root);
    }

    #[cfg(unix)]
    #[test]
    fn destination_validation_rejects_broken_symlink_before_staging() {
        let root = test_root("broken-destination");
        let state_root = create_state_root(&root);
        let destination = root.join("broken-destination");
        symlink(root.join("missing-target"), &destination).unwrap();

        let error = super::plan_clone_destination(&destination, &state_root).unwrap_err();
        assert!(error.to_string().contains("already exists"));

        fs::remove_file(&destination).unwrap();
        cleanup_owned_root(&root);
    }

    #[cfg(windows)]
    #[test]
    fn windows_git_cli_local_path_removes_only_supported_verbatim_prefixes() {
        assert_eq!(
            super::git_cli_local_path(Path::new(r"\\?\C:\Temp\Winds Clone")).unwrap(),
            PathBuf::from(r"C:\Temp\Winds Clone")
        );
        assert_eq!(
            super::git_cli_local_path(Path::new(r"\\?\UNC\server\share\Winds Clone")).unwrap(),
            PathBuf::from(r"\\server\share\Winds Clone")
        );
        assert!(super::git_cli_local_path(Path::new(r"\\?\UNC\server")).is_err());
        assert!(super::git_cli_local_path(Path::new(r"\\?\UNC\")).is_err());
        assert!(super::git_cli_local_path(Path::new(r"\\?\Volume{abc}\repo")).is_err());
    }
}
