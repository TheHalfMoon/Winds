#![cfg(unix)]

use serde_json::Value;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn winds_never_invokes_prohibited_downstream_git_operations() {
    let root = unique_temp_dir("winds-prohibited-git");
    let repo = root.join("repo");
    let winds_home = root.join("winds-home");
    let shim_dir = root.join("shim");
    let trace_path = root.join("git-trace.bin");
    let real_git = real_git_path();
    fs::create_dir_all(&repo).unwrap();

    git(&real_git, &repo, &["init", "-b", "main"]);
    git(
        &real_git,
        &repo,
        &["config", "user.email", "winds-test@example.invalid"],
    );
    git(&real_git, &repo, &["config", "user.name", "Winds Test"]);
    fs::write(repo.join("result.txt"), "base\n").unwrap();
    git(&real_git, &repo, &["add", "result.txt"]);
    git(&real_git, &repo, &["commit", "-m", "base"]);
    let base_oid = git_text(&real_git, &repo, &["rev-parse", "HEAD"]);

    git(&real_git, &repo, &["switch", "-c", "candidate"]);
    fs::write(repo.join("result.txt"), "candidate\n").unwrap();
    git(&real_git, &repo, &["commit", "-am", "candidate"]);
    let candidate_oid = git_text(&real_git, &repo, &["rev-parse", "HEAD"]);
    git(&real_git, &repo, &["switch", "main"]);

    install_git_trace_shim(&shim_dir);

    let verify = traced_winds(
        &winds_home,
        &shim_dir,
        &trace_path,
        &real_git,
        &[
            "verify",
            "--repo",
            repo.to_str().unwrap(),
            "--base",
            &base_oid,
            "--candidate",
            &candidate_oid,
            "--check",
            "true",
            "--timeout-secs",
            "5",
        ],
    );
    assert_success(&verify);
    let verify_json: Value = serde_json::from_slice(&verify.stdout).unwrap();
    let run_id = verify_json["run_id"].as_str().unwrap().to_owned();
    let worktree = PathBuf::from(verify_json["worktree_path"].as_str().unwrap());

    let promote = traced_winds(
        &winds_home,
        &shim_dir,
        &trace_path,
        &real_git,
        &[
            "promote",
            "--repo",
            repo.to_str().unwrap(),
            "--run",
            &run_id,
        ],
    );
    assert_success(&promote);

    fs::write(worktree.join("manual.txt"), "manual edit\n").unwrap();
    let recover = traced_winds(
        &winds_home,
        &shim_dir,
        &trace_path,
        &real_git,
        &["recover", "--repo", repo.to_str().unwrap()],
    );
    assert!(!recover.status.success());
    let recover_json: Value = serde_json::from_slice(&recover.stdout).unwrap();
    assert!(
        recover_json["runs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|run| run["run_id"] == run_id && run["status"] == "MANUAL_RECOVERY_REQUIRED")
    );

    let trace = fs::read(&trace_path).unwrap();
    assert_no_prohibited_git_operations(&trace);

    remove_owned_temp_dir(&root);
}

#[test]
fn binary_trace_parser_preserves_embedded_delimiters_and_empty_args() {
    let trace = b"4\0-C\0/tmp/with\ttab\nand-newline\0status\0\0";
    let invocations = parse_trace(trace);

    assert_eq!(invocations.len(), 1);
    assert_eq!(invocations[0].len(), 4);
    assert_eq!(invocations[0][0], b"-C");
    assert_eq!(invocations[0][1], b"/tmp/with\ttab\nand-newline");
    assert_eq!(invocations[0][2], b"status");
    assert!(invocations[0][3].is_empty());
}

fn assert_no_prohibited_git_operations(trace: &[u8]) {
    let invocations = parse_trace(trace);
    let rendered = render_invocations(&invocations);

    assert!(
        !invocations.is_empty(),
        "Git shim recorded no Winds invocations"
    );
    assert!(
        invocations
            .iter()
            .filter_map(|args| git_subcommand_index(args).map(|index| &args[index]))
            .any(|subcommand| subcommand == b"worktree"),
        "Git shim did not observe the expected worktree path: {rendered}"
    );

    for forbidden in [b"merge".as_slice(), b"rebase", b"cherry-pick", b"push"] {
        assert!(
            !invocations.iter().any(|args| {
                git_subcommand_index(args)
                    .map(|index| args[index].as_slice() == forbidden)
                    .unwrap_or(false)
            }),
            "Winds invoked prohibited git operation `{}`: {rendered}",
            String::from_utf8_lossy(forbidden)
        );
    }

    for args in &invocations {
        let Some(subcommand_index) = git_subcommand_index(args) else {
            continue;
        };

        if args[subcommand_index] == b"clean" {
            assert!(
                !args[subcommand_index + 1..]
                    .iter()
                    .any(|arg| is_force_flag(arg)),
                "Winds invoked prohibited force-clean operation: {rendered}"
            );
        }

        if args[subcommand_index] == b"worktree"
            && args.get(subcommand_index + 1).map(Vec::as_slice) == Some(b"remove")
        {
            assert!(
                !args[subcommand_index + 2..]
                    .iter()
                    .any(|arg| is_force_flag(arg)),
                "Winds invoked prohibited force-remove operation: {rendered}"
            );
        }
    }
}

fn parse_trace(trace: &[u8]) -> Vec<Vec<Vec<u8>>> {
    let mut fields = trace.split(|byte| *byte == 0);
    let mut invocations = Vec::new();

    while let Some(count_bytes) = fields.next() {
        if count_bytes.is_empty() {
            assert!(fields.next().is_none(), "unexpected empty trace record");
            break;
        }

        let count = std::str::from_utf8(count_bytes)
            .expect("trace argc must be ASCII")
            .parse::<usize>()
            .expect("trace argc must be numeric");
        let mut args = Vec::with_capacity(count);
        for _ in 0..count {
            args.push(
                fields
                    .next()
                    .expect("trace record ended before argc arguments")
                    .to_vec(),
            );
        }
        invocations.push(args);
    }

    invocations
}

fn git_subcommand_index(args: &[Vec<u8>]) -> Option<usize> {
    let mut index = 0;
    while index < args.len() {
        match args[index].as_slice() {
            b"-c" | b"-C" => index += 2,
            _ => return Some(index),
        }
    }
    None
}

fn is_force_flag(arg: &[u8]) -> bool {
    arg == b"--force"
        || (arg.starts_with(b"-") && !arg.starts_with(b"--") && arg[1..].contains(&b'f'))
}

fn render_invocations(invocations: &[Vec<Vec<u8>>]) -> String {
    invocations
        .iter()
        .map(|args| {
            args.iter()
                .map(|arg| format!("{:?}", String::from_utf8_lossy(arg)))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn install_git_trace_shim(shim_dir: &Path) {
    fs::create_dir_all(shim_dir).unwrap();
    let shim = shim_dir.join("git");
    fs::write(
        &shim,
        r#"#!/bin/sh
{
    printf '%s\000' "$#"
    for arg in "$@"; do
        printf '%s\000' "$arg"
    done
} >> "$WINDS_GIT_TRACE"
exec "$WINDS_REAL_GIT" "$@"
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&shim).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(shim, permissions).unwrap();
}

fn traced_winds(
    home: &Path,
    shim_dir: &Path,
    trace_path: &Path,
    real_git: &Path,
    args: &[&str],
) -> Output {
    let mut paths = vec![shim_dir.to_path_buf()];
    if let Some(current_path) = env::var_os("PATH") {
        paths.extend(env::split_paths(&current_path));
    }
    let traced_path = env::join_paths(paths).unwrap();

    Command::new(env!("CARGO_BIN_EXE_winds"))
        .args(args)
        .env("WINDS_HOME", home)
        .env("WINDS_GIT_TRACE", trace_path)
        .env("WINDS_REAL_GIT", real_git)
        .env("PATH", traced_path)
        .output()
        .unwrap()
}

fn real_git_path() -> PathBuf {
    let output = Command::new("sh")
        .args(["-c", "command -v git"])
        .output()
        .unwrap();
    assert_success(&output);
    PathBuf::from(String::from_utf8(output.stdout).unwrap().trim())
}

fn git(real_git: &Path, repo: &Path, args: &[&str]) {
    let output = Command::new(real_git)
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert_success(&output);
}

fn git_text(real_git: &Path, repo: &Path, args: &[&str]) -> String {
    let output = Command::new(real_git)
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert_success(&output);
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn remove_owned_temp_dir(path: &Path) {
    let temp = env::temp_dir().canonicalize().unwrap();
    let canonical = path.canonicalize().unwrap();
    assert_eq!(canonical.parent(), Some(temp.as_path()));
    assert!(
        canonical
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap()
            .starts_with("winds-prohibited-git-")
    );
    fs::remove_dir_all(canonical).unwrap();
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = env::temp_dir().join(format!("{prefix}-{nanos}-{}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}
