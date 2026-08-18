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
const HISTORY_WRITE_LOCK_DB: &str = ".winds-history-write-lock.sqlite3";
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
                "terminal transcript byte quota exceeds the Winds implementation safety maximum of {HARD_MAX_TRANSCRIPT_BYTES} bytes"
            )
            .into());
        }
        let policy = Self {
            command_history_enabled,
            transcript_enabled: true,
            transcript_byte_quota,
            total_history_byte_quota,
        };
        if u64::try_from(minimum_manifest_bytes("x", policy)?)? > total_history_byte_quota {
            return Err(
                "total terminal history byte quota is too small for mandatory history metadata"
                    .into(),
            );
        }
        Ok(policy)
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
    reader_active: bool,
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
        if policy.transcript_enabled
            && u64::try_from(minimum_manifest_bytes(execution_id, policy)?)?
                > policy.total_history_byte_quota
        {
            return Err(
                "terminal history quota cannot hold mandatory metadata for this execution".into(),
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
        state.reader_active = true;
        drop(state);
        let total_quota =
            usize::try_from(self.policy.total_history_byte_quota).unwrap_or(usize::MAX);
        Ok(Box::new(HistoryReader {
            inner: reader,
            quota: self.policy.transcript_byte_quota.min(total_quota),
            state: Arc::clone(&self.state),
            eof_observed: false,
        }))
    }

    pub(crate) fn persist(&self) -> Result<Option<PersistedSessionHistory>> {
        if !self.policy.transcript_enabled {
            return Ok(None);
        }

        let (observed_bytes, retained, capture_complete, quota_truncated) = {
            let state = self
                .state
                .lock()
                .map_err(|_| "terminal history state lock is poisoned")?;
            if state.reader_active {
                return Err(
                    "drop the terminal output reader before persisting transcript history so retention metadata is final"
                        .into(),
                );
            }
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
        let mut retained = retained;
        let (manifest, manifest_bytes, transcript_name, required_bytes) = loop {
            let retained_bytes = u64::try_from(retained.len())?;
            let transcript_truncated =
                quota_truncated || !capture_complete || observed_bytes > retained_bytes;
            let (manifest, manifest_bytes, transcript_name) = build_history_manifest(
                &self.execution_id,
                self.policy,
                &storage_key,
                observed_bytes,
                &retained,
                capture_complete,
                transcript_truncated,
            )?;
            let required_bytes = retained_bytes
                .checked_add(u64::try_from(manifest_bytes.len())?)
                .ok_or("terminal history logical byte size overflowed")?;
            if required_bytes <= self.policy.total_history_byte_quota {
                break (manifest, manifest_bytes, transcript_name, required_bytes);
            }
            let manifest_bytes_u64 = u64::try_from(manifest_bytes.len())?;
            let available_for_transcript = self
                .policy
                .total_history_byte_quota
                .checked_sub(manifest_bytes_u64)
                .ok_or("terminal history quota cannot hold mandatory manifest metadata")?;
            let available_for_transcript =
                usize::try_from(available_for_transcript).unwrap_or(usize::MAX);
            if available_for_transcript >= retained.len() {
                return Err("terminal history quota accounting did not converge".into());
            }
            retained.truncate(available_for_transcript);
        };
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
                return match remove_owned_history_session(history_root, &session_dir) {
                    Ok(()) => Err(error),
                    Err(cleanup_error) => Err(format!(
                        "terminal history write failed: {error}; owned-session cleanup also failed: {cleanup_error}"
                    )
                    .into()),
                };
            }
            let usage = history_logical_bytes(history_root)?;
            if usage > self.policy.total_history_byte_quota {
                return match remove_owned_history_session(history_root, &session_dir) {
                    Ok(()) => Err(format!(
                        "terminal history quota verification failed after write: {usage} > {}",
                        self.policy.total_history_byte_quota
                    )
                    .into()),
                    Err(cleanup_error) => Err(format!(
                        "terminal history quota verification failed after write: {usage} > {}; owned-session cleanup also failed: {cleanup_error}",
                        self.policy.total_history_byte_quota
                    )
                    .into()),
                };
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
        if buffer.is_empty() {
            return Ok(0);
        }
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
        if let Ok(mut state) = self.state.lock() {
            state.reader_active = false;
            if !self.eof_observed {
                state.capture_complete = false;
            }
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
            || (argument.chars().any(char::is_whitespace)
                && contains_sensitive_url_like_token(argument))
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

fn contains_sensitive_url_like_token(argument: &str) -> bool {
    argument.split_whitespace().any(|token| {
        let token = token.trim_matches(|ch: char| {
            matches!(
                ch,
                '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' | ',' | ';'
            )
        });
        let Some((scheme, rest)) = token.split_once("://") else {
            return false;
        };
        if !is_valid_url_scheme(scheme) {
            return false;
        }
        let authority_end = rest
            .char_indices()
            .find_map(|(index, ch)| matches!(ch, '/' | '?' | '#').then_some(index))
            .unwrap_or(rest.len());
        let authority = &rest[..authority_end];
        authority.contains('@') || rest.contains('?') || rest.contains('#')
    })
}

fn is_valid_url_scheme(scheme: &str) -> bool {
    !scheme.is_empty()
        && scheme.chars().enumerate().all(|(index, ch)| {
            if index == 0 {
                ch.is_ascii_alphabetic()
            } else {
                ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.')
            }
        })
}

fn sanitize_url_like_argument(argument: &str) -> Option<String> {
    if argument.chars().any(char::is_whitespace) {
        return None;
    }
    let (scheme, rest) = argument.split_once("://")?;
    if !is_valid_url_scheme(scheme) {
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
    validate_terminal_history_identity(state_root, execution_id)?;
    let history_root = state_root.join("history");
    ensure_private_directory(&history_root)?;

    let lock_database = state_root.join(HISTORY_WRITE_LOCK_DB);
    ensure_private_lock_database(&lock_database)?;
    let lock_connection = Connection::open_with_flags(
        &lock_database,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )?;
    lock_connection.busy_timeout(Duration::from_secs(5))?;
    lock_connection.execute_batch("BEGIN IMMEDIATE")?;
    let result = operation(&history_root);
    let release = lock_connection.execute_batch("ROLLBACK");
    match (result, release) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error.into()),
    }
}

fn validate_terminal_history_identity(state_root: &Path, execution_id: &str) -> Result<()> {
    let database = state_root.join("winds.db");
    let connection = Connection::open_with_flags(
        &database,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.execute_batch("BEGIN IMMEDIATE")?;
    let result = (|| -> Result<()> {
        let terminal_count: i64 = connection.query_row(
            "SELECT COUNT(*)
             FROM executions e
             INNER JOIN terminal_sessions t ON t.execution_id = e.execution_id
             WHERE e.execution_id = ?1 AND e.kind = ?2",
            params![
                execution_id,
                crate::domain::ExecutionKind::Terminal.as_str()
            ],
            |row| row.get(0),
        )?;
        if terminal_count != 1 {
            return Err(
                "terminal history state root does not contain the matching complete terminal execution"
                    .into(),
            );
        }
        Ok(())
    })();
    let release = connection.execute_batch("ROLLBACK");
    match (result, release) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) => Err(error.into()),
    }
}

fn ensure_private_lock_database(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err("terminal history write lock must be a real file".into());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            options.mode(0o600);
            match options.open(path) {
                Ok(file) => file.sync_all()?,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error.into()),
            }
            let metadata = fs::symlink_metadata(path)?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err("terminal history write lock must be a real file".into());
            }
        }
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

fn ensure_private_directory(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err("terminal history path must be a real directory".into());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut builder = DirBuilder::new();
            builder.recursive(false);
            #[cfg(unix)]
            builder.mode(0o700);
            match builder.create(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error.into()),
            }
            let metadata = fs::symlink_metadata(path)?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err("terminal history path must be a real directory".into());
            }
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
    if !is_history_storage_key(new_storage_key) {
        return Err("terminal history storage key is invalid".into());
    }
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
        remove_owned_history_session(history_root, &entry.path)?;
        existing = existing.saturating_sub(entry.logical_bytes);
    }
    if existing > budget {
        return Err("terminal history quota could not be satisfied by retention pruning".into());
    }
    Ok(())
}

fn remove_owned_history_session(history_root: &Path, target: &Path) -> Result<()> {
    let history_metadata = fs::symlink_metadata(history_root)?;
    if history_metadata.file_type().is_symlink() || !history_metadata.is_dir() {
        return Err("terminal history root must be a real owned directory before deletion".into());
    }
    let target_metadata = fs::symlink_metadata(target)?;
    if target_metadata.file_type().is_symlink() || !target_metadata.is_dir() {
        return Err("terminal history deletion target must be a real directory".into());
    }
    let canonical_history_root = fs::canonicalize(history_root)?;
    let canonical_target = fs::canonicalize(target)?;
    if canonical_target == canonical_history_root
        || canonical_target.parent() != Some(canonical_history_root.as_path())
    {
        return Err("terminal history deletion target is outside the owned history root".into());
    }
    let name = canonical_target
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("terminal history deletion target name is not valid UTF-8")?;
    if !is_history_storage_key(name) {
        return Err("terminal history deletion target is not an owned session directory".into());
    }
    fs::remove_dir_all(&canonical_target)?;
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
        if !is_history_storage_key(&name) {
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

fn is_history_storage_key(name: &str) -> bool {
    let Some(digest) = name.strip_prefix("session-") else {
        return false;
    };
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
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

fn minimum_manifest_bytes(execution_id: &str, policy: SessionHistoryPolicy) -> Result<usize> {
    let storage_key = history_storage_key(execution_id);
    let (_, manifest_bytes, _) = build_history_manifest(
        execution_id,
        policy,
        &storage_key,
        u64::MAX,
        &[],
        false,
        true,
    )?;
    Ok(manifest_bytes.len())
}

fn build_history_manifest(
    execution_id: &str,
    policy: SessionHistoryPolicy,
    storage_key: &str,
    observed_bytes: u64,
    retained: &[u8],
    capture_complete: bool,
    transcript_truncated: bool,
) -> Result<(SessionHistoryManifest, Vec<u8>, String)> {
    let transcript_digest = lower_sha256(retained);
    let transcript_name = format!("transcript.{transcript_digest}.bin");
    let transcript_relative = PathBuf::from("history")
        .join(storage_key)
        .join(&transcript_name);
    let transcript = HistoryBlobRef {
        relative_path: utf8_relative(&transcript_relative)?,
        sha256: transcript_digest,
        captured_bytes: retained.len(),
    };
    let manifest = SessionHistoryManifest {
        schema_version: HISTORY_SCHEMA_VERSION,
        execution_id: execution_id.to_owned(),
        local_only: true,
        secret_filtering: SECRET_FILTERING_MODE.to_owned(),
        policy,
        transcript_observed_bytes: observed_bytes,
        transcript_retained_bytes: retained.len(),
        transcript_capture_complete: capture_complete,
        transcript_truncated,
        transcript,
    };
    validate_manifest(&manifest)?;
    let manifest_bytes = serde_json::to_vec(&manifest)?;
    Ok((manifest, manifest_bytes, transcript_name))
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
        SessionHistoryRecorder, ensure_private_directory, history_logical_bytes,
        history_storage_key, lower_sha256, persisted_arguments, prune_for_write,
        remove_owned_history_session, sanitize_persisted_arguments, with_history_write_lock,
    };
    use crate::domain::{ExecutionKind, FactSource};
    use crate::store::{NewExecution, NewTerminalSession, NewWorkspace, Store};
    use rusqlite::{Connection, OpenFlags};
    use std::fs;
    use std::io::{Cursor, Read};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Barrier};
    use std::thread;

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
    fn enabled_command_history_persists_sanitized_arguments() {
        let policy = SessionHistoryPolicy::command_history_only();
        assert_eq!(
            persisted_arguments(
                &["--api-key".to_owned(), "sk-super-secret".to_owned()],
                policy
            ),
            vec!["--api-key".to_owned(), REDACTED.to_owned()]
        );
    }

    #[test]
    fn policy_enforces_per_session_and_total_quotas() {
        assert!(SessionHistoryPolicy::local_bounded(true, 0, 10).is_err());
        assert!(
            SessionHistoryPolicy::local_bounded(false, HARD_MAX_TRANSCRIPT_BYTES + 1, u64::MAX)
                .is_err()
        );
        assert!(SessionHistoryPolicy::local_bounded(false, 10, 10).is_err());
        assert!(SessionHistoryPolicy::local_bounded(false, 1_024, 1_024).is_ok());
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
            "curl https://user:pass@example.com/repo".to_owned(),
            "curl <https://user:pass@example.com/repo>".to_owned(),
            "curl https://example.com/repo?opaque=value#frag".to_owned(),
            "curl https://example.com/repo".to_owned(),
            "ordinary".to_owned(),
        ];
        let sanitized = sanitize_persisted_arguments(&arguments);
        assert_eq!(sanitized[0], "--api-key");
        assert_eq!(sanitized[1], REDACTED);
        assert_eq!(sanitized[2], REDACTED);
        assert_eq!(sanitized[3], REDACTED);
        assert_eq!(sanitized[4], REDACTED);
        assert_eq!(sanitized[5], REDACTED);
        assert_eq!(sanitized[6], REDACTED);
        assert_eq!(sanitized[7], REDACTED);
        assert_eq!(sanitized[8], "curl https://example.com/repo");
        assert_eq!(sanitized[9], "ordinary");
        let joined = sanitized.join(" ");
        assert!(!joined.contains("super-secret"));
        assert!(!joined.contains("hunter2"));
        assert!(!joined.contains("user:pass"));
        assert!(!joined.contains("token=abc"));
        assert!(!joined.contains("Bearer abc"));
        assert!(!joined.contains("opaque=value"));
    }

    #[test]
    fn concurrent_history_root_initialization_is_idempotent_and_fail_closed() {
        let root = TestRoot::new("history-root-race");
        let history = Arc::new(root.path().join("history"));
        let barrier = Arc::new(Barrier::new(8));
        let handles = (0..8)
            .map(|_| {
                let history = Arc::clone(&history);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    ensure_private_directory(&history)
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap().unwrap();
        }
        let metadata = fs::symlink_metadata(history.as_path()).unwrap();
        assert!(!metadata.file_type().is_symlink());
        assert!(metadata.is_dir());
    }

    #[test]
    fn history_filesystem_lock_does_not_hold_winds_database_writer_lock() {
        let root = TestRoot::new("history-lock-separation");
        let state_root = state_with_terminal_executions(&root, &["history-lock-separation"]);
        with_history_write_lock(&state_root, "history-lock-separation", |_| {
            let connection = Connection::open_with_flags(
                state_root.join("winds.db"),
                OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
            )
            .unwrap();
            connection
                .execute_batch("BEGIN IMMEDIATE; ROLLBACK")
                .unwrap();
            Ok(())
        })
        .unwrap();
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
        let manifest_bytes =
            fs::read(state_root.join(&persisted.manifest_blob.relative_path)).unwrap();
        assert_eq!(manifest_bytes.len(), persisted.manifest_blob.captured_bytes);
        assert_eq!(
            lower_sha256(&manifest_bytes),
            persisted.manifest_blob.sha256
        );
        assert!(history_logical_bytes(&state_root.join("history")).unwrap() <= 16_384);
    }

    #[test]
    fn total_quota_can_equal_transcript_limit_and_reserves_manifest_space_by_truncation() {
        let root = TestRoot::new("quota-equality");
        let state_root = state_with_terminal_executions(&root, &["quota-equality"]);
        let policy = SessionHistoryPolicy::local_bounded(false, 1_024, 1_024).unwrap();
        let recorder =
            SessionHistoryRecorder::new_local("quota-equality", policy, &state_root).unwrap();
        capture_all(&recorder, &vec![b'x'; 1_024]);
        let persisted = recorder.persist().unwrap().unwrap();
        assert!(persisted.manifest.transcript_retained_bytes < 1_024);
        assert!(persisted.manifest.transcript_truncated);
        assert!(history_logical_bytes(&state_root.join("history")).unwrap() <= 1_024);
    }

    #[test]
    fn active_output_reader_blocks_persistence_until_drop() {
        let root = TestRoot::new("active-reader");
        let state_root = state_with_terminal_executions(&root, &["active-reader"]);
        let policy = SessionHistoryPolicy::local_bounded(false, 8, 16_384).unwrap();
        let recorder =
            SessionHistoryRecorder::new_local("active-reader", policy, &state_root).unwrap();
        let reader = recorder
            .wrap_output_reader(Box::new(Cursor::new(b"abcdefgh".to_vec())))
            .unwrap();
        assert!(recorder.persist().is_err());
        drop(reader);
        let persisted = recorder.persist().unwrap().unwrap();
        assert!(!persisted.manifest.transcript_capture_complete);
        assert!(persisted.manifest.transcript_truncated);
    }

    #[test]
    fn zero_length_read_does_not_fake_eof_or_complete_capture() {
        let root = TestRoot::new("zero-read");
        let state_root = state_with_terminal_executions(&root, &["zero-read"]);
        let policy = SessionHistoryPolicy::local_bounded(false, 8, 16_384).unwrap();
        let recorder = SessionHistoryRecorder::new_local("zero-read", policy, &state_root).unwrap();
        let mut reader = recorder
            .wrap_output_reader(Box::new(Cursor::new(b"abcdefgh".to_vec())))
            .unwrap();
        let mut empty = [];
        assert_eq!(reader.read(&mut empty).unwrap(), 0);
        drop(reader);
        let persisted = recorder.persist().unwrap().unwrap();
        assert!(!persisted.manifest.transcript_capture_complete);
        assert!(persisted.manifest.transcript_truncated);
        assert_eq!(persisted.manifest.transcript_observed_bytes, 0);
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
    fn incomplete_terminal_row_cannot_receive_history() {
        let root = TestRoot::new("missing-terminal-row");
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
        store
            .create_execution(
                NewExecution {
                    execution_id: "missing-terminal-row",
                    workspace_id: "workspace-history",
                    kind: ExecutionKind::Terminal,
                    request_source: FactSource::CallerRequested,
                    execution_domain: "native-test",
                },
                2,
            )
            .unwrap();
        drop(store);
        let state_root = fs::canonicalize(state_root).unwrap();
        let policy = SessionHistoryPolicy::local_bounded(false, 8, 16_384).unwrap();
        let recorder =
            SessionHistoryRecorder::new_local("missing-terminal-row", policy, &state_root).unwrap();
        capture_all(&recorder, b"safe");
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
        for execution_id in ["a", "b"] {
            let dir = history.join(history_storage_key(execution_id));
            fs::create_dir(&dir).unwrap();
            fs::write(dir.join("blob"), b"1234").unwrap();
        }
        assert_eq!(history_logical_bytes(&history).unwrap(), 8);
        prune_for_write(&history, &history_storage_key("new"), 8, 8).unwrap();
        assert_eq!(history_logical_bytes(&history).unwrap(), 0);
    }

    #[test]
    fn recursive_history_delete_rejects_root_outside_and_unrecognized_targets() {
        let root = TestRoot::new("delete-ownership");
        let history = root.path().join("history");
        fs::create_dir(&history).unwrap();
        let owned = history.join(history_storage_key("owned"));
        fs::create_dir(&owned).unwrap();
        fs::write(owned.join("blob"), b"safe").unwrap();
        remove_owned_history_session(&history, &owned).unwrap();
        assert!(!owned.exists());

        assert!(remove_owned_history_session(&history, &history).is_err());

        let outside = root.path().join(history_storage_key("outside"));
        fs::create_dir(&outside).unwrap();
        assert!(remove_owned_history_session(&history, &outside).is_err());

        let unexpected = history.join("session-not-a-sha256");
        fs::create_dir(&unexpected).unwrap();
        assert!(remove_owned_history_session(&history, &unexpected).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn recursive_history_delete_rejects_symlink_target() {
        use std::os::unix::fs::symlink;

        let root = TestRoot::new("delete-symlink");
        let history = root.path().join("history");
        fs::create_dir(&history).unwrap();
        let outside = root.path().join("outside");
        fs::create_dir(&outside).unwrap();
        let link = history.join(history_storage_key("linked"));
        symlink(&outside, &link).unwrap();
        assert!(remove_owned_history_session(&history, &link).is_err());
        assert!(outside.exists());
    }
}
