use crate::store::Result;
use rusqlite::{Connection, OpenFlags, params};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

pub(crate) const HARD_MAX_TRANSCRIPT_BYTES: usize = 8 * 1024 * 1024;
const MAX_EXECUTION_ID_BYTES: usize = 512;
const HISTORY_SCHEMA_VERSION: u32 = 1;
const REDACTED: &str = "<winds:redacted>";
const HISTORY_DISABLED: &str = "<winds:history-disabled>";
const SECRET_FILTERING_MODE: &str = "BEST_EFFORT_METADATA_REDACTION_NOT_SECRET_DETECTION";

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct SessionHistoryPolicy {
    command_history_enabled: bool,
    transcript_enabled: bool,
    transcript_byte_quota: usize,
    total_history_byte_quota: u64,
}

impl SessionHistoryPolicy {
    pub const fn disabled() -> Self {
        Self {
            command_history_enabled: false,
            transcript_enabled: false,
            transcript_byte_quota: 0,
            total_history_byte_quota: 0,
        }
    }

    pub const fn command_history_only() -> Self {
        Self {
            command_history_enabled: true,
            transcript_enabled: false,
            transcript_byte_quota: 0,
            total_history_byte_quota: 0,
        }
    }

    pub fn local_bounded(
        command_history_enabled: bool,
        transcript_byte_quota: usize,
        total_history_byte_quota: u64,
    ) -> Result<Self> {
        if transcript_byte_quota == 0 {
            return Err(
                "enabled terminal transcript history requires a non-zero byte quota".into(),
            );
        }
        if transcript_byte_quota > HARD_MAX_TRANSCRIPT_BYTES {
            return Err(format!(
                "terminal transcript byte quota exceeds the Spec 003 T056 hard maximum of {HARD_MAX_TRANSCRIPT_BYTES} bytes"
            )
            .into());
        }
        if total_history_byte_quota < u64::try_from(transcript_byte_quota)? {
            return Err(
                "total terminal history byte quota must be at least the per-session transcript quota"
                    .into(),
            );
        }
        Ok(Self {
            command_history_enabled,
            transcript_enabled: true,
            transcript_byte_quota,
            total_history_byte_quota,
        })
    }
}

impl Default for SessionHistoryPolicy {
    fn default() -> Self {
        Self::disabled()
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HistoryBlobRef {
    pub relative_path: String,
    pub sha256: String,
    pub captured_bytes: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SessionHistoryManifest {
    pub schema_version: u32,
    pub execution_id: String,
    pub local_only: bool,
    pub secret_filtering: String,
    pub policy: SessionHistoryPolicy,
    pub transcript_observed_bytes: u64,
    pub transcript_retained_bytes: usize,
    pub transcript_capture_complete: bool,
    pub transcript_truncated: bool,
    pub transcript: HistoryBlobRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedSessionHistory {
    pub manifest: SessionHistoryManifest,
    pub manifest_blob: HistoryBlobRef,
}

#[derive(Debug, Default)]
struct TranscriptState {
    observed_bytes: u64,
    retained: Vec<u8>,
    quota_truncated: bool,
    reader_taken: bool,
    capture_complete: bool,
    persisted: bool,
}

pub(crate) struct SessionHistoryRecorder {
    execution_id: String,
    policy: SessionHistoryPolicy,
    state_root: Option<PathBuf>,
    state: Arc<Mutex<TranscriptState>>,
}

impl SessionHistoryRecorder {
    pub(crate) fn new_disabled(execution_id: &str) -> Result<Self> {
        Self::new(execution_id, SessionHistoryPolicy::disabled(), None)
    }

    pub(crate) fn new_local(
        execution_id: &str,
        policy: SessionHistoryPolicy,
        state_root: &Path,
    ) -> Result<Self> {
        if !policy.transcript_enabled {
            return Err(
                "local terminal history recorder requires transcript history to be enabled".into(),
            );
        }
        Self::new(execution_id, policy, Some(state_root))
    }

    fn new(
        execution_id: &str,
        policy: SessionHistoryPolicy,
        state_root: Option<&Path>,
    ) -> Result<Self> {
        if execution_id.is_empty()
            || execution_id.len() > MAX_EXECUTION_ID_BYTES
            || execution_id.chars().any(char::is_control)
        {
            return Err(
                "session history execution id is empty, too long, or contains control characters"
                    .into(),
            );
        }
        let state_root = match state_root {
            Some(root) => Some(validate_state_root(root)?),
            None => None,
        };
        Ok(Self {
            execution_id: execution_id.to_owned(),
            policy,
            state_root,
            state: Arc::new(Mutex::new(TranscriptState::default())),
        })
    }

    pub(crate) fn policy(&self) -> SessionHistoryPolicy {
        self.policy
    }

    pub(crate) fn wrap_output_reader(
        &self,
        reader: Box<dyn Read + Send>,
    ) -> Result<Box<dyn Read + Send>> {
        if !self.policy.transcript_enabled {
            return Ok(reader);
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| "terminal history state lock is poisoned")?;
        if state.reader_taken {
            return Err("terminal history output reader has already been wrapped".into());
        }
        if state.persisted {
            return Err("terminal history was already persisted before output capture".into());
        }
        state.reader_taken = true;
        drop(state);
        Ok(Box::new(HistoryReader {
            inner: reader,
            quota: self.policy.transcript_byte_quota,
            state: Arc::clone(&self.state),
            eof_observed: false,
        }))
    }

    pub(crate) fn persist(&self) -> Result<Option<PersistedSessionHistory>> {
        if !self.policy.transcript_enabled {
            return Ok(None);
        }
        if Arc::strong_count(&self.state) != 1 {
            return Err(
                "drop the terminal output reader before persisting transcript history so retention metadata is final"
                    .into(),
            );
        }

        let (observed_bytes, retained, capture_complete, quota_truncated) = {
            let state = self
                .state
                .lock()
                .map_err(|_| "terminal history state lock is poisoned")?;
            if !state.reader_taken {
                return Err(
                    "terminal transcript history cannot be persisted before an output reader was captured"
                        .into(),
                );
            }
            if state.persisted {
                return Err("session history has already been persisted".into());
            }
            (
                state.observed_bytes,
                state.retained.clone(),
                state.capture_complete,
                state.quota_truncated,
            )
        };
        let state_root = self
            .state_root
            .as_ref()
            .ok_or("enabled transcript history is missing its local state root")?;
        let storage_key = history_storage_key(&self.execution_id);
        let transcript_truncated = quota_truncated || !capture_complete;
        let transcript_digest = lower_sha256(&retained);
        let transcript_name = format!("transcript.{transcript_digest}.bin");
        let transcript_relative = PathBuf::from("history")
            .join(&storage_key)
            .join(&transcript_name);
        let transcript = HistoryBlobRef {
            relative_path: utf8_relative(&transcript_relative)?,
            sha256: transcript_digest,
            captured_bytes: retained.len(),
        };
        let manifest = SessionHistoryManifest {
            schema_version: HISTORY_SCHEMA_VERSION,
            execution_id: self.execution_id.clone(),
            local_only: true,
            secret_filtering: SECRET_FILTERING_MODE.to_owned(),
            policy: self.policy,
            transcript_observed_bytes: observed_bytes,
            transcript_retained_bytes: retained.len(),
            transcript_capture_complete: capture_complete,
            transcript_truncated,
            transcript,
        };
        validate_manifest(&manifest)?;
        let manifest_bytes = serde_json::to_vec(&manifest)?;
        let manifest_digest = lower_sha256(&manifest_bytes);
        let manifest_name = format!("manifest.{manifest_digest}.json");
        let manifest_relative = PathBuf::from("history")
            .join(&storage_key)
            .join(&manifest_name);
        let manifest_blob = HistoryBlobRef {
            relative_path: utf8_relative(&manifest_relative)?,
            sha256: manifest_digest,
            captured_bytes: manifest_bytes.len(),
        };
        let required_bytes = u64::try_from(retained.len())?
            .checked_add(u64::try_from(manifest_bytes.len())?)
            .ok_or("terminal history logical byte size overflowed")?;
        if required_bytes > self.policy.total_history_byte_quota {
            return Err(format!(
                "one terminal history record requires {required_bytes} bytes, exceeding its total history quota of {} bytes",
                self.policy.total_history_byte_quota
            )
            .into());
        }

        with_history_write_lock(state_root, &self.execution_id, |history_root| {
            prune_for_write(
                history_root,
                &storage_key,
                required_bytes,
                self.policy.total_history_byte_quota,
            )?;
            let session_dir = history_root.join(&storage_key);
            create_private_directory(&session_dir)?;
            let write_result = (|| -> Result<()> {
                write_private_file(&session_dir.join(&transcript_name), &retained)?;
                write_private_file(&session_dir.join(&manifest_name), &manifest_bytes)?;
                Ok(())
            })();
            if let Err(error) = write_result {
                let _ = fs::remove_dir_all(&session_dir);
                return Err(error);
            }
            let usage = history_logical_bytes(history_root)?;
            if usage > self.policy.total_history_byte_quota {
                let _ = fs::remove_dir_all(&session_dir);
                return Err(format!(
                    "terminal history quota verification failed after write: {usage} > {}",
                    self.policy.total_history_byte_quota
                )
                .into());
            }
            Ok(())
        })?;
        self.state
            .lock()
            .map_err(|_| "terminal history state lock is poisoned")?
            .persisted = true;
        Ok(Some(PersistedSessionHistory {
            manifest,
            manifest_blob,
        }))
    }
}

struct HistoryReader {
    inner: Box<dyn Read + Send>,
    quota: usize,
    state: Arc<Mutex<TranscriptState>>,
    eof_observed: bool,
}

impl Read for HistoryReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buffer)?;
        if read == 0 {
            self.eof_observed = true;
            self.state
                .lock()
                .map_err(|_| std::io::Error::other("terminal history state lock is poisoned"))?
                .capture_complete = true;
            return Ok(0);
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| std::io::Error::other("terminal history state lock is poisoned"))?;
        state.observed_bytes = state
            .observed_bytes
            .saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
        let keep = self.quota.saturating_sub(state.retained.len()).min(read);
        state.retained.extend_from_slice(&buffer[..keep]);
        state.quota_truncated |= keep < read;
        Ok(read)
    }
}

impl Drop for HistoryReader {
    fn drop(&mut self) {
        if self.eof_observed {
            return;
        }
        if let Ok(mut state) = self.state.lock() {
            state.capture_complete = false;
        }
    }
}

pub(crate) fn persisted_arguments(
    arguments: &[String],
    policy: SessionHistoryPolicy,
) -> Vec<String> {
    if !policy.command_history_enabled {
        return vec![HISTORY_DISABLED.to_owned()];
    }
    sanitize_persisted_arguments(arguments)
}

pub(crate) fn sanitize_persisted_arguments(arguments: &[String]) -> Vec<String> {
    let mut sanitized = Vec::with_capacity(arguments.len());
    let mut redact_next = false;
    for argument in arguments {
        if redact_next {
            sanitized.push(REDACTED.to_owned());
            redact_next = false;
            continue;
        }
        let lower = argument.to_ascii_lowercase();
        if is_secret_option(&lower) {
            sanitized.push(argument.clone());
            redact_next = true;
        } else if contains_obvious_secret_assignment(&lower)
            || lower.contains("authorization:")
            || lower.contains("proxy-authorization:")
        {
            sanitized.push(REDACTED.to_owned());
        } else if let Some(url) = sanitize_url_like_argument(argument) {
            sanitized.push(url);
        } else {
            sanitized.push(argument.clone());
        }
    }
    sanitized
}

fn is_secret_option(lower: &str) -> bool {
    matches!(
        lower.trim_start_matches('-').replace('_', "-").as_str(),
        "api-key"
            | "apikey"
            | "token"
            | "access-token"
            | "auth-token"
            | "authorization"
            | "password"
            | "passwd"
            | "secret"
            | "client-secret"
            | "database-url"
            | "dsn"
    )
}

fn contains_obvious_secret_assignment(lower: &str) -> bool {
    [
        "api_key=",
        "api-key=",
        "apikey=",
        "token=",
        "access_token=",
        "access-token=",
        "auth_token=",
        "auth-token=",
        "authorization=",
        "password=",
        "passwd=",
        "secret=",
        "client_secret=",
        "client-secret=",
        "database_url=",
        "database-url=",
        "dsn=",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn sanitize_url_like_argument(argument: &str) -> Option<String> {
    if argument.chars().any(char::is_whitespace) {
        return None;
    }
    let (scheme, rest) = argument.split_once("://")?;
    let valid_scheme = !scheme.is_empty()
        && scheme.chars().enumerate().all(|(index, ch)| {
            if index == 0 {
                ch.is_ascii_alphabetic()
            } else {
                ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.')
            }
        });
    if !valid_scheme {
        return Some(REDACTED.to_owned());
    }
    let tail = rest
        .char_indices()
        .find_map(|(index, ch)| matches!(ch, '?' | '#').then_some(index))
        .unwrap_or(rest.len());
    let without_tail = &rest[..tail];
    let authority_end = without_tail.find('/').unwrap_or(without_tail.len());
    let raw_authority = &without_tail[..authority_end];
    let authority = raw_authority
        .rsplit_once('@')
        .map_or(raw_authority, |(_, host)| host);
    if authority.is_empty() && !scheme.eq_ignore_ascii_case("file") {
        return Some(REDACTED.to_owned());
    }
    Some(format!(
        "{}://{authority}{}",
        scheme.to_ascii_lowercase(),
        &without_tail[authority_end..]
    ))
}

fn validate_state_root(state_root: &Path) -> Result<PathBuf> {
    if !state_root.is_absolute() {
        return Err("terminal history state root must be absolute".into());
    }
    let canonical = fs::canonicalize(state_root)?;
    if !canonical.is_dir() {
        return Err("terminal history state root must be a directory".into());
    }
    let db = canonical.join("winds.db");
    let metadata = fs::symlink_metadata(&db)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("terminal history state root must contain the real Winds database".into());
    }
    Ok(canonical)
}

fn with_history_write_lock<T>(
    state_root: &Path,
    execution_id: &str,
    operation: impl FnOnce(&Path) -> Result<T>,
) -> Result<T> {
    let database = state_root.join("winds.db");
    let connection = Connection::open_with_flags(&database, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.execute_batch("BEGIN IMMEDIATE")?;
    let result = (|| -> Result<T> {
        let terminal_count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM executions WHERE execution_id = ?1 AND kind = 'TERMINAL'",
            params![execution_id],
            |row| row.get(0),
        )?;
        if terminal_count != 1 {
            return Err(
                "terminal history state root does not contain the matching terminal execution"
                    .into(),
            );
        }
        let history_root = state_root.join("history");
        ensure_private_directory(&history_root)?;
        operation(&history_root)
    })();
    let rollback = connection.execute_batch("ROLLBACK");
    match (result, rollback) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error.into()),
    }
}

fn ensure_private_directory(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err("terminal history path must be a real directory".into());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            create_private_directory(path)?;
        }
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

fn create_private_directory(path: &Path) -> Result<()> {
    let mut builder = DirBuilder::new();
    builder.recursive(false);
    #[cfg(unix)]
    builder.mode(0o700);
    builder.create(path)?;
    Ok(())
}

fn write_private_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

#[derive(Debug)]
struct RetainedHistoryDir {
    path: PathBuf,
    logical_bytes: u64,
    modified: SystemTime,
}

fn prune_for_write(
    history_root: &Path,
    new_storage_key: &str,
    required_bytes: u64,
    total_quota: u64,
) -> Result<()> {
    if history_root.join(new_storage_key).exists() {
        return Err("terminal history for this execution already exists or is incomplete".into());
    }
    let mut entries = retained_history_dirs(history_root)?;
    let mut existing = entries.iter().try_fold(0_u64, |sum, entry| {
        sum.checked_add(entry.logical_bytes)
            .ok_or("terminal history logical byte size overflowed")
    })?;
    let budget = total_quota
        .checked_sub(required_bytes)
        .ok_or("terminal history record exceeds total history quota")?;
    entries.sort_by(|left, right| {
        left.modified
            .cmp(&right.modified)
            .then_with(|| left.path.cmp(&right.path))
    });
    for entry in entries {
        if existing <= budget {
            break;
        }
        fs::remove_dir_all(&entry.path)?;
        existing = existing.saturating_sub(entry.logical_bytes);
    }
    if existing > budget {
        return Err("terminal history quota could not be satisfied by retention pruning".into());
    }
    Ok(())
}

fn retained_history_dirs(history_root: &Path) -> Result<Vec<RetainedHistoryDir>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(history_root)? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err("terminal history root contains an unexpected non-directory entry".into());
        }
        let name = entry
            .file_name()
            .to_str()
            .ok_or("terminal history directory name is not valid UTF-8")?
            .to_owned();
        if !name.starts_with("session-") {
            return Err("terminal history root contains an unrecognized directory".into());
        }
        entries.push(RetainedHistoryDir {
            logical_bytes: session_logical_bytes(&entry.path())?,
            modified: metadata.modified()?,
            path: entry.path(),
        });
    }
    Ok(entries)
}

fn session_logical_bytes(session_dir: &Path) -> Result<u64> {
    let mut total = 0_u64;
    for entry in fs::read_dir(session_dir)? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err("terminal history session contains an unexpected non-file entry".into());
        }
        total = total
            .checked_add(metadata.len())
            .ok_or("terminal history logical byte size overflowed")?;
    }
    Ok(total)
}

fn history_logical_bytes(history_root: &Path) -> Result<u64> {
    retained_history_dirs(history_root)?
        .into_iter()
        .try_fold(0_u64, |sum, entry| {
            sum.checked_add(entry.logical_bytes)
                .ok_or_else(|| "terminal history logical byte size overflowed".into())
        })
}

fn validate_manifest(manifest: &SessionHistoryManifest) -> Result<()> {
    if manifest.schema_version != HISTORY_SCHEMA_VERSION
        || !manifest.local_only
        || manifest.secret_filtering != SECRET_FILTERING_MODE
    {
        return Err("session history manifest has unsupported semantics".into());
    }
    let retained = u64::try_from(manifest.transcript_retained_bytes)?;
    if retained > manifest.transcript_observed_bytes
        || manifest.transcript_retained_bytes > manifest.policy.transcript_byte_quota
        || manifest.transcript_truncated
            != (!manifest.transcript_capture_complete
                || manifest.transcript_observed_bytes > retained)
        || manifest.transcript.captured_bytes != manifest.transcript_retained_bytes
    {
        return Err("session history retention metadata is inconsistent".into());
    }
    Ok(())
}

fn history_storage_key(execution_id: &str) -> String {
    format!("session-{}", lower_sha256(execution_id.as_bytes()))
}

fn lower_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn utf8_relative(path: &Path) -> Result<String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| "terminal history relative path is not valid UTF-8".into())
}

#[cfg(test)]
mod tests {
    use super::{
        HARD_MAX_TRANSCRIPT_BYTES, HISTORY_DISABLED, REDACTED, SessionHistoryPolicy,
        SessionHistoryRecorder, history_logical_bytes, persisted_arguments, prune_for_write,
        sanitize_persisted_arguments,
    };
    use crate::domain::{ExecutionKind, FactSource};
    use crate::store::{NewExecution, NewTerminalSession, NewWorkspace, Store};
    use std::fs;
    use std::io::{Cursor, Read};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new(name: &str) -> Self {
            let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "winds-t056-{name}-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(fs::canonicalize(path).unwrap())
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn state_with_terminal_executions(root: &TestRoot, execution_ids: &[&str]) -> PathBuf {
        let state_root = root.path().join("state");
        let workspace_root = root.path().join("workspace");
        fs::create_dir(&workspace_root).unwrap();
        let workspace_root = fs::canonicalize(workspace_root).unwrap();
        let mut store = Store::open(&state_root).unwrap();
        store
            .create_workspace(
                NewWorkspace {
                    workspace_id: "workspace-history",
                    canonical_worktree_root: workspace_root.to_str().unwrap(),
                    git_common_dir: workspace_root.join(".git").to_str().unwrap(),
                },
                1,
            )
            .unwrap();
        let no_arguments: Vec<String> = Vec::new();
        for (index, execution_id) in execution_ids.iter().enumerate() {
            store
                .create_terminal_execution(
                    NewExecution {
                        execution_id,
                        workspace_id: "workspace-history",
                        kind: ExecutionKind::Terminal,
                        request_source: FactSource::CallerRequested,
                        execution_domain: "native-test",
                    },
                    NewTerminalSession {
                        execution_id,
                        profile_id: "history-test-profile",
                        shell_executable: "/history-test-shell",
                        shell_arguments: &no_arguments,
                        requested_cwd: workspace_root.to_str().unwrap(),
                        initial_cols: Some(80),
                        initial_rows: Some(24),
                    },
                    i64::try_from(index + 2).unwrap(),
                )
                .unwrap();
        }
        drop(store);
        fs::canonicalize(state_root).unwrap()
    }

    fn capture_all(recorder: &SessionHistoryRecorder, bytes: &[u8]) {
        let mut reader = recorder
            .wrap_output_reader(Box::new(Cursor::new(bytes.to_vec())))
            .unwrap();
        let mut observed = Vec::new();
        reader.read_to_end(&mut observed).unwrap();
        assert_eq!(observed, bytes);
        drop(reader);
    }

    #[test]
    fn disabled_policy_withholds_command_text() {
        let policy = SessionHistoryPolicy::disabled();
        assert_eq!(
            persisted_arguments(&["secret-looking-command".to_owned()], policy),
            vec![HISTORY_DISABLED.to_owned()]
        );
    }

    #[test]
    fn policy_enforces_per_session_and_total_quotas() {
        assert!(SessionHistoryPolicy::local_bounded(true, 0, 10).is_err());
        assert!(
            SessionHistoryPolicy::local_bounded(false, HARD_MAX_TRANSCRIPT_BYTES + 1, u64::MAX)
                .is_err()
        );
        assert!(SessionHistoryPolicy::local_bounded(false, 10, 9).is_err());
        assert!(SessionHistoryPolicy::local_bounded(false, 10, 10_000).is_ok());
    }

    #[test]
    fn metadata_sanitizer_redacts_obvious_secret_shapes_and_url_credentials() {
        let arguments = vec![
            "--api-key".to_owned(),
            "sk-super-secret".to_owned(),
            "PASSWORD=hunter2".to_owned(),
            "https://user:pass@example.com/repo?token=abc#frag".to_owned(),
            "Authorization: Bearer abc".to_owned(),
            "ordinary".to_owned(),
        ];
        let sanitized = sanitize_persisted_arguments(&arguments);
        assert_eq!(sanitized[0], "--api-key");
        assert_eq!(sanitized[1], REDACTED);
        assert_eq!(sanitized[2], REDACTED);
        assert_eq!(sanitized[3], REDACTED);
        assert_eq!(sanitized[4], REDACTED);
        assert_eq!(sanitized[5], "ordinary");
        let joined = sanitized.join(" ");
        assert!(!joined.contains("super-secret"));
        assert!(!joined.contains("hunter2"));
        assert!(!joined.contains("user:pass"));
        assert!(!joined.contains("token=abc"));
        assert!(!joined.contains("Bearer abc"));
    }

    #[test]
    fn bounded_transcript_records_quota_and_complete_capture_truth() {
        let root = TestRoot::new("bounded");
        let state_root = state_with_terminal_executions(&root, &["execution-one"]);
        let policy = SessionHistoryPolicy::local_bounded(true, 5, 16_384).unwrap();
        let recorder =
            SessionHistoryRecorder::new_local("execution-one", policy, &state_root).unwrap();
        capture_all(&recorder, b"abcdefgh");
        let persisted = recorder.persist().unwrap().unwrap();
        assert_eq!(persisted.manifest.transcript_observed_bytes, 8);
        assert_eq!(persisted.manifest.transcript_retained_bytes, 5);
        assert!(persisted.manifest.transcript_capture_complete);
        assert!(persisted.manifest.transcript_truncated);
        assert_eq!(
            fs::read(state_root.join(&persisted.manifest.transcript.relative_path)).unwrap(),
            b"abcde"
        );
        assert!(history_logical_bytes(&state_root.join("history")).unwrap() <= 16_384);
    }

    #[test]
    fn early_reader_drop_is_explicitly_truncated_not_falsely_complete() {
        let root = TestRoot::new("incomplete");
        let state_root = state_with_terminal_executions(&root, &["execution-two"]);
        let policy = SessionHistoryPolicy::local_bounded(false, 8, 16_384).unwrap();
        let recorder =
            SessionHistoryRecorder::new_local("execution-two", policy, &state_root).unwrap();
        let mut reader = recorder
            .wrap_output_reader(Box::new(Cursor::new(b"abcdefgh".to_vec())))
            .unwrap();
        let mut partial = [0_u8; 3];
        reader.read_exact(&mut partial).unwrap();
        drop(reader);
        let persisted = recorder.persist().unwrap().unwrap();
        assert_eq!(&partial, b"abc");
        assert!(!persisted.manifest.transcript_capture_complete);
        assert!(persisted.manifest.transcript_truncated);
        assert_eq!(persisted.manifest.transcript_observed_bytes, 3);
        assert_eq!(persisted.manifest.transcript_retained_bytes, 3);
    }

    #[test]
    fn transcript_history_cannot_persist_before_output_reader_capture() {
        let root = TestRoot::new("no-reader");
        let state_root = state_with_terminal_executions(&root, &["execution-three"]);
        let policy = SessionHistoryPolicy::local_bounded(false, 8, 16_384).unwrap();
        let recorder =
            SessionHistoryRecorder::new_local("execution-three", policy, &state_root).unwrap();
        assert!(recorder.persist().is_err());
        assert!(!state_root.join("history").exists());
    }

    #[test]
    fn wrong_state_root_cannot_receive_terminal_history() {
        let root = TestRoot::new("wrong-root");
        let correct = state_with_terminal_executions(&root, &["execution-four"]);
        let other_root = TestRoot::new("other-state");
        let other = state_with_terminal_executions(&other_root, &["other-execution"]);
        let policy = SessionHistoryPolicy::local_bounded(false, 8, 16_384).unwrap();
        let recorder = SessionHistoryRecorder::new_local("execution-four", policy, &other).unwrap();
        capture_all(&recorder, b"safe");
        assert!(recorder.persist().is_err());
        assert!(!other.join("history").exists());
        assert!(!correct.join("history").exists());
    }

    #[test]
    fn total_quota_prunes_old_sessions_across_repeated_terminal_history() {
        let root = TestRoot::new("retention");
        let state_root = state_with_terminal_executions(
            &root,
            &["retention-one", "retention-two", "retention-three"],
        );
        let policy = SessionHistoryPolicy::local_bounded(false, 4, 1_024).unwrap();
        for execution_id in ["retention-one", "retention-two", "retention-three"] {
            let recorder =
                SessionHistoryRecorder::new_local(execution_id, policy, &state_root).unwrap();
            capture_all(&recorder, b"abcdefgh");
            recorder.persist().unwrap().unwrap();
            assert!(history_logical_bytes(&state_root.join("history")).unwrap() <= 1_024);
        }
        let retained_count = fs::read_dir(state_root.join("history")).unwrap().count();
        assert!(retained_count < 3);
    }

    #[test]
    fn quota_helper_prunes_existing_sessions_before_new_write() {
        let root = TestRoot::new("retention-helper");
        let history = root.path().join("history");
        fs::create_dir(&history).unwrap();
        for name in ["session-a", "session-b"] {
            let dir = history.join(name);
            fs::create_dir(&dir).unwrap();
            fs::write(dir.join("blob"), b"1234").unwrap();
        }
        assert_eq!(history_logical_bytes(&history).unwrap(), 8);
        prune_for_write(&history, "session-new", 8, 8).unwrap();
        assert_eq!(history_logical_bytes(&history).unwrap(), 0);
    }
}
