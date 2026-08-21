#[cfg(test)]
mod agentic_authority;
#[allow(
    dead_code,
    reason = "Spec 006 T078 fake Claude structured CLI; real Claude and prompts remain blocked until T080"
)]
mod agentic_claude;
#[allow(
    dead_code,
    reason = "Spec 006 T079 bounded Codex connected proof; no generic runtime surface or automatic execution"
)]
mod agentic_codex;
#[cfg(test)]
mod agentic_context;
#[allow(
    dead_code,
    reason = "Spec 006 T072 fixture-only runtime discovery; real Agent work remains task-gated"
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
#[cfg(test)]
#[allow(dead_code)]
mod t079_codex_connected_tests;

use crate::check::run_check;
use crate::domain::{CheckEvidence, CheckStatus, Eligibility, EvidenceReport, PromotionReport};
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

    let check_run = run_check(&worktree, check_command, Duration::from_secs(timeout_secs))
        .map_err(|error| format!("required check failed to execute: {error}"))?;
    let head_after = repo.worktree_head(&worktree)?;
    let clean_after = repo.worktree_is_clean(&worktree)?;
    let mut warnings = Vec::new();

    if head_after != candidate_oid {
        warnings.push("candidate HEAD changed while evidence was being collected".to_owned());
    }
    if !clean_after {
        warnings.push("required check mutated candidate worktree state".to_owned());
    }
    if check_run.stdout.truncated || check_run.stderr.truncated {
        warnings.push(
            "required check output exceeded the capture cap; evidence is incomplete".to_owned(),
        );
    }

    let eligibility =
        if check_run.status != CheckStatus::Pass || head_after != candidate_oid || !clean_after {
            Eligibility::Blocked
        } else if check_run.stdout.truncated || check_run.stderr.truncated {
            Eligibility::Warning
        } else {
            Eligibility::Eligible
        };

    let stdout = store.write_blob(
        &run_id,
        "check.stdout",
        &check_run.stdout.bytes,
        check_run.stdout.truncated,
    )?;
    let stderr = store.write_blob(
        &run_id,
        "check.stderr",
        &check_run.stderr.bytes,
        check_run.stderr.truncated,
    )?;
    let evidence = CheckEvidence {
        status: check_run.status,
        exit_code: check_run.exit_code,
        duration_ms: check_run.duration_ms,
        stdout,
        stderr,
    };
    let finished = unix_ms()?;
    store.record_check(&run_id, &evidence, finished)?;
    store.record_observed_state(&run_id, &head_after, clean_after, finished)?;
    store.set_eligibility(&run_id, eligibility, finished)?;
    store.persist_evidence_report(
        &EvidenceReport {
            schema: "winds-evidence-v1".to_owned(),
            run_id: run_id.clone(),
            candidate_oid,
            candidate_tree,
            check_command: check_command.to_owned(),
            evidence,
            head_after,
            clean_after,
            eligibility,
            warnings,
        },
        finished,
    )?;
    repo.remove_locked_worktree(&worktree)?;
    store.mark_workspace_removed(&run_id, unix_ms()?)?;
    Ok(())
}

fn promote(flags: HashMap<String, String>) -> Result<()> {
    ensure_allowed_flags(&flags, &["repo", "run", "home"])?;
    let repo_arg = required(&flags, "repo")?;
    let run_id = required(&flags, "run")?;
    let repo = Repo::open(Path::new(repo_arg))?;
    let _git_lock = repo.acquire_mutation_lock()?;
    repo.require_clean_primary()?;
    let home = winds_home(flags.get("home").map(String::as_str), &repo)?;
    let mut store = Store::open(&home)?;
    let run = store.load_run(run_id)?;
    if run.eligibility != Some(Eligibility::Eligible) {
        return Err("verification run is not eligible for promotion".into());
    }
    let main_oid = repo.resolve_commit("refs/heads/main")?;
    if main_oid != run.base_oid {
        return Err("main moved after verification; candidate must be re-verified".into());
    }
    let candidate_oid = repo.resolve_commit(&run.candidate_oid)?;
    let candidate_tree = repo.tree_oid(&candidate_oid)?;
    if candidate_oid != run.candidate_oid || candidate_tree != run.candidate_tree {
        return Err("candidate identity no longer matches verified evidence".into());
    }
    repo.fast_forward_main(&candidate_oid)?;
    let promoted_main_oid = repo.resolve_commit("refs/heads/main")?;
    let promoted_main_tree = repo.tree_oid(&promoted_main_oid)?;
    if promoted_main_oid != candidate_oid || promoted_main_tree != candidate_tree {
        return Err("promotion result did not match the verified candidate".into());
    }
    let now = unix_ms()?;
    store.mark_promoted(run_id, &promoted_main_oid, &promoted_main_tree, now)?;
    store.persist_promotion_report(
        &PromotionReport {
            schema: "winds-promotion-v1".to_owned(),
            run_id: run_id.to_owned(),
            base_oid: run.base_oid,
            candidate_oid,
            candidate_tree,
            promoted_main_oid,
            promoted_main_tree,
        },
        now,
    )?;
    Ok(())
}

fn recover(flags: HashMap<String, String>) -> Result<()> {
    ensure_allowed_flags(&flags, &["repo", "run", "home"])?;
    let repo_arg = required(&flags, "repo")?;
    let requested_run = flags.get("run").map(String::as_str);
    let repo = Repo::open(Path::new(repo_arg))?;
    let _git_lock = repo.acquire_mutation_lock()?;
    let home = winds_home(flags.get("home").map(String::as_str), &repo)?;
    let mut store = Store::open(&home)?;
    let runs = store.recoverable_runs(requested_run)?;
    for run in runs {
        let worktree = PathBuf::from(&run.worktree_path);
        if worktree.exists() {
            repo.remove_locked_worktree(&worktree)?;
        }
        store.mark_workspace_removed(&run.run_id, unix_ms()?)?;
    }
    Ok(())
}

fn parse_flags(args: Vec<String>) -> Result<HashMap<String, String>> {
    let mut flags = HashMap::new();
    let mut index = 0;
    while index < args.len() {
        let key = args
            .get(index)
            .ok_or_else(|| usage())?
            .strip_prefix("--")
            .ok_or_else(|| usage())?;
        let value = args.get(index + 1).ok_or_else(|| usage())?;
        if flags.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(format!("duplicate flag --{key}").into());
        }
        index += 2;
    }
    Ok(flags)
}

fn ensure_allowed_flags(flags: &HashMap<String, String>, allowed: &[&str]) -> Result<()> {
    for key in flags.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("unsupported flag --{key}").into());
        }
    }
    Ok(())
}

fn required<'a>(flags: &'a HashMap<String, String>, key: &str) -> Result<&'a str> {
    flags
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| format!("missing --{key}").into())
}

fn require_required_check_runtime() -> Result<()> {
    let path = env::var_os("PATH").ok_or("PATH is not set")?;
    if env::split_paths(&path).next().is_none() {
        return Err("PATH contains no search roots".into());
    }
    Ok(())
}

fn winds_home(explicit: Option<&str>, repo: &Repo) -> Result<PathBuf> {
    let path = match explicit {
        Some(path) => PathBuf::from(path),
        None => repo.root().join(".winds"),
    };
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn utf8_path<'a>(path: &'a Path, label: &str) -> Result<&'a str> {
    path.to_str()
        .ok_or_else(|| format!("{label} is not valid UTF-8").into())
}

fn unix_ms() -> Result<i64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .try_into()?)
}

fn new_run_id() -> Result<String> {
    Ok(format!("run-{}-{}", std::process::id(), unix_ms()?))
}

fn usage() -> &'static str {
    "usage: winds <verify|promote|recover|workspace-open|workspace-clone|profiles|run|terminal-proof|execution> [--key value ...]"
}
