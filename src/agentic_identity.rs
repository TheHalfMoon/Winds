use super::{Result, Store, validate_agentic_identity_text, validate_agentic_identity_timestamp};
use crate::domain::WindsSessionRecord;
use rusqlite::{OptionalExtension, params};

#[cfg(test)]
#[path = "t071_agentic_continuity_tests.rs"]
mod t071_agentic_continuity_tests;

#[allow(
    dead_code,
    reason = "Spec 006 T071 fixture-first continuity API; product/runtime callers land in later tasks"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ContinuationResolution {
    Selected(WindsSessionRecord),
    Candidates(Vec<WindsSessionRecord>),
}

#[allow(
    dead_code,
    reason = "Spec 006 T071 fixture-first continuity API; product/runtime callers land in later tasks"
)]
impl Store {
    pub(crate) fn start_new_winds_session(
        &self,
        workstream_id: &str,
        session_id: &str,
        display_name: &str,
        now_ms: i64,
    ) -> Result<()> {
        self.create_winds_session(
            super::NewWindsSession {
                session_id,
                workstream_id,
                display_name,
            },
            now_ms,
        )
    }

    pub(crate) fn create_new_task_with_session(
        &mut self,
        workspace_id: &str,
        workstream_id: &str,
        workstream_display_name: &str,
        session_id: &str,
        session_display_name: &str,
        now_ms: i64,
    ) -> Result<()> {
        validate_agentic_identity_text(workstream_id, "workstream id")?;
        validate_agentic_identity_text(workstream_display_name, "workstream display name")?;
        validate_agentic_identity_text(session_id, "Winds session id")?;
        validate_agentic_identity_text(session_display_name, "Winds session display name")?;
        validate_agentic_identity_timestamp(now_ms, "new task creation time")?;
        self.load_workspace(workspace_id)?;

        let tx = self.connection.transaction()?;
        tx.execute(
            "INSERT INTO workstreams(
                workstream_id, workspace_id, display_name, created_unix_ms, updated_unix_ms
             ) VALUES (?1, ?2, ?3, ?4, ?4)",
            params![workstream_id, workspace_id, workstream_display_name, now_ms],
        )?;
        tx.execute(
            "INSERT INTO winds_sessions(
                session_id, workstream_id, display_name, created_unix_ms, updated_unix_ms
             ) VALUES (?1, ?2, ?3, ?4, ?4)",
            params![session_id, workstream_id, session_display_name, now_ms],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn fork_winds_session(
        &mut self,
        origin_session_id: &str,
        session_id: &str,
        display_name: &str,
        now_ms: i64,
    ) -> Result<()> {
        validate_agentic_identity_text(origin_session_id, "origin Winds session id")?;
        validate_agentic_identity_text(session_id, "forked Winds session id")?;
        validate_agentic_identity_text(display_name, "forked Winds session display name")?;
        validate_agentic_identity_timestamp(now_ms, "forked Winds session creation time")?;
        if origin_session_id == session_id {
            return Err("forked Winds session id must differ from its origin".into());
        }

        let tx = self.connection.transaction()?;
        let origin = tx
            .query_row(
                "SELECT workstream_id, created_unix_ms
                 FROM winds_sessions WHERE session_id = ?1",
                params![origin_session_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()?
            .ok_or_else(|| format!("unknown origin Winds session: {origin_session_id}"))?;
        if now_ms < origin.1 {
            return Err("fork creation time cannot precede origin session creation time".into());
        }

        tx.execute(
            "INSERT INTO winds_sessions(
                session_id, workstream_id, display_name, created_unix_ms, updated_unix_ms
             ) VALUES (?1, ?2, ?3, ?4, ?4)",
            params![session_id, origin.0, display_name, now_ms],
        )?;
        tx.execute(
            "INSERT INTO winds_session_origins(session_id, workstream_id, origin_session_id)
             VALUES (?1, ?2, ?3)",
            params![session_id, origin.0, origin_session_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn load_winds_session_origin(
        &self,
        session_id: &str,
    ) -> Result<Option<WindsSessionRecord>> {
        Ok(self
            .connection
            .query_row(
                "SELECT origin.session_id, origin.workstream_id, origin.display_name,
                        origin.created_unix_ms, origin.updated_unix_ms
                 FROM winds_session_origins relation
                 INNER JOIN winds_sessions origin
                    ON origin.session_id = relation.origin_session_id
                   AND origin.workstream_id = relation.workstream_id
                 WHERE relation.session_id = ?1",
                params![session_id],
                |row| {
                    Ok(WindsSessionRecord {
                        session_id: row.get(0)?,
                        workstream_id: row.get(1)?,
                        display_name: row.get(2)?,
                        created_unix_ms: row.get(3)?,
                        updated_unix_ms: row.get(4)?,
                    })
                },
            )
            .optional()?)
    }

    pub(crate) fn resolve_winds_continuation(
        &self,
        workspace_id: &str,
        selector: Option<&str>,
    ) -> Result<ContinuationResolution> {
        self.load_workspace(workspace_id)?;
        if let Some(selector) = selector {
            validate_agentic_identity_text(selector, "continuation selector")?;
        }

        let mut statement = self.connection.prepare(
            "SELECT session.session_id, session.workstream_id, session.display_name,
                    session.created_unix_ms, session.updated_unix_ms
             FROM winds_sessions session
             INNER JOIN workstreams workstream
                ON workstream.workstream_id = session.workstream_id
             WHERE workstream.workspace_id = ?1
             ORDER BY session.created_unix_ms, session.session_id",
        )?;
        let sessions = statement
            .query_map(params![workspace_id], |row| {
                Ok(WindsSessionRecord {
                    session_id: row.get(0)?,
                    workstream_id: row.get(1)?,
                    display_name: row.get(2)?,
                    created_unix_ms: row.get(3)?,
                    updated_unix_ms: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let candidates = match selector {
            Some(selector) => {
                if let Some(exact) = sessions
                    .iter()
                    .find(|session| session.session_id == selector)
                    .cloned()
                {
                    return Ok(ContinuationResolution::Selected(exact));
                }
                sessions
                    .into_iter()
                    .filter(|session| session.display_name == selector)
                    .collect::<Vec<_>>()
            }
            None => sessions,
        };

        match candidates.len() {
            0 => Err("no Winds session matches the continuation request".into()),
            1 => Ok(ContinuationResolution::Selected(
                candidates.into_iter().next().expect("one candidate"),
            )),
            _ => Ok(ContinuationResolution::Candidates(candidates)),
        }
    }
}
