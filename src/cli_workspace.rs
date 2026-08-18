use crate::command::history::SessionHistoryPolicy;
use crate::command::{ExplicitCommandRequest, run_explicit_command_with_history_policy};
use crate::domain::{ExecutionKind, ExecutionStatus, FactSource};
use crate::execution::TerminalExecution;
use crate::git::Repo;
use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
use crate::git::terminal::TerminalSize;
use crate::git::workspace::open_existing_workspace;
use crate::git::workspace_clone::clone_and_register_workspace;
use crate::git::workspace_inventory::inventory_workspace_environment;
#[cfg(windows)]
use crate::git::wsl::discover_wsl_distributions;
use crate::store::Store;
use crate::{
    Result, ensure_allowed_flags, required, resolve_without_creation, unix_ms, utf8_path,
    winds_home,
};
use rusqlite::{Connection, ErrorCode, OpenFlags, OptionalExtension, TransactionBehavior, params};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub(crate) fn dispatch(command: &str, flags: HashMap<String, String>) -> Result<()> {
    match command {
        "workspace-open" => workspace_open(flags),
        "workspace-clone" => workspace_clone(flags),
        "profiles" => profiles(flags),
        "run" => run_command(flags),
        "terminal-proof" => terminal_proof(flags),
        "execution" => execution(flags),
        _ => Err("unknown Spec 003 T057 CLI command".into()),
    }
}

fn workspace_open(flags: HashMap<String, String>) -> Result<()> {
    ensure_allowed_flags(&flags, &["repo", "home"])?;
    let repo_arg = required(&flags, "repo")?;
    let repo = Repo::open(Path::new(repo_arg))?;
    let home = winds_home(flags.get("home").map(String::as_str), &repo)?;
    let workspace = open_existing_workspace(Path::new(repo_arg), &home, unix_ms()?)?;
    let _store = open_reconciled_cli_store(&home)?;
    print_json(&workspace)
}

fn workspace_clone(flags: HashMap<String, String>) -> Result<()> {
    ensure_allowed_flags(&flags, &["remote", "destination", "home"])?;
    let remote = required(&flags, "remote")?;
    let destination = Path::new(required(&flags, "destination")?);
    let home = standalone_winds_home_for_clone(
        flags.get("home").map(String::as_str),
        destination,
        remote,
    )?;
    let workspace = clone_and_register_workspace(remote, destination, &home, unix_ms()?)?;
    let _store = open_reconciled_cli_store(&home)?;
    print_json(&workspace)
}

fn profiles(flags: HashMap<String, String>) -> Result<()> {
    ensure_allowed_flags(&flags, &["repo", "home"])?;
    let repo_arg = required(&flags, "repo")?;
    let repo = Repo::open(Path::new(repo_arg))?;
    let home = winds_home(flags.get("home").map(String::as_str), &repo)?;
    let workspace = open_existing_workspace(Path::new(repo_arg), &home, unix_ms()?)?;
    let _store = open_reconciled_cli_store(&home)?;
    let inventory = inventory_workspace_environment(&workspace)?;
    let native_shell_profiles = discover_native_shell_profiles(&inventory)?;
    let wsl = wsl_inventory();

    print_json(&json!({
        "workspace": workspace,
        "inventory": inventory,
        "native_shell_profiles": native_shell_profiles,
        "wsl": wsl,
    }))
}

fn run_command(flags: HashMap<String, String>) -> Result<()> {
    ensure_allowed_flags(
        &flags,
        &[
            "repo",
            "execution-id",
            "executable",
            "args-json",
            "history",
            "home",
        ],
    )?;
    let repo_arg = required(&flags, "repo")?;
    let execution_id = required(&flags, "execution-id")?;
    let executable = Path::new(required(&flags, "executable")?);
    let arguments = parse_arguments(flags.get("args-json").map(String::as_str))?;
    let history_policy = parse_history_policy(flags.get("history").map(String::as_str))?;

    let repo = Repo::open(Path::new(repo_arg))?;
    let home = winds_home(flags.get("home").map(String::as_str), &repo)?;
    let workspace = open_existing_workspace(Path::new(repo_arg), &home, unix_ms()?)?;
    let cwd = Path::new(&workspace.canonical_worktree_root);
    let mut store = open_reconciled_cli_store(&home)?;
    let _lease = acquire_execution_lease(&home, execution_id)?;
    let result = run_explicit_command_with_history_policy(
        &mut store,
        ExplicitCommandRequest {
            execution_id,
            workspace_id: &workspace.workspace_id,
            executable,
            arguments: &arguments,
            cwd,
        },
        history_policy,
    )?;
    let snapshot = execution_snapshot(&store, execution_id)?;

    print_json(&json!({
        "result": {
            "exit_code": result.exit_code,
            "duration_ms": result.duration_ms,
        },
        "execution": snapshot,
    }))
}

fn terminal_proof(flags: HashMap<String, String>) -> Result<()> {
    ensure_allowed_flags(
        &flags,
        &["repo", "execution-id", "profile-id", "rows", "cols", "home"],
    )?;
    let repo_arg = required(&flags, "repo")?;
    let execution_id = required(&flags, "execution-id")?;
    let profile_id = required(&flags, "profile-id")?;
    let size = parse_terminal_size(
        flags.get("rows").map(String::as_str),
        flags.get("cols").map(String::as_str),
    )?;

    let repo = Repo::open(Path::new(repo_arg))?;
    let home = winds_home(flags.get("home").map(String::as_str), &repo)?;
    let workspace = open_existing_workspace(Path::new(repo_arg), &home, unix_ms()?)?;
    let inventory = inventory_workspace_environment(&workspace)?;
    let native_shell_profiles = discover_native_shell_profiles(&inventory)?;
    let profile = select_profile(&native_shell_profiles, profile_id)?;
    let cwd = Path::new(&workspace.canonical_worktree_root);
    let mut store = open_reconciled_cli_store(&home)?;
    let _lease = acquire_execution_lease(&home, execution_id)?;

    let exit = {
        let mut execution = TerminalExecution::start_native(
            &mut store,
            execution_id,
            &workspace.workspace_id,
            profile,
            cwd,
            size,
        )?;
        execution.terminate()?
    };
    let snapshot = execution_snapshot(&store, execution_id)?;

    print_json(&json!({
        "proof": {
            "profile_id": profile.profile_id,
            "rows": size.rows,
            "cols": size.cols,
            "exit_code": exit.exit_code,
            "signal": exit.signal,
        },
        "execution": snapshot,
    }))
}

fn execution(flags: HashMap<String, String>) -> Result<()> {
    ensure_allowed_flags(&flags, &["repo", "execution-id", "home"])?;
    let repo_arg = required(&flags, "repo")?;
    let execution_id = required(&flags, "execution-id")?;
    let repo = Repo::open(Path::new(repo_arg))?;
    let home = winds_home(flags.get("home").map(String::as_str), &repo)?;
    let mut store = open_reconciled_cli_store(&home)?;
    require_execution_repo(&store, execution_id, &repo)?;
    print_json(&execution_snapshot_with_ownership_truth(
        &home,
        &mut store,
        execution_id,
    )?)
}

fn open_reconciled_cli_store(home: &Path) -> Result<Store> {
    let mut store = Store::open(home)?;
    reconcile_unowned_cli_executions(home, &mut store, ExecutionKind::Terminal)?;
    reconcile_unowned_cli_executions(home, &mut store, ExecutionKind::ShellCommand)?;
    Ok(store)
}

fn reconcile_unowned_cli_executions(
    home: &Path,
    store: &mut Store,
    kind: ExecutionKind,
) -> Result<()> {
    let execution_ids = nonfinal_execution_ids(home, kind)?;
    for execution_id in execution_ids {
        match probe_execution_lease(home, &execution_id)? {
            LeaseProbe::Active => {}
            LeaseProbe::Acquired(lease) => {
                let result =
                    reconcile_unowned_execution_row(home, store, &execution_id, kind, unix_ms()?);
                drop(lease);
                result?;
            }
        }
    }
    Ok(())
}

fn execution_snapshot_with_ownership_truth(
    home: &Path,
    store: &mut Store,
    execution_id: &str,
) -> Result<Value> {
    loop {
        // Build the complete point-in-time view before proving ownership. The value is
        // intentionally discarded because an active-owner proof requires a fresh read.
        execution_snapshot(store, execution_id)?;
        let execution = store.load_execution(execution_id)?;
        if !matches!(
            execution.status,
            ExecutionStatus::Requested | ExecutionStatus::Running
        ) {
            return execution_snapshot(store, execution_id);
        }

        match probe_execution_lease(home, execution_id)? {
            LeaseProbe::Active => return execution_snapshot(store, execution_id),
            LeaseProbe::Acquired(lease) => {
                let result = reconcile_unowned_execution_row(
                    home,
                    store,
                    execution_id,
                    execution.kind,
                    unix_ms()?,
                );
                drop(lease);
                result?;
            }
        }
    }
}

fn reconcile_unowned_execution_row(
    home: &Path,
    store: &mut Store,
    execution_id: &str,
    kind: ExecutionKind,
    now_ms: i64,
) -> Result<()> {
    if kind == ExecutionKind::ShellCommand {
        let execution = store.load_execution(execution_id)?;
        if execution.status == ExecutionStatus::Running {
            let command = store.load_shell_command(execution_id)?;
            if command.exit_source == Some(FactSource::WindsObserved) {
                store.finalize_shell_command_from_observation(execution_id)?;
                return Ok(());
            }
        }
    }

    let database = home.join("winds.db");
    let mut connection = Connection::open_with_flags(
        database,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )?;
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let row = tx
        .query_row(
            "SELECT status, requested_unix_ms
             FROM executions
             WHERE execution_id = ?1 AND kind = ?2",
            params![execution_id, kind.as_str()],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        )
        .optional()?
        .ok_or_else(|| {
            format!("unknown Winds execution during restart reconciliation: {execution_id}")
        })?;
    let status = ExecutionStatus::from_db(&row.0).ok_or_else(|| {
        format!(
            "unknown execution status during restart reconciliation for {execution_id}: {}",
            row.0
        )
    })?;
    if !matches!(
        status,
        ExecutionStatus::Requested | ExecutionStatus::Running
    ) {
        tx.commit()?;
        return Ok(());
    }
    if now_ms < row.1 {
        return Err(format!(
            "restart reconciliation time precedes execution request time: {execution_id}"
        )
        .into());
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
        return Err(format!(
            "targeted restart reconciliation lost its non-final execution row: {execution_id}"
        )
        .into());
    }
    if kind == ExecutionKind::Terminal {
        tx.execute(
            "UPDATE terminal_sessions
             SET close_reason = 'OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN'
             WHERE execution_id = ?1",
            params![execution_id],
        )?;
    }
    let event_kind = match kind {
        ExecutionKind::Terminal => "TerminalOwnershipLostAfterRestart",
        ExecutionKind::ShellCommand => "ShellCommandOwnershipLostAfterRestart",
    };
    tx.execute(
        "INSERT INTO execution_events(execution_id, kind, fact_source, created_unix_ms)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            execution_id,
            event_kind,
            FactSource::WindsObserved.as_str(),
            now_ms,
        ],
    )?;
    tx.commit()?;
    Ok(())
}

fn nonfinal_execution_ids(home: &Path, kind: ExecutionKind) -> Result<Vec<String>> {
    let database = home.join("winds.db");
    let connection = Connection::open_with_flags(
        database,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )?;
    let mut statement = connection.prepare(
        "SELECT execution_id
         FROM executions
         WHERE kind = ?1 AND status IN (?2, ?3)
         ORDER BY requested_unix_ms, execution_id",
    )?;
    let execution_ids = statement
        .query_map(
            params![
                kind.as_str(),
                ExecutionStatus::Requested.as_str(),
                ExecutionStatus::Running.as_str(),
            ],
            |row| row.get::<_, String>(0),
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(execution_ids)
}

fn acquire_execution_lease(home: &Path, execution_id: &str) -> Result<ExecutionLease> {
    match probe_execution_lease(home, execution_id)? {
        LeaseProbe::Acquired(lease) => Ok(lease),
        LeaseProbe::Active => Err(format!(
            "execution ownership lease is already held by another live Winds process: {execution_id}"
        )
        .into()),
    }
}

enum LeaseProbe {
    Acquired(ExecutionLease),
    Active,
}

struct ExecutionLease {
    connection: Option<Connection>,
}

impl Drop for ExecutionLease {
    fn drop(&mut self) {
        if let Some(connection) = self.connection.take() {
            let _ = connection.execute_batch("ROLLBACK");
        }
    }
}

fn probe_execution_lease(home: &Path, execution_id: &str) -> Result<LeaseProbe> {
    if execution_id.is_empty() || execution_id.chars().any(char::is_control) {
        return Err("execution ownership lease requires a non-empty control-free identity".into());
    }
    let path = home.join(execution_lease_filename(execution_id));
    let connection = Connection::open_with_flags(
        &path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )?;
    connection.busy_timeout(Duration::from_millis(0))?;
    match connection.execute_batch("BEGIN IMMEDIATE") {
        Ok(()) => Ok(LeaseProbe::Acquired(ExecutionLease {
            connection: Some(connection),
        })),
        Err(error) if sqlite_busy_or_locked(&error) => Ok(LeaseProbe::Active),
        Err(error) => Err(format!(
            "execution ownership lease could not be checked for {execution_id}: {error}"
        )
        .into()),
    }
}

fn execution_lease_filename(execution_id: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(b"WindsExecutionOwnershipLeaseV1\0");
    digest.update(execution_id.as_bytes());
    let hex: String = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("execution-ownership-{hex}.sqlite3")
}

fn sqlite_busy_or_locked(error: &rusqlite::Error) -> bool {
    matches!(
        error.sqlite_error_code(),
        Some(ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked)
    )
}

fn require_execution_repo(store: &Store, execution_id: &str, repo: &Repo) -> Result<()> {
    let execution = store.load_execution(execution_id)?;
    let workspace = store.load_workspace(&execution.workspace_id)?;
    let repo_root = utf8_path(repo.root(), "repository path")?;
    if workspace.canonical_worktree_root != repo_root {
        return Err(format!(
            "execution {execution_id} belongs to a different Winds workspace than --repo"
        )
        .into());
    }
    Ok(())
}

fn execution_snapshot(store: &Store, execution_id: &str) -> Result<Value> {
    let execution = store.load_execution(execution_id)?;
    let execution_domain: Value =
        serde_json::from_str(&execution.execution_domain).map_err(|error| {
            format!("persisted execution domain is not valid JSON for {execution_id}: {error}")
        })?;
    let events = store
        .execution_events(execution_id)?
        .into_iter()
        .map(|event| {
            json!({
                "event_id": event.event_id,
                "execution_id": event.execution_id,
                "kind": event.kind,
                "source": event.source,
                "created_unix_ms": event.created_unix_ms,
            })
        })
        .collect::<Vec<_>>();

    let (terminal, shell_command) = match execution.kind {
        ExecutionKind::Terminal => {
            let session = store.load_terminal_session(execution_id)?;
            (
                Some(json!({
                    "execution_id": session.execution_id,
                    "profile_id": session.profile_id,
                    "shell_executable": session.shell_executable,
                    "shell_arguments": session.shell_arguments,
                    "requested_cwd": session.requested_cwd,
                    "initial_cols": session.initial_cols,
                    "initial_rows": session.initial_rows,
                    "close_reason": session.close_reason,
                })),
                None,
            )
        }
        ExecutionKind::ShellCommand => {
            let command = store.load_shell_command(execution_id)?;
            (
                None,
                Some(json!({
                    "execution_id": command.execution_id,
                    "executable": command.executable,
                    "arguments": command.arguments,
                    "command_source": command.command_source,
                    "requested_cwd": command.requested_cwd,
                    "cwd_source": command.cwd_source,
                    "exit_code": command.exit_code,
                    "exit_source": command.exit_source,
                    "observed_end_unix_ms": command.observed_end_unix_ms,
                })),
            )
        }
    };

    Ok(json!({
        "execution_id": execution.execution_id,
        "workspace_id": execution.workspace_id,
        "kind": execution.kind,
        "request_source": execution.request_source,
        "execution_domain": execution_domain,
        "status": execution.status,
        "status_source": execution.status_source,
        "requested_unix_ms": execution.requested_unix_ms,
        "started_unix_ms": execution.started_unix_ms,
        "ended_unix_ms": execution.ended_unix_ms,
        "duration_ms": execution.duration_ms,
        "terminal": terminal,
        "shell_command": shell_command,
        "events": events,
    }))
}

fn select_profile<'a>(profiles: &'a [ShellProfile], profile_id: &str) -> Result<&'a ShellProfile> {
    profiles
        .iter()
        .find(|profile| profile.profile_id == profile_id)
        .ok_or_else(|| format!("unknown or unavailable native shell profile: {profile_id}").into())
}

fn parse_arguments(value: Option<&str>) -> Result<Vec<String>> {
    value
        .map(serde_json::from_str::<Vec<String>>)
        .transpose()
        .map_err(|error| format!("--args-json must be a JSON array of strings: {error}").into())
        .map(|arguments| arguments.unwrap_or_default())
}

fn parse_history_policy(value: Option<&str>) -> Result<SessionHistoryPolicy> {
    match value.unwrap_or("command") {
        "command" => Ok(SessionHistoryPolicy::command_history_only()),
        "disabled" => Ok(SessionHistoryPolicy::disabled()),
        other => Err(format!(
            "unsupported --history value {other:?}; expected command or disabled"
        )
        .into()),
    }
}

fn parse_terminal_size(rows: Option<&str>, cols: Option<&str>) -> Result<TerminalSize> {
    let rows = rows.map(str::parse::<u16>).transpose()?.unwrap_or(24);
    let cols = cols.map(str::parse::<u16>).transpose()?.unwrap_or(80);
    if rows == 0 || cols == 0 {
        return Err("--rows and --cols must both be greater than zero".into());
    }
    Ok(TerminalSize { rows, cols })
}

fn standalone_winds_home_for_clone(
    explicit: Option<&str>,
    destination: &Path,
    remote: &str,
) -> Result<PathBuf> {
    if !destination.is_absolute() {
        return Err("clone destination must be an absolute path".into());
    }
    if destination.exists() {
        return Err(format!(
            "clone destination already exists: {}",
            destination.display()
        )
        .into());
    }
    let planned_destination = resolve_without_creation(destination)?;

    let path = if let Some(path) = explicit {
        PathBuf::from(path)
    } else if let Some(path) = env::var_os("WINDS_HOME") {
        PathBuf::from(path)
    } else {
        let home = env::var_os("HOME").ok_or("HOME is not set; pass --home or WINDS_HOME")?;
        PathBuf::from(home).join(".winds")
    };
    let absolute = if path.is_absolute() {
        path
    } else {
        env::current_dir()?.join(path)
    };
    let resolved = resolve_without_creation(&absolute)?;
    if planned_destination.starts_with(&resolved) || resolved.starts_with(&planned_destination) {
        return Err("clone destination and Winds state root must not overlap".into());
    }
    require_clone_state_external_to_local_remote(remote, &resolved)?;

    fs::create_dir_all(&resolved)?;
    let canonical = resolved.canonicalize()?;
    utf8_path(&canonical, "WINDS_HOME")?;
    Ok(canonical)
}

fn require_clone_state_external_to_local_remote(remote: &str, state_root: &Path) -> Result<()> {
    if let Some((scheme, _)) = remote.split_once("://") {
        if scheme.eq_ignore_ascii_case("file") {
            return Err(
                "T057 workspace-clone requires an absolute local path instead of file:// so Winds can prove the clone-source/state-root boundary"
                    .into(),
            );
        }
        return Ok(());
    }

    let remote_path = Path::new(remote);
    if !remote_path.is_absolute() {
        return Ok(());
    }
    let canonical_remote = remote_path
        .canonicalize()
        .map_err(|error| format!("local clone remote cannot be canonicalized: {error}"))?;
    if state_root.starts_with(&canonical_remote) {
        return Err("Winds state root must live outside the local clone source".into());
    }
    if let Ok(repo) = Repo::open(&canonical_remote) {
        repo.require_external_state_path(state_root)?;
    }
    Ok(())
}

#[cfg(windows)]
fn wsl_inventory() -> Value {
    match discover_wsl_distributions() {
        Ok(distributions) => json!({
            "availability": "AVAILABLE",
            "distributions": distributions,
        }),
        Err(error) => json!({
            "availability": "UNAVAILABLE",
            "reason": error.to_string(),
            "distributions": [],
        }),
    }
}

#[cfg(not(windows))]
fn wsl_inventory() -> Value {
    json!({
        "availability": "UNSUPPORTED_ON_HOST",
        "distributions": [],
    })
}

fn print_json(value: &impl Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        execution_lease_filename, parse_arguments, parse_history_policy, parse_terminal_size,
    };
    use crate::command::history::SessionHistoryPolicy;

    #[test]
    fn arguments_default_to_empty_and_parse_string_arrays() {
        assert_eq!(parse_arguments(None).unwrap(), Vec::<String>::new());
        assert_eq!(
            parse_arguments(Some(r#"["--flag","value"]"#)).unwrap(),
            vec!["--flag".to_owned(), "value".to_owned()]
        );
        assert!(parse_arguments(Some(r#"["ok",42]"#)).is_err());
    }

    #[test]
    fn command_history_is_default_and_can_be_disabled() {
        assert_eq!(
            parse_history_policy(None).unwrap(),
            SessionHistoryPolicy::command_history_only()
        );
        assert_eq!(
            parse_history_policy(Some("disabled")).unwrap(),
            SessionHistoryPolicy::disabled()
        );
        assert!(parse_history_policy(Some("transcript")).is_err());
    }

    #[test]
    fn terminal_size_defaults_and_rejects_zero() {
        let size = parse_terminal_size(None, None).unwrap();
        assert_eq!(size.rows, 24);
        assert_eq!(size.cols, 80);
        assert!(parse_terminal_size(Some("0"), None).is_err());
        assert!(parse_terminal_size(None, Some("0")).is_err());
    }

    #[test]
    fn ownership_lease_filename_is_deterministic_and_path_safe() {
        let first = execution_lease_filename("workspace/execution:1");
        let second = execution_lease_filename("workspace/execution:1");
        assert_eq!(first, second);
        assert!(first.starts_with("execution-ownership-"));
        assert!(first.ends_with(".sqlite3"));
        assert!(!first.contains('/'));
        assert!(!first.contains(':'));
    }
}
