from pathlib import Path

store_path = Path("src/store.rs")
store = store_path.read_text()

save_marker = "    pub fn save_evidence(&mut self, report: &EvidenceReport, now_ms: i64) -> Result<()> {\n"
assert store.count(save_marker) == 1, "save_evidence marker drifted"
observe_and_save = '''    pub(crate) fn observe_and_save_evidence(
        &mut self,
        repo: &crate::git::Repo,
        run_id: &str,
        now_ms: i64,
    ) -> Result<EvidenceReport> {
        let persisted = self
            .connection
            .query_row(
                "SELECT repo_path, base_oid, candidate_ref, candidate_oid, candidate_tree,\n\
                        worktree_path, check_command, timeout_secs, state\n\
                 FROM candidate_runs WHERE run_id = ?1",
                params![run_id],
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
                        row.get::<_, String>(8)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown Winds run for evidence observation: {run_id}"))?;

        if persisted.8 != "READY" {
            return Err(format!(
                "verification evidence observation requires READY candidate state, found {}",
                persisted.8
            )
            .into());
        }
        let observed_repo_path = repo
            .root()
            .to_str()
            .ok_or("verification repository path is not valid UTF-8")?;
        if observed_repo_path != persisted.0 {
            return Err("verification repository does not match persisted candidate run".into());
        }

        let worktree = PathBuf::from(&persisted.5);
        let timeout_secs = u64::try_from(persisted.7)?;
        let check_run = crate::check::run_check(
            &worktree,
            &persisted.6,
            std::time::Duration::from_secs(timeout_secs),
        )
        .map_err(|error| format!("required check failed to execute: {error}"))?;
        let head_after = repo.worktree_head(&worktree)?;
        let clean_after = repo.worktree_is_clean(&worktree)?;
        let mut warnings = Vec::new();

        if head_after != persisted.3 {
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

        let eligibility = if check_run.status != crate::domain::CheckStatus::Pass
            || head_after != persisted.3
            || !clean_after
        {
            Eligibility::Blocked
        } else if check_run.stdout.truncated || check_run.stderr.truncated {
            Eligibility::Warning
        } else {
            Eligibility::Eligible
        };

        let stdout = self.write_blob(
            run_id,
            "check.stdout",
            &check_run.stdout.bytes,
            check_run.stdout.truncated,
        )?;
        let stderr = self.write_blob(
            run_id,
            "check.stderr",
            &check_run.stderr.bytes,
            check_run.stderr.truncated,
        )?;

        let report = EvidenceReport {
            schema_version: 1,
            run_id: run_id.to_owned(),
            authority: "WINDS_OBSERVED",
            repo_path: persisted.0,
            base_oid: persisted.1,
            candidate_ref: persisted.2,
            candidate_oid: persisted.3,
            candidate_tree: persisted.4,
            worktree_path: persisted.5,
            check: CheckEvidence {
                authority: "WINDS_OBSERVED",
                command: persisted.6,
                status: check_run.status,
                exit_code: check_run.exit_code,
                duration_ms: check_run.duration_ms,
                stdout,
                stderr,
            },
            eligibility,
            warnings,
        };
        self.save_evidence(&report, now_ms)?;
        Ok(report)
    }

    fn save_evidence(&mut self, report: &EvidenceReport, now_ms: i64) -> Result<()> {
'''
store = store.replace(save_marker, observe_and_save, 1)

load_marker = "    pub fn load_run(&self, run_id: &str) -> Result<StoredRun> {\n"
assert store.count(load_marker) == 1, "load_run marker drifted"
test_bridge = '''    #[cfg(test)]
    pub(crate) fn save_evidence_for_test(
        &mut self,
        report: &EvidenceReport,
        now_ms: i64,
    ) -> Result<()> {
        self.save_evidence(report, now_ms)
    }

'''
store = store.replace(load_marker, test_bridge + load_marker, 1)
store_path.write_text(store)

main_path = Path("src/main.rs")
main = main_path.read_text()
old_import = "use crate::domain::{CheckEvidence, CheckStatus, Eligibility, EvidenceReport, PromotionReport};"
new_import = "use crate::domain::{CheckStatus, Eligibility, PromotionReport};"
assert main.count(old_import) == 1, "main domain import marker drifted"
main = main.replace(old_import, new_import, 1)

start_token = "    let check_run = run_check(&worktree, check_command, Duration::from_secs(timeout_secs))"
end_token = "    store.save_evidence(&report, unix_ms()?)?;"
assert main.count(start_token) == 1, "verify observation start marker drifted"
assert main.count(end_token) == 1, "verify evidence persistence marker drifted"
start = main.index(start_token)
end = main.index(end_token, start) + len(end_token)
main = main[:start] + "    let report = store.observe_and_save_evidence(&repo, &run_id, unix_ms()?)?;" + main[end:]
main_path.write_text(main)

tests_path = Path("src/t083_agentic_candidate_evidence_tests.rs")
tests = tests_path.read_text()
occurrences = tests.count(".save_evidence(")
assert occurrences == 3, f"unexpected save_evidence focused-test call count: {occurrences}"
tests = tests.replace(".save_evidence(", ".save_evidence_for_test(")
tests_path.write_text(tests)
