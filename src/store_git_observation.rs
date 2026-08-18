use super::{Result, Store};
use crate::domain::{ExecutionKind, FactSource};
use crate::git::GIT_WORKTREE_STATE_FORMAT;
use rusqlite::{OptionalExtension, params};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitObservationBoundary {
    Before,
    After,
}

impl GitObservationBoundary {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Before => "BEFORE",
            Self::After => "AFTER",
        }
    }

    fn from_db(value: &str) -> Option<Self> {
        match value {
            "BEFORE" => Some(Self::Before),
            "AFTER" => Some(Self::After),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitObservationAvailability {
    Observed,
    Unavailable,
}

impl GitObservationAvailability {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Observed => "OBSERVED",
            Self::Unavailable => "UNAVAILABLE",
        }
    }

    fn from_db(value: &str) -> Option<Self> {
        match value {
            "OBSERVED" => Some(Self::Observed),
            "UNAVAILABLE" => Some(Self::Unavailable),
            _ => None,
        }
    }
}

pub(crate) struct NewExecutionGitObservation<'a> {
    pub execution_id: &'a str,
    pub boundary: GitObservationBoundary,
    pub availability: GitObservationAvailability,
    pub head_oid: Option<&'a str>,
    pub branch: Option<&'a str>,
    pub detached: Option<bool>,
    pub dirty: Option<bool>,
    pub worktree_state_sha256: Option<&'a str>,
    pub observed_unix_ms: Option<i64>,
}

#[allow(
    dead_code,
    reason = "Spec 003 T055 typed read surface; CLI/timeline caller lands in T057"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExecutionGitObservationRecord {
    pub execution_id: String,
    pub boundary: GitObservationBoundary,
    pub availability: GitObservationAvailability,
    pub source: FactSource,
    pub head_oid: Option<String>,
    pub branch: Option<String>,
    pub detached: Option<bool>,
    pub dirty: Option<bool>,
    pub worktree_state_format: Option<String>,
    pub worktree_state_sha256: Option<String>,
    pub observed_unix_ms: Option<i64>,
}

impl Store {
    pub(crate) fn record_execution_git_observation(
        &mut self,
        observation: NewExecutionGitObservation<'_>,
    ) -> Result<()> {
        let worktree_state_format = validate_new_observation(&observation)?;
        let tx = self.connection.transaction()?;
        let kind = tx
            .query_row(
                "SELECT kind FROM executions WHERE execution_id = ?1",
                params![observation.execution_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or_else(|| {
                format!(
                    "unknown Winds execution for Git observation: {}",
                    observation.execution_id
                )
            })?;
        if kind != ExecutionKind::ShellCommand.as_str() {
            return Err(
                "Git command-boundary observations require SHELL_COMMAND execution kind".into(),
            );
        }

        tx.execute(
            "INSERT INTO execution_git_observations(
                execution_id, boundary, availability, fact_source,
                head_oid, branch, detached, dirty,
                worktree_state_format, worktree_state_sha256, observed_unix_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                observation.execution_id,
                observation.boundary.as_str(),
                observation.availability.as_str(),
                FactSource::WindsObserved.as_str(),
                observation.head_oid,
                observation.branch,
                observation.detached.map(bool_to_i64),
                observation.dirty.map(bool_to_i64),
                worktree_state_format,
                observation.worktree_state_sha256,
                observation.observed_unix_ms,
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    #[allow(
        dead_code,
        reason = "Spec 003 T055 typed read surface; CLI/timeline caller lands in T057"
    )]
    pub(crate) fn load_execution_git_observations(
        &self,
        execution_id: &str,
    ) -> Result<Vec<ExecutionGitObservationRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT execution_id, boundary, availability, fact_source,
                    head_oid, branch, detached, dirty,
                    worktree_state_format, worktree_state_sha256, observed_unix_ms
             FROM execution_git_observations
             WHERE execution_id = ?1
             ORDER BY CASE boundary WHEN 'BEFORE' THEN 0 WHEN 'AFTER' THEN 1 ELSE 2 END",
        )?;
        let rows = statement
            .query_map(params![execution_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, Option<i64>>(10)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let mut observations = Vec::with_capacity(rows.len());
        for row in rows {
            let boundary = GitObservationBoundary::from_db(&row.1)
                .ok_or_else(|| format!("unknown Git observation boundary in store: {}", row.1))?;
            let availability = GitObservationAvailability::from_db(&row.2).ok_or_else(|| {
                format!("unknown Git observation availability in store: {}", row.2)
            })?;
            let source = FactSource::from_db(&row.3)
                .ok_or_else(|| format!("unknown Git observation source in store: {}", row.3))?;
            if source != FactSource::WindsObserved {
                return Err(format!(
                    "Git command-boundary observation has invalid source in store: {}",
                    row.3
                )
                .into());
            }
            let record = ExecutionGitObservationRecord {
                execution_id: row.0,
                boundary,
                availability,
                source,
                head_oid: row.4,
                branch: row.5,
                detached: optional_bool_from_db(row.6, "detached")?,
                dirty: optional_bool_from_db(row.7, "dirty")?,
                worktree_state_format: row.8,
                worktree_state_sha256: row.9,
                observed_unix_ms: row.10,
            };
            validate_loaded_observation(&record)?;
            observations.push(record);
        }
        Ok(observations)
    }
}

fn validate_new_observation(
    observation: &NewExecutionGitObservation<'_>,
) -> Result<Option<&'static str>> {
    if observation.execution_id.is_empty() {
        return Err("Git observation requires non-empty execution identity".into());
    }
    if observation.observed_unix_ms.is_some_and(|value| value < 0) {
        return Err("Git observation time cannot be negative".into());
    }
    match observation.availability {
        GitObservationAvailability::Unavailable => {
            if observation.head_oid.is_some()
                || observation.branch.is_some()
                || observation.detached.is_some()
                || observation.dirty.is_some()
                || observation.worktree_state_sha256.is_some()
            {
                return Err(
                    "UNAVAILABLE Git observation cannot contain synthesized Git state".into(),
                );
            }
            Ok(None)
        }
        GitObservationAvailability::Observed => {
            let detached = observation
                .detached
                .ok_or("OBSERVED Git observation requires detached state")?;
            observation
                .dirty
                .ok_or("OBSERVED Git observation requires dirty state")?;
            let digest = observation
                .worktree_state_sha256
                .ok_or("OBSERVED Git observation requires worktree-state digest")?;
            validate_optional_nonempty(observation.head_oid, "Git HEAD object id")?;
            validate_optional_nonempty(observation.branch, "Git branch")?;
            if !is_lower_hex_sha256(digest) {
                return Err(
                    "Git worktree-state digest must be a lowercase SHA-256 hex value".into(),
                );
            }
            if detached {
                if observation.branch.is_some() {
                    return Err("detached Git observation cannot contain a branch name".into());
                }
                if observation.head_oid.is_none() {
                    return Err("detached Git observation requires an exact HEAD object id".into());
                }
            } else if observation.branch.is_none() {
                return Err("attached Git observation requires a branch name".into());
            }
            Ok(Some(GIT_WORKTREE_STATE_FORMAT))
        }
    }
}

fn validate_loaded_observation(record: &ExecutionGitObservationRecord) -> Result<()> {
    if record.observed_unix_ms.is_some_and(|value| value < 0) {
        return Err("stored Git observation time cannot be negative".into());
    }
    match record.availability {
        GitObservationAvailability::Unavailable => {
            if record.head_oid.is_some()
                || record.branch.is_some()
                || record.detached.is_some()
                || record.dirty.is_some()
                || record.worktree_state_format.is_some()
                || record.worktree_state_sha256.is_some()
            {
                return Err("stored UNAVAILABLE Git observation contains fabricated state".into());
            }
        }
        GitObservationAvailability::Observed => {
            let detached = record
                .detached
                .ok_or("stored OBSERVED Git observation is missing detached state")?;
            record
                .dirty
                .ok_or("stored OBSERVED Git observation is missing dirty state")?;
            if record.worktree_state_format.as_deref() != Some(GIT_WORKTREE_STATE_FORMAT) {
                return Err("stored Git worktree-state format is unknown".into());
            }
            let digest = record
                .worktree_state_sha256
                .as_deref()
                .ok_or("stored OBSERVED Git observation is missing worktree-state digest")?;
            if !is_lower_hex_sha256(digest) {
                return Err("stored Git worktree-state digest is invalid".into());
            }
            validate_optional_nonempty(record.head_oid.as_deref(), "stored Git HEAD object id")?;
            validate_optional_nonempty(record.branch.as_deref(), "stored Git branch")?;
            if detached {
                if record.branch.is_some() || record.head_oid.is_none() {
                    return Err("stored detached Git observation is internally inconsistent".into());
                }
            } else if record.branch.is_none() {
                return Err("stored attached Git observation is missing its branch name".into());
            }
        }
    }
    Ok(())
}

fn validate_optional_nonempty(value: Option<&str>, label: &str) -> Result<()> {
    if value.is_some_and(str::is_empty) {
        return Err(format!("{label} cannot be empty").into());
    }
    Ok(())
}

fn bool_to_i64(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

fn optional_bool_from_db(value: Option<i64>, label: &str) -> Result<Option<bool>> {
    match value {
        None => Ok(None),
        Some(0) => Ok(Some(false)),
        Some(1) => Ok(Some(true)),
        Some(other) => Err(format!("stored Git {label} boolean is invalid: {other}").into()),
    }
}

fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::{GitObservationAvailability, GitObservationBoundary, NewExecutionGitObservation};
    use crate::domain::{ExecutionKind, FactSource};
    use crate::store::{NewExecution, NewShellCommand, NewWorkspace, Store};
    use rusqlite::Connection;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

    fn test_home(name: &str) -> PathBuf {
        let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
        let home = std::env::temp_dir().join(format!(
            "winds-t055-store-{name}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&home).unwrap();
        home
    }

    fn store_with_shell_command(name: &str) -> (PathBuf, Store) {
        let home = test_home(name);
        let mut store = Store::open(&home).unwrap();
        store
            .create_workspace(
                NewWorkspace {
                    workspace_id: "workspace-1",
                    canonical_worktree_root: "/tmp/example",
                    git_common_dir: "/tmp/example/.git",
                },
                10,
            )
            .unwrap();
        let arguments = vec!["status".to_owned()];
        store
            .create_shell_command_execution(
                NewExecution {
                    execution_id: "command-1",
                    workspace_id: "workspace-1",
                    kind: ExecutionKind::ShellCommand,
                    request_source: FactSource::CallerRequested,
                    execution_domain: "native-test",
                },
                NewShellCommand {
                    execution_id: "command-1",
                    executable: "/usr/bin/git",
                    arguments: &arguments,
                    command_source: FactSource::CallerRequested,
                    requested_cwd: "/tmp/example",
                    cwd_source: FactSource::CallerRequested,
                },
                20,
            )
            .unwrap();
        (home, store)
    }

    #[test]
    fn observed_and_unavailable_git_states_round_trip_without_candidate_evidence() {
        let (home, mut store) = store_with_shell_command("round-trip");
        let digest = "0".repeat(64);
        store
            .record_execution_git_observation(NewExecutionGitObservation {
                execution_id: "command-1",
                boundary: GitObservationBoundary::Before,
                availability: GitObservationAvailability::Observed,
                head_oid: Some("abc123"),
                branch: Some("main"),
                detached: Some(false),
                dirty: Some(true),
                worktree_state_sha256: Some(&digest),
                observed_unix_ms: Some(21),
            })
            .unwrap();
        store
            .record_execution_git_observation(NewExecutionGitObservation {
                execution_id: "command-1",
                boundary: GitObservationBoundary::After,
                availability: GitObservationAvailability::Unavailable,
                head_oid: None,
                branch: None,
                detached: None,
                dirty: None,
                worktree_state_sha256: None,
                observed_unix_ms: Some(22),
            })
            .unwrap();

        let observations = store.load_execution_git_observations("command-1").unwrap();
        assert_eq!(observations.len(), 2);
        assert_eq!(observations[0].boundary, GitObservationBoundary::Before);
        assert_eq!(
            observations[0].availability,
            GitObservationAvailability::Observed
        );
        assert_eq!(observations[0].source, FactSource::WindsObserved);
        assert_eq!(observations[0].head_oid.as_deref(), Some("abc123"));
        assert_eq!(observations[0].branch.as_deref(), Some("main"));
        assert_eq!(observations[0].detached, Some(false));
        assert_eq!(observations[0].dirty, Some(true));
        assert_eq!(
            observations[0].worktree_state_sha256.as_deref(),
            Some(digest.as_str())
        );
        assert_eq!(observations[1].boundary, GitObservationBoundary::After);
        assert_eq!(
            observations[1].availability,
            GitObservationAvailability::Unavailable
        );
        assert_eq!(observations[1].head_oid, None);
        assert_eq!(observations[1].dirty, None);

        let candidate_events: i64 = store
            .connection
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .unwrap();
        let evidence_reports: i64 = store
            .connection
            .query_row("SELECT COUNT(*) FROM evidence_reports", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(candidate_events, 0);
        assert_eq!(evidence_reports, 0);

        drop(store);
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn duplicate_boundary_is_rejected() {
        let (home, mut store) = store_with_shell_command("duplicate");
        store
            .record_execution_git_observation(NewExecutionGitObservation {
                execution_id: "command-1",
                boundary: GitObservationBoundary::Before,
                availability: GitObservationAvailability::Unavailable,
                head_oid: None,
                branch: None,
                detached: None,
                dirty: None,
                worktree_state_sha256: None,
                observed_unix_ms: Some(21),
            })
            .unwrap();
        let duplicate = store.record_execution_git_observation(NewExecutionGitObservation {
            execution_id: "command-1",
            boundary: GitObservationBoundary::Before,
            availability: GitObservationAvailability::Unavailable,
            head_oid: None,
            branch: None,
            detached: None,
            dirty: None,
            worktree_state_sha256: None,
            observed_unix_ms: Some(22),
        });
        assert!(duplicate.is_err());
        assert_eq!(
            store
                .load_execution_git_observations("command-1")
                .unwrap()
                .len(),
            1
        );
        drop(store);
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn unavailable_observation_rejects_fabricated_state() {
        let (home, mut store) = store_with_shell_command("unavailable-state");
        let result = store.record_execution_git_observation(NewExecutionGitObservation {
            execution_id: "command-1",
            boundary: GitObservationBoundary::Before,
            availability: GitObservationAvailability::Unavailable,
            head_oid: None,
            branch: None,
            detached: None,
            dirty: Some(false),
            worktree_state_sha256: None,
            observed_unix_ms: Some(21),
        });
        assert!(result.is_err());
        assert!(
            store
                .load_execution_git_observations("command-1")
                .unwrap()
                .is_empty()
        );
        drop(store);
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn observed_state_rejects_invalid_digest_and_detached_branch() {
        let (home, mut store) = store_with_shell_command("invalid-observed");
        let invalid_digest = store.record_execution_git_observation(NewExecutionGitObservation {
            execution_id: "command-1",
            boundary: GitObservationBoundary::Before,
            availability: GitObservationAvailability::Observed,
            head_oid: Some("abc123"),
            branch: Some("main"),
            detached: Some(false),
            dirty: Some(false),
            worktree_state_sha256: Some("not-a-digest"),
            observed_unix_ms: Some(21),
        });
        assert!(invalid_digest.is_err());

        let digest = "0".repeat(64);
        let detached_branch = store.record_execution_git_observation(NewExecutionGitObservation {
            execution_id: "command-1",
            boundary: GitObservationBoundary::Before,
            availability: GitObservationAvailability::Observed,
            head_oid: Some("abc123"),
            branch: Some("main"),
            detached: Some(true),
            dirty: Some(false),
            worktree_state_sha256: Some(&digest),
            observed_unix_ms: Some(21),
        });
        assert!(detached_branch.is_err());

        drop(store);
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn store_open_upgrades_a_0004_database_with_the_forward_only_git_observation_table() {
        let home = test_home("migration");
        let connection = Connection::open(home.join("winds.db")).unwrap();
        connection
            .execute_batch(include_str!("../migrations/0001_init.sql"))
            .unwrap();
        connection
            .execute_batch(include_str!(
                "../migrations/0002_workspace_execution_ledger.sql"
            ))
            .unwrap();
        connection
            .execute_batch(include_str!(
                "../migrations/0003_workspace_clone_origins.sql"
            ))
            .unwrap();
        connection
            .execute_batch(include_str!("../migrations/0004_shell_commands.sql"))
            .unwrap();
        let before: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'execution_git_observations'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(before, 0);
        drop(connection);

        let store = Store::open(&home).unwrap();
        let after: i64 = store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'execution_git_observations'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(after, 1);

        drop(store);
        fs::remove_dir_all(home).unwrap();
    }
}
