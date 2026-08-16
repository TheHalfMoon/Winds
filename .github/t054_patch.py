from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one marker, found {count}")
    return text.replace(old, new, 1)


def patch_domain(root: Path) -> None:
    path = root / "src/domain.rs"
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "pub enum ExecutionKind {\n    Terminal,\n}",
        "pub enum ExecutionKind {\n    Terminal,\n    ShellCommand,\n}",
        "ExecutionKind variants",
    )
    text = replace_once(
        text,
        "        match self {\n            Self::Terminal => \"TERMINAL\",\n        }",
        "        match self {\n            Self::Terminal => \"TERMINAL\",\n            Self::ShellCommand => \"SHELL_COMMAND\",\n        }",
        "ExecutionKind as_str",
    )
    text = replace_once(
        text,
        "        match value {\n            \"TERMINAL\" => Some(Self::Terminal),\n            _ => None,\n        }",
        "        match value {\n            \"TERMINAL\" => Some(Self::Terminal),\n            \"SHELL_COMMAND\" => Some(Self::ShellCommand),\n            _ => None,\n        }",
        "ExecutionKind from_db",
    )
    marker = "#[derive(Debug, Clone, Serialize)]\npub struct BlobEvidence {"
    record = '''#[allow(\n    dead_code,\n    reason = "Spec 003 T054 command-record backend API; CLI/timeline caller lands in T057"\n)]\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct ShellCommandRecord {\n    pub execution_id: String,\n    pub executable: String,\n    pub arguments: Vec<String>,\n    pub command_source: FactSource,\n    pub requested_cwd: String,\n    pub cwd_source: FactSource,\n    pub exit_code: Option<i32>,\n    pub exit_source: Option<FactSource>,\n}\n\n'''
    text = replace_once(text, marker, record + marker, "ShellCommandRecord insertion")
    path.write_text(text, encoding="utf-8")


def patch_store(root: Path) -> None:
    path = root / "src/store.rs"
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "    ExecutionRecord, ExecutionStatus, FactSource, StoredRun, TerminalCloseReason,\n    TerminalSessionRecord, WorkspaceRecord,\n",
        "    ExecutionRecord, ExecutionStatus, FactSource, ShellCommandRecord, StoredRun,\n    TerminalCloseReason, TerminalSessionRecord, WorkspaceRecord,\n",
        "store domain imports",
    )
    text = replace_once(
        text,
        "    deferred_terminal_finalizations: Vec<DeferredTerminalFinalization>,\n}",
        "    deferred_terminal_finalizations: Vec<DeferredTerminalFinalization>,\n    deferred_shell_command_finalizations: Vec<DeferredShellCommandFinalization>,\n}",
        "Store deferred field",
    )
    marker = "#[derive(Debug, Clone)]\nstruct DeferredTerminalFinalization {\n    execution_id: String,\n    finalization: TerminalFinalization,\n}\n"
    extra = marker + '''\n#[derive(Debug, Clone, Copy)]\npub(crate) struct ShellCommandFinalization {\n    pub exit_code: Option<i32>,\n    pub ended_unix_ms: i64,\n}\n\n#[derive(Debug, Clone)]\nstruct DeferredShellCommandFinalization {\n    execution_id: String,\n    finalization: ShellCommandFinalization,\n}\n'''
    text = replace_once(text, marker, extra, "command finalization structs")

    terminal_new = '''pub struct NewTerminalSession<'a> {\n    pub execution_id: &'a str,\n    pub profile_id: &'a str,\n    pub shell_executable: &'a str,\n    pub shell_arguments: &'a [String],\n    pub requested_cwd: &'a str,\n    pub initial_cols: Option<u16>,\n    pub initial_rows: Option<u16>,\n}\n'''
    command_new = terminal_new + '''\n#[allow(\n    dead_code,\n    reason = "Spec 003 T054 command-record backend API; CLI/timeline caller lands in T057"\n)]\npub struct NewShellCommand<'a> {\n    pub execution_id: &'a str,\n    pub executable: &'a str,\n    pub arguments: &'a [String],\n    pub command_source: FactSource,\n    pub requested_cwd: &'a str,\n    pub cwd_source: FactSource,\n}\n'''
    text = replace_once(text, terminal_new, command_new, "NewShellCommand insertion")

    text = replace_once(
        text,
        '''        connection.execute_batch(include_str!(\n            "../migrations/0003_workspace_clone_origins.sql"\n        ))?;\n        Ok(Self {\n            connection,\n            home: home.to_path_buf(),\n            deferred_terminal_finalizations: Vec::new(),\n        })''',
        '''        connection.execute_batch(include_str!(\n            "../migrations/0003_workspace_clone_origins.sql"\n        ))?;\n        connection.execute_batch(include_str!("../migrations/0004_shell_commands.sql"))?;\n        Ok(Self {\n            connection,\n            home: home.to_path_buf(),\n            deferred_terminal_finalizations: Vec::new(),\n            deferred_shell_command_finalizations: Vec::new(),\n        })''',
        "Store open migration",
    )

    methods_marker = "    pub fn mark_terminal_running(&mut self, execution_id: &str, now_ms: i64) -> Result<()> {"
    command_methods = r'''    pub fn create_shell_command_execution(
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

    pub fn mark_shell_command_running(&mut self, execution_id: &str, now_ms: i64) -> Result<()> {
        let tx = self.connection.transaction()?;
        let (status, requested_unix_ms, started_unix_ms) =
            shell_command_execution_state(&tx, execution_id)?;
        if status != ExecutionStatus::Requested || started_unix_ms.is_some() {
            return Err(format!(
                "shell command cannot start from persisted state {}: {execution_id}",
                status.as_str()
            )
            .into());
        }
        if now_ms < requested_unix_ms {
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
                now_ms,
                ExecutionStatus::Requested.as_str(),
            ],
        )?;
        if updated != 1 {
            return Err("shell-command RUNNING transition lost its expected REQUESTED row".into());
        }
        insert_execution_event(
            &tx,
            execution_id,
            "ShellCommandStarted",
            FactSource::WindsObserved,
            now_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn mark_shell_command_failed_to_start(
        &mut self,
        execution_id: &str,
        now_ms: i64,
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
        if now_ms < requested_unix_ms {
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
                now_ms,
                ExecutionStatus::Requested.as_str(),
            ],
        )?;
        if updated != 1 {
            return Err("shell-command FAILED_TO_START transition lost its expected row".into());
        }
        insert_execution_event(
            &tx,
            execution_id,
            "ShellCommandFailedToStart",
            FactSource::WindsObserved,
            now_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn mark_shell_command_start_persistence_failed(
        &mut self,
        execution_id: &str,
        started_unix_ms: i64,
        ended_unix_ms: i64,
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
        if started_unix_ms < requested_unix_ms || ended_unix_ms < started_unix_ms {
            return Err("shell-command start-persistence recovery timestamps are inconsistent".into());
        }
        let duration_ms = ended_unix_ms - started_unix_ms;
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
        insert_execution_event(
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
        ended_unix_ms: i64,
    ) -> Result<()> {
        let tx = self.connection.transaction()?;
        let (status, _requested_unix_ms, started_unix_ms) =
            shell_command_execution_state(&tx, execution_id)?;
        let started_unix_ms = started_unix_ms.ok_or("running shell command has no start time")?;
        if status != ExecutionStatus::Running {
            return Err(format!(
                "shell command cannot be interrupted from persisted state {}: {execution_id}",
                status.as_str()
            )
            .into());
        }
        if ended_unix_ms < started_unix_ms {
            return Err("shell-command end time cannot precede its start time".into());
        }
        let duration_ms = ended_unix_ms - started_unix_ms;
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
        insert_execution_event(
            &tx,
            execution_id,
            "ShellCommandInterrupted",
            FactSource::WindsObserved,
            ended_unix_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn mark_shell_command_exited(
        &mut self,
        execution_id: &str,
        exit_code: Option<i32>,
        ended_unix_ms: i64,
    ) -> Result<()> {
        let tx = self.connection.transaction()?;
        let (status, _requested_unix_ms, started_unix_ms) =
            shell_command_execution_state(&tx, execution_id)?;
        let started_unix_ms = started_unix_ms.ok_or("running shell command has no start time")?;
        if status != ExecutionStatus::Running {
            return Err(format!(
                "shell command cannot exit from persisted state {}: {execution_id}",
                status.as_str()
            )
            .into());
        }
        if ended_unix_ms < started_unix_ms {
            return Err("shell-command end time cannot precede its start time".into());
        }
        let duration_ms = ended_unix_ms - started_unix_ms;
        let updated = tx.execute(
            "UPDATE executions
             SET status = ?2, status_source = ?3, ended_unix_ms = ?4, duration_ms = ?5
             WHERE execution_id = ?1 AND status = ?6",
            params![
                execution_id,
                ExecutionStatus::Exited.as_str(),
                FactSource::WindsObserved.as_str(),
                ended_unix_ms,
                duration_ms,
                ExecutionStatus::Running.as_str(),
            ],
        )?;
        if updated != 1 {
            return Err("shell-command EXITED transition lost its RUNNING row".into());
        }
        let updated_command = tx.execute(
            "UPDATE shell_commands SET exit_code = ?2, exit_source = ?3 WHERE execution_id = ?1",
            params![
                execution_id,
                exit_code.map(i64::from),
                FactSource::WindsObserved.as_str(),
            ],
        )?;
        if updated_command != 1 {
            return Err("shell-command exit update lost its typed record".into());
        }
        insert_execution_event(
            &tx,
            execution_id,
            "ShellCommandExited",
            FactSource::WindsObserved,
            ended_unix_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn apply_shell_command_finalization(
        &mut self,
        execution_id: &str,
        finalization: ShellCommandFinalization,
    ) -> Result<()> {
        self.mark_shell_command_exited(
            execution_id,
            finalization.exit_code,
            finalization.ended_unix_ms,
        )
    }

    pub(crate) fn defer_shell_command_finalization(
        &mut self,
        execution_id: &str,
        finalization: ShellCommandFinalization,
    ) {
        if let Some(existing) = self
            .deferred_shell_command_finalizations
            .iter_mut()
            .find(|pending| pending.execution_id == execution_id)
        {
            existing.finalization = finalization;
            return;
        }
        self.deferred_shell_command_finalizations
            .push(DeferredShellCommandFinalization {
                execution_id: execution_id.to_owned(),
                finalization,
            });
    }

    pub fn retry_deferred_shell_command_finalizations(&mut self) -> Result<usize> {
        let pending = std::mem::take(&mut self.deferred_shell_command_finalizations);
        let mut completed = 0_usize;
        let mut failed = Vec::new();
        let mut failures = Vec::new();
        for item in pending {
            match self.apply_shell_command_finalization(&item.execution_id, item.finalization) {
                Ok(()) => completed += 1,
                Err(error) => {
                    failures.push(format!("{}: {error}", item.execution_id));
                    failed.push(item);
                }
            }
        }
        self.deferred_shell_command_finalizations = failed;
        if failures.is_empty() {
            Ok(completed)
        } else {
            Err(format!(
                "{} deferred shell-command finalization(s) remain pending: {}",
                failures.len(),
                failures.join("; ")
            )
            .into())
        }
    }

    pub fn pending_shell_command_finalization_count(&self) -> usize {
        self.deferred_shell_command_finalizations.len()
    }

    pub fn reconcile_unowned_shell_commands_after_restart(&mut self, now_ms: i64) -> Result<usize> {
        self.retry_deferred_shell_command_finalizations()?;
        let tx = self.connection.transaction()?;
        let execution_ids = {
            let mut statement = tx.prepare(
                "SELECT e.execution_id
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
                    |row| row.get::<_, String>(0),
                )?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        for execution_id in &execution_ids {
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
            insert_execution_event(
                &tx,
                execution_id,
                "ShellCommandOwnershipLostAfterRestart",
                FactSource::WindsObserved,
                now_ms,
            )?;
        }
        tx.commit()?;
        Ok(execution_ids.len())
    }

'''
    text = replace_once(text, methods_marker, command_methods + methods_marker, "command methods insertion")

    load_marker = "    pub fn load_terminal_session(&self, execution_id: &str) -> Result<TerminalSessionRecord> {"
    load_methods = r'''    pub fn load_shell_command(&self, execution_id: &str) -> Result<ShellCommandRecord> {
        let row = self
            .connection
            .query_row(
                "SELECT execution_id, executable, arguments_json, command_source,
                        requested_cwd, cwd_source, exit_code, exit_source
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

'''
    text = replace_once(text, load_marker, load_methods + load_marker, "load shell command insertion")

    helper_marker = "#[allow(\n    dead_code,\n    reason = \"Spec 003 T044 persistence substrate; runtime callers land in later slices\"\n)]\nfn insert_execution_event("
    helper = r'''fn shell_command_execution_state(
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

'''
    text = replace_once(text, helper_marker, helper + helper_marker, "shell command state helper")
    path.write_text(text, encoding="utf-8")


def patch_main(root: Path) -> None:
    path = root / "src/main.rs"
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "mod check;\nmod domain;\n",
        "mod check;\n#[allow(\n    dead_code,\n    reason = \"Spec 003 T054 explicit-command backend API; CLI caller lands in T057\"\n)]\nmod command;\nmod domain;\n",
        "command module declaration",
    )
    path.write_text(text, encoding="utf-8")


def patch_windows_workflow(root: Path) -> None:
    path = root / ".github/workflows/windows-terminal.yml"
    text = path.read_text(encoding="utf-8")
    text = text.replace('      - "src/check.rs"\n', '      - "src/check.rs"\n      - "src/command.rs"\n', 2)
    if text.count('      - "src/command.rs"\n') != 2:
        raise SystemExit("windows workflow command path insertion failed")
    text = text.replace('      - "src/wsl_launch.rs"\n', '      - "src/wsl_launch.rs"\n      - "migrations/0004_shell_commands.sql"\n', 2)
    if text.count('      - "migrations/0004_shell_commands.sql"\n') != 2:
        raise SystemExit("windows workflow migration path insertion failed")
    test_marker = "      - name: Terminal ledger persistence tests\n        run: cargo test --locked --bin winds store::persistence_tests -- --test-threads=1\n"
    test_replacement = test_marker + "      - name: Explicit command observability tests\n        run: cargo test --locked --bin winds command::tests -- --test-threads=1\n"
    text = replace_once(text, test_marker, test_replacement, "windows command test step")
    path.write_text(text, encoding="utf-8")


def write_new_files(root: Path) -> None:
    migration = root / "migrations/0004_shell_commands.sql"
    if migration.exists():
        raise SystemExit("0004_shell_commands.sql already exists")
    migration.write_text(
        '''CREATE TABLE IF NOT EXISTS shell_commands (\n    execution_id TEXT PRIMARY KEY REFERENCES executions(execution_id),\n    executable TEXT NOT NULL,\n    arguments_json TEXT NOT NULL,\n    command_source TEXT NOT NULL,\n    requested_cwd TEXT NOT NULL,\n    cwd_source TEXT NOT NULL,\n    exit_code INTEGER,\n    exit_source TEXT,\n    CHECK (exit_code IS NULL OR exit_source IS NOT NULL)\n);\n\nCREATE INDEX IF NOT EXISTS idx_shell_commands_executable\n    ON shell_commands(executable, execution_id);\n''',
        encoding="utf-8",
    )

    command = root / "src/command.rs"
    if command.exists():
        raise SystemExit("src/command.rs already exists")
    command.write_text(r'''use crate::domain::{ExecutionKind, ExecutionStatus, FactSource};
use crate::git::shell_profiles::ShellExecutionDomain;
use crate::store::{
    NewExecution, NewShellCommand, Result, ShellCommandFinalization, Store,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ExplicitCommandRequest<'a> {
    pub execution_id: &'a str,
    pub workspace_id: &'a str,
    pub executable: &'a Path,
    pub arguments: &'a [String],
    pub cwd: &'a Path,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplicitCommandResult {
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
}

pub fn run_explicit_command(
    store: &mut Store,
    request: ExplicitCommandRequest<'_>,
) -> Result<ExplicitCommandResult> {
    store.retry_deferred_shell_command_finalizations()?;
    if request.execution_id.is_empty() || request.workspace_id.is_empty() {
        return Err("explicit command requires non-empty execution/workspace identity".into());
    }
    let executable = validate_executable(request.executable)?;
    if request.arguments.iter().any(|argument| argument.contains('\0')) {
        return Err("explicit command arguments may not contain NUL bytes".into());
    }
    let cwd = validate_workspace_cwd(store, request.workspace_id, request.cwd)?;
    let execution_domain = serde_json::to_string(&ShellExecutionDomain::NativeHost {
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
    })?;
    let requested_unix_ms = unix_ms()?;
    store.create_shell_command_execution(
        NewExecution {
            execution_id: request.execution_id,
            workspace_id: request.workspace_id,
            kind: ExecutionKind::ShellCommand,
            request_source: FactSource::CallerRequested,
            execution_domain: &execution_domain,
        },
        NewShellCommand {
            execution_id: request.execution_id,
            executable: &executable,
            arguments: request.arguments,
            command_source: FactSource::CallerRequested,
            requested_cwd: &cwd,
            cwd_source: FactSource::CallerRequested,
        },
        requested_unix_ms,
    )?;

    let mut child = match Command::new(&executable)
        .args(request.arguments)
        .current_dir(&cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            let failed_unix_ms = unix_ms()?;
            let persist = store.mark_shell_command_failed_to_start(request.execution_id, failed_unix_ms);
            return match persist {
                Ok(()) => Err(format!("explicit command failed to start: {error}").into()),
                Err(persist_error) => Err(format!(
                    "explicit command failed to start: {error}; FAILED_TO_START persistence also failed: {persist_error}"
                )
                .into()),
            };
        }
    };

    let started_unix_ms = match unix_ms() {
        Ok(value) => value,
        Err(error) => {
            let cleanup_proven = cleanup_owned_child(&mut child);
            return Err(format!(
                "explicit command child started but start time could not be recorded: {error}; owned-child cleanup {}",
                if cleanup_proven { "succeeded" } else { "failed" }
            )
            .into());
        }
    };

    if let Err(persist_error) = store.mark_shell_command_running(request.execution_id, started_unix_ms) {
        let cleanup_proven = cleanup_owned_child(&mut child);
        let ended_unix_ms = unix_ms().unwrap_or(started_unix_ms);
        let repair = if cleanup_proven {
            store.mark_shell_command_start_persistence_failed(
                request.execution_id,
                started_unix_ms,
                ended_unix_ms,
            )
        } else {
            Ok(())
        };
        let repair_note = match repair {
            Ok(()) if cleanup_proven => "interrupted cleanup state persisted".to_owned(),
            Ok(()) => "cleanup was not proven; request remains non-final for restart reconciliation".to_owned(),
            Err(error) => format!("cleanup-state persistence also failed: {error}"),
        };
        return Err(format!(
            "explicit command child started but RUNNING persistence failed: {persist_error}; owned-child cleanup {}; {repair_note}",
            if cleanup_proven { "succeeded" } else { "failed" }
        )
        .into());
    }

    let status = match child.wait() {
        Ok(status) => status,
        Err(wait_error) => {
            let cleanup_proven = cleanup_owned_child(&mut child);
            let ended_unix_ms = unix_ms().unwrap_or(started_unix_ms);
            let persist = if cleanup_proven {
                store.mark_shell_command_interrupted(request.execution_id, ended_unix_ms)
            } else {
                Ok(())
            };
            return Err(format!(
                "failed waiting for explicit command: {wait_error}; owned-child cleanup {}; ledger repair {}",
                if cleanup_proven { "succeeded" } else { "failed" },
                if persist.is_ok() { "succeeded" } else { "failed" }
            )
            .into());
        }
    };
    let ended_unix_ms = unix_ms()?;
    let exit_code = status.code();
    let finalization = ShellCommandFinalization {
        exit_code,
        ended_unix_ms,
    };
    if let Err(error) = store.apply_shell_command_finalization(request.execution_id, finalization) {
        store.defer_shell_command_finalization(request.execution_id, finalization);
        return Err(format!(
            "explicit command exited but final ledger persistence failed and remains retryable: {error}"
        )
        .into());
    }
    let execution = store.load_execution(request.execution_id)?;
    if execution.status != ExecutionStatus::Exited {
        return Err("explicit command finalization did not persist EXITED state".into());
    }
    Ok(ExplicitCommandResult {
        exit_code,
        duration_ms: execution.duration_ms.unwrap_or(0),
    })
}

fn validate_executable(path: &Path) -> Result<String> {
    if !path.is_absolute() {
        return Err("explicit command executable must be an absolute path".into());
    }
    let value = path
        .to_str()
        .ok_or("explicit command executable path is not valid UTF-8")?;
    if value.contains('\0') {
        return Err("explicit command executable may not contain NUL bytes".into());
    }
    Ok(value.to_owned())
}

fn validate_workspace_cwd(store: &Store, workspace_id: &str, cwd: &Path) -> Result<String> {
    if !cwd.is_absolute() {
        return Err("explicit command cwd must be an absolute path".into());
    }
    let canonical_cwd = fs::canonicalize(cwd)?;
    if !canonical_cwd.is_dir() {
        return Err("explicit command cwd must be a directory".into());
    }
    let workspace = store.load_workspace(workspace_id)?;
    let workspace_root = PathBuf::from(&workspace.canonical_worktree_root);
    if !canonical_cwd.starts_with(&workspace_root) {
        return Err("explicit command cwd must remain inside the registered workspace".into());
    }
    canonical_cwd
        .to_str()
        .map(str::to_owned)
        .ok_or_else(|| "explicit command cwd is not valid UTF-8".into())
}

fn cleanup_owned_child(child: &mut Child) -> bool {
    match child.try_wait() {
        Ok(Some(_)) => true,
        Ok(None) => {
            let _ = child.kill();
            child.wait().is_ok()
        }
        Err(_) => {
            let _ = child.kill();
            child.wait().is_ok()
        }
    }
}

fn unix_ms() -> Result<i64> {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    Ok(i64::try_from(millis)?)
}

#[cfg(test)]
mod tests {
    use super::{ExplicitCommandRequest, run_explicit_command};
    use crate::domain::{ExecutionKind, ExecutionStatus, FactSource};
    use crate::store::{NewExecution, NewShellCommand, NewWorkspace, Store};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new(name: &str) -> Self {
            let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "winds-t054-{name}-{}-{sequence}",
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

    fn store_with_workspace(root: &TestRoot) -> Store {
        let home = root.path().join("state");
        let workspace_root = root.path().join("workspace");
        fs::create_dir(&workspace_root).unwrap();
        let canonical_workspace = fs::canonicalize(&workspace_root).unwrap();
        let mut store = Store::open(&home).unwrap();
        store
            .create_workspace(
                NewWorkspace {
                    workspace_id: "workspace-1",
                    canonical_worktree_root: canonical_workspace.to_str().unwrap(),
                    git_common_dir: canonical_workspace.join(".git").to_str().unwrap(),
                },
                1,
            )
            .unwrap();
        store
    }

    fn workspace_path(root: &TestRoot) -> PathBuf {
        fs::canonicalize(root.path().join("workspace")).unwrap()
    }

    #[cfg(unix)]
    fn command_parts(exit_code: i32, marker: bool) -> (PathBuf, Vec<String>) {
        let script = if marker {
            format!("printf '__WINDS_COMMAND_END_spoof__\\n'; exit {exit_code}")
        } else {
            format!("exit {exit_code}")
        };
        (PathBuf::from("/bin/sh"), vec!["-c".to_owned(), script])
    }

    #[cfg(windows)]
    fn command_parts(exit_code: i32, marker: bool) -> (PathBuf, Vec<String>) {
        let executable = PathBuf::from(std::env::var_os("COMSPEC").expect("COMSPEC on Windows CI"));
        let body = if marker {
            format!("echo __WINDS_COMMAND_END_spoof__ & exit /B {exit_code}")
        } else {
            format!("exit /B {exit_code}")
        };
        (
            executable,
            vec!["/D".to_owned(), "/C".to_owned(), body],
        )
    }

    #[test]
    fn explicit_command_records_structured_intent_and_observed_exit() {
        let root = TestRoot::new("success");
        let mut store = store_with_workspace(&root);
        let cwd = workspace_path(&root);
        let (executable, arguments) = command_parts(7, false);
        let result = run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "command-1",
                workspace_id: "workspace-1",
                executable: &executable,
                arguments: &arguments,
                cwd: &cwd,
            },
        )
        .unwrap();
        assert_eq!(result.exit_code, Some(7));
        let execution = store.load_execution("command-1").unwrap();
        assert_eq!(execution.kind, ExecutionKind::ShellCommand);
        assert_eq!(execution.request_source, FactSource::CallerRequested);
        assert_eq!(execution.status, ExecutionStatus::Exited);
        assert_eq!(execution.status_source, FactSource::WindsObserved);
        assert!(execution.started_unix_ms.is_some());
        assert!(execution.ended_unix_ms.is_some());
        assert!(execution.duration_ms.is_some());
        let command = store.load_shell_command("command-1").unwrap();
        assert_eq!(command.executable, executable.to_str().unwrap());
        assert_eq!(command.arguments, arguments);
        assert_eq!(command.command_source, FactSource::CallerRequested);
        assert_eq!(command.requested_cwd, cwd.to_str().unwrap());
        assert_eq!(command.cwd_source, FactSource::CallerRequested);
        assert_eq!(command.exit_code, Some(7));
        assert_eq!(command.exit_source, Some(FactSource::WindsObserved));
    }

    #[test]
    fn marker_like_child_output_cannot_create_shell_reported_telemetry() {
        let root = TestRoot::new("spoof");
        let mut store = store_with_workspace(&root);
        let cwd = workspace_path(&root);
        let (executable, arguments) = command_parts(0, true);
        run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "command-spoof",
                workspace_id: "workspace-1",
                executable: &executable,
                arguments: &arguments,
                cwd: &cwd,
            },
        )
        .unwrap();
        assert_eq!(store.shell_command_count_for_workspace("workspace-1").unwrap(), 1);
        let command = store.load_shell_command("command-spoof").unwrap();
        assert_eq!(command.command_source, FactSource::CallerRequested);
        assert_eq!(command.exit_source, Some(FactSource::WindsObserved));
        let events = store.execution_events("command-spoof").unwrap();
        assert!(events.iter().all(|event| event.source != FactSource::ShellReported));
    }

    #[test]
    fn spawn_failure_is_explicit_and_never_claims_a_start_or_duration() {
        let root = TestRoot::new("failed-start");
        let mut store = store_with_workspace(&root);
        let cwd = workspace_path(&root);
        let executable = cwd.join("missing-executable");
        let arguments = Vec::new();
        let result = run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "command-failed",
                workspace_id: "workspace-1",
                executable: &executable,
                arguments: &arguments,
                cwd: &cwd,
            },
        );
        assert!(result.is_err());
        let execution = store.load_execution("command-failed").unwrap();
        assert_eq!(execution.status, ExecutionStatus::FailedToStart);
        assert_eq!(execution.started_unix_ms, None);
        assert!(execution.ended_unix_ms.is_some());
        assert_eq!(execution.duration_ms, None);
        let command = store.load_shell_command("command-failed").unwrap();
        assert_eq!(command.exit_code, None);
        assert_eq!(command.exit_source, None);
    }

    #[test]
    fn cwd_outside_registered_workspace_fails_before_persistence() {
        let root = TestRoot::new("outside-cwd");
        let mut store = store_with_workspace(&root);
        let outside = root.path().join("outside");
        fs::create_dir(&outside).unwrap();
        let outside = fs::canonicalize(outside).unwrap();
        let (executable, arguments) = command_parts(0, false);
        let result = run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "command-outside",
                workspace_id: "workspace-1",
                executable: &executable,
                arguments: &arguments,
                cwd: &outside,
            },
        );
        assert!(result.is_err());
        assert!(store.load_execution("command-outside").is_err());
        assert_eq!(store.shell_command_count_for_workspace("workspace-1").unwrap(), 0);
    }

    #[test]
    fn restart_reconciliation_marks_nonfinal_command_ownership_unknown() {
        let root = TestRoot::new("restart");
        let mut store = store_with_workspace(&root);
        let cwd = workspace_path(&root);
        let (executable, arguments) = command_parts(0, false);
        store
            .create_shell_command_execution(
                NewExecution {
                    execution_id: "command-restart",
                    workspace_id: "workspace-1",
                    kind: ExecutionKind::ShellCommand,
                    request_source: FactSource::CallerRequested,
                    execution_domain: "native-test",
                },
                NewShellCommand {
                    execution_id: "command-restart",
                    executable: executable.to_str().unwrap(),
                    arguments: &arguments,
                    command_source: FactSource::CallerRequested,
                    requested_cwd: cwd.to_str().unwrap(),
                    cwd_source: FactSource::CallerRequested,
                },
                10,
            )
            .unwrap();
        store.mark_shell_command_running("command-restart", 11).unwrap();
        let reconciled = store.reconcile_unowned_shell_commands_after_restart(20).unwrap();
        assert_eq!(reconciled, 1);
        let execution = store.load_execution("command-restart").unwrap();
        assert_eq!(execution.status, ExecutionStatus::OwnershipLost);
        assert_eq!(execution.ended_unix_ms, None);
        assert_eq!(execution.duration_ms, None);
        let command = store.load_shell_command("command-restart").unwrap();
        assert_eq!(command.exit_code, None);
        assert_eq!(command.exit_source, None);
    }
}
''', encoding="utf-8")


def main() -> None:
    if len(__import__("sys").argv) != 2:
        raise SystemExit("usage: t054_patch.py <repo-root>")
    root = Path(__import__("sys").argv[1]).resolve()
    patch_domain(root)
    patch_store(root)
    patch_main(root)
    patch_windows_workflow(root)
    write_new_files(root)


if __name__ == "__main__":
    main()
