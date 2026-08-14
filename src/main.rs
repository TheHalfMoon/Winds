mod check;
mod domain;
mod git;
mod store;

use crate::check::run_check;
use crate::domain::{CheckEvidence, CheckStatus, Eligibility, EvidenceReport, PromotionReport};
use crate::git::Repo;
use crate::store::{NewRun, Store};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::error::Error;
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
        _ => Err(usage().into()),
    }
}

fn verify(flags: HashMap<String, String>) -> Result<()> {
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

    let home = winds_home(flags.get("home").map(String::as_str))?;
    let mut store = Store::open(&home)?;
    let repo = Repo::open(Path::new(repo_arg))?;
    repo.require_clean_primary()?;

    let base_oid = repo.resolve_commit(base_ref)?;
    let candidate_oid = repo.resolve_commit(candidate_ref)?;
    let candidate_tree = repo.tree_oid(&candidate_oid)?;
    let run_id = new_run_id()?;
    let run_branch = format!("winds/run/{run_id}");
    let worktree = home.join("worktrees").join(&run_id);
    let repo_path = repo.root().to_string_lossy().into_owned();
    let worktree_path = worktree.to_string_lossy().into_owned();
    let now = unix_ms()?;

    let git_lock = repo.acquire_mutation_lock()?;
    store.create_run(
        NewRun {
            run_id: &run_id,
            repo_path: &repo_path,
            base_oid: &base_oid,
            candidate_ref,
            candidate_oid: &candidate_oid,
            candidate_tree: &candidate_tree,
            run_branch: &run_branch,
            worktree_path: &worktree_path,
            check_command,
            timeout_secs,
        },
        now,
    )?;
    repo.add_locked_worktree(
        &worktree,
        &candidate_oid,
        &run_branch,
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

    let report = EvidenceReport {
        schema_version: 1,
        run_id: run_id.clone(),
        authority: "WINDS_OBSERVED",
        repo_path,
        base_oid,
        candidate_ref: candidate_ref.to_owned(),
        candidate_oid,
        candidate_tree,
        run_branch,
        worktree_path,
        check: CheckEvidence {
            authority: "WINDS_OBSERVED",
            command: check_command.to_owned(),
            status: check_run.status,
            exit_code: check_run.exit_code,
            duration_ms: check_run.duration_ms,
            stdout,
            stderr,
        },
        eligibility,
        warnings,
    };
    store.save_evidence(&report, unix_ms()?)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn promote(flags: HashMap<String, String>) -> Result<()> {
    let repo_arg = required(&flags, "repo")?;
    let run_id = required(&flags, "run")?;
    let home = winds_home(flags.get("home").map(String::as_str))?;
    let mut store = Store::open(&home)?;
    let run = store.load_run(run_id)?;
    let repo = Repo::open(Path::new(repo_arg))?;

    if repo.root().to_string_lossy() != run.repo_path {
        return Err("promotion repository does not match the verified run".into());
    }
    if run.eligibility != Eligibility::Eligible {
        return Err("only ELIGIBLE evidence can be promoted in Winds 0.1".into());
    }

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
    if recheck.status != CheckStatus::Pass || recheck.stdout.truncated || recheck.stderr.truncated {
        return Err("promotion recheck did not produce complete PASS evidence".into());
    }
    if repo.worktree_head(&worktree)? != run.candidate_oid || !repo.worktree_is_clean(&worktree)? {
        return Err("candidate changed during promotion recheck; promotion blocked".into());
    }

    let selected_branch = format!("winds/selected/{}", run.run_id);
    let _git_lock = repo.acquire_mutation_lock()?;
    repo.create_selected_branch(&selected_branch, &run.candidate_oid)?;
    store.record_promotion(
        &run.run_id,
        &selected_branch,
        &run.candidate_oid,
        unix_ms()?,
    )?;

    let report = PromotionReport {
        run_id: run.run_id,
        authority: "HUMAN_DECIDED",
        branch: selected_branch,
        commit_oid: run.candidate_oid,
        candidate_tree: run.candidate_tree,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn recover(flags: HashMap<String, String>) -> Result<()> {
    let repo_arg = required(&flags, "repo")?;
    let home = winds_home(flags.get("home").map(String::as_str))?;
    let mut store = Store::open(&home)?;
    let repo = Repo::open(Path::new(repo_arg))?;
    let repo_path = repo.root().to_string_lossy().into_owned();
    let inventory = repo.worktree_paths()?;
    let runs = store.runs_for_repo(&repo_path)?;
    let mut outcomes = Vec::new();

    for run in runs {
        let path = PathBuf::from(&run.worktree_path);
        let registered = inventory.iter().any(|known| known == &path);
        let exact_head = registered
            && path.exists()
            && repo
                .worktree_head(&path)
                .map(|head| head == run.candidate_oid)
                .unwrap_or(false);
        let clean = exact_head && repo.worktree_is_clean(&path).unwrap_or(false);

        let status = if exact_head && clean {
            if run.state == "PROVISIONING" {
                store.mark_recovered_ready(&run.run_id, unix_ms()?)?;
                "RECOVERED_READY"
            } else {
                "PRESENT"
            }
        } else {
            let reason = if !registered {
                "worktree is not registered in Git inventory"
            } else if !path.exists() {
                "registered worktree path is missing"
            } else if !exact_head {
                "worktree HEAD does not match the recorded candidate"
            } else {
                "worktree contains unverified changes"
            };
            store.mark_recovery_required(&run.run_id, reason, unix_ms()?)?;
            "MANUAL_RECOVERY_REQUIRED"
        };

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
    Ok(())
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

fn required<'a>(flags: &'a HashMap<String, String>, name: &str) -> Result<&'a str> {
    flags
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| format!("missing required flag --{name}").into())
}

fn winds_home(explicit: Option<&str>) -> Result<PathBuf> {
    let path = if let Some(path) = explicit {
        PathBuf::from(path)
    } else if let Some(path) = env::var_os("WINDS_HOME") {
        PathBuf::from(path)
    } else {
        let home = env::var_os("HOME").ok_or("HOME is not set; pass --home or WINDS_HOME")?;
        PathBuf::from(home).join(".winds")
    };
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn new_run_id() -> Result<String> {
    Ok(format!("run-{}-{}", unix_ms()?, std::process::id()))
}

fn unix_ms() -> Result<i64> {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    i64::try_from(millis).map_err(|_| "system time exceeds SQLite integer range".into())
}

fn usage() -> &'static str {
    "usage:\n  winds verify --repo PATH --base REF --candidate REF --check COMMAND [--timeout-secs N] [--home PATH]\n  winds promote --repo PATH --run RUN_ID [--home PATH]\n  winds recover --repo PATH [--home PATH]"
}
