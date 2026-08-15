use crate::domain::{
    BlobEvidence, CheckEvidence, Eligibility, EvidenceReport, ExecutionEventRecord, ExecutionKind,
    ExecutionRecord, ExecutionStatus, FactSource, StoredRun, TerminalSessionRecord,
    WorkspaceRecord,
};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub struct Store {
    connection: Connection,
    home: PathBuf,
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

pub struct NewWorkspace<'a> {
    pub workspace_id: &'a str,
    pub canonical_worktree_root: &'a str,
    pub git_common_dir: &'a str,
}

pub struct NewExecution<'a> {
    pub execution_id: &'a str,
    pub workspace_id: &'a str,
    pub kind: ExecutionKind,
    pub request_source: FactSource,
    pub execution_domain: &'a str,
}

pub struct NewTerminalSession<'a> {
    pub execution_id: &'a str,
    pub profile_id: &'a str,
    pub shell_executable: &'a str,
    pub shell_arguments: &'a [String],
    pub requested_cwd: &'a str,
    pub initial_cols: Option<u16>,
    pub initial_rows: Option<u16>,
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
        Ok(Self {
            connection,
            home: home.to_path_buf(),
        })
    }

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
        let shell_arguments_json = serde_json::to_string(session.shell_arguments)?;
        self.connection.execute(
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
        Ok(())
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

        Ok(TerminalSessionRecord {
            execution_id: row.0,
            profile_id: row.1,
            shell_executable: row.2,
            shell_arguments: serde_json::from_str(&row.3)?,
            requested_cwd: row.4,
            initial_cols: row.5.map(u16::try_from).transpose()?,
            initial_rows: row.6.map(u16::try_from).transpose()?,
            close_reason: row.7,
        })
    }

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

    pub fn save_evidence(&mut self, report: &EvidenceReport, now_ms: i64) -> Result<()> {
        let json = serde_json::to_string(report)?;
        let tx = self.connection.transaction()?;
        tx.execute(
            "UPDATE candidate_runs SET state = 'VERIFIED' WHERE run_id = ?1",
            params![report.run_id],
        )?;
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

#[cfg(test)]
mod persistence_tests {
    use super::{NewExecution, NewTerminalSession, NewWorkspace, Store};
    use crate::domain::{ExecutionKind, ExecutionStatus, FactSource};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

    fn test_home(name: &str) -> PathBuf {
        let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
        let home = std::env::temp_dir().join(format!(
            "winds-store-{name}-{}-{sequence}",
            std::process::id()
        ));
        if home.exists() {
            fs::remove_dir_all(&home).unwrap();
        }
        home
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
        assert_eq!(workspace.last_opened_unix_ms, 100);

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
        fs::remove_dir_all(home).unwrap();
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
        fs::remove_dir_all(home).unwrap();
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
