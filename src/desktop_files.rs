use crate::git::{Repo, observe_worktree_state, run_read_only_git_bytes};
use crate::store::{Result, Store};
use rusqlite::OptionalExtension;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

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
    let store = Store::open(home)?;
    require_current_binding(&store, &request.binding)?;
    let root = Path::new(&request.binding.worktree_root);
    let path = resolve_regular_file_without_symlinks(root, &request.path)?;
    let metadata = fs::metadata(&path)?;
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
        fs::File::open(&path)?
            .take(MAX_FILE_PREVIEW_BYTES + 1)
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

fn resolve_regular_file_without_symlinks(root: &Path, value: &str) -> Result<PathBuf> {
    let relative = validated_relative_path(value)?;
    let canonical_root = root.canonicalize()?;
    let mut current = canonical_root.clone();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            unreachable!()
        };
        current.push(part);
        let metadata = fs::symlink_metadata(&current)?;
        if metadata.file_type().is_symlink() {
            return Err("desktop file preview refuses symlink paths".into());
        }
    }
    let canonical_file = current.canonicalize()?;
    if !canonical_file.starts_with(&canonical_root) {
        return Err("desktop file preview escaped the canonical worktree".into());
    }
    if !canonical_file.is_file() {
        return Err("desktop file preview target is not a regular file".into());
    }
    Ok(canonical_file)
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
