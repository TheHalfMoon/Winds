use crate::domain::{BlobEvidence, Eligibility, EvidenceReport, StoredRun};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs;
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
    pub run_branch: &'a str,
    pub worktree_path: &'a str,
    pub check_command: &'a str,
    pub timeout_secs: u64,
}

impl Store {
    pub fn open(home: &Path) -> Result<Self> {
        fs::create_dir_all(home)?;
        fs::create_dir_all(home.join("blobs"))?;
        let connection = Connection::open(home.join("winds.db"))?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        connection.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS tasks (
                task_id TEXT PRIMARY KEY,
                base_oid TEXT NOT NULL,
                created_unix_ms INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS candidate_runs (
                run_id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL REFERENCES tasks(task_id),
                repo_path TEXT NOT NULL,
                candidate_ref TEXT NOT NULL,
                candidate_oid TEXT NOT NULL,
                candidate_tree TEXT NOT NULL,
                run_branch TEXT NOT NULL,
                worktree_path TEXT NOT NULL,
                check_command TEXT NOT NULL,
                timeout_secs INTEGER NOT NULL,
                state TEXT NOT NULL,
                created_unix_ms INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS events (
                event_id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_id TEXT NOT NULL,
                kind TEXT NOT NULL,
                authority TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_unix_ms INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS evidence_reports (
                run_id TEXT PRIMARY KEY REFERENCES candidate_runs(run_id),
                eligibility TEXT NOT NULL,
                report_json TEXT NOT NULL,
                created_unix_ms INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS promotions (
                run_id TEXT PRIMARY KEY REFERENCES candidate_runs(run_id),
                branch TEXT NOT NULL,
                commit_oid TEXT NOT NULL,
                created_unix_ms INTEGER NOT NULL
            );
            ",
        )?;
        Ok(Self {
            connection,
            home: home.to_path_buf(),
        })
    }

    pub fn create_run(&mut self, run: NewRun<'_>, now_ms: i64) -> Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute(
            "INSERT INTO tasks(task_id, base_oid, created_unix_ms) VALUES (?1, ?2, ?3)",
            params![run.run_id, run.base_oid, now_ms],
        )?;
        tx.execute(
            "INSERT INTO candidate_runs(
                run_id, task_id, repo_path, candidate_ref, candidate_oid, candidate_tree,
                run_branch, worktree_path, check_command, timeout_secs, state, created_unix_ms
             ) VALUES (?1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'PROVISIONING', ?10)",
            params![
                run.run_id,
                run.repo_path,
                run.candidate_ref,
                run.candidate_oid,
                run.candidate_tree,
                run.run_branch,
                run.worktree_path,
                run.check_command,
                run.timeout_secs,
                now_ms,
            ],
        )?;
        insert_event(&tx, run.run_id, "WorkspaceProvisionRequested", "WINDS_OBSERVED", "{}", now_ms)?;
        tx.commit()?;
        Ok(())
    }

    pub fn mark_workspace_ready(&self, run_id: &str, now_ms: i64) -> Result<()> {
        self.connection.execute(
            "UPDATE candidate_runs SET state = 'READY' WHERE run_id = ?1",
            params![run_id],
        )?;
        insert_event(
            &self.connection,
            run_id,
            "WorkspaceReady",
            "WINDS_OBSERVED",
            "{}",
            now_ms,
        )?;
        Ok(())
    }

    pub fn write_blob(&self, run_id: &str, name: &str, bytes: &[u8], truncated: bool) -> Result<BlobEvidence> {
        let dir = self.home.join("blobs").join(run_id);
        fs::create_dir_all(&dir)?;
        let path = dir.join(name);
        fs::write(&path, bytes)?;
        let relative = path.strip_prefix(&self.home)?.to_string_lossy().into_owned();
        let digest = Sha256::digest(bytes);
        let sha256 = digest.iter().map(|byte| format!("{byte:02x}")).collect();
        Ok(BlobEvidence {
            relative_path: relative,
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
            "INSERT OR REPLACE INTO evidence_reports(run_id, eligibility, report_json, created_unix_ms)
             VALUES (?1, ?2, ?3, ?4)",
            params![report.run_id, report.eligibility.as_str(), json, now_ms],
        )?;
        insert_event(&tx, &report.run_id, "EvidenceBuilt", "WINDS_OBSERVED", "{}", now_ms)?;
        tx.commit()?;
        Ok(())
    }

    pub fn load_run(&self, run_id: &str) -> Result<StoredRun> {
        let row = self
            .connection
            .query_row(
                "SELECT r.run_id, r.repo_path, t.base_oid, r.candidate_ref, r.candidate_oid,
                        r.candidate_tree, r.run_branch, r.worktree_path, r.check_command,
                        r.timeout_secs, e.eligibility
                 FROM candidate_runs r
                 JOIN tasks t ON t.task_id = r.task_id
                 LEFT JOIN evidence_reports e ON e.run_id = r.run_id
                 WHERE r.run_id = ?1",
                params![run_id],
                |row| {
                    let eligibility: Option<String> = row.get(10)?;
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, String>(8)?,
                        row.get::<_, u64>(9)?,
                        eligibility,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown Winds run: {run_id}"))?;

        let eligibility = match row.10.as_deref() {
            Some("ELIGIBLE") => Eligibility::Eligible,
            Some("WARNING") => Eligibility::Warning,
            _ => Eligibility::Blocked,
        };
        Ok(StoredRun {
            run_id: row.0,
            repo_path: row.1,
            base_oid: row.2,
            candidate_ref: row.3,
            candidate_oid: row.4,
            candidate_tree: row.5,
            run_branch: row.6,
            worktree_path: row.7,
            check_command: row.8,
            timeout_secs: row.9,
            eligibility,
        })
    }

    pub fn record_promotion(&mut self, run_id: &str, branch: &str, commit_oid: &str, now_ms: i64) -> Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute(
            "INSERT INTO promotions(run_id, branch, commit_oid, created_unix_ms) VALUES (?1, ?2, ?3, ?4)",
            params![run_id, branch, commit_oid, now_ms],
        )?;
        insert_event(
            &tx,
            run_id,
            "DecisionRecorded",
            "HUMAN_DECIDED",
            "{\"decision\":\"promote\"}",
            now_ms,
        )?;
        insert_event(&tx, run_id, "PromotionCreated", "WINDS_OBSERVED", "{}", now_ms)?;
        tx.commit()?;
        Ok(())
    }
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
