use crate::command::history::SessionHistoryPolicy;
use crate::command::{ExplicitCommandRequest, run_explicit_command_with_history_policy};
use crate::domain::ExecutionKind;
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
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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
    print_json(&workspace)
}

fn workspace_clone(flags: HashMap<String, String>) -> Result<()> {
    ensure_allowed_flags(&flags, &["remote", "destination", "home"])?;
    let remote = required(&flags, "remote")?;
    let destination = Path::new(required(&flags, "destination")?);
    let home = standalone_winds_home_for_clone(flags.get("home").map(String::as_str), destination)?;
    let workspace = clone_and_register_workspace(remote, destination, &home, unix_ms()?)?;
    print_json(&workspace)
}

fn profiles(flags: HashMap<String, String>) -> Result<()> {
    ensure_allowed_flags(&flags, &["repo", "home"])?;
    let repo_arg = required(&flags, "repo")?;
    let repo = Repo::open(Path::new(repo_arg))?;
    let home = winds_home(flags.get("home").map(String::as_str), &repo)?;
    let workspace = open_existing_workspace(Path::new(repo_arg), &home, unix_ms()?)?;
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
    let mut store = Store::open(&home)?;
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
    let mut store = Store::open(&home)?;

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
    let store = Store::open(&home)?;
    require_execution_repo(&store, execution_id, &repo)?;
    print_json(&execution_snapshot(&store, execution_id)?)
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

fn standalone_winds_home_for_clone(explicit: Option<&str>, destination: &Path) -> Result<PathBuf> {
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
    fs::create_dir_all(&resolved)?;
    let canonical = resolved.canonicalize()?;
    utf8_path(&canonical, "WINDS_HOME")?;
    Ok(canonical)
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
    use super::{parse_arguments, parse_history_policy, parse_terminal_size};
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
}
