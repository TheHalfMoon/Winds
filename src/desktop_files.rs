use crate::git::{Repo, observe_worktree_state, run_read_only_git_bytes};
use crate::store::{Result, Store};
use rusqlite::OptionalExtension;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

#[cfg(unix)]
use std::ffi::CString;
#[cfg(windows)]
use std::ffi::{OsString, c_void};
#[cfg(windows)]
use std::mem::MaybeUninit;
#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(windows)]
use std::os::windows::ffi::{OsStrExt, OsStringExt};
#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;
#[cfg(windows)]
use std::os::windows::io::{AsRawHandle, FromRawHandle};

const MAX_FILE_LIST_ENTRIES: usize = 4096;
const MAX_FILE_PREVIEW_BYTES: u64 = 256 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopRightDockTarget {
    pub workspace_id: String,
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopRightDockBinding {
    pub workspace_id: String,
    pub session_id: String,
    pub worktree_root: String,
    pub git_common_dir: String,
    pub workflow_run_id: Option<String>,
    pub stage_run_id: Option<String>,
    pub candidate_oid: Option<String>,
    pub candidate_tree: Option<String>,
    pub head_oid: Option<String>,
    pub tree_oid: Option<String>,
    pub worktree_state_sha256: String,
    pub binding_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopFileKind {
    File,
    Symlink,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopFileEntry {
    pub path: String,
    pub kind: DesktopFileKind,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopFilesResponse {
    pub binding: DesktopRightDockBinding,
    pub entries: Vec<DesktopFileEntry>,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopFilePreviewState {
    Text,
    Binary,
    TooLarge,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopFilePreviewRequest {
    pub binding: DesktopRightDockBinding,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopFilePreviewResponse {
    pub binding: DesktopRightDockBinding,
    pub path: String,
    pub state: DesktopFilePreviewState,
    pub content: Option<String>,
    pub byte_len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBoundDockRequest {
    pub binding: DesktopRightDockBinding,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopChangeEntry {
    pub path: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopChangesResponse {
    pub binding: DesktopRightDockBinding,
    pub entries: Vec<DesktopChangeEntry>,
    pub diff: String,
    pub diff_lossy: bool,
}

pub fn desktop_right_dock_bind(
    home: &Path,
    target: DesktopRightDockTarget,
) -> Result<DesktopRightDockBinding> {
    let store = Store::open(home)?;
    current_binding(&store, &target)
}

pub fn desktop_right_dock_files(
    home: &Path,
    request: DesktopBoundDockRequest,
) -> Result<DesktopFilesResponse> {
    let store = Store::open(home)?;
    require_current_binding(&store, &request.binding)?;
    let root = Path::new(&request.binding.worktree_root);
    let output = run_read_only_git_bytes(
        root,
        [
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ],
        "desktop Files inventory",
    )?;
    let mut entries = Vec::new();
    let mut truncated = false;
    for raw in output
        .split(|byte| *byte == 0)
        .filter(|value| !value.is_empty())
    {
        if entries.len() == MAX_FILE_LIST_ENTRIES {
            truncated = true;
            break;
        }
        let path = std::str::from_utf8(raw)
            .map_err(|_| "desktop Files inventory contains a non-UTF-8 path")?;
        let relative = validated_relative_path(path)?;
        let metadata = match fs::symlink_metadata(root.join(&relative)) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        let kind = if metadata.file_type().is_symlink() {
            DesktopFileKind::Symlink
        } else if metadata.is_file() {
            DesktopFileKind::File
        } else {
            return Err(format!("desktop Files inventory returned non-file path: {path}").into());
        };
        entries.push(DesktopFileEntry {
            path: path.to_owned(),
            kind,
        });
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    require_current_binding(&store, &request.binding)?;
    Ok(DesktopFilesResponse {
        binding: request.binding,
        entries,
        truncated,
    })
}

pub fn desktop_right_dock_preview_file(
    home: &Path,
    request: DesktopFilePreviewRequest,
) -> Result<DesktopFilePreviewResponse> {
    desktop_right_dock_preview_file_inner(home, request, || {})
}

#[cfg(all(test, unix))]
pub(crate) fn desktop_right_dock_preview_file_with_open_hook<F>(
    home: &Path,
    request: DesktopFilePreviewRequest,
    before_final_open: F,
) -> Result<DesktopFilePreviewResponse>
where
    F: FnOnce(),
{
    desktop_right_dock_preview_file_inner(home, request, before_final_open)
}

fn desktop_right_dock_preview_file_inner<F>(
    home: &Path,
    request: DesktopFilePreviewRequest,
    before_final_open: F,
) -> Result<DesktopFilePreviewResponse>
where
    F: FnOnce(),
{
    let store = Store::open(home)?;
    require_current_binding(&store, &request.binding)?;
    let root = Path::new(&request.binding.worktree_root);
    let relative = validated_relative_path(&request.path)?;
    let file = open_regular_file_beneath_root(root, &relative, before_final_open)?;
    let metadata = file.metadata()?;
    let byte_len = metadata.len();
    let response = if byte_len > MAX_FILE_PREVIEW_BYTES {
        DesktopFilePreviewResponse {
            binding: request.binding.clone(),
            path: request.path.clone(),
            state: DesktopFilePreviewState::TooLarge,
            content: None,
            byte_len,
        }
    } else {
        let mut bytes = Vec::with_capacity(byte_len as usize);
        file.take(MAX_FILE_PREVIEW_BYTES + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_FILE_PREVIEW_BYTES {
            return Err("desktop file preview changed size while being read".into());
        }
        if bytes.contains(&0) {
            DesktopFilePreviewResponse {
                binding: request.binding.clone(),
                path: request.path.clone(),
                state: DesktopFilePreviewState::Binary,
                content: None,
                byte_len,
            }
        } else {
            match String::from_utf8(bytes) {
                Ok(content) => DesktopFilePreviewResponse {
                    binding: request.binding.clone(),
                    path: request.path.clone(),
                    state: DesktopFilePreviewState::Text,
                    content: Some(content),
                    byte_len,
                },
                Err(_) => DesktopFilePreviewResponse {
                    binding: request.binding.clone(),
                    path: request.path.clone(),
                    state: DesktopFilePreviewState::Binary,
                    content: None,
                    byte_len,
                },
            }
        }
    };
    require_current_binding(&store, &request.binding)?;
    Ok(response)
}

pub fn desktop_right_dock_changes(
    home: &Path,
    request: DesktopBoundDockRequest,
) -> Result<DesktopChangesResponse> {
    let store = Store::open(home)?;
    require_current_binding(&store, &request.binding)?;
    let root = Path::new(&request.binding.worktree_root);
    let status = run_read_only_git_bytes(
        root,
        [
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--no-renames",
        ],
        "desktop Changes status",
    )?;
    let entries = parse_porcelain_v1_status(&status)?;
    let unstaged = run_read_only_git_bytes(
        root,
        [
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--no-color",
            "--no-renames",
            "--",
        ],
        "desktop unstaged diff",
    )?;
    let staged = run_read_only_git_bytes(
        root,
        [
            "diff",
            "--cached",
            "--no-ext-diff",
            "--no-textconv",
            "--no-color",
            "--no-renames",
            "--",
        ],
        "desktop staged diff",
    )?;
    let diff_lossy =
        std::str::from_utf8(&unstaged).is_err() || std::str::from_utf8(&staged).is_err();
    let mut diff = String::new();
    if !staged.is_empty() {
        diff.push_str("# Staged changes\n");
        diff.push_str(&String::from_utf8_lossy(&staged));
        if !diff.ends_with('\n') {
            diff.push('\n');
        }
    }
    if !unstaged.is_empty() {
        if !diff.is_empty() {
            diff.push('\n');
        }
        diff.push_str("# Unstaged changes\n");
        diff.push_str(&String::from_utf8_lossy(&unstaged));
    }
    require_current_binding(&store, &request.binding)?;
    Ok(DesktopChangesResponse {
        binding: request.binding,
        entries,
        diff,
        diff_lossy,
    })
}

fn current_binding(
    store: &Store,
    target: &DesktopRightDockTarget,
) -> Result<DesktopRightDockBinding> {
    let session = store.load_winds_session(&target.session_id)?;
    let workstream = store.load_workstream(&session.workstream_id)?;
    if workstream.workspace_id != target.workspace_id {
        return Err("desktop right dock Session does not belong to requested Workspace".into());
    }
    let workspace = store.load_workspace(&target.workspace_id)?;
    let root = PathBuf::from(&workspace.canonical_worktree_root);
    let common = PathBuf::from(&workspace.git_common_dir);
    let observation = observe_worktree_state(&root, &common)?;
    let repo = Repo::open(&root)?;
    let tree_oid = observation
        .head_oid
        .as_deref()
        .map(|head| repo.tree_oid(head))
        .transpose()?;
    let workflow_identity =
        latest_workflow_identity(store, &target.session_id, &target.workspace_id)?;
    let (workflow_run_id, stage_run_id, candidate_oid, candidate_tree) = match workflow_identity {
        Some(identity) => (
            Some(identity.workflow_run_id),
            Some(identity.stage_run_id),
            identity.candidate_oid,
            identity.candidate_tree,
        ),
        None => (None, None, None, None),
    };
    let mut binding = DesktopRightDockBinding {
        workspace_id: target.workspace_id.clone(),
        session_id: target.session_id.clone(),
        worktree_root: workspace.canonical_worktree_root,
        git_common_dir: workspace.git_common_dir,
        workflow_run_id,
        stage_run_id,
        candidate_oid,
        candidate_tree,
        head_oid: observation.head_oid,
        tree_oid,
        worktree_state_sha256: observation.worktree_state_sha256,
        binding_digest: String::new(),
    };
    binding.binding_digest = binding_digest(&binding);
    Ok(binding)
}

fn require_current_binding(store: &Store, expected: &DesktopRightDockBinding) -> Result<()> {
    if expected.binding_digest != binding_digest(expected) {
        return Err("desktop right dock binding digest is invalid".into());
    }
    let current = current_binding(
        store,
        &DesktopRightDockTarget {
            workspace_id: expected.workspace_id.clone(),
            session_id: expected.session_id.clone(),
        },
    )?;
    if current != *expected {
        return Err(
            "desktop right dock binding is stale; refresh the exact Session binding".into(),
        );
    }
    Ok(())
}

struct BoundWorkflowIdentity {
    workflow_run_id: String,
    stage_run_id: String,
    candidate_oid: Option<String>,
    candidate_tree: Option<String>,
}

fn latest_workflow_identity(
    store: &Store,
    session_id: &str,
    workspace_id: &str,
) -> Result<Option<BoundWorkflowIdentity>> {
    let stage = store
        .connection
        .query_row(
            "SELECT workflow.workflow_run_id, stage.stage_run_id
             FROM workflow_actor_bindings actor
             JOIN workflow_stage_runs stage ON stage.stage_run_id = actor.stage_run_id
             JOIN workflow_runs workflow ON workflow.workflow_run_id = stage.workflow_run_id
             JOIN winds_sessions session
               ON session.session_id = actor.winds_session_id
              AND session.workstream_id = workflow.workstream_id
             WHERE actor.winds_session_id = ?1
               AND workflow.workspace_id = ?2
             ORDER BY actor.bound_unix_ms DESC, actor.binding_id DESC
             LIMIT 1",
            rusqlite::params![session_id, workspace_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;
    let Some((workflow_run_id, stage_run_id)) = stage else {
        return Ok(None);
    };
    let candidate = store
        .connection
        .query_row(
            "SELECT candidate_oid, candidate_tree
             FROM workflow_artifact_baselines
             WHERE stage_run_id = ?1
               AND candidate_oid IS NOT NULL
               AND candidate_tree IS NOT NULL
             ORDER BY created_unix_ms DESC, baseline_id DESC
             LIMIT 1",
            rusqlite::params![stage_run_id],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                ))
            },
        )
        .optional()?
        .unwrap_or((None, None));
    match (&candidate.0, &candidate.1) {
        (None, None) | (Some(_), Some(_)) => {}
        _ => return Err("desktop right dock candidate identity is incomplete".into()),
    }
    Ok(Some(BoundWorkflowIdentity {
        workflow_run_id,
        stage_run_id,
        candidate_oid: candidate.0,
        candidate_tree: candidate.1,
    }))
}

fn binding_digest(binding: &DesktopRightDockBinding) -> String {
    let mut hasher = Sha256::new();
    for value in [
        binding.workspace_id.as_str(),
        binding.session_id.as_str(),
        binding.worktree_root.as_str(),
        binding.git_common_dir.as_str(),
        binding.workflow_run_id.as_deref().unwrap_or("<none>"),
        binding.stage_run_id.as_deref().unwrap_or("<none>"),
        binding.candidate_oid.as_deref().unwrap_or("<none>"),
        binding.candidate_tree.as_deref().unwrap_or("<none>"),
        binding.head_oid.as_deref().unwrap_or("<unborn>"),
        binding.tree_oid.as_deref().unwrap_or("<unborn>"),
        binding.worktree_state_sha256.as_str(),
    ] {
        hasher.update(value.as_bytes());
        hasher.update([0]);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn validated_relative_path(value: &str) -> Result<PathBuf> {
    if value.is_empty() {
        return Err("desktop file path must not be empty".into());
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return Err("desktop file path must be relative to the canonical worktree".into());
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => normalized.push(value),
            _ => return Err("desktop file path traversal is not allowed".into()),
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err("desktop file path must identify a file".into());
    }
    Ok(normalized)
}

#[cfg(unix)]
fn open_regular_file_beneath_root<F>(
    root: &Path,
    relative: &Path,
    before_final_open: F,
) -> Result<fs::File>
where
    F: FnOnce(),
{
    let mut directory = open_unix_absolute_directory(root)?;
    let components = relative
        .components()
        .map(|component| match component {
            Component::Normal(value) => Ok(value),
            _ => Err("desktop file path must contain only normal relative components".into()),
        })
        .collect::<Result<Vec<_>>>()?;
    let (file_name, parent_components) = components
        .split_last()
        .ok_or("desktop file path must identify a file")?;
    for component in parent_components {
        directory = open_unix_directory_at(
            directory.as_raw_fd(),
            component,
            "desktop file preview directory",
        )?;
    }
    before_final_open();
    let name = unix_component_cstring(file_name, "desktop file preview")?;
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
        )
    };
    if fd < 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ELOOP) {
            return Err("desktop file preview refuses symlink final targets".into());
        }
        return Err(format!(
            "desktop file preview could not open the final file without following links: {error}"
        )
        .into());
    }
    let file = unsafe { fs::File::from_raw_fd(fd) };
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err("desktop file preview requires a real regular file".into());
    }
    Ok(file)
}

#[cfg(unix)]
fn open_unix_absolute_directory(path: &Path) -> Result<fs::File> {
    if !path.is_absolute() {
        return Err("desktop canonical worktree root must be absolute".into());
    }
    let root_name = CString::new("/").expect("static root path has no NUL");
    let root_fd = unsafe {
        libc::open(
            root_name.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if root_fd < 0 {
        return Err(format!(
            "desktop filesystem root could not be opened safely: {}",
            std::io::Error::last_os_error()
        )
        .into());
    }
    let mut directory = unsafe { fs::File::from_raw_fd(root_fd) };
    for component in path.components() {
        match component {
            Component::RootDir => {}
            Component::Normal(value) => {
                directory = open_unix_directory_at(
                    directory.as_raw_fd(),
                    value,
                    "desktop canonical worktree component",
                )?;
            }
            _ => {
                return Err(
                    "desktop canonical worktree root contains a non-canonical component".into(),
                );
            }
        }
    }
    Ok(directory)
}

#[cfg(unix)]
fn open_unix_directory_at(parent_fd: i32, name: &std::ffi::OsStr, label: &str) -> Result<fs::File> {
    let name = unix_component_cstring(name, label)?;
    let fd = unsafe {
        libc::openat(
            parent_fd,
            name.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(format!(
            "{label} could not be opened without following links: {}",
            std::io::Error::last_os_error()
        )
        .into());
    }
    Ok(unsafe { fs::File::from_raw_fd(fd) })
}

#[cfg(unix)]
fn unix_component_cstring(name: &std::ffi::OsStr, label: &str) -> Result<CString> {
    CString::new(name.as_bytes())
        .map_err(|_| format!("{label} contains an embedded NUL byte").into())
}

#[cfg(windows)]
const WINDOWS_FILE_SHARE_READ: u32 = 0x0000_0001;
#[cfg(windows)]
const WINDOWS_FILE_SHARE_WRITE: u32 = 0x0000_0002;
#[cfg(windows)]
const WINDOWS_FILE_SHARE_DELETE: u32 = 0x0000_0004;
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
const WINDOWS_FINAL_PATH_BUFFER: usize = 32_768;
#[cfg(windows)]
const WINDOWS_GENERIC_READ: u32 = 0x8000_0000;
#[cfg(windows)]
const WINDOWS_SYNCHRONIZE: u32 = 0x0010_0000;
#[cfg(windows)]
const WINDOWS_FILE_LIST_DIRECTORY: u32 = 0x0000_0001;
#[cfg(windows)]
const WINDOWS_FILE_TRAVERSE: u32 = 0x0000_0020;
#[cfg(windows)]
const WINDOWS_FILE_READ_ATTRIBUTES: u32 = 0x0000_0080;
#[cfg(windows)]
const WINDOWS_OBJ_CASE_INSENSITIVE: u32 = 0x0000_0040;
#[cfg(windows)]
const WINDOWS_FILE_OPEN: u32 = 0x0000_0001;
#[cfg(windows)]
const WINDOWS_FILE_DIRECTORY_FILE: u32 = 0x0000_0001;
#[cfg(windows)]
const WINDOWS_FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x0000_0020;
#[cfg(windows)]
const WINDOWS_FILE_NON_DIRECTORY_FILE: u32 = 0x0000_0040;
#[cfg(windows)]
const WINDOWS_FILE_OPEN_REPARSE_POINT: u32 = 0x0020_0000;

#[cfg(windows)]
#[repr(C)]
struct WindowsFileAttributeTagInfo {
    file_attributes: u32,
    _reparse_tag: u32,
}

#[cfg(windows)]
#[repr(C)]
struct WindowsUnicodeString {
    length: u16,
    maximum_length: u16,
    buffer: *mut u16,
}

#[cfg(windows)]
#[repr(C)]
struct WindowsObjectAttributes {
    length: u32,
    root_directory: *mut c_void,
    object_name: *mut WindowsUnicodeString,
    attributes: u32,
    security_descriptor: *mut c_void,
    security_quality_of_service: *mut c_void,
}

#[cfg(windows)]
#[repr(C)]
union WindowsIoStatusValue {
    status: i32,
    pointer: *mut c_void,
}

#[cfg(windows)]
#[repr(C)]
struct WindowsIoStatusBlock {
    value: WindowsIoStatusValue,
    information: usize,
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetFileInformationByHandleEx(
        file_handle: *mut c_void,
        file_information_class: i32,
        file_information: *mut c_void,
        buffer_size: u32,
    ) -> i32;
    fn GetFinalPathNameByHandleW(
        file_handle: *mut c_void,
        file_path: *mut u16,
        file_path_size: u32,
        flags: u32,
    ) -> u32;
}

#[cfg(windows)]
#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtCreateFile(
        file_handle: *mut *mut c_void,
        desired_access: u32,
        object_attributes: *mut WindowsObjectAttributes,
        io_status_block: *mut WindowsIoStatusBlock,
        allocation_size: *mut i64,
        file_attributes: u32,
        share_access: u32,
        create_disposition: u32,
        create_options: u32,
        ea_buffer: *mut c_void,
        ea_length: u32,
    ) -> i32;
}

#[cfg(windows)]
fn open_regular_file_beneath_root<F>(
    root: &Path,
    relative: &Path,
    before_final_open: F,
) -> Result<fs::File>
where
    F: FnOnce(),
{
    let mut directory = open_windows_root(root)?;
    require_windows_object_type(&directory, true, "desktop canonical worktree root")?;
    let root_handle_path = windows_final_path(&directory, "desktop canonical worktree root")?;
    if normalize_windows_handle_path(&root_handle_path) != normalize_windows_handle_path(root) {
        return Err("desktop canonical worktree root handle does not match its stored path".into());
    }

    let components = relative
        .components()
        .map(|component| match component {
            Component::Normal(value) => Ok(value),
            _ => Err("desktop file path must contain only normal relative components".into()),
        })
        .collect::<Result<Vec<_>>>()?;
    let (file_name, parent_components) = components
        .split_last()
        .ok_or("desktop file path must identify a file")?;
    for component in parent_components {
        directory = open_windows_relative_object(
            &directory,
            component,
            true,
            "desktop file preview directory",
        )?;
        require_windows_object_type(&directory, true, "desktop file preview directory")?;
    }

    before_final_open();
    let file = open_windows_relative_object(&directory, file_name, false, "desktop file preview")?;
    require_windows_object_type(&file, false, "desktop file preview")?;
    Ok(file)
}

#[cfg(windows)]
fn open_windows_root(path: &Path) -> Result<fs::File> {
    let mut options = fs::OpenOptions::new();
    options
        .access_mode(WINDOWS_FILE_READ_ATTRIBUTES | WINDOWS_SYNCHRONIZE)
        .share_mode(WINDOWS_FILE_SHARE_READ | WINDOWS_FILE_SHARE_WRITE | WINDOWS_FILE_SHARE_DELETE)
        .custom_flags(WINDOWS_FILE_FLAG_OPEN_REPARSE_POINT | WINDOWS_FILE_FLAG_BACKUP_SEMANTICS);
    options.open(path).map_err(|error| {
        format!("desktop canonical worktree root could not be opened without following its final reparse point: {error}").into()
    })
}

#[cfg(windows)]
fn open_windows_relative_object(
    parent: &fs::File,
    name: &std::ffi::OsStr,
    directory: bool,
    label: &str,
) -> Result<fs::File> {
    let mut name_wide = name.encode_wide().collect::<Vec<_>>();
    if name_wide.is_empty() || name_wide.contains(&0) {
        return Err(format!("{label} has an invalid Windows path component").into());
    }
    let byte_len = name_wide
        .len()
        .checked_mul(std::mem::size_of::<u16>())
        .ok_or("Windows path component length overflow")?;
    let byte_len = u16::try_from(byte_len)
        .map_err(|_| format!("{label} Windows path component is too long"))?;
    let mut unicode = WindowsUnicodeString {
        length: byte_len,
        maximum_length: byte_len,
        buffer: name_wide.as_mut_ptr(),
    };
    let mut attributes = WindowsObjectAttributes {
        length: u32::try_from(std::mem::size_of::<WindowsObjectAttributes>())?,
        root_directory: parent.as_raw_handle(),
        object_name: &mut unicode,
        attributes: WINDOWS_OBJ_CASE_INSENSITIVE,
        security_descriptor: std::ptr::null_mut(),
        security_quality_of_service: std::ptr::null_mut(),
    };
    let mut io_status = WindowsIoStatusBlock {
        value: WindowsIoStatusValue { status: 0 },
        information: 0,
    };
    let mut handle = std::ptr::null_mut();
    let desired_access = if directory {
        WINDOWS_FILE_LIST_DIRECTORY
            | WINDOWS_FILE_TRAVERSE
            | WINDOWS_FILE_READ_ATTRIBUTES
            | WINDOWS_SYNCHRONIZE
    } else {
        WINDOWS_GENERIC_READ | WINDOWS_FILE_READ_ATTRIBUTES | WINDOWS_SYNCHRONIZE
    };
    let create_options = WINDOWS_FILE_OPEN_REPARSE_POINT
        | WINDOWS_FILE_SYNCHRONOUS_IO_NONALERT
        | if directory {
            WINDOWS_FILE_DIRECTORY_FILE
        } else {
            WINDOWS_FILE_NON_DIRECTORY_FILE
        };
    let status = unsafe {
        NtCreateFile(
            &mut handle,
            desired_access,
            &mut attributes,
            &mut io_status,
            std::ptr::null_mut(),
            0,
            WINDOWS_FILE_SHARE_READ | WINDOWS_FILE_SHARE_WRITE | WINDOWS_FILE_SHARE_DELETE,
            WINDOWS_FILE_OPEN,
            create_options,
            std::ptr::null_mut(),
            0,
        )
    };
    if status < 0 || handle.is_null() {
        return Err(format!(
            "{label} could not be opened relative to its owned parent without following reparse points: NTSTATUS 0x{:08x}",
            status as u32
        )
        .into());
    }
    Ok(unsafe { fs::File::from_raw_handle(handle) })
}

#[cfg(windows)]
fn require_windows_object_type(handle: &fs::File, directory: bool, label: &str) -> Result<()> {
    let mut info = MaybeUninit::<WindowsFileAttributeTagInfo>::uninit();
    let result = unsafe {
        GetFileInformationByHandleEx(
            handle.as_raw_handle(),
            WINDOWS_FILE_ATTRIBUTE_TAG_INFO_CLASS,
            info.as_mut_ptr().cast::<c_void>(),
            std::mem::size_of::<WindowsFileAttributeTagInfo>() as u32,
        )
    };
    if result == 0 {
        return Err(format!(
            "{label} handle attributes cannot be inspected: {}",
            std::io::Error::last_os_error()
        )
        .into());
    }
    let info = unsafe { info.assume_init() };
    let is_directory = info.file_attributes & WINDOWS_FILE_ATTRIBUTE_DIRECTORY != 0;
    if info.file_attributes & WINDOWS_FILE_ATTRIBUTE_REPARSE_POINT != 0 || is_directory != directory
    {
        return Err(format!("{label} is a reparse point or has the wrong object type").into());
    }
    Ok(())
}

#[cfg(windows)]
fn windows_final_path(handle: &fs::File, label: &str) -> Result<PathBuf> {
    let mut buffer = vec![0_u16; WINDOWS_FINAL_PATH_BUFFER];
    let length = unsafe {
        GetFinalPathNameByHandleW(
            handle.as_raw_handle(),
            buffer.as_mut_ptr(),
            u32::try_from(buffer.len())?,
            0,
        )
    };
    if length == 0 {
        return Err(format!(
            "{label} final path cannot be inspected: {}",
            std::io::Error::last_os_error()
        )
        .into());
    }
    let length = usize::try_from(length)?;
    if length >= buffer.len() {
        return Err(format!("{label} final path exceeded the bounded Windows buffer").into());
    }
    Ok(PathBuf::from(OsString::from_wide(&buffer[..length])))
}

#[cfg(windows)]
fn normalize_windows_handle_path(path: &Path) -> PathBuf {
    let value = path.as_os_str().to_string_lossy();
    if let Some(rest) = value.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{rest}"));
    }
    if let Some(rest) = value.strip_prefix(r"\\?\") {
        return PathBuf::from(rest);
    }
    path.to_path_buf()
}

#[cfg(not(any(unix, windows)))]
fn open_regular_file_beneath_root<F>(
    _root: &Path,
    _relative: &Path,
    _before_final_open: F,
) -> Result<fs::File>
where
    F: FnOnce(),
{
    Err("desktop file preview safe open is unsupported on this platform".into())
}

fn parse_porcelain_v1_status(output: &[u8]) -> Result<Vec<DesktopChangeEntry>> {
    let mut entries = Vec::new();
    for field in output
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
    {
        if field.len() < 4 || field[2] != b' ' {
            return Err("desktop Changes received malformed Git porcelain output".into());
        }
        let status = std::str::from_utf8(&field[..2])
            .map_err(|_| "desktop Changes status code is not UTF-8")?;
        let path =
            std::str::from_utf8(&field[3..]).map_err(|_| "desktop Changes path is not UTF-8")?;
        validated_relative_path(path)?;
        entries.push(DesktopChangeEntry {
            path: path.to_owned(),
            status: status.to_owned(),
        });
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(entries)
}
