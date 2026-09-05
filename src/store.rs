use crate::domain::{
    BlobEvidence, CheckEvidence, Eligibility, EvidenceReport, ExecutionEventRecord, ExecutionKind,
    ExecutionRecord, ExecutionStatus, FactSource, ShellCommandRecord, StoredRun,
    TerminalCloseReason, TerminalSessionRecord, WorkspaceRecord,
};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

#[path = "agentic_identity.rs"]
pub(crate) mod agentic_identity;
#[path = "store_git_observation.rs"]
pub(crate) mod git_observation;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub struct Store {
    pub(crate) connection: Connection,
    home: PathBuf,
    deferred_terminal_finalizations: Vec<DeferredTerminalFinalization>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TerminalFinalization {
    Exited {
        ended_unix_ms: Option<i64>,
    },
    Interrupted {
        ended_unix_ms: Option<i64>,
        reason: TerminalCloseReason,
    },
    OwnershipLost {
        observed_unix_ms: Option<i64>,
    },
}

#[derive(Debug, Clone)]
struct DeferredTerminalFinalization {
    execution_id: String,
    finalization: TerminalFinalization,
}

pub struct NewRun<'a> {
    pub run_id: &'a str,
    pub repo_path: &'a str,
    pub base_oid: &'a str,
    pub candidate_ref: &'a str,
    pub candidate_oid: &'a str,
    pub candidate_tree: &'a str,
    pub worktree_path: &'a str,
    pub check_command: &'a str,
    pub timeout_secs: u64,
}

#[allow(
    dead_code,
    reason = "Spec 003 T044 persistence substrate; runtime callers land in later slices"
)]
pub struct NewWorkspace<'a> {
    pub workspace_id: &'a str,
    pub canonical_worktree_root: &'a str,
    pub git_common_dir: &'a str,
}

#[allow(
    dead_code,
    reason = "Spec 006 T070 persistence substrate; product session semantics land in T071"
)]
pub struct NewWorkstream<'a> {
    pub workstream_id: &'a str,
    pub workspace_id: &'a str,
    pub display_name: &'a str,
}

#[allow(
    dead_code,
    reason = "Spec 006 T070 persistence substrate; product session semantics land in T071"
)]
pub struct NewWindsSession<'a> {
    pub session_id: &'a str,
    pub workstream_id: &'a str,
    pub display_name: &'a str,
}

#[allow(
    dead_code,
    reason = "Spec 003 T044 persistence substrate; runtime callers land in later slices"
)]
pub struct NewExecution<'a> {
    pub execution_id: &'a str,
    pub workspace_id: &'a str,
    pub kind: ExecutionKind,
    pub request_source: FactSource,
    pub execution_domain: &'a str,
}

#[allow(
    dead_code,
    reason = "Spec 003 T044 persistence substrate; runtime callers land in later slices"
)]
pub struct NewTerminalSession<'a> {
    pub execution_id: &'a str,
    pub profile_id: &'a str,
    pub shell_executable: &'a str,
    pub shell_arguments: &'a [String],
    pub requested_cwd: &'a str,
    pub initial_cols: Option<u16>,
    pub initial_rows: Option<u16>,
}

#[allow(
    dead_code,
    reason = "Spec 003 T054 command-record backend API; CLI/timeline caller lands in T057"
)]
pub struct NewShellCommand<'a> {
    pub execution_id: &'a str,
    pub executable: &'a str,
    pub arguments: &'a [String],
    pub command_source: FactSource,
    pub requested_cwd: &'a str,
    pub cwd_source: FactSource,
}

pub struct RecoverableRun {
    pub run_id: String,
    pub candidate_oid: String,
    pub worktree_path: String,
    pub state: String,
}

impl Store {
    pub fn open(home: &Path) -> Result<Self> {
        fs::create_dir_all(home)?;
        fs::create_dir_all(home.join("blobs"))?;
        let connection = Connection::open(home.join("winds.db"))?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        connection.execute_batch(include_str!("../migrations/0001_init.sql"))?;
        connection.execute_batch(include_str!(
            "../migrations/0002_workspace_execution_ledger.sql"
        ))?;
        connection.execute_batch(include_str!(
            "../migrations/0003_workspace_clone_origins.sql"
        ))?;
        connection.execute_batch(include_str!("../migrations/0004_shell_commands.sql"))?;
        connection.execute_batch(include_str!(
            "../migrations/0005_execution_git_observations.sql"
        ))?;
        connection.execute_batch(include_str!("../migrations/0006_agentic_identity.sql"))?;
        connection.execute_batch(include_str!(
            "../migrations/0007_agentic_session_origins.sql"
        ))?;
        connection.execute_batch(include_str!(
            "../migrations/0008_runtime_session_bindings.sql"
        ))?;
        Ok(Self {
            connection,
            home: home.to_path_buf(),
            deferred_terminal_finalizations: Vec::new(),
        })
    }
}

#[allow(
    dead_code,
    reason = "Spec 003 T044 persistence substrate; runtime callers land in later slices"
)]
impl Store {
    pub fn create_workspace(&self, workspace: NewWorkspace<'_>, now_ms: i64) -> Result<()> {
        self.connection.execute(
            "INSERT INTO workspaces(
                workspace_id, canonical_worktree_root, git_common_dir,
                created_unix_ms, last_opened_unix_ms
             ) VALUES (?1, ?2, ?3, ?4, ?4)",
            params![
                workspace.workspace_id,
                workspace.canonical_worktree_root,
                workspace.git_common_dir,
                now_ms,
            ],
        )?;
        Ok(())
    }

    pub fn mark_workspace_opened(&self, workspace_id: &str, now_ms: i64) -> Result<()> {
        let updated = self.connection.execute(
            "UPDATE workspaces SET last_opened_unix_ms = ?2 WHERE workspace_id = ?1",
            params![workspace_id, now_ms],
        )?;
        if updated != 1 {
            return Err(format!("unknown Winds workspace: {workspace_id}").into());
        }
        Ok(())
    }

    pub fn load_workspace(&self, workspace_id: &str) -> Result<WorkspaceRecord> {
        let workspace = self
            .connection
            .query_row(
                "SELECT workspace_id, canonical_worktree_root, git_common_dir,
                        created_unix_ms, last_opened_unix_ms
                 FROM workspaces WHERE workspace_id = ?1",
                params![workspace_id],
                |row| {
                    Ok(WorkspaceRecord {
                        workspace_id: row.get(0)?,
                        canonical_worktree_root: row.get(1)?,
                        git_common_dir: row.get(2)?,
                        created_unix_ms: row.get(3)?,
                        last_opened_unix_ms: row.get(4)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown Winds workspace: {workspace_id}"))?;
        Ok(workspace)
    }

    pub fn register_cloned_workspace(
        &mut self,
        workspace: NewWorkspace<'_>,
        remote_identity: &str,
        now_ms: i64,
    ) -> Result<()> {
        let tx = self.connection.transaction()?;
        let existing = tx
            .query_row(
                "SELECT canonical_worktree_root, git_common_dir
                 FROM workspaces WHERE workspace_id = ?1",
                params![workspace.workspace_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;

        if let Some((canonical_worktree_root, git_common_dir)) = existing {
            if canonical_worktree_root != workspace.canonical_worktree_root
                || git_common_dir != workspace.git_common_dir
            {
                return Err(format!(
                    "stored workspace identity conflicts with observed Git identity: {}",
                    workspace.workspace_id
                )
                .into());
            }
            tx.execute(
                "UPDATE workspaces SET last_opened_unix_ms = ?2 WHERE workspace_id = ?1",
                params![workspace.workspace_id, now_ms],
            )?;
        } else {
            tx.execute(
                "INSERT INTO workspaces(
                    workspace_id, canonical_worktree_root, git_common_dir,
                    created_unix_ms, last_opened_unix_ms
                 ) VALUES (?1, ?2, ?3, ?4, ?4)",
                params![
                    workspace.workspace_id,
                    workspace.canonical_worktree_root,
                    workspace.git_common_dir,
                    now_ms,
                ],
            )?;
        }

        tx.execute(
            "INSERT INTO workspace_clone_origins(workspace_id, remote_identity, recorded_unix_ms)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(workspace_id) DO UPDATE SET
                 remote_identity = excluded.remote_identity,
                 recorded_unix_ms = excluded.recorded_unix_ms",
            params![workspace.workspace_id, remote_identity, now_ms],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn create_execution(&mut self, execution: NewExecution<'_>, now_ms: i64) -> Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute(
            "INSERT INTO executions(
                execution_id, workspace_id, kind, request_source, execution_domain,
                status, status_source, requested_unix_ms,
                started_unix_ms, ended_unix_ms, duration_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?4, ?7, NULL, NULL, NULL)",
            params![
                execution.execution_id,
                execution.workspace_id,
                execution.kind.as_str(),
                execution.request_source.as_str(),
                execution.execution_domain,
                ExecutionStatus::Requested.as_str(),
                now_ms,
            ],
        )?;
        insert_execution_event(
            &tx,
            execution.execution_id,
            "ExecutionRequested",
            execution.request_source,
            now_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn create_terminal_execution(
        &mut self,
        execution: NewExecution<'_>,
        session: NewTerminalSession<'_>,
        now_ms: i64,
    ) -> Result<()> {
        if execution.kind != ExecutionKind::Terminal {
            return Err("terminal execution persistence requires TERMINAL execution kind".into());
        }
        if execution.execution_id != session.execution_id {
            return Err("terminal execution/session identities do not match".into());
        }
        let shell_arguments_json = serde_json::to_string(session.shell_arguments)?;
        let tx = self.connection.transaction()?;
        tx.execute(
            "INSERT INTO executions(
            execution_id, workspace_id, kind, request_source, execution_domain,
            status, status_source, requested_unix_ms,
            started_unix_ms, ended_unix_ms, duration_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?4, ?7, NULL, NULL, NULL)",
            params![
                execution.execution_id,
                execution.workspace_id,
                execution.kind.as_str(),
                execution.request_source.as_str(),
                execution.execution_domain,
                ExecutionStatus::Requested.as_str(),
                now_ms,
            ],
        )?;
        insert_execution_event(
            &tx,
            execution.execution_id,
            "ExecutionRequested",
            execution.request_source,
            now_ms,
        )?;
        tx.execute(
            "INSERT INTO terminal_sessions(
            execution_id, profile_id, shell_executable, shell_arguments_json,
            requested_cwd, initial_cols, initial_rows, close_reason
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)",
            params![
                session.execution_id,
                session.profile_id,
                session.shell_executable,
                shell_arguments_json,
                session.requested_cwd,
                session.initial_cols.map(i64::from),
                session.initial_rows.map(i64::from),
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn create_shell_command_execution(
        &mut self,
        execution: NewExecution<'_>,
        command: NewShellCommand<'_>,
        now_ms: i64,
    ) -> Result<()> {
        if execution.kind != ExecutionKind::ShellCommand {
            return Err("shell-command persistence requires SHELL_COMMAND execution kind".into());
        }
        if execution.execution_id != command.execution_id {
            return Err("shell-command execution/record identities do not match".into());
        }
        if command.command_source == FactSource::WindsObserved
            || command.cwd_source == FactSource::WindsObserved
        {
            return Err(
                "explicit/shell-reported command intent cannot be persisted as WINDS_OBSERVED"
                    .into(),
            );
        }
        let arguments_json = serde_json::to_string(command.arguments)?;
        let tx = self.connection.transaction()?;
        tx.execute(
            "INSERT INTO executions(
                execution_id, workspace_id, kind, request_source, execution_domain,
                status, status_source, requested_unix_ms,
                started_unix_ms, ended_unix_ms, duration_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?4, ?7, NULL, NULL, NULL)",
            params![
                execution.execution_id,
                execution.workspace_id,
                execution.kind.as_str(),
                execution.request_source.as_str(),
                execution.execution_domain,
                ExecutionStatus::Requested.as_str(),
                now_ms,
            ],
        )?;
        insert_execution_event(
            &tx,
            execution.execution_id,
            "ExecutionRequested",
            execution.request_source,
            now_ms,
        )?;
        tx.execute(
            "INSERT INTO shell_commands(
                execution_id, executable, arguments_json, command_source,
                requested_cwd, cwd_source, exit_code, exit_source
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL)",
            params![
                command.execution_id,
                command.executable,
                arguments_json,
                command.command_source.as_str(),
                command.requested_cwd,
                command.cwd_source.as_str(),
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn mark_shell_command_running(
        &mut self,
        execution_id: &str,
        started_unix_ms: Option<i64>,
    ) -> Result<()> {
        let tx = self.connection.transaction()?;
        let (status, requested_unix_ms, persisted_started_unix_ms) =
            shell_command_execution_state(&tx, execution_id)?;
        if status != ExecutionStatus::Requested || persisted_started_unix_ms.is_some() {
            return Err(format!(
                "shell command cannot start from persisted state {}: {execution_id}",
                status.as_str()
            )
            .into());
        }
        if started_unix_ms.is_some_and(|value| value < requested_unix_ms) {
            return Err("shell-command start time cannot precede its request time".into());
        }
        let updated = tx.execute(
            "UPDATE executions
             SET status = ?2, status_source = ?3, started_unix_ms = ?4,
                 ended_unix_ms = NULL, duration_ms = NULL
             WHERE execution_id = ?1 AND status = ?5",
            params![
                execution_id,
                ExecutionStatus::Running.as_str(),
                FactSource::WindsObserved.as_str(),
                started_unix_ms,
                ExecutionStatus::Requested.as_str(),
            ],
        )?;
        if updated != 1 {
            return Err("shell-command RUNNING transition lost its expected REQUESTED row".into());
        }
        insert_execution_event_if_time(
            &tx,
            execution_id,
            "ShellCommandStarted",
            FactSource::WindsObserved,
            started_unix_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn mark_shell_command_failed_to_start(
        &mut self,
        execution_id: &str,
        failed_unix_ms: Option<i64>,
    ) -> Result<()> {
        let tx = self.connection.transaction()?;
        let (status, requested_unix_ms, started_unix_ms) =
            shell_command_execution_state(&tx, execution_id)?;
        if status != ExecutionStatus::Requested || started_unix_ms.is_some() {
            return Err(format!(
                "shell command cannot fail-to-start from persisted state {}: {execution_id}",
                status.as_str()
            )
            .into());
        }
        if failed_unix_ms.is_some_and(|value| value < requested_unix_ms) {
            return Err("shell-command failure time cannot precede its request time".into());
        }
        let updated = tx.execute(
            "UPDATE executions
             SET status = ?2, status_source = ?3, ended_unix_ms = ?4, duration_ms = NULL
             WHERE execution_id = ?1 AND status = ?5",
            params![
                execution_id,
                ExecutionStatus::FailedToStart.as_str(),
                FactSource::WindsObserved.as_str(),
                failed_unix_ms,
                ExecutionStatus::Requested.as_str(),
            ],
        )?;
        if updated != 1 {
            return Err("shell-command FAILED_TO_START transition lost its expected row".into());
        }
        insert_execution_event_if_time(
            &tx,
            execution_id,
            "ShellCommandFailedToStart",
            FactSource::WindsObserved,
            failed_unix_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn mark_shell_command_start_persistence_failed(
        &mut self,
        execution_id: &str,
        started_unix_ms: Option<i64>,
        ended_unix_ms: Option<i64>,
    ) -> Result<()> {
        let tx = self.connection.transaction()?;
        let (status, requested_unix_ms, persisted_started_unix_ms) =
            shell_command_execution_state(&tx, execution_id)?;
        if status != ExecutionStatus::Requested || persisted_started_unix_ms.is_some() {
            return Err(format!(
                "shell-command start-persistence recovery cannot run from persisted state {}: {execution_id}",
                status.as_str()
            )
            .into());
        }
        validate_optional_command_times(requested_unix_ms, started_unix_ms, ended_unix_ms)?;
        let duration_ms = optional_duration_ms(started_unix_ms, ended_unix_ms)?;
        let updated = tx.execute(
            "UPDATE executions
             SET status = ?2, status_source = ?3, started_unix_ms = ?4,
                 ended_unix_ms = ?5, duration_ms = ?6
             WHERE execution_id = ?1 AND status = ?7",
            params![
                execution_id,
                ExecutionStatus::Interrupted.as_str(),
                FactSource::WindsObserved.as_str(),
                started_unix_ms,
                ended_unix_ms,
                duration_ms,
                ExecutionStatus::Requested.as_str(),
            ],
        )?;
        if updated != 1 {
            return Err("shell-command start-persistence recovery lost its REQUESTED row".into());
        }
        insert_execution_event_if_time(
            &tx,
            execution_id,
            "ShellCommandStartPersistenceFailed",
            FactSource::WindsObserved,
            ended_unix_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn mark_shell_command_interrupted(
        &mut self,
        execution_id: &str,
        ended_unix_ms: Option<i64>,
    ) -> Result<()> {
        let tx = self.connection.transaction()?;
        let (status, requested_unix_ms, started_unix_ms) =
            shell_command_execution_state(&tx, execution_id)?;
        if status != ExecutionStatus::Running {
            return Err(format!(
                "shell command cannot be interrupted from persisted state {}: {execution_id}",
                status.as_str()
            )
            .into());
        }
        validate_optional_command_times(requested_unix_ms, started_unix_ms, ended_unix_ms)?;
        let duration_ms = optional_duration_ms(started_unix_ms, ended_unix_ms)?;
        let updated = tx.execute(
            "UPDATE executions
             SET status = ?2, status_source = ?3, ended_unix_ms = ?4, duration_ms = ?5
             WHERE execution_id = ?1 AND status = ?6",
            params![
                execution_id,
                ExecutionStatus::Interrupted.as_str(),
                FactSource::WindsObserved.as_str(),
                ended_unix_ms,
                duration_ms,
                ExecutionStatus::Running.as_str(),
            ],
        )?;
        if updated != 1 {
            return Err("shell-command INTERRUPTED transition lost its RUNNING row".into());
        }
        insert_execution_event_if_time(
            &tx,
            execution_id,
            "ShellCommandInterrupted",
            FactSource::WindsObserved,
            ended_unix_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn mark_shell_command_ownership_lost(
        &mut self,
        execution_id: &str,
        observed_unix_ms: Option<i64>,
    ) -> Result<()> {
        let tx = self.connection.transaction()?;
        let (status, requested_unix_ms, started_unix_ms) =
            shell_command_execution_state(&tx, execution_id)?;
        if !matches!(
            status,
            ExecutionStatus::Requested | ExecutionStatus::Running
        ) {
            return Err(format!(
                "shell-command ownership cannot be lost from persisted state {}: {execution_id}",
                status.as_str()
            )
            .into());
        }
        let observation_floor = started_unix_ms
            .unwrap_or(requested_unix_ms)
            .max(requested_unix_ms);
        if observed_unix_ms.is_some_and(|value| value < observation_floor) {
            return Err(
                "shell-command ownership-loss observation cannot precede its observed start/request time"
                    .into(),
            );
        }
        let updated = tx.execute(
            "UPDATE executions
             SET status = ?2, status_source = ?3,
                 ended_unix_ms = NULL, duration_ms = NULL
             WHERE execution_id = ?1 AND status IN (?4, ?5)",
            params![
                execution_id,
                ExecutionStatus::OwnershipLost.as_str(),
                FactSource::WindsObserved.as_str(),
                ExecutionStatus::Requested.as_str(),
                ExecutionStatus::Running.as_str(),
            ],
        )?;
        if updated != 1 {
            return Err("shell-command ownership-loss transition lost its non-final row".into());
        }
        insert_execution_event_if_time(
            &tx,
            execution_id,
            "ShellCommandOwnershipLost",
            FactSource::WindsObserved,
            observed_unix_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn record_shell_command_exit_observation(
        &mut self,
        execution_id: &str,
        exit_code: Option<i32>,
        observed_end_unix_ms: Option<i64>,
    ) -> Result<()> {
        let tx = self.connection.transaction()?;
        let (status, requested_unix_ms, started_unix_ms) =
            shell_command_execution_state(&tx, execution_id)?;
        if status != ExecutionStatus::Running {
            return Err(format!(
                "shell-command exit cannot be observed from persisted state {}: {execution_id}",
                status.as_str()
            )
            .into());
        }
        if exit_code.is_none() && observed_end_unix_ms.is_none() {
            return Err(
                "shell-command exit observation requires an exit code or observed end time".into(),
            );
        }
        validate_optional_command_times(requested_unix_ms, started_unix_ms, observed_end_unix_ms)?;
        let updated = tx.execute(
            "UPDATE shell_commands
             SET exit_code = ?2, exit_source = ?3, observed_end_unix_ms = ?4
             WHERE execution_id = ?1 AND exit_source IS NULL",
            params![
                execution_id,
                exit_code.map(i64::from),
                FactSource::WindsObserved.as_str(),
                observed_end_unix_ms,
            ],
        )?;
        if updated != 1 {
            return Err(
                "shell-command exit observation was already recorded or lost its typed row".into(),
            );
        }
        tx.commit()?;
        Ok(())
    }

    pub fn finalize_shell_command_from_observation(&mut self, execution_id: &str) -> Result<()> {
        let tx = self.connection.transaction()?;
        let row = tx
            .query_row(
                "SELECT e.status, e.requested_unix_ms, e.started_unix_ms,
                        c.exit_code, c.exit_source, c.observed_end_unix_ms
                 FROM executions e
                 INNER JOIN shell_commands c ON c.execution_id = e.execution_id
                 WHERE e.execution_id = ?1 AND e.kind = ?2",
                params![execution_id, ExecutionKind::ShellCommand.as_str()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Option<i64>>(2)?,
                        row.get::<_, Option<i64>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<i64>>(5)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown Winds shell command execution: {execution_id}"))?;
        let status = ExecutionStatus::from_db(&row.0)
            .ok_or_else(|| format!("unknown shell-command execution status in store: {}", row.0))?;
        if status != ExecutionStatus::Running {
            return Err(format!(
                "shell-command completion cannot finalize from persisted state {}: {execution_id}",
                status.as_str()
            )
            .into());
        }
        if row.4.as_deref() != Some(FactSource::WindsObserved.as_str())
            || (row.3.is_none() && row.5.is_none())
        {
            return Err(
                "shell-command completion requires a durable WINDS_OBSERVED exit fact".into(),
            );
        }
        validate_optional_command_times(row.1, row.2, row.5)?;
        let duration_ms = optional_duration_ms(row.2, row.5)?;
        let updated = tx.execute(
            "UPDATE executions
             SET status = ?2, status_source = ?3, ended_unix_ms = ?4, duration_ms = ?5
             WHERE execution_id = ?1 AND status = ?6",
            params![
                execution_id,
                ExecutionStatus::Exited.as_str(),
                FactSource::WindsObserved.as_str(),
                row.5,
                duration_ms,
                ExecutionStatus::Running.as_str(),
            ],
        )?;
        if updated != 1 {
            return Err("shell-command EXITED transition lost its RUNNING row".into());
        }
        insert_execution_event_if_time(
            &tx,
            execution_id,
            "ShellCommandExited",
            FactSource::WindsObserved,
            row.5,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn finalize_observed_shell_commands(&mut self) -> Result<usize> {
        let execution_ids = {
            let mut statement = self.connection.prepare(
                "SELECT e.execution_id
                 FROM executions e
                 INNER JOIN shell_commands c ON c.execution_id = e.execution_id
                 WHERE e.kind = ?1 AND e.status = ?2 AND c.exit_source = ?3
                   AND (c.exit_code IS NOT NULL OR c.observed_end_unix_ms IS NOT NULL)
                 ORDER BY e.requested_unix_ms, e.execution_id",
            )?;
            statement
                .query_map(
                    params![
                        ExecutionKind::ShellCommand.as_str(),
                        ExecutionStatus::Running.as_str(),
                        FactSource::WindsObserved.as_str(),
                    ],
                    |row| row.get::<_, String>(0),
                )?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        for execution_id in &execution_ids {
            self.finalize_shell_command_from_observation(execution_id)?;
        }
        Ok(execution_ids.len())
    }

    pub fn reconcile_unowned_shell_commands_after_restart(&mut self, now_ms: i64) -> Result<usize> {
        self.finalize_observed_shell_commands()?;
        let tx = self.connection.transaction()?;
        let executions = {
            let mut statement = tx.prepare(
                "SELECT e.execution_id, e.requested_unix_ms, e.started_unix_ms
                 FROM executions e
                 INNER JOIN shell_commands c ON c.execution_id = e.execution_id
                 WHERE e.kind = ?1 AND e.status IN (?2, ?3)
                 ORDER BY e.requested_unix_ms, e.execution_id",
            )?;
            statement
                .query_map(
                    params![
                        ExecutionKind::ShellCommand.as_str(),
                        ExecutionStatus::Requested.as_str(),
                        ExecutionStatus::Running.as_str(),
                    ],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, i64>(1)?,
                            row.get::<_, Option<i64>>(2)?,
                        ))
                    },
                )?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        for (execution_id, requested_unix_ms, started_unix_ms) in &executions {
            let updated = tx.execute(
                "UPDATE executions
                 SET status = ?2, status_source = ?3,
                     ended_unix_ms = NULL, duration_ms = NULL
                 WHERE execution_id = ?1 AND status IN (?4, ?5)",
                params![
                    execution_id,
                    ExecutionStatus::OwnershipLost.as_str(),
                    FactSource::WindsObserved.as_str(),
                    ExecutionStatus::Requested.as_str(),
                    ExecutionStatus::Running.as_str(),
                ],
            )?;
            if updated != 1 {
                return Err(format!(
                    "shell-command restart reconciliation lost its non-final row: {execution_id}"
                )
                .into());
            }
            let observation_floor = started_unix_ms
                .unwrap_or(*requested_unix_ms)
                .max(*requested_unix_ms);
            insert_execution_event(
                &tx,
                execution_id,
                "ShellCommandOwnershipLostAfterRestart",
                FactSource::WindsObserved,
                now_ms.max(observation_floor),
            )?;
        }
        tx.commit()?;
        Ok(executions.len())
    }

    pub fn mark_terminal_running(&mut self, execution_id: &str, now_ms: i64) -> Result<()> {
        let tx = self.connection.transaction()?;
        let (status, requested_unix_ms, started_unix_ms) =
            terminal_execution_state(&tx, execution_id)?;
        if status != ExecutionStatus::Requested || started_unix_ms.is_some() {
            return Err(format!(
                "terminal execution cannot start from persisted state {}: {execution_id}",
                status.as_str()
            )
            .into());
        }
        if now_ms < requested_unix_ms {
            return Err("terminal start time cannot precede its request time".into());
        }
        let updated = tx.execute(
            "UPDATE executions
         SET status = ?2, status_source = ?3, started_unix_ms = ?4,
             ended_unix_ms = NULL, duration_ms = NULL
         WHERE execution_id = ?1 AND status = ?5",
            params![
                execution_id,
                ExecutionStatus::Running.as_str(),
                FactSource::WindsObserved.as_str(),
                now_ms,
                ExecutionStatus::Requested.as_str(),
            ],
        )?;
        if updated != 1 {
            return Err("terminal RUNNING transition lost its expected REQUESTED row".into());
        }
        insert_execution_event(
            &tx,
            execution_id,
            "TerminalStarted",
            FactSource::WindsObserved,
            now_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn mark_terminal_failed_to_start(&mut self, execution_id: &str, now_ms: i64) -> Result<()> {
        let tx = self.connection.transaction()?;
        let (status, requested_unix_ms, started_unix_ms) =
            terminal_execution_state(&tx, execution_id)?;
        if status != ExecutionStatus::Requested || started_unix_ms.is_some() {
            return Err(format!(
                "terminal execution cannot fail-to-start from persisted state {}: {execution_id}",
                status.as_str()
            )
            .into());
        }
        if now_ms < requested_unix_ms {
            return Err("terminal failure time cannot precede its request time".into());
        }
        let updated = tx.execute(
            "UPDATE executions
         SET status = ?2, status_source = ?3, ended_unix_ms = ?4, duration_ms = NULL
         WHERE execution_id = ?1 AND status = ?5",
            params![
                execution_id,
                ExecutionStatus::FailedToStart.as_str(),
                FactSource::WindsObserved.as_str(),
                now_ms,
                ExecutionStatus::Requested.as_str(),
            ],
        )?;
        if updated != 1 {
            return Err("FAILED_TO_START transition lost its expected REQUESTED row".into());
        }
        set_terminal_close_reason(&tx, execution_id, TerminalCloseReason::FailedToStart)?;
        insert_execution_event(
            &tx,
            execution_id,
            "TerminalFailedToStart",
            FactSource::WindsObserved,
            now_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn mark_terminal_start_persistence_failed(
        &mut self,
        execution_id: &str,
        started_unix_ms: i64,
        ended_unix_ms: Option<i64>,
    ) -> Result<()> {
        let tx = self.connection.transaction()?;
        let (status, requested_unix_ms, persisted_started_unix_ms) =
            terminal_execution_state(&tx, execution_id)?;
        if status != ExecutionStatus::Requested || persisted_started_unix_ms.is_some() {
            return Err(format!(
            "terminal start-persistence recovery cannot run from persisted state {}: {execution_id}",
            status.as_str()
        )
        .into());
        }
        if started_unix_ms < requested_unix_ms
            || ended_unix_ms.is_some_and(|value| value < started_unix_ms)
        {
            return Err("terminal start-persistence recovery timestamps are inconsistent".into());
        }
        let duration_ms = ended_unix_ms.map(|value| value - started_unix_ms);
        let updated = tx.execute(
            "UPDATE executions
         SET status = ?2, status_source = ?3, started_unix_ms = ?4,
             ended_unix_ms = ?5, duration_ms = ?6
         WHERE execution_id = ?1 AND status = ?7",
            params![
                execution_id,
                ExecutionStatus::Interrupted.as_str(),
                FactSource::WindsObserved.as_str(),
                started_unix_ms,
                ended_unix_ms,
                duration_ms,
                ExecutionStatus::Requested.as_str(),
            ],
        )?;
        if updated != 1 {
            return Err("start-persistence recovery lost its expected REQUESTED row".into());
        }
        set_terminal_close_reason(
            &tx,
            execution_id,
            TerminalCloseReason::StartPersistenceFailed,
        )?;
        insert_execution_event_if_time(
            &tx,
            execution_id,
            "TerminalStartPersistenceFailed",
            FactSource::WindsObserved,
            ended_unix_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn mark_terminal_exited(
        &mut self,
        execution_id: &str,
        ended_unix_ms: Option<i64>,
    ) -> Result<()> {
        finalize_running_terminal(
            &mut self.connection,
            execution_id,
            ExecutionStatus::Exited,
            TerminalCloseReason::ProcessExited,
            "TerminalExited",
            ended_unix_ms,
        )
    }

    pub fn mark_terminal_interrupted(
        &mut self,
        execution_id: &str,
        reason: TerminalCloseReason,
        ended_unix_ms: Option<i64>,
    ) -> Result<()> {
        if !matches!(
            reason,
            TerminalCloseReason::TerminatedByWinds | TerminalCloseReason::ClosedByWinds
        ) {
            return Err(
                "terminal interruption requires a controlled close/terminate reason".into(),
            );
        }
        finalize_running_terminal(
            &mut self.connection,
            execution_id,
            ExecutionStatus::Interrupted,
            reason,
            "TerminalInterrupted",
            ended_unix_ms,
        )
    }

    pub(crate) fn apply_terminal_finalization(
        &mut self,
        execution_id: &str,
        finalization: TerminalFinalization,
    ) -> Result<()> {
        match finalization {
            TerminalFinalization::Exited { ended_unix_ms } => {
                self.mark_terminal_exited(execution_id, ended_unix_ms)
            }
            TerminalFinalization::Interrupted {
                ended_unix_ms,
                reason,
            } => self.mark_terminal_interrupted(execution_id, reason, ended_unix_ms),
            TerminalFinalization::OwnershipLost { observed_unix_ms } => self
                .mark_terminal_ownership_lost(
                    execution_id,
                    "TerminalOwnershipLostAfterCleanupFailure",
                    observed_unix_ms,
                ),
        }
    }

    pub(crate) fn defer_terminal_finalization(
        &mut self,
        execution_id: &str,
        finalization: TerminalFinalization,
    ) {
        if let Some(existing) = self
            .deferred_terminal_finalizations
            .iter_mut()
            .find(|pending| pending.execution_id == execution_id)
        {
            existing.finalization = finalization;
            return;
        }
        self.deferred_terminal_finalizations
            .push(DeferredTerminalFinalization {
                execution_id: execution_id.to_owned(),
                finalization,
            });
    }

    pub fn retry_deferred_terminal_finalizations(&mut self) -> Result<usize> {
        let pending = std::mem::take(&mut self.deferred_terminal_finalizations);
        let mut completed = 0_usize;
        let mut failed = Vec::new();
        let mut failures = Vec::new();
        for item in pending {
            match self.apply_terminal_finalization(&item.execution_id, item.finalization) {
                Ok(()) => completed += 1,
                Err(error) => {
                    failures.push(format!("{}: {error}", item.execution_id));
                    failed.push(item);
                }
            }
        }
        self.deferred_terminal_finalizations = failed;
        if failures.is_empty() {
            Ok(completed)
        } else {
            Err(format!(
                "{} deferred terminal finalization(s) remain pending: {}",
                failures.len(),
                failures.join("; ")
            )
            .into())
        }
    }

    pub fn pending_terminal_finalization_count(&self) -> usize {
        self.deferred_terminal_finalizations.len()
    }

    fn mark_terminal_ownership_lost(
        &mut self,
        execution_id: &str,
        event_kind: &str,
        observed_unix_ms: Option<i64>,
    ) -> Result<()> {
        let tx = self.connection.transaction()?;
        let (status, requested_unix_ms, started_unix_ms) =
            terminal_execution_state(&tx, execution_id)?;
        if !matches!(
            status,
            ExecutionStatus::Requested | ExecutionStatus::Running
        ) {
            return Err(format!(
                "terminal ownership cannot be lost from persisted state {}: {execution_id}",
                status.as_str()
            )
            .into());
        }
        let observation_floor = started_unix_ms
            .unwrap_or(requested_unix_ms)
            .max(requested_unix_ms);
        if observed_unix_ms.is_some_and(|value| value < observation_floor) {
            return Err(
                "terminal ownership-loss observation cannot precede its observed start/request time"
                    .into(),
            );
        }
        let updated = tx.execute(
            "UPDATE executions
         SET status = ?2, status_source = ?3,
             ended_unix_ms = NULL, duration_ms = NULL
         WHERE execution_id = ?1 AND status IN (?4, ?5)",
            params![
                execution_id,
                ExecutionStatus::OwnershipLost.as_str(),
                FactSource::WindsObserved.as_str(),
                ExecutionStatus::Requested.as_str(),
                ExecutionStatus::Running.as_str(),
            ],
        )?;
        if updated != 1 {
            return Err(
                "terminal ownership-loss transition lost its expected non-final row".into(),
            );
        }
        set_terminal_close_reason(
            &tx,
            execution_id,
            TerminalCloseReason::OwnershipLostProcessStateUnknown,
        )?;
        insert_execution_event_if_time(
            &tx,
            execution_id,
            event_kind,
            FactSource::WindsObserved,
            observed_unix_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn reconcile_unowned_terminal_sessions_after_restart(
        &mut self,
        now_ms: i64,
    ) -> Result<usize> {
        self.retry_deferred_terminal_finalizations()?;
        let tx = self.connection.transaction()?;
        let executions = {
            let mut statement = tx.prepare(
                "SELECT e.execution_id, t.execution_id, e.requested_unix_ms, e.started_unix_ms
             FROM executions e
             LEFT JOIN terminal_sessions t ON t.execution_id = e.execution_id
             WHERE e.kind = ?1 AND e.status IN (?2, ?3)
             ORDER BY e.requested_unix_ms, e.execution_id",
            )?;
            statement
                .query_map(
                    params![
                        ExecutionKind::Terminal.as_str(),
                        ExecutionStatus::Requested.as_str(),
                        ExecutionStatus::Running.as_str(),
                    ],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, Option<String>>(1)?,
                            row.get::<_, i64>(2)?,
                            row.get::<_, Option<i64>>(3)?,
                        ))
                    },
                )?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };

        for (execution_id, terminal_session_id, requested_unix_ms, started_unix_ms) in &executions {
            let updated = tx.execute(
                "UPDATE executions
             SET status = ?2, status_source = ?3,
                 ended_unix_ms = NULL, duration_ms = NULL
             WHERE execution_id = ?1 AND status IN (?4, ?5)",
                params![
                    execution_id,
                    ExecutionStatus::OwnershipLost.as_str(),
                    FactSource::WindsObserved.as_str(),
                    ExecutionStatus::Requested.as_str(),
                    ExecutionStatus::Running.as_str(),
                ],
            )?;
            if updated != 1 {
                return Err(format!(
                    "terminal ownership-loss reconciliation lost its non-final row: {execution_id}"
                )
                .into());
            }
            if terminal_session_id.is_some() {
                set_terminal_close_reason(
                    &tx,
                    execution_id,
                    TerminalCloseReason::OwnershipLostProcessStateUnknown,
                )?;
            }
            let observation_floor = started_unix_ms
                .unwrap_or(*requested_unix_ms)
                .max(*requested_unix_ms);
            insert_execution_event(
                &tx,
                execution_id,
                "TerminalOwnershipLostAfterRestart",
                FactSource::WindsObserved,
                now_ms.max(observation_floor),
            )?;
        }
        tx.commit()?;
        Ok(executions.len())
    }

    pub fn load_execution(&self, execution_id: &str) -> Result<ExecutionRecord> {
        let row = self
            .connection
            .query_row(
                "SELECT execution_id, workspace_id, kind, request_source, execution_domain,
                        status, status_source, requested_unix_ms,
                        started_unix_ms, ended_unix_ms, duration_ms
                 FROM executions WHERE execution_id = ?1",
                params![execution_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, i64>(7)?,
                        row.get::<_, Option<i64>>(8)?,
                        row.get::<_, Option<i64>>(9)?,
                        row.get::<_, Option<i64>>(10)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown Winds execution: {execution_id}"))?;

        let kind = ExecutionKind::from_db(&row.2)
            .ok_or_else(|| format!("unknown execution kind in store: {}", row.2))?;
        let request_source = FactSource::from_db(&row.3)
            .ok_or_else(|| format!("unknown execution request source in store: {}", row.3))?;
        let status = ExecutionStatus::from_db(&row.5)
            .ok_or_else(|| format!("unknown execution status in store: {}", row.5))?;
        let status_source = FactSource::from_db(&row.6)
            .ok_or_else(|| format!("unknown execution status source in store: {}", row.6))?;
        let duration_ms = row.10.map(u64::try_from).transpose()?;

        Ok(ExecutionRecord {
            execution_id: row.0,
            workspace_id: row.1,
            kind,
            request_source,
            execution_domain: row.4,
            status,
            status_source,
            requested_unix_ms: row.7,
            started_unix_ms: row.8,
            ended_unix_ms: row.9,
            duration_ms,
        })
    }

    pub fn record_execution_event(
        &self,
        execution_id: &str,
        kind: &str,
        source: FactSource,
        now_ms: i64,
    ) -> Result<()> {
        insert_execution_event(&self.connection, execution_id, kind, source, now_ms)?;
        Ok(())
    }

    pub fn execution_events(&self, execution_id: &str) -> Result<Vec<ExecutionEventRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT event_id, execution_id, kind, fact_source, created_unix_ms
             FROM execution_events
             WHERE execution_id = ?1
             ORDER BY created_unix_ms, event_id",
        )?;
        let raw_rows = statement
            .query_map(params![execution_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let mut events = Vec::with_capacity(raw_rows.len());
        for row in raw_rows {
            let source = FactSource::from_db(&row.3)
                .ok_or_else(|| format!("unknown execution event source in store: {}", row.3))?;
            events.push(ExecutionEventRecord {
                event_id: row.0,
                execution_id: row.1,
                kind: row.2,
                source,
                created_unix_ms: row.4,
            });
        }
        Ok(events)
    }

    pub fn create_terminal_session(&self, session: NewTerminalSession<'_>) -> Result<()> {
        let kind = self
            .connection
            .query_row(
                "SELECT kind FROM executions WHERE execution_id = ?1",
                params![session.execution_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or_else(|| {
                format!(
                    "unknown Winds execution for terminal session: {}",
                    session.execution_id
                )
            })?;
        if kind != ExecutionKind::Terminal.as_str() {
            return Err("terminal session persistence requires TERMINAL execution kind".into());
        }
        let shell_arguments_json = serde_json::to_string(session.shell_arguments)?;
        self.connection.execute(
            "INSERT INTO terminal_sessions(
                execution_id, profile_id, shell_executable, shell_arguments_json,
                requested_cwd, initial_cols, initial_rows, close_reason
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)",
            params![
                session.execution_id,
                session.profile_id,
                shell_executable,
                shell_arguments_json,
                session.requested_cwd,
                session.initial_cols.map(i64::from),
                session.initial_rows.map(i64::from),
            ],
        )?;
        Ok(())
    }

    pub fn load_shell_command(&self, execution_id: &str) -> Result<ShellCommandRecord> {
        let row = self
            .connection
            .query_row(
                "SELECT execution_id, executable, arguments_json, command_source,
                        requested_cwd, cwd_source, exit_code, exit_source, observed_end_unix_ms
                 FROM shell_commands WHERE execution_id = ?1",
                params![execution_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, Option<i64>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                        row.get::<_, Option<i64>>(8)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown Winds shell command: {execution_id}"))?;
        let command_source = FactSource::from_db(&row.3)
            .ok_or_else(|| format!("unknown shell-command source in store: {}", row.3))?;
        let cwd_source = FactSource::from_db(&row.5)
            .ok_or_else(|| format!("unknown shell-command cwd source in store: {}", row.5))?;
        let exit_source = row
            .7
            .as_deref()
            .map(|value| {
                FactSource::from_db(value)
                    .ok_or_else(|| format!("unknown shell-command exit source in store: {value}"))
            })
            .transpose()?;
        Ok(ShellCommandRecord {
            execution_id: row.0,
            executable: row.1,
            arguments: serde_json::from_str(&row.2)?,
            command_source,
            requested_cwd: row.4,
            cwd_source,
            exit_code: row.6.map(i32::try_from).transpose()?,
            exit_source,
            observed_end_unix_ms: row.8,
        })
    }

    #[cfg(test)]
    pub(crate) fn shell_command_count_for_workspace(&self, workspace_id: &str) -> Result<usize> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*)
             FROM shell_commands c
             INNER JOIN executions e ON e.execution_id = c.execution_id
             WHERE e.workspace_id = ?1",
            params![workspace_id],
            |row| row.get(0),
        )?;
        Ok(usize::try_from(count)?)
    }

    pub fn load_terminal_session(&self, execution_id: &str) -> Result<TerminalSessionRecord> {
        let row = self
            .connection
            .query_row(
                "SELECT execution_id, profile_id, shell_executable, shell_arguments_json,
                        requested_cwd, initial_cols, initial_rows, close_reason
                 FROM terminal_sessions WHERE execution_id = ?1",
                params![execution_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<i64>>(5)?,
                        row.get::<_, Option<i64>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown Winds terminal session: {execution_id}"))?;

        let close_reason = row
            .7
            .as_deref()
            .map(|value| {
                TerminalCloseReason::from_db(value)
                    .ok_or_else(|| format!("unknown terminal close reason in store: {value}"))
            })
            .transpose()?;

        Ok(TerminalSessionRecord {
            execution_id: row.0,
            profile_id: row.1,
            shell_executable: row.2,
            shell_arguments: serde_json::from_str(&row.3)?,
            requested_cwd: row.4,
            initial_cols: row.5.map(u16::try_from).transpose()?,
            initial_rows: row.6.map(u16::try_from).transpose()?,
            close_reason,
        })
    }
}

impl Store {
    pub fn create_run(&mut self, run: NewRun<'_>, now_ms: i64) -> Result<()> {
        let timeout_secs = i64::try_from(run.timeout_secs)?;
        let tx = self.connection.transaction()?;
        tx.execute(
            "INSERT INTO candidate_runs(
                run_id, repo_path, base_oid, candidate_ref, candidate_oid, candidate_tree,
                worktree_path, check_command, timeout_secs, state, created_unix_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'PROVISIONING', ?10)",
            params![
                run.run_id,
                run.repo_path,
                run.base_oid,
                run.candidate_ref,
                run.candidate_oid,
                run.candidate_tree,
                run.worktree_path,
                run.check_command,
                timeout_secs,
                now_ms,
            ],
        )?;
        insert_event(
            &tx,
            run.run_id,
            "WorkspaceProvisionRequested",
            "WINDS_OBSERVED",
            "{}",
            now_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn mark_workspace_ready(&mut self, run_id: &str, now_ms: i64) -> Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute(
            "UPDATE candidate_runs SET state = 'READY' WHERE run_id = ?1",
            params![run_id],
        )?;
        insert_event(
            &tx,
            run_id,
            "WorkspaceReady",
            "WINDS_OBSERVED",
            "{}",
            now_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn write_blob(
        &self,
        run_id: &str,
        name: &str,
        bytes: &[u8],
        truncated: bool,
    ) -> Result<BlobEvidence> {
        let digest = Sha256::digest(bytes);
        let sha256: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
        let dir = self.home.join("blobs").join(run_id);
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{name}.{sha256}"));

        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                file.write_all(bytes)?;
                file.sync_all()?;
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                if !existing_blob_matches(&path, bytes)? {
                    return Err(format!(
                        "existing evidence blob does not match its content-addressed path: {}",
                        path.display()
                    )
                    .into());
                }
            }
            Err(error) => return Err(error.into()),
        }

        let relative = path.strip_prefix(&self.home)?;
        let relative_path = relative
            .to_str()
            .ok_or("evidence blob path is not valid UTF-8")?
            .to_owned();
        Ok(BlobEvidence {
            relative_path,
            sha256,
            captured_bytes: bytes.len(),
            truncated,
        })
    }

    pub(crate) fn save_evidence(&mut self, report: &EvidenceReport, now_ms: i64) -> Result<()> {
        let tx = self.connection.transaction()?;
        let persisted = tx
            .query_row(
                "SELECT repo_path, base_oid, candidate_ref, candidate_oid, candidate_tree,
                        worktree_path, check_command, state
                 FROM candidate_runs WHERE run_id = ?1",
                params![report.run_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown Winds run for evidence: {}", report.run_id))?;

        if persisted.7 != "READY" {
            return Err(format!(
                "verification evidence requires READY candidate state, found {}",
                persisted.7
            )
            .into());
        }
        if report.schema_version != 1 {
            return Err("unsupported verification evidence schema version".into());
        }
        if report.authority != "WINDS_OBSERVED" || report.check.authority != "WINDS_OBSERVED" {
            return Err("verification evidence authority must be WINDS_OBSERVED".into());
        }

        let bindings = [
            ("repository path", report.repo_path.as_str(), persisted.0.as_str()),
            ("base OID", report.base_oid.as_str(), persisted.1.as_str()),
            ("candidate ref", report.candidate_ref.as_str(), persisted.2.as_str()),
            ("candidate OID", report.candidate_oid.as_str(), persisted.3.as_str()),
            ("candidate tree", report.candidate_tree.as_str(), persisted.4.as_str()),
            ("worktree path", report.worktree_path.as_str(), persisted.5.as_str()),
            ("check command", report.check.command.as_str(), persisted.6.as_str()),
        ];
        for (label, reported, expected) in bindings {
            if reported != expected {
                return Err(format!(
                    "verification evidence {label} does not match persisted candidate run"
                )
                .into());
            }
        }

        const HEAD_CHANGED: &str = "candidate HEAD changed while evidence was being collected";
        const WORKTREE_MUTATED: &str = "required check mutated candidate worktree state";
        const OUTPUT_TRUNCATED: &str =
            "required check output exceeded the capture cap; evidence is incomplete";

        if report.warnings.iter().any(|warning| {
            !matches!(
                warning.as_str(),
                HEAD_CHANGED | WORKTREE_MUTATED | OUTPUT_TRUNCATED
            )
        }) {
            return Err("verification evidence contains an unknown warning".into());
        }
        if report
            .warnings
            .iter()
            .enumerate()
            .any(|(index, warning)| report.warnings[..index].contains(warning))
        {
            return Err("verification evidence contains duplicate warnings".into());
        }

        let head_changed = report.warnings.iter().any(|warning| warning == HEAD_CHANGED);
        let worktree_mutated = report
            .warnings
            .iter()
            .any(|warning| warning == WORKTREE_MUTATED);
        let output_truncated = report.check.stdout.truncated || report.check.stderr.truncated;
        let truncation_warning = report
            .warnings
            .iter()
            .any(|warning| warning == OUTPUT_TRUNCATED);
        if output_truncated != truncation_warning {
            return Err("verification evidence truncation warning is inconsistent".into());
        }
        if report.check.status == crate::domain::CheckStatus::Pass
            && report.check.exit_code != Some(0)
        {
            return Err("PASS verification evidence requires exit code 0".into());
        }
        if report.check.status == crate::domain::CheckStatus::Fail
            && report.check.exit_code == Some(0)
        {
            return Err("FAIL verification evidence cannot carry exit code 0".into());
        }

        let expected_eligibility = if report.check.status != crate::domain::CheckStatus::Pass
            || head_changed
            || worktree_mutated
        {
            Eligibility::Blocked
        } else if output_truncated {
            Eligibility::Warning
        } else {
            Eligibility::Eligible
        };
        if report.eligibility != expected_eligibility {
            return Err(format!(
                "verification evidence eligibility {} is inconsistent with observed report state {}",
                report.eligibility.as_str(),
                expected_eligibility.as_str()
            )
            .into());
        }

        let json = serde_json::to_string(report)?;
        let updated = tx.execute(
            "UPDATE candidate_runs SET state = 'VERIFIED' WHERE run_id = ?1 AND state = 'READY'",
            params![report.run_id],
        )?;
        if updated != 1 {
            return Err("verification evidence lost its expected READY candidate row".into());
        }
        tx.execute(
            "INSERT INTO evidence_reports(run_id, eligibility, report_json, created_unix_ms)
             VALUES (?1, ?2, ?3, ?4)",
            params![report.run_id, report.eligibility.as_str(), json, now_ms],
        )?;
        insert_event(
            &tx,
            &report.run_id,
            "EvidenceBuilt",
            "WINDS_OBSERVED",
            "{}",
            now_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn load_run(&self, run_id: &str) -> Result<StoredRun> {
        let row = self
            .connection
            .query_row(
                "SELECT r.run_id, r.repo_path, r.candidate_oid, r.candidate_tree,
                        r.worktree_path, r.check_command, r.timeout_secs, e.eligibility
                 FROM candidate_runs r
                 LEFT JOIN evidence_reports e ON e.run_id = r.run_id
                 WHERE r.run_id = ?1",
                params![run_id],
                |row| {
                    let timeout_secs: i64 = row.get(6)?;
                    let eligibility: Option<String> = row.get(7)?;
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        timeout_secs,
                        eligibility,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown Winds run: {run_id}"))?;

        let eligibility = match row.7.as_deref() {
            Some("ELIGIBLE") => Eligibility::Eligible,
            Some("WARNING") => Eligibility::Warning,
            _ => Eligibility::Blocked,
        };
        Ok(StoredRun {
            run_id: row.0,
            repo_path: row.1,
            candidate_oid: row.2,
            candidate_tree: row.3,
            worktree_path: row.4,
            check_command: row.5,
            timeout_secs: u64::try_from(row.6)?,
            eligibility,
        })
    }

    pub fn runs_for_repo(&self, repo_path: &str) -> Result<Vec<RecoverableRun>> {
        let mut statement = self.connection.prepare(
            "SELECT run_id, candidate_oid, worktree_path, state
             FROM candidate_runs WHERE repo_path = ?1 ORDER BY created_unix_ms, run_id",
        )?;
        let rows = statement.query_map(params![repo_path], |row| {
            Ok(RecoverableRun {
                run_id: row.get(0)?,
                candidate_oid: row.get(1)?,
                worktree_path: row.get(2)?,
                state: row.get(3)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn record_recovery_required(
        &mut self,
        run_id: &str,
        reason: &str,
        now_ms: i64,
    ) -> Result<()> {
        let payload = serde_json::to_string(&serde_json::json!({ "reason": reason }))?;
        let tx = self.connection.transaction()?;
        insert_event(
            &tx,
            run_id,
            "RecoveryRequired",
            "WINDS_OBSERVED",
            &payload,
            now_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn record_promotion_recheck(
        &mut self,
        run_id: &str,
        evidence: &CheckEvidence,
        now_ms: i64,
    ) -> Result<()> {
        let payload = serde_json::to_string(evidence)?;
        let tx = self.connection.transaction()?;
        insert_event(
            &tx,
            run_id,
            "PromotionRecheckObserved",
            "WINDS_OBSERVED",
            &payload,
            now_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn record_promotion(
        &mut self,
        run_id: &str,
        branch: &str,
        commit_oid: &str,
        now_ms: i64,
    ) -> Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute(
            "INSERT INTO promotions(run_id, branch, commit_oid, created_unix_ms) VALUES (?1, ?2, ?3, ?4)",
            params![run_id, branch, commit_oid, now_ms],
        )?;
        insert_event(
            &tx,
            run_id,
            "DecisionRecorded",
            "CALLER_REQUESTED",
            "{\"decision\":\"promote\"}",
            now_ms,
        )?;
        insert_event(
            &tx,
            run_id,
            "PromotionCreated",
            "WINDS_OBSERVED",
            "{}",
            now_ms,
        )?;
        tx.commit()?;
        Ok(())
    }
}

fn existing_blob_matches(path: &Path, expected: &[u8]) -> Result<bool> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() {
        return Ok(false);
    }

    let mut file = open_existing_blob(path)?;
    let metadata = file.metadata()?;
    if !metadata.file_type().is_file() || metadata.len() != u64::try_from(expected.len())? {
        return Ok(false);
    }

    let mut offset = 0_usize;
    let mut buffer = [0_u8; 8192];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let end = offset
            .checked_add(count)
            .ok_or("existing evidence blob comparison overflow")?;
        if end > expected.len() || expected[offset..end] != buffer[..count] {
            return Ok(false);
        }
        offset = end;
    }
    Ok(offset == expected.len())
}

#[cfg(unix)]
fn open_existing_blob(path: &Path) -> Result<File> {
    Ok(OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?)
}

#[cfg(not(unix))]
fn open_existing_blob(_path: &Path) -> Result<File> {
    Err("existing evidence blob validation is unsupported on this platform".into())
}

fn terminal_execution_state(
    connection: &Connection,
    execution_id: &str,
) -> Result<(ExecutionStatus, i64, Option<i64>)> {
    let row = connection
        .query_row(
            "SELECT e.status, e.requested_unix_ms, e.started_unix_ms
             FROM executions e
             INNER JOIN terminal_sessions t ON t.execution_id = e.execution_id
             WHERE e.execution_id = ?1 AND e.kind = ?2",
            params![execution_id, ExecutionKind::Terminal.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| format!("unknown persisted terminal execution: {execution_id}"))?;
    let status = ExecutionStatus::from_db(&row.0)
        .ok_or_else(|| format!("unknown execution status in store: {}", row.0))?;
    Ok((status, row.1, row.2))
}

fn set_terminal_close_reason(
    connection: &Connection,
    execution_id: &str,
    reason: TerminalCloseReason,
) -> Result<()> {
    let updated = connection.execute(
        "UPDATE terminal_sessions SET close_reason = ?2 WHERE execution_id = ?1",
        params![execution_id, reason.as_str()],
    )?;
    if updated != 1 {
        return Err(format!("unknown persisted terminal session: {execution_id}").into());
    }
    Ok(())
}

fn finalize_running_terminal(
    connection: &mut Connection,
    execution_id: &str,
    status: ExecutionStatus,
    close_reason: TerminalCloseReason,
    event_kind: &str,
    ended_unix_ms: Option<i64>,
) -> Result<()> {
    if !matches!(
        status,
        ExecutionStatus::Exited | ExecutionStatus::Interrupted
    ) {
        return Err("terminal finalization requires EXITED or INTERRUPTED status".into());
    }
    let tx = connection.transaction()?;
    let (current_status, _requested_unix_ms, started_unix_ms) =
        terminal_execution_state(&tx, execution_id)?;
    if current_status != ExecutionStatus::Running {
        return Err(format!(
            "terminal execution cannot finalize from persisted state {}: {execution_id}",
            current_status.as_str()
        )
        .into());
    }
    let started_unix_ms =
        started_unix_ms.ok_or("RUNNING terminal execution is missing its observed start time")?;
    if ended_unix_ms.is_some_and(|value| value < started_unix_ms) {
        return Err("terminal end time cannot precede its observed start time".into());
    }
    let duration_ms = ended_unix_ms.map(|value| value - started_unix_ms);
    let updated = tx.execute(
        "UPDATE executions
         SET status = ?2, status_source = ?3,
             ended_unix_ms = ?4, duration_ms = ?5
         WHERE execution_id = ?1 AND status = ?6",
        params![
            execution_id,
            status.as_str(),
            FactSource::WindsObserved.as_str(),
            ended_unix_ms,
            duration_ms,
            ExecutionStatus::Running.as_str(),
        ],
    )?;
    if updated != 1 {
        return Err("terminal finalization lost its expected RUNNING row".into());
    }
    set_terminal_close_reason(&tx, execution_id, close_reason)?;
    insert_execution_event_if_time(
        &tx,
        execution_id,
        event_kind,
        FactSource::WindsObserved,
        ended_unix_ms,
    )?;
    tx.commit()?;
    Ok(())
}

fn insert_event(
    connection: &Connection,
    run_id: &str,
    kind: &str,
    authority: &str,
    payload_json: &str,
    now_ms: i64,
) -> rusqlite::Result<()> {
    connection.execute(
        "INSERT INTO events(run_id, kind, authority, payload_json, created_unix_ms)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![run_id, kind, authority, payload_json, now_ms],
    )?;
    Ok(())
}

fn insert_execution_event_if_time(
    connection: &Connection,
    execution_id: &str,
    kind: &str,
    source: FactSource,
    observed_unix_ms: Option<i64>,
) -> Result<()> {
    if let Some(observed_unix_ms) = observed_unix_ms {
        insert_execution_event(connection, execution_id, kind, source, observed_unix_ms)?;
    }
    Ok(())
}

fn validate_optional_command_times(
    requested_unix_ms: i64,
    started_unix_ms: Option<i64>,
    ended_unix_ms: Option<i64>,
) -> Result<()> {
    if started_unix_ms.is_some_and(|value| value < requested_unix_ms) {
        return Err("shell-command start time cannot precede its request time".into());
    }
    if ended_unix_ms.is_some_and(|value| value < requested_unix_ms) {
        return Err("shell-command end time cannot precede its request time".into());
    }
    if let (Some(started), Some(ended)) = (started_unix_ms, ended_unix_ms)
        && ended < started
    {
        return Err("shell-command end time cannot precede its start time".into());
    }
    Ok(())
}

fn optional_duration_ms(
    started_unix_ms: Option<i64>,
    ended_unix_ms: Option<i64>,
) -> Result<Option<i64>> {
    match (started_unix_ms, ended_unix_ms) {
        (Some(started), Some(ended)) => Ok(Some(
            ended
                .checked_sub(started)
                .ok_or("shell-command duration overflow")?,
        )),
        _ => Ok(None),
    }
}

fn shell_command_execution_state(
    connection: &Connection,
    execution_id: &str,
) -> Result<(ExecutionStatus, i64, Option<i64>)> {
    let row = connection
        .query_row(
            "SELECT e.status, e.requested_unix_ms, e.started_unix_ms
             FROM executions e
             INNER JOIN shell_commands c ON c.execution_id = e.execution_id
             WHERE e.execution_id = ?1 AND e.kind = ?2",
            params![execution_id, ExecutionKind::ShellCommand.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| format!("unknown Winds shell command execution: {execution_id}"))?;
    let status = ExecutionStatus::from_db(&row.0)
        .ok_or_else(|| format!("unknown shell-command execution status in store: {}", row.0))?;
    Ok((status, row.1, row.2))
}

#[allow(
    dead_code,
    reason = "Spec 003 T044 persistence substrate; runtime callers land in later slices"
)]
fn insert_execution_event(
    connection: &Connection,
    execution_id: &str,
    kind: &str,
    source: FactSource,
    now_ms: i64,
) -> rusqlite::Result<()> {
    connection.execute(
        "INSERT INTO execution_events(execution_id, kind, fact_source, created_unix_ms)
         VALUES (?1, ?2, ?3, ?4)",
        params![execution_id, kind, source.as_str(), now_ms],
    )?;
    Ok(())
}

#[allow(
    dead_code,
    reason = "Spec 006 T070 persistence substrate; product session semantics land in T071"
)]
impl Store {
    pub fn create_workstream(&self, workstream: NewWorkstream<'_>, now_ms: i64) -> Result<()> {
        validate_agentic_identity_text(workstream.workstream_id, "workstream id")?;
        validate_agentic_identity_text(workstream.display_name, "workstream display name")?;
        validate_agentic_identity_timestamp(now_ms, "workstream creation time")?;
        self.load_workspace(workstream.workspace_id)?;
        self.connection.execute(
            "INSERT INTO workstreams(
                workstream_id, workspace_id, display_name, created_unix_ms, updated_unix_ms
             ) VALUES (?1, ?2, ?3, ?4, ?4)",
            params![
                workstream.workstream_id,
                workstream.workspace_id,
                workstream.display_name,
                now_ms,
            ],
        )?;
        Ok(())
    }

    pub fn load_workstream(&self, workstream_id: &str) -> Result<crate::domain::WorkstreamRecord> {
        self.connection
            .query_row(
                "SELECT workstream_id, workspace_id, display_name,
                        created_unix_ms, updated_unix_ms
                 FROM workstreams WHERE workstream_id = ?1",
                params![workstream_id],
                |row| {
                    Ok(crate::domain::WorkstreamRecord {
                        workstream_id: row.get(0)?,
                        workspace_id: row.get(1)?,
                        display_name: row.get(2)?,
                        created_unix_ms: row.get(3)?,
                        updated_unix_ms: row.get(4)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown Winds workstream: {workstream_id}").into())
    }

    pub fn list_workstreams(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<crate::domain::WorkstreamRecord>> {
        self.load_workspace(workspace_id)?;
        let mut statement = self.connection.prepare(
            "SELECT workstream_id, workspace_id, display_name,
                    created_unix_ms, updated_unix_ms
             FROM workstreams
             WHERE workspace_id = ?1
             ORDER BY created_unix_ms, workstream_id",
        )?;
        let rows = statement.query_map(params![workspace_id], |row| {
            Ok(crate::domain::WorkstreamRecord {
                workstream_id: row.get(0)?,
                workspace_id: row.get(1)?,
                display_name: row.get(2)?,
                created_unix_ms: row.get(3)?,
                updated_unix_ms: row.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn rename_workstream(
        &self,
        workstream_id: &str,
        display_name: &str,
        now_ms: i64,
    ) -> Result<()> {
        validate_agentic_identity_text(display_name, "workstream display name")?;
        validate_agentic_identity_timestamp(now_ms, "workstream rename time")?;
        let existing = self.load_workstream(workstream_id)?;
        if now_ms < existing.created_unix_ms {
            return Err("workstream rename time cannot precede creation time".into());
        }
        if now_ms < existing.updated_unix_ms {
            return Err("workstream rename time cannot precede current update time".into());
        }
        let updated = self.connection.execute(
            "UPDATE workstreams
             SET display_name = ?2, updated_unix_ms = ?3
             WHERE workstream_id = ?1 AND updated_unix_ms <= ?3",
            params![workstream_id, display_name, now_ms],
        )?;
        if updated != 1 {
            return Err("workstream rename lost monotonic update race".into());
        }
        Ok(())
    }

    pub fn create_winds_session(&self, session: NewWindsSession<'_>, now_ms: i64) -> Result<()> {
        validate_agentic_identity_text(session.session_id, "Winds session id")?;
        validate_agentic_identity_text(session.display_name, "Winds session display name")?;
        validate_agentic_identity_timestamp(now_ms, "Winds session creation time")?;
        self.load_workstream(session.workstream_id)?;
        self.connection.execute(
            "INSERT INTO winds_sessions(
                session_id, workstream_id, display_name, created_unix_ms, updated_unix_ms
             ) VALUES (?1, ?2, ?3, ?4, ?4)",
            params![
                session.session_id,
                session.workstream_id,
                session.display_name,
                now_ms,
            ],
        )?;
        Ok(())
    }

    pub fn load_winds_session(
        &self,
        session_id: &str,
    ) -> Result<crate::domain::WindsSessionRecord> {
        self.connection
            .query_row(
                "SELECT session_id, workstream_id, display_name,
                        created_unix_ms, updated_unix_ms
                 FROM winds_sessions WHERE session_id = ?1",
                params![session_id],
                |row| {
                    Ok(crate::domain::WindsSessionRecord {
                        session_id: row.get(0)?,
                        workstream_id: row.get(1)?,
                        display_name: row.get(2)?,
                        created_unix_ms: row.get(3)?,
                        updated_unix_ms: row.get(4)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown Winds session: {session_id}").into())
    }

    pub fn list_winds_sessions(
        &self,
        workstream_id: &str,
    ) -> Result<Vec<crate::domain::WindsSessionRecord>> {
        self.load_workstream(workstream_id)?;
        let mut statement = self.connection.prepare(
            "SELECT session_id, workstream_id, display_name,
                    created_unix_ms, updated_unix_ms
             FROM winds_sessions
             WHERE workstream_id = ?1
             ORDER BY created_unix_ms, session_id",
        )?;
        let rows = statement.query_map(params![workstream_id], |row| {
            Ok(crate::domain::WindsSessionRecord {
                session_id: row.get(0)?,
                workstream_id: row.get(1)?,
                display_name: row.get(2)?,
                created_unix_ms: row.get(3)?,
                updated_unix_ms: row.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn rename_winds_session(
        &self,
        session_id: &str,
        display_name: &str,
        now_ms: i64,
    ) -> Result<()> {
        validate_agentic_identity_text(display_name, "Winds session display name")?;
        validate_agentic_identity_timestamp(now_ms, "Winds session rename time")?;
        let existing = self.load_winds_session(session_id)?;
        if now_ms < existing.created_unix_ms {
            return Err("Winds session rename time cannot precede creation time".into());
        }
        if now_ms < existing.updated_unix_ms {
            return Err("Winds session rename time cannot precede current update time".into());
        }
        let updated = self.connection.execute(
            "UPDATE winds_sessions
             SET display_name = ?2, updated_unix_ms = ?3
             WHERE session_id = ?1 AND updated_unix_ms <= ?3",
            params![session_id, display_name, now_ms],
        )?;
        if updated != 1 {
            return Err("Winds session rename lost monotonic update race".into());
        }
        Ok(())
    }
}

fn validate_agentic_identity_text(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(format!("{label} must not be empty").into());
    }
    Ok(())
}

fn validate_agentic_identity_timestamp(value: i64, label: &str) -> Result<()> {
    if value < 0 {
        return Err(format!("{label} must not be negative").into());
    }
    Ok(())
}

#[cfg(test)]
mod persistence_tests {
    use super::{NewExecution, NewTerminalSession, NewWorkspace, Store, TerminalFinalization};
    use crate::domain::{ExecutionKind, ExecutionStatus, FactSource, TerminalCloseReason};
    use std::fs;
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

    fn test_home(name: &str) -> PathBuf {
        let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
        let home = std::env::temp_dir().join(format!(
            "winds-store-{name}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&home).unwrap();
        home
    }

    fn remove_file_if_exists(path: &Path) {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => panic!("failed to remove test file {}: {error}", path.display()),
        }
    }

    fn cleanup_test_home(home: &Path) {
        remove_file_if_exists(&home.join("winds.db-wal"));
        remove_file_if_exists(&home.join("winds.db-shm"));
        remove_file_if_exists(&home.join("winds.db-journal"));
        remove_file_if_exists(&home.join("winds.db"));
        fs::remove_dir(home.join("blobs")).unwrap();
        fs::remove_dir(home).unwrap();
    }

    #[test]
    fn workspace_execution_ledger_is_separate_and_source_labeled() {
        let home = test_home("execution-ledger");
        let mut store = Store::open(&home).unwrap();

        store
            .create_workspace(
                NewWorkspace {
                    workspace_id: "workspace-1",
                    canonical_worktree_root: "/tmp/example",
                    git_common_dir: "/tmp/example/.git",
                },
                100,
            )
            .unwrap();
        store.mark_workspace_opened("workspace-1", 105).unwrap();
        store
            .create_execution(
                NewExecution {
                    execution_id: "execution-1",
                    workspace_id: "workspace-1",
                    kind: ExecutionKind::Terminal,
                    request_source: FactSource::CallerRequested,
                    execution_domain: "host-linux",
                },
                110,
            )
            .unwrap();

        let shell_arguments = vec!["--login".to_owned()];
        store
            .create_terminal_session(NewTerminalSession {
                execution_id: "execution-1",
                profile_id: "bash-login",
                shell_executable: "/bin/bash",
                shell_arguments: &shell_arguments,
                requested_cwd: "/tmp/example",
                initial_cols: Some(120),
                initial_rows: Some(40),
            })
            .unwrap();
        store
            .record_execution_event(
                "execution-1",
                "ShellTelemetryReceived",
                FactSource::ShellReported,
                120,
            )
            .unwrap();

        let workspace = store.load_workspace("workspace-1").unwrap();
        assert_eq!(workspace.canonical_worktree_root, "/tmp/example");
        assert_eq!(workspace.git_common_dir, "/tmp/example/.git");
        assert_eq!(workspace.created_unix_ms, 100);
        assert_eq!(workspace.last_opened_unix_ms, 105);

        let execution = store.load_execution("execution-1").unwrap();
        assert_eq!(execution.kind, ExecutionKind::Terminal);
        assert_eq!(execution.request_source, FactSource::CallerRequested);
        assert_eq!(execution.status, ExecutionStatus::Requested);
        assert_eq!(execution.status_source, FactSource::CallerRequested);
        assert_eq!(execution.requested_unix_ms, 110);
        assert_eq!(execution.started_unix_ms, None);
        assert_eq!(execution.ended_unix_ms, None);
        assert_eq!(execution.duration_ms, None);

        let terminal = store.load_terminal_session("execution-1").unwrap();
        assert_eq!(terminal.profile_id, "bash-login");
        assert_eq!(terminal.shell_executable, "/bin/bash");
        assert_eq!(terminal.shell_arguments, shell_arguments);
        assert_eq!(terminal.requested_cwd, "/tmp/example");
        assert_eq!(terminal.initial_cols, Some(120));
        assert_eq!(terminal.initial_rows, Some(40));
        assert_eq!(terminal.close_reason, None);

        let oversized_dimension = store.connection.execute(
            "UPDATE terminal_sessions SET initial_cols = 65536 WHERE execution_id = ?1",
            rusqlite::params!["execution-1"],
        );
        assert!(oversized_dimension.is_err());

        let execution_events = store.execution_events("execution-1").unwrap();
        assert_eq!(execution_events.len(), 2);
        assert_eq!(execution_events[0].kind, "ExecutionRequested");
        assert_eq!(execution_events[0].source, FactSource::CallerRequested);
        assert_eq!(execution_events[1].kind, "ShellTelemetryReceived");
        assert_eq!(execution_events[1].source, FactSource::ShellReported);

        let candidate_event_count: i64 = store
            .connection
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(candidate_event_count, 0);

        drop(store);
        cleanup_test_home(&home);
    }

    #[test]
    fn failed_launch_can_end_without_claiming_a_process_start_or_duration() {
        let home = test_home("failed-launch");
        let mut store = Store::open(&home).unwrap();
        store
            .create_workspace(
                NewWorkspace {
                    workspace_id: "workspace-1",
                    canonical_worktree_root: "/tmp/example",
                    git_common_dir: "/tmp/example/.git",
                },
                200,
            )
            .unwrap();
        store
            .create_execution(
                NewExecution {
                    execution_id: "execution-1",
                    workspace_id: "workspace-1",
                    kind: ExecutionKind::Terminal,
                    request_source: FactSource::CallerRequested,
                    execution_domain: "host-linux",
                },
                210,
            )
            .unwrap();

        store
            .connection
            .execute(
                "UPDATE executions
                 SET status = 'FAILED_TO_START',
                     status_source = 'WINDS_OBSERVED',
                     ended_unix_ms = ?2
                 WHERE execution_id = ?1",
                rusqlite::params!["execution-1", 215_i64],
            )
            .unwrap();

        let execution = store.load_execution("execution-1").unwrap();
        assert_eq!(execution.status, ExecutionStatus::FailedToStart);
        assert_eq!(execution.status_source, FactSource::WindsObserved);
        assert_eq!(execution.started_unix_ms, None);
        assert_eq!(execution.ended_unix_ms, Some(215));
        assert_eq!(execution.duration_ms, None);

        let invalid_duration = store.connection.execute(
            "UPDATE executions SET duration_ms = 5 WHERE execution_id = ?1",
            rusqlite::params!["execution-1"],
        );
        assert!(invalid_duration.is_err());

        drop(store);
        cleanup_test_home(&home);
    }

    #[test]
    fn terminal_lifecycle_transitions_are_atomic_typed_and_timed() {
        let home = test_home("terminal-lifecycle");
        let mut store = Store::open(&home).unwrap();
        store
            .create_workspace(
                NewWorkspace {
                    workspace_id: "workspace-1",
                    canonical_worktree_root: "/tmp/example",
                    git_common_dir: "/tmp/example/.git",
                },
                100,
            )
            .unwrap();
        let shell_arguments = Vec::new();
        store
            .create_terminal_execution(
                NewExecution {
                    execution_id: "execution-1",
                    workspace_id: "workspace-1",
                    kind: ExecutionKind::Terminal,
                    request_source: FactSource::CallerRequested,
                    execution_domain: "host-linux",
                },
                NewTerminalSession {
                    execution_id: "execution-1",
                    profile_id: "profile-1",
                    shell_executable: "/bin/sh",
                    shell_arguments: &shell_arguments,
                    requested_cwd: "/tmp/example",
                    initial_cols: Some(80),
                    initial_rows: Some(24),
                },
                110,
            )
            .unwrap();
        store.mark_terminal_running("execution-1", 120).unwrap();
        store
            .mark_terminal_exited("execution-1", Some(155))
            .unwrap();

        let execution = store.load_execution("execution-1").unwrap();
        assert_eq!(execution.status, ExecutionStatus::Exited);
        assert_eq!(execution.status_source, FactSource::WindsObserved);
        assert_eq!(execution.started_unix_ms, Some(120));
        assert_eq!(execution.ended_unix_ms, Some(155));
        assert_eq!(execution.duration_ms, Some(35));
        let terminal = store.load_terminal_session("execution-1").unwrap();
        assert_eq!(
            terminal.close_reason,
            Some(TerminalCloseReason::ProcessExited)
        );
        let events = store.execution_events("execution-1").unwrap();
        assert_eq!(
            events
                .iter()
                .map(|event| event.kind.as_str())
                .collect::<Vec<_>>(),
            ["ExecutionRequested", "TerminalStarted", "TerminalExited"]
        );
        assert_eq!(events[0].source, FactSource::CallerRequested);
        assert!(
            events[1..]
                .iter()
                .all(|event| event.source == FactSource::WindsObserved)
        );
        assert!(
            store
                .mark_terminal_exited("execution-1", Some(160))
                .is_err()
        );

        drop(store);
        cleanup_test_home(&home);
    }

    #[test]
    fn terminal_failed_and_interrupted_states_are_explicit() {
        let home = test_home("terminal-final-states");
        let mut store = Store::open(&home).unwrap();
        store
            .create_workspace(
                NewWorkspace {
                    workspace_id: "workspace-1",
                    canonical_worktree_root: "/tmp/example",
                    git_common_dir: "/tmp/example/.git",
                },
                100,
            )
            .unwrap();
        let shell_arguments = Vec::new();
        for execution_id in ["failed", "interrupted"] {
            store
                .create_terminal_execution(
                    NewExecution {
                        execution_id,
                        workspace_id: "workspace-1",
                        kind: ExecutionKind::Terminal,
                        request_source: FactSource::CallerRequested,
                        execution_domain: "host-linux",
                    },
                    NewTerminalSession {
                        execution_id,
                        profile_id: "profile-1",
                        shell_executable: "/bin/sh",
                        shell_arguments: &shell_arguments,
                        requested_cwd: "/tmp/example",
                        initial_cols: Some(80),
                        initial_rows: Some(24),
                    },
                    110,
                )
                .unwrap();
        }
        store.mark_terminal_failed_to_start("failed", 115).unwrap();
        store.mark_terminal_running("interrupted", 120).unwrap();
        store
            .mark_terminal_interrupted(
                "interrupted",
                TerminalCloseReason::TerminatedByWinds,
                Some(150),
            )
            .unwrap();

        let failed = store.load_execution("failed").unwrap();
        assert_eq!(failed.status, ExecutionStatus::FailedToStart);
        assert_eq!(failed.started_unix_ms, None);
        assert_eq!(failed.ended_unix_ms, Some(115));
        assert_eq!(failed.duration_ms, None);
        assert_eq!(
            store.load_terminal_session("failed").unwrap().close_reason,
            Some(TerminalCloseReason::FailedToStart)
        );
        let interrupted = store.load_execution("interrupted").unwrap();
        assert_eq!(interrupted.status, ExecutionStatus::Interrupted);
        assert_eq!(interrupted.duration_ms, Some(30));
        assert_eq!(
            store
                .load_terminal_session("interrupted")
                .unwrap()
                .close_reason,
            Some(TerminalCloseReason::TerminatedByWinds)
        );

        drop(store);
        cleanup_test_home(&home);
    }

    #[test]
    fn restart_reconciliation_marks_nonfinal_sessions_ownership_lost_without_pid_claims() {
        let home = test_home("ownership-lost");
        let mut store = Store::open(&home).unwrap();
        store
            .create_workspace(
                NewWorkspace {
                    workspace_id: "workspace-1",
                    canonical_worktree_root: "/tmp/example",
                    git_common_dir: "/tmp/example/.git",
                },
                100,
            )
            .unwrap();
        let shell_arguments = Vec::new();
        for execution_id in ["requested", "running"] {
            store
                .create_terminal_execution(
                    NewExecution {
                        execution_id,
                        workspace_id: "workspace-1",
                        kind: ExecutionKind::Terminal,
                        request_source: FactSource::CallerRequested,
                        execution_domain: "host-linux",
                    },
                    NewTerminalSession {
                        execution_id,
                        profile_id: "profile-1",
                        shell_executable: "/bin/sh",
                        shell_arguments: &shell_arguments,
                        requested_cwd: "/tmp/example",
                        initial_cols: Some(80),
                        initial_rows: Some(24),
                    },
                    110,
                )
                .unwrap();
        }
        store.mark_terminal_running("running", 120).unwrap();

        assert_eq!(
            store
                .reconcile_unowned_terminal_sessions_after_restart(200)
                .unwrap(),
            2
        );
        assert_eq!(
            store
                .reconcile_unowned_terminal_sessions_after_restart(210)
                .unwrap(),
            0
        );
        for execution_id in ["requested", "running"] {
            let execution = store.load_execution(execution_id).unwrap();
            assert_eq!(execution.status, ExecutionStatus::OwnershipLost);
            assert_eq!(execution.status_source, FactSource::WindsObserved);
            assert_eq!(execution.ended_unix_ms, None);
            assert_eq!(execution.duration_ms, None);
            assert_eq!(
                store
                    .load_terminal_session(execution_id)
                    .unwrap()
                    .close_reason,
                Some(TerminalCloseReason::OwnershipLostProcessStateUnknown)
            );
        }
        assert_eq!(
            store.load_execution("requested").unwrap().started_unix_ms,
            None
        );
        assert_eq!(
            store.load_execution("running").unwrap().started_unix_ms,
            Some(120)
        );

        let column_names = {
            let mut statement = store
                .connection
                .prepare("PRAGMA table_info(terminal_sessions)")
                .unwrap();
            statement
                .query_map([], |row| row.get::<_, String>(1))
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap()
        };
        assert!(!column_names.iter().any(|name| name.contains("pid")));

        drop(store);
        cleanup_test_home(&home);
    }

    #[test]
    fn atomic_terminal_request_rolls_back_if_session_insert_fails() {
        let home = test_home("terminal-request-rollback");
        let mut store = Store::open(&home).unwrap();
        store
            .create_workspace(
                NewWorkspace {
                    workspace_id: "workspace-1",
                    canonical_worktree_root: "/tmp/example",
                    git_common_dir: "/tmp/example/.git",
                },
                100,
            )
            .unwrap();
        let shell_arguments = Vec::new();
        let result = store.create_terminal_execution(
            NewExecution {
                execution_id: "execution-invalid",
                workspace_id: "workspace-1",
                kind: ExecutionKind::Terminal,
                request_source: FactSource::CallerRequested,
                execution_domain: "host-linux",
            },
            NewTerminalSession {
                execution_id: "execution-invalid",
                profile_id: "profile-1",
                shell_executable: "/bin/sh",
                shell_arguments: &shell_arguments,
                requested_cwd: "/tmp/example",
                initial_cols: Some(0),
                initial_rows: Some(24),
            },
            110,
        );
        assert!(result.is_err());
        assert!(store.load_execution("execution-invalid").is_err());
        let event_count: i64 = store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM execution_events WHERE execution_id = ?1",
                rusqlite::params!["execution-invalid"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(event_count, 0);

        drop(store);
        cleanup_test_home(&home);
    }

    #[test]
    fn deferred_terminal_finalization_can_be_retried_after_owner_drop() {
        let home = test_home("terminal-deferred-finalization");
        let mut store = Store::open(&home).unwrap();
        store
            .create_workspace(
                NewWorkspace {
                    workspace_id: "workspace-1",
                    canonical_worktree_root: "/tmp/example",
                    git_common_dir: "/tmp/example/.git",
                },
                100,
            )
            .unwrap();
        let shell_arguments = Vec::new();
        store
            .create_terminal_execution(
                NewExecution {
                    execution_id: "execution-deferred",
                    workspace_id: "workspace-1",
                    kind: ExecutionKind::Terminal,
                    request_source: FactSource::CallerRequested,
                    execution_domain: "host-linux",
                },
                NewTerminalSession {
                    execution_id: "execution-deferred",
                    profile_id: "profile-1",
                    shell_executable: "/bin/sh",
                    shell_arguments: &shell_arguments,
                    requested_cwd: "/tmp/example",
                    initial_cols: Some(80),
                    initial_rows: Some(24),
                },
                110,
            )
            .unwrap();
        store
            .mark_terminal_running("execution-deferred", 120)
            .unwrap();
        store.defer_terminal_finalization(
            "execution-deferred",
            TerminalFinalization::Interrupted {
                ended_unix_ms: Some(150),
                reason: TerminalCloseReason::ClosedByWinds,
            },
        );
        assert_eq!(store.pending_terminal_finalization_count(), 1);
        assert_eq!(store.retry_deferred_terminal_finalizations().unwrap(), 1);
        assert_eq!(store.pending_terminal_finalization_count(), 0);
        let execution = store.load_execution("execution-deferred").unwrap();
        assert_eq!(execution.status, ExecutionStatus::Interrupted);
        assert_eq!(execution.duration_ms, Some(30));
        assert_eq!(
            store
                .load_terminal_session("execution-deferred")
                .unwrap()
                .close_reason,
            Some(TerminalCloseReason::ClosedByWinds)
        );

        drop(store);
        cleanup_test_home(&home);
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::existing_blob_matches;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn existing_blob_rejects_symlink_even_when_target_bytes_match() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "winds-existing-blob-symlink-{nanos}-{}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let target = root.join("target");
        let link = root.join("blob");
        fs::write(&target, b"evidence").unwrap();
        symlink(&target, &link).unwrap();

        assert!(!existing_blob_matches(&link, b"evidence").unwrap());

        fs::remove_file(&link).unwrap();
        fs::remove_file(&target).unwrap();
        fs::remove_dir(&root).unwrap();
    }
}
