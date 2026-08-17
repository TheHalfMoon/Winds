use crate::domain::BlobEvidence;
use crate::store::{Result, Store};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::sync::{Arc, Mutex};

pub(crate) const HARD_MAX_TRANSCRIPT_BYTES: usize = 8 * 1024 * 1024;
const HISTORY_SCHEMA_VERSION: u32 = 1;
const REDACTED: &str = "<winds:redacted>";
const HISTORY_DISABLED: &str = "<winds:history-disabled>";
const SECRET_FILTERING_MODE: &str = "BEST_EFFORT_METADATA_REDACTION_NOT_SECRET_DETECTION";

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct SessionHistoryPolicy {
    command_history_enabled: bool,
    transcript_enabled: bool,
    transcript_byte_quota: usize,
}

impl SessionHistoryPolicy {
    pub const fn disabled() -> Self {
        Self {
            command_history_enabled: false,
            transcript_enabled: false,
            transcript_byte_quota: 0,
        }
    }

    pub const fn command_history_only() -> Self {
        Self {
            command_history_enabled: true,
            transcript_enabled: false,
            transcript_byte_quota: 0,
        }
    }

    pub fn local_bounded(
        command_history_enabled: bool,
        transcript_byte_quota: usize,
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
        Ok(Self {
            command_history_enabled,
            transcript_enabled: true,
            transcript_byte_quota,
        })
    }

    fn any_persistence_enabled(self) -> bool {
        self.command_history_enabled || self.transcript_enabled
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
    pub truncated: bool,
}

impl From<BlobEvidence> for HistoryBlobRef {
    fn from(value: BlobEvidence) -> Self {
        Self {
            relative_path: value.relative_path,
            sha256: value.sha256,
            captured_bytes: value.captured_bytes,
            truncated: value.truncated,
        }
    }
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
    pub transcript_truncated: bool,
    pub transcript: Option<HistoryBlobRef>,
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
    truncated: bool,
    reader_taken: bool,
    persisted: bool,
}

pub(crate) struct SessionHistoryRecorder {
    execution_id: String,
    policy: SessionHistoryPolicy,
    state: Arc<Mutex<TranscriptState>>,
}

impl SessionHistoryRecorder {
    pub(crate) fn new(execution_id: &str, policy: SessionHistoryPolicy) -> Result<Self> {
        if execution_id.is_empty() || execution_id.chars().any(char::is_control) {
            return Err(
                "session history requires a non-empty control-character-free execution id".into(),
            );
        }
        Ok(Self {
            execution_id: execution_id.to_owned(),
            policy,
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
        }))
    }

    pub(crate) fn persist(&self, store: &Store) -> Result<Option<PersistedSessionHistory>> {
        if !self.policy.any_persistence_enabled() {
            return Ok(None);
        }
        if self.policy.transcript_enabled && Arc::strong_count(&self.state) != 1 {
            return Err(
                "drop the terminal output reader before persisting transcript history so retained/truncated metadata is final"
                    .into(),
            );
        }

        let (observed_bytes, retained, truncated) = {
            let state = self
                .state
                .lock()
                .map_err(|_| "terminal history state lock is poisoned")?;
            if state.persisted {
                return Err("session history has already been persisted".into());
            }
            (
                state.observed_bytes,
                state.retained.clone(),
                state.truncated,
            )
        };

        let storage_key = history_storage_key(&self.execution_id);
        let transcript = if self.policy.transcript_enabled {
            Some(
                store
                    .write_blob(&storage_key, "terminal-transcript", &retained, truncated)?
                    .into(),
            )
        } else {
            None
        };
        let manifest = SessionHistoryManifest {
            schema_version: HISTORY_SCHEMA_VERSION,
            execution_id: self.execution_id.clone(),
            local_only: true,
            secret_filtering: SECRET_FILTERING_MODE.to_owned(),
            policy: self.policy,
            transcript_observed_bytes: observed_bytes,
            transcript_retained_bytes: retained.len(),
            transcript_truncated: truncated,
            transcript,
        };
        validate_manifest(&manifest)?;
        let manifest_blob = store
            .write_blob(
                &storage_key,
                "history-manifest",
                &serde_json::to_vec(&manifest)?,
                false,
            )?
            .into();
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
}

impl Read for HistoryReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buffer)?;
        if read == 0 {
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
        state.truncated |= keep < read;
        Ok(read)
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
        || manifest.transcript_truncated != (manifest.transcript_observed_bytes > retained)
    {
        return Err("session history retention metadata is inconsistent".into());
    }
    match (&manifest.transcript, manifest.policy.transcript_enabled) {
        (Some(blob), true)
            if blob.captured_bytes == manifest.transcript_retained_bytes
                && blob.truncated == manifest.transcript_truncated => {}
        (None, false)
            if manifest.transcript_observed_bytes == 0
                && manifest.transcript_retained_bytes == 0
                && !manifest.transcript_truncated => {}
        _ => return Err("session history transcript metadata is inconsistent".into()),
    }
    Ok(())
}

fn history_storage_key(execution_id: &str) -> String {
    let digest = Sha256::digest(execution_id.as_bytes());
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("history-{hex}")
}

#[cfg(test)]
mod tests {
    use super::{
        HARD_MAX_TRANSCRIPT_BYTES, HISTORY_DISABLED, REDACTED, SessionHistoryPolicy,
        SessionHistoryRecorder, persisted_arguments, sanitize_persisted_arguments,
    };
    use crate::store::Store;
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
            Self(path)
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

    #[test]
    fn disabled_policy_persists_nothing_and_command_text_can_be_withheld() {
        let root = TestRoot::new("disabled");
        let store = Store::open(root.path()).unwrap();
        let policy = SessionHistoryPolicy::disabled();
        let recorder = SessionHistoryRecorder::new("../../unsafe-id", policy).unwrap();
        assert_eq!(
            persisted_arguments(&["secret-looking-command".to_owned()], policy),
            vec![HISTORY_DISABLED.to_owned()]
        );
        assert!(recorder.persist(&store).unwrap().is_none());
        assert_eq!(fs::read_dir(root.path().join("blobs")).unwrap().count(), 0);
    }

    #[test]
    fn policy_enforces_nonzero_bounded_transcript_quota() {
        assert!(SessionHistoryPolicy::local_bounded(true, 0).is_err());
        assert!(SessionHistoryPolicy::local_bounded(false, HARD_MAX_TRANSCRIPT_BYTES + 1).is_err());
        assert!(SessionHistoryPolicy::local_bounded(false, 1).is_ok());
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
    fn bounded_transcript_records_quota_and_truncation_truth() {
        let root = TestRoot::new("bounded");
        let store = Store::open(root.path()).unwrap();
        let policy = SessionHistoryPolicy::local_bounded(true, 5).unwrap();
        let recorder = SessionHistoryRecorder::new("execution/with-path-chars", policy).unwrap();
        let mut reader = recorder
            .wrap_output_reader(Box::new(Cursor::new(b"abcdefgh".to_vec())))
            .unwrap();
        let mut observed = Vec::new();
        reader.read_to_end(&mut observed).unwrap();
        assert_eq!(observed, b"abcdefgh");
        assert!(recorder.persist(&store).is_err());
        drop(reader);

        let persisted = recorder.persist(&store).unwrap().unwrap();
        assert_eq!(persisted.manifest.transcript_observed_bytes, 8);
        assert_eq!(persisted.manifest.transcript_retained_bytes, 5);
        assert!(persisted.manifest.transcript_truncated);
        let transcript = persisted.manifest.transcript.as_ref().unwrap();
        assert_eq!(
            fs::read(root.path().join(&transcript.relative_path)).unwrap(),
            b"abcde"
        );
        assert!(
            persisted
                .manifest_blob
                .relative_path
                .starts_with("blobs/history-")
        );
        assert!(!persisted.manifest_blob.relative_path.contains(".."));
        assert!(recorder.persist(&store).is_err());
    }
}
