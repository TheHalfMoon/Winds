from pathlib import Path
import sys


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one marker, found {count}")
    return text.replace(old, new, 1)


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: t055_patch.py <repo-root>")
    root = Path(sys.argv[1]).resolve()

    migration = root / "migrations/0005_shell_command_git_observations.sql"
    if migration.exists():
        raise SystemExit("T055 migration already exists")
    migration.write_text(
        """CREATE TABLE IF NOT EXISTS shell_command_git_observations (\n"
        "    execution_id TEXT PRIMARY KEY REFERENCES shell_commands(execution_id),\n"
        "    before_source TEXT NOT NULL,\n"
        "    before_head_known INTEGER NOT NULL CHECK (before_head_known IN (0, 1)),\n"
        "    before_head_oid TEXT,\n"
        "    before_dirty INTEGER CHECK (before_dirty IS NULL OR before_dirty IN (0, 1)),\n"
        "    after_source TEXT,\n"
        "    after_head_known INTEGER NOT NULL DEFAULT 0 CHECK (after_head_known IN (0, 1)),\n"
        "    after_head_oid TEXT,\n"
        "    after_dirty INTEGER CHECK (after_dirty IS NULL OR after_dirty IN (0, 1)),\n"
        "    CHECK (before_head_known = 1 OR before_head_oid IS NULL),\n"
        "    CHECK (after_head_known = 1 OR after_head_oid IS NULL),\n"
        "    CHECK (\n"
        "        after_source IS NOT NULL\n"
        "        OR (after_head_known = 0 AND after_head_oid IS NULL AND after_dirty IS NULL)\n"
        "    )\n"
        ");\n",
        encoding="utf-8",
    )

    domain = root / "src/domain.rs"
    text = domain.read_text(encoding="utf-8")
    anchor = """#[allow(\n    dead_code,\n    reason = \"Spec 003 T044 persistence substrate; runtime callers land in later slices\"\n)]\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct ExecutionRecord {\n"""
    insertion = """#[allow(\n    dead_code,\n    reason = \"Spec 003 T055 backend API; CLI/timeline caller lands in T057\"\n)]\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct LightweightGitObservation {\n    pub head_known: bool,\n    pub head_oid: Option<String>,\n    pub dirty: Option<bool>,\n}\n\n#[allow(\n    dead_code,\n    reason = \"Spec 003 T055 backend API; CLI/timeline caller lands in T057\"\n)]\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct ShellCommandGitObservationsRecord {\n    pub execution_id: String,\n    pub before_source: FactSource,\n    pub before: LightweightGitObservation,\n    pub after_source: Option<FactSource>,\n    pub after: LightweightGitObservation,\n}\n\n""" + anchor
    text = replace_once(text, anchor, insertion, "domain Git observation records")
    domain.write_text(text, encoding="utf-8")

    workspace = root / "src/workspace.rs"
    text = workspace.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "use crate::store::{NewWorkspace, Store};\n",
        "use crate::domain::{LightweightGitObservation, WorkspaceRecord};\nuse crate::store::{NewWorkspace, Store};\n",
        "workspace observation imports",
    )
    anchor = """pub(super) fn inspect_existing_workspace(\n    path: &Path,\n    canonical_state_root: &Path,\n) -> Result<WorkspaceInspection> {\n    let repo = open_worktree(path)?;\n    let observation = inspect_worktree(&repo)?;\n    require_canonical_external_state_root(&repo, canonical_state_root)?;\n    Ok(observation)\n}\n\n"""
    addition = anchor + """pub(crate) fn observe_registered_workspace_git_state(\n    workspace: &WorkspaceRecord,\n) -> LightweightGitObservation {\n    let unknown = || LightweightGitObservation {\n        head_known: false,\n        head_oid: None,\n        dirty: None,\n    };\n    let Ok(repo) = Repo::open(Path::new(&workspace.canonical_worktree_root)) else {\n        return unknown();\n    };\n    if repo.root().to_str() != Some(workspace.canonical_worktree_root.as_str())\n        || repo.common_dir.to_str() != Some(workspace.git_common_dir.as_str())\n    {\n        return unknown();\n    }\n\n    let (head_known, head_oid) = match branch_name(&repo)\n        .and_then(|branch| exact_head(&repo, branch.as_deref()))\n    {\n        Ok(head_oid) => (true, head_oid),\n        Err(_) => (false, None),\n    };\n    let dirty = read_only_status(&repo).ok().map(|status| !status.is_empty());\n    LightweightGitObservation {\n        head_known,\n        head_oid,\n        dirty,\n    }\n}\n\n"""
    text = replace_once(text, anchor, addition, "registered workspace lightweight Git observation")
    workspace.write_text(text, encoding="utf-8")

    store = root / "src/store.rs"
    text = store.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "    ExecutionRecord, ExecutionStatus, FactSource, ShellCommandRecord, StoredRun,\n    TerminalCloseReason, TerminalSessionRecord, WorkspaceRecord,\n",
        "    ExecutionRecord, ExecutionStatus, FactSource, LightweightGitObservation,\n    ShellCommandGitObservationsRecord, ShellCommandRecord, StoredRun, TerminalCloseReason,\n    TerminalSessionRecord, WorkspaceRecord,\n",
        "store Git observation imports",
    )
    text = replace_once(
        text,
        "        connection.execute_batch(include_str!(\"../migrations/0004_shell_commands.sql\"))?;\n",
        "        connection.execute_batch(include_str!(\"../migrations/0004_shell_commands.sql\"))?;\n        connection.execute_batch(include_str!(\n            \"../migrations/0005_shell_command_git_observations.sql\"\n        ))?;\n",
        "store T055 migration",
    )
    old = """    pub fn create_shell_command_execution(\n        &mut self,\n        execution: NewExecution<'_>,\n        command: NewShellCommand<'_>,\n        now_ms: i64,\n    ) -> Result<()> {\n"""
    new = """    pub fn create_shell_command_execution(\n        &mut self,\n        execution: NewExecution<'_>,\n        command: NewShellCommand<'_>,\n        before_git: &LightweightGitObservation,\n        now_ms: i64,\n    ) -> Result<()> {\n"""
    text = replace_once(text, old, new, "create shell command observation parameter")
    text = replace_once(
        text,
        "        let arguments_json = serde_json::to_string(command.arguments)?;\n        let tx = self.connection.transaction()?;\n",
        "        validate_git_observation(before_git)?;\n        let arguments_json = serde_json::to_string(command.arguments)?;\n        let tx = self.connection.transaction()?;\n",
        "validate before Git observation",
    )
    anchor = """        tx.execute(\n            \"INSERT INTO shell_commands(\n                execution_id, executable, arguments_json, command_source,\n                requested_cwd, cwd_source, exit_code, exit_source\n             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL)\",\n            params![\n                command.execution_id,\n                command.executable,\n                arguments_json,\n                command.command_source.as_str(),\n                command.requested_cwd,\n                command.cwd_source.as_str(),\n            ],\n        )?;\n        tx.commit()?;\n"""
    replacement = """        tx.execute(\n            \"INSERT INTO shell_commands(\n                execution_id, executable, arguments_json, command_source,\n                requested_cwd, cwd_source, exit_code, exit_source\n             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL)\",\n            params![\n                command.execution_id,\n                command.executable,\n                arguments_json,\n                command.command_source.as_str(),\n                command.requested_cwd,\n                command.cwd_source.as_str(),\n            ],\n        )?;\n        tx.execute(\n            \"INSERT INTO shell_command_git_observations(\n                execution_id, before_source, before_head_known, before_head_oid, before_dirty\n             ) VALUES (?1, ?2, ?3, ?4, ?5)\",\n            params![\n                command.execution_id,\n                FactSource::WindsObserved.as_str(),\n                i64::from(before_git.head_known),\n                before_git.head_oid.as_deref(),\n                before_git.dirty.map(i64::from),\n            ],\n        )?;\n        tx.commit()?;\n"""
    text = replace_once(text, anchor, replacement, "atomic before Git observation insert")

    anchor = """    pub fn mark_shell_command_running(\n        &mut self,\n        execution_id: &str,\n        started_unix_ms: Option<i64>,\n    ) -> Result<()> {\n"""
    methods = """    pub fn record_shell_command_git_after(\n        &mut self,\n        execution_id: &str,\n        after_git: &LightweightGitObservation,\n    ) -> Result<()> {\n        validate_git_observation(after_git)?;\n        let updated = self.connection.execute(\n            \"UPDATE shell_command_git_observations\n             SET after_source = ?2, after_head_known = ?3, after_head_oid = ?4,\n                 after_dirty = ?5\n             WHERE execution_id = ?1 AND after_source IS NULL\",\n            params![\n                execution_id,\n                FactSource::WindsObserved.as_str(),\n                i64::from(after_git.head_known),\n                after_git.head_oid.as_deref(),\n                after_git.dirty.map(i64::from),\n            ],\n        )?;\n        if updated != 1 {\n            return Err(\n                \"shell-command after-Git observation was already recorded or lost its typed row\"\n                    .into(),\n            );\n        }\n        Ok(())\n    }\n\n""" + anchor
    text = replace_once(text, anchor, methods, "after Git observation writer")

    anchor = """    pub fn load_shell_command(&self, execution_id: &str) -> Result<ShellCommandRecord> {\n"""
    loader = """    pub fn load_shell_command_git_observations(\n        &self,\n        execution_id: &str,\n    ) -> Result<ShellCommandGitObservationsRecord> {\n        let row = self\n            .connection\n            .query_row(\n                \"SELECT execution_id, before_source, before_head_known, before_head_oid,\n                        before_dirty, after_source, after_head_known, after_head_oid, after_dirty\n                 FROM shell_command_git_observations WHERE execution_id = ?1\",\n                params![execution_id],\n                |row| {\n                    Ok((\n                        row.get::<_, String>(0)?,\n                        row.get::<_, String>(1)?,\n                        row.get::<_, i64>(2)?,\n                        row.get::<_, Option<String>>(3)?,\n                        row.get::<_, Option<i64>>(4)?,\n                        row.get::<_, Option<String>>(5)?,\n                        row.get::<_, i64>(6)?,\n                        row.get::<_, Option<String>>(7)?,\n                        row.get::<_, Option<i64>>(8)?,\n                    ))\n                },\n            )\n            .optional()?\n            .ok_or_else(|| format!(\"unknown Winds shell-command Git observations: {execution_id}\"))?;\n        let before_source = FactSource::from_db(&row.1)\n            .ok_or_else(|| format!(\"unknown before-Git observation source in store: {}\", row.1))?;\n        let after_source = row\n            .5\n            .as_deref()\n            .map(|value| {\n                FactSource::from_db(value)\n                    .ok_or_else(|| format!(\"unknown after-Git observation source in store: {value}\"))\n            })\n            .transpose()?;\n        Ok(ShellCommandGitObservationsRecord {\n            execution_id: row.0,\n            before_source,\n            before: LightweightGitObservation {\n                head_known: sqlite_bool(row.2, \"before_head_known\")?,\n                head_oid: row.3,\n                dirty: sqlite_optional_bool(row.4, \"before_dirty\")?,\n            },\n            after_source,\n            after: LightweightGitObservation {\n                head_known: sqlite_bool(row.6, \"after_head_known\")?,\n                head_oid: row.7,\n                dirty: sqlite_optional_bool(row.8, \"after_dirty\")?,\n            },\n        })\n    }\n\n""" + anchor
    text = replace_once(text, anchor, loader, "Git observation loader")

    anchor = """fn terminal_execution_state(\n    connection: &Connection,\n    execution_id: &str,\n) -> Result<(ExecutionStatus, i64, Option<i64>)> {\n"""
    helpers = """fn validate_git_observation(observation: &LightweightGitObservation) -> Result<()> {\n    if !observation.head_known && observation.head_oid.is_some() {\n        return Err(\"unknown Git HEAD observation cannot contain an object id\".into());\n    }\n    if observation.head_oid.as_deref().is_some_and(str::is_empty) {\n        return Err(\"observed Git HEAD object id cannot be empty\".into());\n    }\n    Ok(())\n}\n\nfn sqlite_bool(value: i64, field: &str) -> Result<bool> {\n    match value {\n        0 => Ok(false),\n        1 => Ok(true),\n        _ => Err(format!(\"invalid SQLite boolean in {field}: {value}\").into()),\n    }\n}\n\nfn sqlite_optional_bool(value: Option<i64>, field: &str) -> Result<Option<bool>> {\n    value.map(|value| sqlite_bool(value, field)).transpose()\n}\n\n""" + anchor
    text = replace_once(text, anchor, helpers, "Git observation validation/parsing helpers")
    store.write_text(text, encoding="utf-8")

    command = root / "src/command.rs"
    text = command.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "use crate::git::shell_profiles::ShellExecutionDomain;\n",
        "use crate::git::shell_profiles::ShellExecutionDomain;\nuse crate::git::workspace::observe_registered_workspace_git_state;\n",
        "command Git observer import",
    )
    text = replace_once(
        text,
        "use crate::domain::{ExecutionKind, ExecutionStatus, FactSource};\n",
        "use crate::domain::{ExecutionKind, ExecutionStatus, FactSource, WorkspaceRecord};\n",
        "command workspace record import",
    )
    old = """    let cwd = validate_workspace_cwd(store, request.workspace_id, request.cwd)?;\n    let execution_domain = serde_json::to_string(&ShellExecutionDomain::NativeHost {\n"""
    new = """    let workspace = store.load_workspace(request.workspace_id)?;\n    let cwd = validate_workspace_cwd(&workspace, request.cwd)?;\n    let before_git = observe_registered_workspace_git_state(&workspace);\n    let execution_domain = serde_json::to_string(&ShellExecutionDomain::NativeHost {\n"""
    text = replace_once(text, old, new, "capture registered workspace before Git observation")
    text = replace_once(
        text,
        "        },\n        requested_unix_ms,\n    )?;\n\n    let mut child = match Command::new(&executable)\n",
        "        },\n        &before_git,\n        requested_unix_ms,\n    )?;\n\n    let mut child = match Command::new(&executable)\n",
        "persist before Git observation with command request",
    )
    old = """        let repair = if cleanup_proven {\n            store.mark_shell_command_start_persistence_failed(\n                request.execution_id,\n                started_unix_ms,\n                ended_unix_ms,\n            )\n        } else {\n"""
    new = """        if cleanup_proven {\n            record_final_git_observation(store, request.execution_id, &workspace);\n        }\n        let repair = if cleanup_proven {\n            store.mark_shell_command_start_persistence_failed(\n                request.execution_id,\n                started_unix_ms,\n                ended_unix_ms,\n            )\n        } else {\n"""
    text = replace_once(text, old, new, "after Git observation for proven start-persistence cleanup")
    old = """            let persist = if cleanup_proven {\n                store.mark_shell_command_interrupted(request.execution_id, ended_unix_ms)\n            } else {\n"""
    new = """            if cleanup_proven {\n                record_final_git_observation(store, request.execution_id, &workspace);\n            }\n            let persist = if cleanup_proven {\n                store.mark_shell_command_interrupted(request.execution_id, ended_unix_ms)\n            } else {\n"""
    text = replace_once(text, old, new, "after Git observation for proven wait-error cleanup")
    text = replace_once(
        text,
        "    let ended_unix_ms = trustworthy_wall_time_after(requested_unix_ms, started_unix_ms);\n    let exit_code = status.code();\n",
        "    record_final_git_observation(store, request.execution_id, &workspace);\n    let ended_unix_ms = trustworthy_wall_time_after(requested_unix_ms, started_unix_ms);\n    let exit_code = status.code();\n",
        "after Git observation for natural exit",
    )
    old = """fn validate_workspace_cwd(store: &Store, workspace_id: &str, cwd: &Path) -> Result<String> {\n    if !cwd.is_absolute() {\n        return Err(\"explicit command cwd must be an absolute path\".into());\n    }\n    let canonical_cwd = fs::canonicalize(cwd)?;\n    if !canonical_cwd.is_dir() {\n        return Err(\"explicit command cwd must be a directory\".into());\n    }\n    let workspace = store.load_workspace(workspace_id)?;\n    let workspace_root = PathBuf::from(&workspace.canonical_worktree_root);\n"""
    new = """fn validate_workspace_cwd(workspace: &WorkspaceRecord, cwd: &Path) -> Result<String> {\n    if !cwd.is_absolute() {\n        return Err(\"explicit command cwd must be an absolute path\".into());\n    }\n    let canonical_cwd = fs::canonicalize(cwd)?;\n    if !canonical_cwd.is_dir() {\n        return Err(\"explicit command cwd must be a directory\".into());\n    }\n    let workspace_root = PathBuf::from(&workspace.canonical_worktree_root);\n"""
    text = replace_once(text, old, new, "reuse loaded workspace during cwd validation")
    anchor = """fn cleanup_owned_child(child: &mut Child) -> bool {\n"""
    helper = """fn record_final_git_observation(\n    store: &mut Store,\n    execution_id: &str,\n    workspace: &WorkspaceRecord,\n) {\n    let observation = observe_registered_workspace_git_state(workspace);\n    // T055 observations are best-effort workspace history. A persistence failure must not\n    // block a directly proven child finalization or turn workspace history into authority.\n    let _ = store.record_shell_command_git_after(execution_id, &observation);\n}\n\n""" + anchor
    text = replace_once(text, anchor, helper, "best-effort final Git observation helper")

    text = replace_once(
        text,
        "    use crate::domain::{ExecutionKind, ExecutionStatus, FactSource};\n",
        "    use crate::domain::{ExecutionKind, ExecutionStatus, FactSource};\n",
        "command test domain import guard",
    )
    # Existing tests use non-Git workspaces. Make the missing-observation contract explicit there.
    marker = """        assert!(command.observed_end_unix_ms.is_some());\n    }\n\n    #[test]\n    fn parser_free_explicit_run_does_not_upgrade_marker_like_output_authority() {\n"""
    replacement = """        assert!(command.observed_end_unix_ms.is_some());\n        let git = store.load_shell_command_git_observations(\"command-1\").unwrap();\n        assert_eq!(git.before_source, FactSource::WindsObserved);\n        assert!(!git.before.head_known);\n        assert_eq!(git.before.head_oid, None);\n        assert_eq!(git.before.dirty, None);\n        assert_eq!(git.after_source, Some(FactSource::WindsObserved));\n        assert!(!git.after.head_known);\n        assert_eq!(git.after.head_oid, None);\n        assert_eq!(git.after.dirty, None);\n    }\n\n    #[test]\n    fn parser_free_explicit_run_does_not_upgrade_marker_like_output_authority() {\n"""
    text = replace_once(text, marker, replacement, "explicit unknown Git observation assertions")

    marker = """        let execution = store.load_execution(\"command-failed\").unwrap();\n"""
    replacement = """        let git = store\n            .load_shell_command_git_observations(\"command-failed\")\n            .unwrap();\n        assert_eq!(git.before_source, FactSource::WindsObserved);\n        assert_eq!(git.after_source, None);\n        let execution = store.load_execution(\"command-failed\").unwrap();\n"""
    text = replace_once(text, marker, replacement, "failed-start has no after boundary")

    # Add one real Git integration fixture proving both HEAD and dirty-state snapshots, and no candidate evidence.
    anchor = """    #[test]\n    fn cwd_outside_registered_workspace_fails_before_persistence() {\n"""
    integration = r'''    fn run_git(cwd: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_COMMON_DIR")
            .arg("-c")
            .arg("core.hooksPath=hooks-disabled")
            .arg("-C")
            .arg(cwd)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }

    fn git_store_with_workspace(root: &TestRoot) -> (Store, PathBuf, String) {
        let home = root.path().join("state-git");
        let workspace_root = root.path().join("workspace-git");
        fs::create_dir(&workspace_root).unwrap();
        run_git(&workspace_root, &["init", "--initial-branch=main"]);
        run_git(&workspace_root, &["config", "user.name", "Winds Test"]);
        run_git(
            &workspace_root,
            &["config", "user.email", "winds@example.invalid"],
        );
        fs::write(workspace_root.join("tracked.txt"), b"before\n").unwrap();
        run_git(&workspace_root, &["add", "--", "tracked.txt"]);
        run_git(
            &workspace_root,
            &["commit", "--no-gpg-sign", "-m", "initial"],
        );
        let canonical_workspace = fs::canonicalize(&workspace_root).unwrap();
        let git_common_dir = fs::canonicalize(workspace_root.join(".git")).unwrap();
        let initial_head = run_git(&workspace_root, &["rev-parse", "HEAD"]);
        let store = Store::open(&home).unwrap();
        store
            .create_workspace(
                NewWorkspace {
                    workspace_id: "workspace-git",
                    canonical_worktree_root: canonical_workspace.to_str().unwrap(),
                    git_common_dir: git_common_dir.to_str().unwrap(),
                },
                1,
            )
            .unwrap();
        (store, canonical_workspace, initial_head)
    }

    #[cfg(unix)]
    fn dirty_command_parts() -> (PathBuf, Vec<String>) {
        (
            PathBuf::from("/bin/sh"),
            vec![
                "-c".to_owned(),
                "printf 'after\\n' > tracked.txt".to_owned(),
            ],
        )
    }

    #[cfg(windows)]
    fn dirty_command_parts() -> (PathBuf, Vec<String>) {
        (
            PathBuf::from(std::env::var_os("COMSPEC").expect("COMSPEC on Windows CI")),
            vec![
                "/D".to_owned(),
                "/C".to_owned(),
                "echo after>tracked.txt".to_owned(),
            ],
        )
    }

    #[test]
    fn explicit_command_records_lightweight_before_after_git_state_without_candidate_evidence() {
        let root = TestRoot::new("git-observation");
        let (mut store, cwd, initial_head) = git_store_with_workspace(&root);
        let (executable, arguments) = dirty_command_parts();
        run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "command-git",
                workspace_id: "workspace-git",
                executable: &executable,
                arguments: &arguments,
                cwd: &cwd,
            },
        )
        .unwrap();

        let git = store.load_shell_command_git_observations("command-git").unwrap();
        assert_eq!(git.before_source, FactSource::WindsObserved);
        assert!(git.before.head_known);
        assert_eq!(git.before.head_oid.as_deref(), Some(initial_head.as_str()));
        assert_eq!(git.before.dirty, Some(false));
        assert_eq!(git.after_source, Some(FactSource::WindsObserved));
        assert!(git.after.head_known);
        assert_eq!(git.after.head_oid.as_deref(), Some(initial_head.as_str()));
        assert_eq!(git.after.dirty, Some(true));
        assert!(store.load_run("command-git").is_err());
        assert!(store.runs_for_repo(cwd.to_str().unwrap()).unwrap().is_empty());
    }

''' + anchor
    text = replace_once(text, anchor, integration, "real Git command observation integration test")
    text = replace_once(
        text,
        "    use std::path::{Path, PathBuf};\n",
        "    use std::path::{Path, PathBuf};\n    use std::process::Command;\n",
        "command test Git process import",
    )
    command.write_text(text, encoding="utf-8")


if __name__ == "__main__":
    main()
