#[cfg(test)]
mod agentic_authority;
#[allow(
    dead_code,
    reason = "Spec 006 T078 fake Claude structured CLI; real Claude and prompts remain blocked until T080"
)]
mod agentic_claude;
#[allow(
    dead_code,
    reason = "Spec 006 T077 fake Codex protocol client; real Codex and prompts remain blocked until T079"
)]
mod agentic_codex;
#[cfg(test)]
mod agentic_context;
#[allow(
    dead_code,
    reason = "Spec 006 T084 deterministic findability seam; no production CLI behavior is added by the focused proof"
)]
mod agentic_find;
#[allow(
    dead_code,
    reason = "Spec 006 T072 fixture-only runtime discovery; real Agent work remains blocked"
)]
mod agentic_runtime;
mod check;
mod cli_workspace;
#[allow(
    dead_code,
    reason = "Spec 003 command backend includes lifecycle surfaces beyond the minimal T057 CLI caller"
)]
mod command;
mod domain;
#[allow(
    dead_code,
    reason = "Spec 003 terminal backend includes lifecycle surfaces beyond the minimal T057 CLI caller"
)]
mod execution;
mod git;
mod store;
#[cfg(test)]
mod t068_store_regression_tests;
#[cfg(test)]
mod t072_agentic_runtime_discovery_tests;
#[cfg(test)]
mod t074_agentic_context_tests;
#[cfg(test)]
mod t075_agentic_authority_tests;
#[cfg(test)]
mod t077_codex_protocol_tests;
#[cfg(test)]
mod t078_claude_structured_tests;

use crate::check::run_check;
use crate::domain::{CheckEvidence, CheckStatus, Eligibility, PromotionReport};
use crate::git::Repo;
use crate::store::{NewRun, Store};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_TIMEOUT_SECS: u64 = 300;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("winds: {error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return Err(usage().into());
    };
    let flags = parse_flags(args.collect())?;
    match command.as_str() {
        "verify" => verify(flags),
        "promote" => promote(flags),
        "recover" => recover(flags),
        "workspace-open" | "workspace-clone" | "profiles" | "run" | "terminal-proof"
        | "execution" => cli_workspace::dispatch(command.as_str(), flags),
        _ => Err(usage().into()),
    }
}

fn verify(flags: HashMap<String, String>) -> Result<()> {
    ensure_allowed_flags(
        &flags,
        &["repo", "base", "candidate", "check", "timeout-secs", "home"],
    )?;
    require_required_check_runtime()?;
    let repo_arg = required(&flags, "repo")?;
    let base_ref = required(&flags, "base")?;
    let candidate_ref = required(&flags, "candidate")?;
    let check_command = required(&flags, "check")?;
    let timeout_secs = flags
        .get("timeout-secs")
        .map(|value| value.parse::<u64>())
        .transpose()?
        .unwrap_or(DEFAULT_TIMEOUT_SECS);
    if timeout_secs == 0 {
        return Err("--timeout-secs must be greater than zero".into());
    }

    let repo = Repo::open(Path::new(repo_arg))?;
    let git_lock = repo.acquire_mutation_lock()?;
    repo.require_clean_primary()?;
    let home = winds_home(flags.get("home").map(String::as_str), &repo)?;
    let mut store = Store::open(&home)?;

    let base_oid = repo.resolve_commit(base_ref)?;
    let candidate_oid = repo.resolve_commit(candidate_ref)?;
    let candidate_tree = repo.tree_oid(&candidate_oid)?;
    let run_id = new_run_id()?;
    let worktree = home.join("worktrees").join(&run_id);
    let repo_path = utf8_path(repo.root(), "repository path")?.to_owned();
    let worktree_path = utf8_path(&worktree, "candidate worktree path")?.to_owned();
    let now = unix_ms()?;

    store.create_run(
        NewRun {
            run_id: &run_id,
            repo_path: &repo_path,
            base_oid: &base_oid,
            candidate_ref,
            candidate_oid: &candidate_oid,
            candidate_tree: &candidate_tree,
            worktree_path: &worktree_path,
            check_command,
            timeout_secs,
        },
        now,
    )?;
    repo.add_locked_worktree(
        &worktree,
        &candidate_oid,
        &format!("Winds verification run {run_id}"),
    )?;
    store.mark_workspace_ready(&run_id, unix_ms()?)?;
    drop(git_lock);

    let report = store.observe_and_save_evidence(&repo, &run_id, unix_ms()?)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if report.eligibility != Eligibility::Eligible {
        return Err(format!("candidate verification is {}", report.eligibility.as_str()).into());
    }
    Ok(())
}

fn promote(flags: HashMap<String, String>) -> Result<()> {
    ensure_allowed_flags(&flags, &["repo", "run", "home"])?;
    require_required_check_runtime()?;
    let repo_arg = required(&flags, "repo")?;
    let run_id = required(&flags, "run")?;
    let repo = Repo::open(Path::new(repo_arg))?;
    let home = winds_home(flags.get("home").map(String::as_str), &repo)?;
    let mut store = Store::open(&home)?;
    let run = store.load_run(run_id)?;

    if utf8_path(repo.root(), "repository path")? != run.repo_path {
        return Err("promotion repository does not match the verified run".into());
    }
    if run.eligibility != Eligibility::Eligible {
        return Err("only ELIGIBLE evidence can be promoted in Winds 0.1".into());
    }

    let _git_lock = repo.acquire_mutation_lock()?;
    let worktree = PathBuf::from(&run.worktree_path);
    if !worktree.exists() {
        return Err("verified candidate worktree is missing; manual recovery is required".into());
    }
    if repo.worktree_head(&worktree)? != run.candidate_oid {
        return Err("candidate HEAD changed after verification; rerun verification".into());
    }
    if !repo.worktree_is_clean(&worktree)? {
        return Err("candidate worktree changed after verification; rerun verification".into());
    }

    let recheck = run_check(
        &worktree,
        &run.check_command,
        Duration::from_secs(run.timeout_secs),
    )
    .map_err(|error| format!("promotion recheck failed to execute: {error}"))?;
    let recheck_stdout = store.write_blob(
        &run.run_id,
        "promotion-recheck.stdout",
        &recheck.stdout.bytes,
        recheck.stdout.truncated,
    )?;
    let recheck_stderr = store.write_blob(
        &run.run_id,
        "promotion-recheck.stderr",
        &recheck.stderr.bytes,
        recheck.stderr.truncated,
    )?;
    let recheck_evidence = CheckEvidence {
        authority: "WINDS_OBSERVED",
        command: run.check_command.clone(),
        status: recheck.status,
        exit_code: recheck.exit_code,
        duration_ms: recheck.duration_ms,
        stdout: recheck_stdout,
        stderr: recheck_stderr,
    };
    store.record_promotion_recheck(&run.run_id, &recheck_evidence, unix_ms()?)?;

    if recheck_evidence.status != CheckStatus::Pass
        || recheck_evidence.stdout.truncated
        || recheck_evidence.stderr.truncated
    {
        return Err("promotion recheck did not produce complete PASS evidence".into());
    }
    if repo.worktree_head(&worktree)? != run.candidate_oid || !repo.worktree_is_clean(&worktree)? {
        return Err("candidate changed during promotion recheck; promotion blocked".into());
    }

    let selected_branch = format!("winds/selected/{}", run.run_id);
    repo.create_selected_branch(&selected_branch, &run.candidate_oid)?;
    store.record_promotion(
        &run.run_id,
        &selected_branch,
        &run.candidate_oid,
        unix_ms()?,
    )?;

    let report = PromotionReport {
        run_id: run.run_id,
        authority: "CALLER_REQUESTED",
        branch: selected_branch,
        commit_oid: run.candidate_oid,
        candidate_tree: run.candidate_tree,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn recover(flags: HashMap<String, String>) -> Result<()> {
    ensure_allowed_flags(&flags, &["repo", "home"])?;
    let repo_arg = required(&flags, "repo")?;
    let repo = Repo::open(Path::new(repo_arg))?;
    let home = winds_home(flags.get("home").map(String::as_str), &repo)?;
    let mut store = Store::open(&home)?;
    let repo_path = utf8_path(repo.root(), "repository path")?.to_owned();
    let _git_lock = repo.acquire_mutation_lock()?;
    let inventory = repo.worktree_paths()?;
    let runs = store.runs_for_repo(&repo_path)?;
    let mut outcomes = Vec::new();
    let mut manual_recovery_required = false;

    for run in runs {
        let path = PathBuf::from(&run.worktree_path);
        let registered = inventory.iter().any(|known| known == &path);
        let inspection = if registered && path.exists() {
            match repo.worktree_head(&path) {
                Ok(head) => match repo.worktree_is_clean(&path) {
                    Ok(clean) => Ok((head == run.candidate_oid, clean)),
                    Err(error) => Err(error),
                },
                Err(error) => Err(error),
            }
        } else {
            Ok((false, false))
        };
        let (exact_head, clean, inspection_error) = match inspection {
            Ok((exact_head, clean)) => (exact_head, clean, None),
            Err(error) => (false, false, Some(error.to_string())),
        };

        let (status, recovery_reason): (&str, Option<String>) = if run.state == "PROVISIONING" {
            (
                "MANUAL_RECOVERY_REQUIRED",
                Some(
                    "run was interrupted during provisioning; automatic ownership cannot be proven"
                        .to_owned(),
                ),
            )
        } else if let Some(error) = inspection_error.as_deref() {
            (
                "MANUAL_RECOVERY_REQUIRED",
                Some(format!("worktree state could not be inspected: {error}")),
            )
        } else if exact_head && clean {
            ("PRESENT", None)
        } else if !registered {
            (
                "MANUAL_RECOVERY_REQUIRED",
                Some("worktree is not registered in Git inventory".to_owned()),
            )
        } else if !path.exists() {
            (
                "MANUAL_RECOVERY_REQUIRED",
                Some("registered worktree path is missing".to_owned()),
            )
        } else if !exact_head {
            (
                "MANUAL_RECOVERY_REQUIRED",
                Some("worktree HEAD does not match the recorded candidate".to_owned()),
            )
        } else {
            (
                "MANUAL_RECOVERY_REQUIRED",
                Some("worktree contains unverified changes".to_owned()),
            )
        };

        if let Some(reason) = recovery_reason.as_deref() {
            manual_recovery_required = true;
            store.record_recovery_required(&run.run_id, reason, unix_ms()?)?;
        }

        outcomes.push(json!({
            "run_id": run.run_id,
            "status": status,
            "worktree_path": run.worktree_path,
        }));
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "authority": "WINDS_OBSERVED",
            "repo_path": repo_path,
            "runs": outcomes,
        }))?
    );
    if manual_recovery_required {
        return Err("one or more Winds runs require manual recovery".into());
    }
    Ok(())
}

#[cfg(unix)]
fn require_required_check_runtime() -> Result<()> {
    Ok(())
}

#[cfg(not(unix))]
fn require_required_check_runtime() -> Result<()> {
    Err(
        "authoritative required-check execution is unsupported on native Windows in Spec 003 T051"
            .into(),
    )
}

fn parse_flags(args: Vec<String>) -> Result<HashMap<String, String>> {
    let mut result = HashMap::new();
    let mut index = 0;
    while index < args.len() {
        let key = args[index]
            .strip_prefix("--")
            .ok_or_else(|| format!("unexpected argument: {}", args[index]))?;
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for --{key}"))?;
        if value.starts_with("--") {
            return Err(format!("missing value for --{key}").into());
        }
        if result.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(format!("duplicate flag --{key}").into());
        }
        index += 2;
    }
    Ok(result)
}

fn ensure_allowed_flags(flags: &HashMap<String, String>, allowed: &[&str]) -> Result<()> {
    for name in flags.keys() {
        if !allowed.contains(&name.as_str()) {
            return Err(format!("unknown flag --{name}").into());
        }
    }
    Ok(())
}

fn required<'a>(flags: &'a HashMap<String, String>, name: &str) -> Result<&'a str> {
    flags
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| format!("missing required flag --{name}").into())
}

fn winds_home(explicit: Option<&str>, repo: &Repo) -> Result<PathBuf> {
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
    repo.require_external_state_path(&resolved)?;
    fs::create_dir_all(&resolved)?;
    let canonical = resolved.canonicalize()?;
    repo.require_external_state_path(&canonical)?;
    utf8_path(&canonical, "WINDS_HOME")?;
    Ok(canonical)
}

fn resolve_without_creation(path: &Path) -> Result<PathBuf> {
    let mut missing: Vec<OsString> = Vec::new();
    let mut cursor = path;
    while !cursor.exists() {
        let name = cursor
            .file_name()
            .ok_or("WINDS_HOME has no existing ancestor")?;
        missing.push(name.to_os_string());
        cursor = cursor
            .parent()
            .ok_or("WINDS_HOME has no existing ancestor")?;
    }

    let mut resolved = cursor.canonicalize()?;
    for component in missing.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn utf8_path<'a>(path: &'a Path, label: &str) -> Result<&'a str> {
    path.to_str().ok_or_else(|| {
        format!("{label} is not valid UTF-8; Winds 0.1 refuses lossy path storage").into()
    })
}

fn new_run_id() -> Result<String> {
    Ok(format!("run-{}-{}", unix_ms()?, std::process::id()))
}

fn unix_ms() -> Result<i64> {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    i64::try_from(millis).map_err(|_| "system time exceeds SQLite integer range".into())
}

fn usage() -> &'static str {
    "usage:\n  winds verify --repo PATH --base REF --candidate REF --check COMMAND [--timeout-secs N] [--home PATH]\n  winds promote --repo PATH --run RUN_ID [--home PATH]\n  winds recover --repo PATH [--home PATH]\n  winds workspace-open --repo PATH [--home PATH]\n  winds workspace-clone --remote REMOTE --destination ABS_PATH [--home PATH]\n  winds profiles --repo PATH [--home PATH]\n  winds run --repo PATH --execution-id ID --executable ABS_PATH [--args-json JSON_ARRAY] [--history command|disabled] [--home PATH]\n  winds terminal-proof --repo PATH --execution-id ID --profile-id PROFILE_ID [--rows N] [--cols N] [--home PATH]\n  winds execution --repo PATH --execution-id ID [--home PATH]"
}
