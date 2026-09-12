use crate::agentic_runtime::{RuntimeBindingOwnership, RuntimeKind};
use crate::domain::workflow::WorkflowContinuationClass;
use crate::model_mesh::{
    ApprovalApplicability, ContextDigest, ContinuityAuthorityClaim, ContinuityClass,
    ContinuityContextCompleteness, ExactModelId, ExactProviderId, IdentityClaim, IdentityDimension,
    IdentitySourceClass, ModelMeshAuthorityEnvelopeV1, ModelMeshContinuityPermissionDescriptorV1,
    TargetDimension, TargetRequest, TargetSelector, adapt_actor_scope,
};
use crate::store::{
    ModelMeshClaimSubject, NewModelMeshTargetRequest, Store, StoredModelMeshContinuityEvent,
    StoredModelMeshIdentityClaim, StoredModelMeshTargetRequest, StoredStageRun, StoredWorkflowRun,
};
use crate::{Result, ensure_allowed_flags, required, unix_ms};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) const MODEL_MESH_COMMAND: &str = "model-mesh";
const MAX_CLI_ITEMS: usize = 256;
const MAX_CLI_VALUE_BYTES: usize = 4096;

#[derive(Debug)]
struct ReadOnlyModelMeshStore {
    connection: Connection,
    db_path: PathBuf,
}

#[derive(Debug)]
struct ReadOnlyRuntimeBinding {
    binding_id: String,
    session_id: String,
    runtime: RuntimeKind,
    executable_sha256: String,
    version: String,
    native_session_id: Option<String>,
    ownership: RuntimeBindingOwnership,
}

impl ReadOnlyModelMeshStore {
    fn open(home: &Path) -> Result<Self> {
        if !home.is_absolute() {
            return Err("model-mesh --home must be an absolute path".into());
        }
        let home_metadata = fs::metadata(home)
            .map_err(|_| "Model Mesh inspection requires an existing initialized --home")?;
        if !home_metadata.is_dir() {
            return Err("Model Mesh inspection --home is not a directory".into());
        }
        let db_path = home.join("winds.db");
        let db_metadata = fs::metadata(&db_path)
            .map_err(|_| "Model Mesh inspection requires an existing initialized winds.db")?;
        if !db_metadata.is_file() {
            return Err("Model Mesh inspection winds.db is not a regular file".into());
        }
        ensure_no_sqlite_sidecars(&db_path)?;
        let uri = immutable_sqlite_uri(&db_path)?;
        let connection = Connection::open_with_flags(
            uri,
            OpenFlags::SQLITE_OPEN_READ_ONLY
                | OpenFlags::SQLITE_OPEN_URI
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        validate_read_only_canonical_schema(&connection)?;
        Ok(Self {
            connection,
            db_path,
        })
    }

    fn assert_source_quiescent(&self) -> Result<()> {
        ensure_no_sqlite_sidecars(&self.db_path)
    }

    fn require_stage_scope(&self, flags: &HashMap<String, String>) -> Result<String> {
        let stage_id = required(flags, "stage-id")?;
        let row = self
            .connection
            .query_row(
                "SELECT workflow.workspace_id, workflow.workstream_id, workflow.workflow_run_id,
                        stage.stage_run_id
                 FROM workflow_stage_runs stage
                 JOIN workflow_runs workflow ON workflow.workflow_run_id = stage.workflow_run_id
                 WHERE stage.stage_run_id = ?1",
                params![stage_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown workflow StageRun: {stage_id}"))?;
        if row.0 != required(flags, "workspace-id")?
            || row.1 != required(flags, "workstream-id")?
            || row.2 != required(flags, "workflow-id")?
        {
            return Err(
                "Model Mesh CLI exact workspace/workstream/workflow/stage identity mismatch".into(),
            );
        }
        Ok(row.3)
    }

    fn actor_binding_ids_for_stage(&self, stage_run_id: &str) -> Result<Vec<String>> {
        let mut statement = self.connection.prepare(
            "SELECT binding_id FROM workflow_actor_bindings
             WHERE stage_run_id = ?1 ORDER BY bound_unix_ms, binding_id",
        )?;
        Ok(statement
            .query_map(params![stage_run_id], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?)
    }

    fn load_runtime_binding(&self, binding_id: &str) -> Result<ReadOnlyRuntimeBinding> {
        let row = self
            .connection
            .query_row(
                "SELECT binding_id, session_id, runtime_kind, observed_executable_path,
                        canonical_executable_path, executable_byte_len, executable_sha256,
                        runtime_version_state, runtime_version, runtime_version_source,
                        native_session_id, ownership_state, bound_unix_ms,
                        ownership_observed_unix_ms
                 FROM runtime_session_bindings WHERE binding_id = ?1",
                params![binding_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, String>(8)?,
                        row.get::<_, String>(9)?,
                        row.get::<_, Option<String>>(10)?,
                        row.get::<_, String>(11)?,
                        row.get::<_, i64>(12)?,
                        row.get::<_, Option<i64>>(13)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown runtime session binding: {binding_id}"))?;
        let runtime = RuntimeKind::from_db(&row.2)
            .ok_or_else(|| format!("unknown runtime kind in store: {}", row.2))?;
        if row.7 != "OBSERVED" || row.9 != "WINDS_LOCALLY_OBSERVED" {
            return Err(
                "persisted runtime binding lacks accepted observed version provenance".into(),
            );
        }
        if row.8.trim().is_empty() || row.12 < 0 || row.5 < 0 {
            return Err(
                "persisted runtime binding contains invalid bounded observation truth".into(),
            );
        }
        if !Path::new(&row.3).is_absolute() || !Path::new(&row.4).is_absolute() {
            return Err("persisted runtime executable identity must use absolute paths".into());
        }
        if row.6.len() != 64
            || !row
                .6
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err("persisted runtime executable SHA-256 is malformed".into());
        }
        let ownership = RuntimeBindingOwnership::from_db(&row.11)
            .ok_or_else(|| format!("unknown runtime binding ownership state: {}", row.11))?;
        match (ownership, row.13) {
            (RuntimeBindingOwnership::Unproven, None) => {}
            (RuntimeBindingOwnership::OwnershipLost, Some(observed)) if observed >= row.12 => {}
            _ => return Err("persisted runtime binding ownership evidence is inconsistent".into()),
        }
        if row
            .10
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err("persisted native runtime session id must not be empty".into());
        }
        Ok(ReadOnlyRuntimeBinding {
            binding_id: row.0,
            session_id: row.1,
            runtime,
            executable_sha256: row.6,
            version: row.8,
            native_session_id: row.10,
            ownership,
        })
    }

    fn actor_availability_json(&self, binding_id: &str) -> Result<Value> {
        let row = self
            .connection
            .query_row(
                "SELECT binding_id, stage_run_id, winds_session_id, runtime_binding_id,
                        continuation_class, bound_unix_ms
                 FROM workflow_actor_bindings WHERE binding_id = ?1",
                params![binding_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown workflow actor binding: {binding_id}"))?;
        let winds_session_id = row
            .2
            .ok_or("stored workflow actor binding is missing explicit Winds session identity")?;
        if row.5 < 0 {
            return Err("stored workflow actor binding time must not be negative".into());
        }
        let continuation = WorkflowContinuationClass::from_db(&row.4)
            .ok_or_else(|| format!("unknown workflow continuation class in store: {}", row.4))?;
        if continuation == WorkflowContinuationClass::Resumed {
            return Err(
                "stored RESUMED workflow continuation lacks an accepted physical Spec 006 resume proof"
                    .into(),
            );
        }
        let runtime = row
            .3
            .as_deref()
            .map(|id| self.load_runtime_binding(id))
            .transpose()?;
        if runtime
            .as_ref()
            .is_some_and(|runtime| runtime.session_id != winds_session_id)
        {
            return Err(
                "workflow actor runtime binding does not belong to the bound Winds session".into(),
            );
        }
        Ok(json!({
            "actor_binding_id":row.0,
            "stage_run_id":row.1,
            "winds_session_id":winds_session_id,
            "workflow_continuation":continuation.as_db_str(),
            "runtime_binding":runtime.as_ref().map(read_only_runtime_binding_json),
            "target_availability":"DURABLE_BINDING_OBSERVED_NOT_LIVE_EXECUTION_READINESS"
        }))
    }

    fn load_target(&self, target_request_id: &str) -> Result<StoredModelMeshTargetRequest> {
        let row = self
            .connection
            .query_row(
                "SELECT request.target_request_id, request.actor_role, request.runtime_kind,
                        request.requested_provider_id, request.requested_model_id,
                        request.selector_class, request.target_descriptor_digest,
                        request.selection_approval_id, request.created_unix_ms,
                        workflow.workspace_id, workflow.workstream_id, workflow.workflow_run_id,
                        stage.stage_run_id, actor.binding_id, actor.winds_session_id
                 FROM model_mesh_target_requests request
                 JOIN workflow_actor_bindings actor
                   ON actor.binding_id = request.actor_binding_id
                  AND actor.stage_run_id = request.stage_run_id
                 JOIN workflow_stage_runs stage ON stage.stage_run_id = request.stage_run_id
                 JOIN workflow_runs workflow ON workflow.workflow_run_id = stage.workflow_run_id
                 JOIN winds_sessions session
                   ON session.session_id = actor.winds_session_id
                  AND session.workstream_id = workflow.workstream_id
                 LEFT JOIN runtime_session_bindings runtime ON runtime.binding_id = actor.runtime_binding_id
                 WHERE request.target_request_id = ?1
                   AND (actor.runtime_binding_id IS NULL OR (
                        runtime.session_id = actor.winds_session_id
                        AND runtime.runtime_kind = request.runtime_kind
                   ))",
                params![target_request_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, i64>(8)?,
                        row.get::<_, String>(9)?,
                        row.get::<_, String>(10)?,
                        row.get::<_, String>(11)?,
                        row.get::<_, String>(12)?,
                        row.get::<_, String>(13)?,
                        row.get::<_, String>(14)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown or invalid Model Mesh target request: {target_request_id}"))?;
        let runtime = RuntimeKind::from_db(&row.2)
            .ok_or_else(|| format!("unknown Model Mesh target runtime: {}", row.2))?;
        let provider = row
            .3
            .as_deref()
            .map(ExactProviderId::new)
            .transpose()
            .map_err(cli_model_mesh_error)?
            .map_or(TargetDimension::Unspecified, TargetDimension::Exact);
        let model = row
            .4
            .as_deref()
            .map(ExactModelId::new)
            .transpose()
            .map_err(cli_model_mesh_error)?
            .map_or(TargetDimension::Unspecified, TargetDimension::Exact);
        let selector = TargetSelector::from_db(&row.5)
            .ok_or_else(|| format!("unknown Model Mesh target selector: {}", row.5))?;
        let descriptor = crate::model_mesh::ModelMeshTargetDescriptorV1::new(
            &row.9, &row.10, &row.11, &row.12, &row.13, &row.14, &row.1, runtime, provider, model,
        )
        .map_err(cli_model_mesh_error)?;
        let request = TargetRequest::new(descriptor, selector).map_err(cli_model_mesh_error)?;
        if request.target_descriptor_digest() != row.6 {
            return Err(
                "stored Model Mesh target descriptor digest does not match canonical joins".into(),
            );
        }
        if row.8 < 0 {
            return Err("stored Model Mesh target creation time must not be negative".into());
        }
        Ok(StoredModelMeshTargetRequest {
            target_request_id: row.0,
            request,
            selection_approval_id: row.7,
            created_unix_ms: row.8,
        })
    }

    fn load_claim(&self, identity_claim_id: &str) -> Result<StoredModelMeshIdentityClaim> {
        let row = self
            .connection
            .query_row(
                "SELECT identity_claim_id, claim_subject, target_request_id, actor_binding_id,
                        dimension, normalized_value, source_class, observation_basis,
                        runtime_binding_id, observed_unix_ms
                 FROM model_mesh_identity_claims WHERE identity_claim_id = ?1",
                params![identity_claim_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, Option<String>>(7)?,
                        row.get::<_, Option<String>>(8)?,
                        row.get::<_, i64>(9)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown Model Mesh identity claim: {identity_claim_id}"))?;
        let subject = match row.1.as_str() {
            "REQUEST_TARGET" => ModelMeshClaimSubject::RequestTarget(
                row.2
                    .ok_or("REQUEST_TARGET claim is missing target request identity")?,
            ),
            "ACTOR" => ModelMeshClaimSubject::Actor(
                row.3
                    .ok_or("ACTOR claim is missing actor binding identity")?,
            ),
            _ => return Err(format!("unknown Model Mesh claim subject: {}", row.1).into()),
        };
        let dimension = IdentityDimension::from_db(&row.4)
            .ok_or_else(|| format!("unknown Model Mesh identity dimension: {}", row.4))?;
        let source = IdentitySourceClass::from_db(&row.6)
            .ok_or_else(|| format!("unknown Model Mesh identity source: {}", row.6))?;
        let claim = IdentityClaim::new(dimension, row.5.as_deref(), source, row.7.as_deref())
            .map_err(cli_model_mesh_error)?;
        if row.9 < 0 {
            return Err("stored Model Mesh identity observation time must not be negative".into());
        }
        self.validate_claim_subject(&subject, row.8.as_deref())?;
        Ok(StoredModelMeshIdentityClaim {
            identity_claim_id: row.0,
            subject,
            claim,
            runtime_binding_id: row.8,
            observed_unix_ms: row.9,
        })
    }

    fn validate_claim_subject(
        &self,
        subject: &ModelMeshClaimSubject,
        runtime_binding_id: Option<&str>,
    ) -> Result<()> {
        let valid = match subject {
            ModelMeshClaimSubject::RequestTarget(target_request_id) => {
                self.connection.query_row(
                    "SELECT EXISTS(SELECT 1 FROM model_mesh_target_requests WHERE target_request_id=?1)",
                    params![target_request_id],
                    |row| row.get::<_, i64>(0),
                )? == 1
                    && runtime_binding_id.is_none()
            }
            ModelMeshClaimSubject::Actor(actor_binding_id) => {
                self.connection.query_row(
                    "SELECT EXISTS(
                        SELECT 1 FROM workflow_actor_bindings actor
                        LEFT JOIN runtime_session_bindings runtime
                          ON runtime.binding_id = actor.runtime_binding_id
                        WHERE actor.binding_id = ?1
                          AND actor.winds_session_id IS NOT NULL
                          AND (?2 IS NULL OR (
                               actor.runtime_binding_id = ?2
                               AND runtime.session_id = actor.winds_session_id
                          )))",
                    params![actor_binding_id, runtime_binding_id],
                    |row| row.get::<_, i64>(0),
                )? == 1
            }
        };
        if !valid {
            return Err(
                "stored Model Mesh identity claim subject/runtime binding is invalid".into(),
            );
        }
        Ok(())
    }

    fn list_claims_for_target(
        &self,
        target_request_id: &str,
    ) -> Result<Vec<StoredModelMeshIdentityClaim>> {
        let target = self.load_target(target_request_id)?;
        let actor_binding_id = target.request.descriptor().actor_binding_id();
        let mut statement = self.connection.prepare(
            "SELECT identity_claim_id FROM model_mesh_identity_claims
             WHERE target_request_id = ?1 OR actor_binding_id = ?2
             ORDER BY observed_unix_ms, identity_claim_id",
        )?;
        let ids = statement
            .query_map(params![target_request_id, actor_binding_id], |row| {
                row.get::<_, String>(0)
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(statement);
        ids.iter().map(|id| self.load_claim(id)).collect()
    }

    fn association_ids(&self, event_id: &str, role: &str) -> Result<Vec<String>> {
        let mut statement = self.connection.prepare(
            "SELECT identity_claim_id FROM model_mesh_continuity_identity_claims
             WHERE continuity_event_id=?1 AND actor_role=?2
             ORDER BY identity_claim_id",
        )?;
        Ok(statement
            .query_map(params![event_id, role], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?)
    }

    fn validate_event_claims(
        &self,
        actor_binding_id: Option<&str>,
        claim_ids: &[String],
        role: &str,
    ) -> Result<()> {
        if actor_binding_id.is_none() && !claim_ids.is_empty() {
            return Err(
                format!("Model Mesh continuity {role} claims require an exact actor").into(),
            );
        }
        if let Some(actor_binding_id) = actor_binding_id {
            for claim_id in claim_ids {
                let claim = self.load_claim(claim_id)?;
                if claim.subject != ModelMeshClaimSubject::Actor(actor_binding_id.to_owned()) {
                    return Err(format!(
                        "Model Mesh continuity {role} identity claim does not belong to exact actor"
                    )
                    .into());
                }
            }
        }
        Ok(())
    }

    fn approval_matches(
        &self,
        approval_id: &str,
        expected: &ModelMeshAuthorityEnvelopeV1,
    ) -> Result<(bool, i64)> {
        let row = self
            .connection
            .query_row(
                "SELECT workspace_id, workstream_id, session_id, content_digest,
                        canonical_content_json, approved_unix_ms
                 FROM agentic_delegation_approvals WHERE approval_id=?1",
                params![approval_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                    ))
                },
            )
            .optional()?;
        let Some(row) = row else {
            return Ok((false, -1));
        };
        let observed = format!("{:x}", Sha256::digest(row.4.as_bytes()));
        if observed != row.3 || row.5 < 0 {
            return Ok((false, row.5));
        }
        let parsed = match ModelMeshAuthorityEnvelopeV1::from_canonical_json(&row.4) {
            Ok(value) => value,
            Err(_) => return Ok((false, row.5)),
        };
        Ok((
            row.0 == parsed.workspace_id()
                && row.1 == parsed.workstream_id()
                && row.2 == parsed.session_id()
                && &parsed == expected,
            row.5,
        ))
    }

    fn validate_actor_in_target_lineage(
        &self,
        target_request_id: &str,
        actor_binding_id: Option<&str>,
        role: &str,
    ) -> Result<()> {
        let Some(actor_binding_id) = actor_binding_id else {
            return Ok(());
        };
        let valid = self.connection.query_row(
            "WITH RECURSIVE lineage(stage_run_id, predecessor_stage_run_id) AS (
                SELECT stage.stage_run_id, stage.predecessor_stage_run_id
                FROM model_mesh_target_requests request
                JOIN workflow_stage_runs stage ON stage.stage_run_id = request.stage_run_id
                WHERE request.target_request_id = ?1
                UNION ALL
                SELECT predecessor.stage_run_id, predecessor.predecessor_stage_run_id
                FROM workflow_stage_runs predecessor
                JOIN lineage current ON predecessor.stage_run_id = current.predecessor_stage_run_id
             )
             SELECT EXISTS(
                SELECT 1
                FROM workflow_actor_bindings actor
                JOIN lineage ON lineage.stage_run_id = actor.stage_run_id
                WHERE actor.binding_id = ?2
                  AND actor.winds_session_id IS NOT NULL
             )",
            params![target_request_id, actor_binding_id],
            |row| row.get::<_, i64>(0),
        )? == 1;
        if !valid {
            return Err(format!(
                "stored Model Mesh continuity {role} actor is outside canonical target StageRun lineage"
            )
            .into());
        }
        Ok(())
    }

    fn load_event(&self, event_id: &str) -> Result<StoredModelMeshContinuityEvent> {
        let row = self
            .connection
            .query_row(
                "SELECT continuity_event_id, target_request_id, source_actor_binding_id,
                        destination_actor_binding_id, continuity_class, context_digest,
                        completeness_state, continuity_permission_digest, authority_approval_id,
                        authority_claim, created_unix_ms
                 FROM model_mesh_continuity_events WHERE continuity_event_id=?1",
                params![event_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, Option<String>>(7)?,
                        row.get::<_, Option<String>>(8)?,
                        row.get::<_, String>(9)?,
                        row.get::<_, i64>(10)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown Model Mesh continuity event: {event_id}"))?;
        let continuity_class = ContinuityClass::from_db(&row.4)
            .ok_or_else(|| format!("unknown Model Mesh continuity class: {}", row.4))?;
        let completeness = ContinuityContextCompleteness::from_db(&row.6)
            .ok_or_else(|| format!("unknown Model Mesh continuity completeness: {}", row.6))?;
        let authority_claim = ContinuityAuthorityClaim::from_db(&row.9)
            .ok_or_else(|| format!("unknown Model Mesh continuity authority claim: {}", row.9))?;
        if row.10 < 0 {
            return Err("stored Model Mesh continuity event time must not be negative".into());
        }
        self.validate_actor_in_target_lineage(&row.1, row.2.as_deref(), "source")?;
        self.validate_actor_in_target_lineage(&row.1, row.3.as_deref(), "destination")?;
        let source_identity_claim_ids = self.association_ids(&row.0, "SOURCE")?;
        let destination_identity_claim_ids = self.association_ids(&row.0, "DESTINATION")?;
        self.validate_event_claims(row.2.as_deref(), &source_identity_claim_ids, "source")?;
        self.validate_event_claims(
            row.3.as_deref(),
            &destination_identity_claim_ids,
            "destination",
        )?;
        let target = self.load_target(&row.1)?;
        if row.10 < target.created_unix_ms {
            return Err("stored Model Mesh continuity event predates its target request".into());
        }
        let descriptor = target.request.descriptor();
        let target_digest = descriptor.digest().map_err(cli_model_mesh_error)?;
        match authority_claim {
            ContinuityAuthorityClaim::Required => {
                let context_digest = row
                    .5
                    .as_deref()
                    .ok_or("authorized Model Mesh continuity requires context digest")?;
                let permission_digest = row
                    .7
                    .as_deref()
                    .ok_or("authorized Model Mesh continuity requires permission digest")?;
                let approval_id = row
                    .8
                    .as_deref()
                    .ok_or("authorized Model Mesh continuity requires approval id")?;
                let permission = ModelMeshContinuityPermissionDescriptorV1::new(
                    descriptor.workflow_run_id(),
                    descriptor.stage_run_id(),
                    row.2.as_deref(),
                    row.3.as_deref(),
                    continuity_class,
                    &target_digest,
                    ContextDigest::exact(context_digest).map_err(cli_model_mesh_error)?,
                )
                .map_err(cli_model_mesh_error)?;
                if permission.digest().map_err(cli_model_mesh_error)? != permission_digest {
                    return Err(
                        "stored Model Mesh continuity permission digest does not match event"
                            .into(),
                    );
                }
                let expected = ModelMeshAuthorityEnvelopeV1::for_continuity_permission(
                    descriptor,
                    &permission,
                )
                .map_err(cli_model_mesh_error)?;
                let (matches, approved_at) = self.approval_matches(approval_id, &expected)?;
                if !matches || row.10 < approved_at {
                    return Err(
                        "stored Model Mesh continuity approval no longer matches exact event"
                            .into(),
                    );
                }
            }
            ContinuityAuthorityClaim::NoAuthorityClaim => {
                if !matches!(
                    continuity_class,
                    ContinuityClass::Unavailable | ContinuityClass::Unproven
                ) || row.7.is_some()
                    || row.8.is_some()
                {
                    return Err("invalid NO_AUTHORITY_CLAIM continuity event".into());
                }
            }
        }
        Ok(StoredModelMeshContinuityEvent {
            continuity_event_id: row.0,
            target_request_id: row.1,
            source_actor_binding_id: row.2,
            destination_actor_binding_id: row.3,
            continuity_class,
            context_digest: row.5,
            completeness,
            continuity_permission_digest: row.7,
            authority_approval_id: row.8,
            authority_claim,
            source_identity_claim_ids,
            destination_identity_claim_ids,
            created_unix_ms: row.10,
        })
    }

    fn list_events_for_target(
        &self,
        target_request_id: &str,
    ) -> Result<Vec<StoredModelMeshContinuityEvent>> {
        self.load_target(target_request_id)?;
        let mut statement = self.connection.prepare(
            "SELECT continuity_event_id FROM model_mesh_continuity_events
             WHERE target_request_id=?1 ORDER BY created_unix_ms, continuity_event_id",
        )?;
        let ids = statement
            .query_map(params![target_request_id], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(statement);
        ids.iter().map(|id| self.load_event(id)).collect()
    }
}

fn normalize_schema_sql(sql: &str) -> String {
    sql.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
        .replace(" if not exists", "")
}

fn read_schema_objects(
    connection: &Connection,
    namespace: &str,
) -> Result<BTreeMap<String, (String, String, String)>> {
    let (table_glob, index_glob, trigger_glob) = match namespace {
        "workflow" => ("workflow_*", "idx_workflow_*", "trg_workflow_*"),
        "model_mesh" => ("model_mesh_*", "idx_model_mesh_*", "trg_model_mesh_*"),
        _ => return Err("unknown read-only schema namespace".into()),
    };
    let mut statement = connection.prepare(
        "SELECT name, type, tbl_name, sql
         FROM sqlite_master
         WHERE name GLOB ?1 OR name GLOB ?2 OR name GLOB ?3
         ORDER BY name",
    )?;
    let rows = statement.query_map(params![table_glob, index_glob, trigger_glob], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
        ))
    })?;
    let mut objects = BTreeMap::new();
    for row in rows {
        let (name, object_type, table_name, sql) = row?;
        let sql = sql.ok_or_else(|| format!("{namespace} schema object has no SQL: {name}"))?;
        objects.insert(name, (object_type, table_name, normalize_schema_sql(&sql)));
    }
    Ok(objects)
}

fn expected_workflow_schema_objects_read_only() -> Result<BTreeMap<String, (String, String, String)>>
{
    let connection = Connection::open_in_memory()?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.execute_batch(include_str!(
        "../migrations/0002_workspace_execution_ledger.sql"
    ))?;
    connection.execute_batch(include_str!("../migrations/0006_agentic_identity.sql"))?;
    connection.execute_batch(include_str!(
        "../migrations/0008_runtime_session_bindings.sql"
    ))?;
    connection.execute_batch(include_str!(
        "../migrations/0010_resumable_workflow_ledger.sql"
    ))?;
    read_schema_objects(&connection, "workflow")
}

fn expected_model_mesh_schema_objects_read_only()
-> Result<BTreeMap<String, (String, String, String)>> {
    let connection = Connection::open_in_memory()?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.execute_batch(include_str!(
        "../migrations/0002_workspace_execution_ledger.sql"
    ))?;
    connection.execute_batch(include_str!("../migrations/0006_agentic_identity.sql"))?;
    connection.execute_batch(include_str!(
        "../migrations/0008_runtime_session_bindings.sql"
    ))?;
    connection.execute_batch(include_str!(
        "../migrations/0009_agentic_delegation_audit.sql"
    ))?;
    connection.execute_batch(include_str!(
        "../migrations/0010_resumable_workflow_ledger.sql"
    ))?;
    connection.execute_batch(include_str!("../migrations/0011_model_mesh_continuity.sql"))?;
    read_schema_objects(&connection, "model_mesh")
}

fn validate_schema_namespace(
    connection: &Connection,
    namespace: &str,
    expected: BTreeMap<String, (String, String, String)>,
) -> Result<()> {
    let observed = read_schema_objects(connection, namespace)?;
    if observed.keys().collect::<Vec<_>>() != expected.keys().collect::<Vec<_>>() {
        let missing = expected
            .keys()
            .filter(|name| !observed.contains_key(*name))
            .cloned()
            .collect::<Vec<_>>();
        let unexpected = observed
            .keys()
            .filter(|name| !expected.contains_key(*name))
            .cloned()
            .collect::<Vec<_>>();
        return Err(format!(
            "{namespace} schema object inventory mismatch; missing={missing:?}; unexpected={unexpected:?}"
        )
        .into());
    }
    for (name, expected_object) in expected {
        let observed_object = observed
            .get(&name)
            .ok_or_else(|| format!("{namespace} schema object missing: {name}"))?;
        if observed_object != &expected_object {
            return Err(format!("{namespace} schema object definition mismatch: {name}").into());
        }
    }
    Ok(())
}

fn validate_read_only_canonical_schema(connection: &Connection) -> Result<()> {
    validate_schema_namespace(
        connection,
        "workflow",
        expected_workflow_schema_objects_read_only()?,
    )?;
    validate_schema_namespace(
        connection,
        "model_mesh",
        expected_model_mesh_schema_objects_read_only()?,
    )
}

fn immutable_sqlite_uri(path: &Path) -> Result<String> {
    let text = path
        .to_str()
        .ok_or("Model Mesh inspection database path is not valid UTF-8")?
        .replace('\\', "/");
    #[cfg(windows)]
    let text = if text.starts_with('/') {
        text
    } else {
        format!("/{text}")
    };
    let mut encoded = String::with_capacity(text.len() + 32);
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b':' | b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            use std::fmt::Write as _;
            write!(&mut encoded, "%{byte:02X}").map_err(|_| "failed to encode SQLite path")?;
        }
    }
    Ok(format!("file://{encoded}?mode=ro&immutable=1"))
}

fn ensure_no_sqlite_sidecars(db_path: &Path) -> Result<()> {
    let text = db_path
        .to_str()
        .ok_or("Model Mesh inspection database path is not valid UTF-8")?;
    for suffix in ["-wal", "-shm", "-journal"] {
        if Path::new(&format!("{text}{suffix}")).exists() {
            return Err(
                "Model Mesh read-only inspection refuses an active SQLite sidecar; retry after the writer closes"
                    .into(),
            );
        }
    }
    Ok(())
}

fn read_only_runtime_binding_json(binding: &ReadOnlyRuntimeBinding) -> Value {
    json!({
        "binding_id":binding.binding_id,
        "winds_session_id":binding.session_id,
        "runtime":binding.runtime.as_str(),
        "executable_sha256":binding.executable_sha256,
        "version_state":"OBSERVED",
        "version":binding.version,
        "version_source":"WINDS_LOCALLY_OBSERVED",
        "native_session_id":binding.native_session_id,
        "ownership":binding.ownership.as_str(),
        "live_session_proven":false
    })
}

pub(crate) fn dispatch(flags: HashMap<String, String>) -> Result<()> {
    let output = execute(flags)?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

pub(crate) fn execute(flags: HashMap<String, String>) -> Result<Value> {
    validate_flag_values(&flags)?;
    let action = required(&flags, "action")?.to_owned();
    match action.as_str() {
        "availability" => availability(flags),
        "request" => record_request(flags),
        "status" => target_status(flags),
        "why-blocked" => why_blocked(flags),
        "continuity" => continuity(flags),
        "reviewer" => reviewer(flags),
        _ => Err(format!("unknown Model Mesh action: {action}").into()),
    }
}

fn availability(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(&flags, &stage_flags(&[]))?;
    let store = open_read_only_store(&flags)?;
    let stage_run_id = store.require_stage_scope(&flags)?;
    let actor_ids = store.actor_binding_ids_for_stage(&stage_run_id)?;
    require_bounded_items(actor_ids.len(), "Model Mesh actor availability")?;
    let actors = actor_ids
        .iter()
        .map(|id| store.actor_availability_json(id))
        .collect::<Result<Vec<_>>>()?;
    let output = json!({
        "action":"availability",
        "stage_run_id":stage_run_id,
        "observation_scope":"DURABLE_LOCAL_STATE_ONLY",
        "live_runtime_discovery":"NOT_PERFORMED",
        "provider_execution":"NOT_PERFORMED",
        "credential_operation":"NOT_PERFORMED",
        "actors":actors,
        "policy_path":"EXPLICIT_POLICY_UNAVAILABLE_FIRST_SLICE"
    });
    store.assert_source_quiescent()?;
    Ok(output)
}

fn record_request(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(
        &flags,
        &stage_flags(&[
            "target-request-id",
            "actor-binding-id",
            "actor-role",
            "runtime",
            "provider-id",
            "model-id",
            "approval-id",
        ]),
    )?;
    let mut store = open_store(&flags)?;
    let (workflow, stage) = require_stage_scope(&store, &flags)?;
    let actor_id = required(&flags, "actor-binding-id")?;
    let actor = store.load_workflow_actor_binding(actor_id)?;
    if actor.stage_run_id != stage.identity.stage_run_id {
        return Err("Model Mesh CLI actor does not belong to the exact requested StageRun".into());
    }
    let session = store.load_winds_session(&actor.winds_session_id)?;
    let runtime_binding = actor
        .runtime_binding_id
        .as_deref()
        .map(|id| store.load_runtime_session_binding(id))
        .transpose()?;
    let scope = adapt_actor_scope(
        &workflow.identity,
        &stage.identity,
        &session,
        &actor,
        runtime_binding.as_ref(),
    )
    .map_err(cli_model_mesh_error)?;
    let descriptor = scope
        .target_descriptor(
            required(&flags, "actor-role")?,
            parse_runtime(required(&flags, "runtime")?)?,
            provider_dimension(flags.get("provider-id").map(String::as_str))?,
            model_dimension(flags.get("model-id").map(String::as_str))?,
        )
        .map_err(cli_model_mesh_error)?;
    let request =
        TargetRequest::new(descriptor, TargetSelector::Human).map_err(cli_model_mesh_error)?;
    let target_request_id = required(&flags, "target-request-id")?;
    let approval_id = required(&flags, "approval-id")?;
    let history = store.list_model_mesh_target_requests_for_stage(&stage.identity.stage_run_id)?;
    require_bounded_items(history.len(), "Model Mesh target history")?;
    if let Some(existing) = history
        .iter()
        .find(|stored| stored.target_request_id == target_request_id)
    {
        if existing.request != request || existing.selection_approval_id != approval_id {
            return Err("Model Mesh target request idempotency collision".into());
        }
        return Ok(request_result("IDEMPOTENT_NO_CHANGE", existing));
    }
    let stored = store.create_model_mesh_target_request(NewModelMeshTargetRequest {
        target_request_id,
        request: &request,
        selection_approval_id: approval_id,
        created_unix_ms: unix_ms()?,
    })?;
    Ok(request_result("INSERTED", &stored))
}

fn request_result(outcome: &str, stored: &StoredModelMeshTargetRequest) -> Value {
    json!({
        "action":"request",
        "append_outcome":outcome,
        "request":stored_target_json(stored),
        "selector":"HUMAN",
        "policy_path":"EXPLICIT_POLICY_UNAVAILABLE_FIRST_SLICE",
        "execution_authority":"NOT_GRANTED_BY_TARGET_SELECTION",
        "provider_execution":"NOT_PERFORMED"
    })
}

fn target_status(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(&flags, &["action", "home", "target-request-id"])?;
    let store = open_read_only_store(&flags)?;
    let output =
        target_status_from_read_only_store(&store, required(&flags, "target-request-id")?)?;
    store.assert_source_quiescent()?;
    Ok(output)
}

fn target_status_from_read_only_store(
    store: &ReadOnlyModelMeshStore,
    target_request_id: &str,
) -> Result<Value> {
    let stored = store.load_target(target_request_id)?;
    let claims = store.list_claims_for_target(target_request_id)?;
    require_bounded_items(claims.len(), "Model Mesh identity claims")?;
    let approval = selection_approval_applicability_read_only(store, &stored)?;
    let actor_json =
        store.actor_availability_json(stored.request.descriptor().actor_binding_id())?;
    let runtime_binding = actor_json
        .get("runtime_binding")
        .cloned()
        .unwrap_or(Value::Null);
    let mut blockers = Vec::new();
    if approval != ApprovalApplicability::Exact {
        blockers.push(format!("SELECTION_APPROVAL_{}", approval_label(approval)));
    }
    blockers.push("LIVE_RUNTIME_DISCOVERY_UNAVAILABLE".to_owned());
    blockers.push("AUTHENTICATION_READINESS_UNKNOWN".to_owned());
    blockers.push("CURRENT_AUTHORITY_BASIS_UNAVAILABLE".to_owned());
    Ok(json!({
        "action":"status",
        "request":stored_target_json(&stored),
        "identity_claims":claims.iter().map(stored_claim_json).collect::<Vec<_>>(),
        "durable_runtime_binding":runtime_binding,
        "target_resolution":{
            "state":"UNAVAILABLE",
            "canonical_resolution":Value::Null,
            "reason":"CURRENT_AUTHORITY_AND_LIVE_RUNTIME_BASIS_UNAVAILABLE"
        },
        "drift":{
            "live_runtime_freshness":"UNAVAILABLE_WITHOUT_DISCOVERY",
            "native_session_truth":actor_json["runtime_binding"]["ownership"].as_str().unwrap_or("UNAVAILABLE"),
            "selection_approval":approval_label(approval),
            "current_authority":"UNKNOWN"
        },
        "authentication_readiness":"UNKNOWN",
        "blockers":blockers,
        "policy_path":"EXPLICIT_POLICY_UNAVAILABLE_FIRST_SLICE",
        "execution_authority":"UNKNOWN_NOT_GRANTED_BY_INSPECTION",
        "outcomes":{
            "target_match":"UNKNOWN",
            "execution_authorized":"UNKNOWN",
            "continuity_proven":"UNKNOWN",
            "verified":"UNKNOWN",
            "human_accepted":"UNKNOWN",
            "landed":"UNKNOWN"
        },
        "provider_execution":"NOT_PERFORMED",
        "credential_operation":"NOT_PERFORMED"
    }))
}

fn why_blocked(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(&flags, &["action", "home", "target-request-id"])?;
    let store = open_read_only_store(&flags)?;
    let mut status =
        target_status_from_read_only_store(&store, required(&flags, "target-request-id")?)?;
    let primary = status["blockers"]
        .as_array()
        .and_then(|values| values.first())
        .cloned()
        .unwrap_or_else(|| json!("NONE"));
    status["action"] = json!("why-blocked");
    status["primary_blocker"] = primary;
    store.assert_source_quiescent()?;
    Ok(status)
}

fn continuity(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(&flags, &["action", "home", "target-request-id"])?;
    let store = open_read_only_store(&flags)?;
    let target_request_id = required(&flags, "target-request-id")?;
    let target = store.load_target(target_request_id)?;
    let events = store.list_events_for_target(target_request_id)?;
    require_bounded_items(events.len(), "Model Mesh continuity events")?;
    let values = events
        .iter()
        .map(|event| continuity_event_json_read_only(&store, event))
        .collect::<Result<Vec<_>>>()?;
    let output = json!({
        "action":"continuity",
        "target_request":stored_target_json(&target),
        "events":values,
        "history_rewritten":false,
        "provider_execution":"NOT_PERFORMED"
    });
    store.assert_source_quiescent()?;
    Ok(output)
}

fn reviewer(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(
        &flags,
        &["action", "home", "target-request-id", "continuity-event-id"],
    )?;
    let store = open_read_only_store(&flags)?;
    let target_request_id = required(&flags, "target-request-id")?;
    let event = store.load_event(required(&flags, "continuity-event-id")?)?;
    if event.target_request_id != target_request_id {
        return Err("reviewer continuity event does not belong to the exact target request".into());
    }
    let target = store.load_target(target_request_id)?;
    let output = json!({
        "action":"reviewer",
        "target_request":stored_target_json(&target),
        "continuity":continuity_event_json_read_only(&store, &event)?,
        "candidate_evidence_freshness":{
            "state":"UNAVAILABLE",
            "reason":"CURRENT_CANDIDATE_ARTIFACT_EVIDENCE_INPUTS_NOT_SUPPLIED_TO_T121"
        },
        "reviewer_context_fresh":false,
        "verified":"UNKNOWN",
        "human_accepted":"UNKNOWN",
        "provider_execution":"NOT_PERFORMED"
    });
    store.assert_source_quiescent()?;
    Ok(output)
}

fn open_read_only_store(flags: &HashMap<String, String>) -> Result<ReadOnlyModelMeshStore> {
    let home = PathBuf::from(required(flags, "home")?);
    ReadOnlyModelMeshStore::open(Path::new(&home))
}

fn open_store(flags: &HashMap<String, String>) -> Result<Store> {
    let home = PathBuf::from(required(flags, "home")?);
    if !home.is_absolute() {
        return Err("model-mesh --home must be an absolute path".into());
    }
    Store::open(Path::new(&home))
}

fn require_stage_scope(
    store: &Store,
    flags: &HashMap<String, String>,
) -> Result<(StoredWorkflowRun, StoredStageRun)> {
    let workflow = store.load_workflow_run(required(flags, "workflow-id")?)?;
    if workflow.identity.workspace_id != required(flags, "workspace-id")?
        || workflow.identity.workstream_id != required(flags, "workstream-id")?
    {
        return Err("Model Mesh CLI exact workspace/workstream/workflow identity mismatch".into());
    }
    let stage = store.load_stage_run(required(flags, "stage-id")?)?;
    if stage.identity.workflow_run_id != workflow.identity.workflow_run_id {
        return Err(
            "Model Mesh CLI StageRun does not belong to the exact requested workflow".into(),
        );
    }
    Ok((workflow, stage))
}

fn stored_target_json(stored: &StoredModelMeshTargetRequest) -> Value {
    let descriptor = stored.request.descriptor();
    json!({
        "target_request_id":stored.target_request_id,
        "workspace_id":descriptor.workspace_id(),
        "workstream_id":descriptor.workstream_id(),
        "workflow_run_id":descriptor.workflow_run_id(),
        "stage_run_id":descriptor.stage_run_id(),
        "actor_binding_id":descriptor.actor_binding_id(),
        "winds_session_id":descriptor.winds_session_id(),
        "actor_role":descriptor.actor_role(),
        "runtime":descriptor.runtime().as_str(),
        "provider":target_provider_json(descriptor.provider()),
        "model":target_model_json(descriptor.model()),
        "selector":stored.request.selector().as_str(),
        "target_descriptor_digest":stored.request.target_descriptor_digest(),
        "selection_approval_id":stored.selection_approval_id,
        "created_unix_ms":stored.created_unix_ms
    })
}

fn stored_claim_json(stored: &StoredModelMeshIdentityClaim) -> Value {
    let subject = match &stored.subject {
        ModelMeshClaimSubject::RequestTarget(id) => json!({"kind":"REQUEST_TARGET","id":id}),
        ModelMeshClaimSubject::Actor(id) => json!({"kind":"ACTOR","id":id}),
    };
    json!({
        "identity_claim_id":stored.identity_claim_id,
        "subject":subject,
        "dimension":stored.claim.dimension.as_str(),
        "value":stored.claim.value,
        "source":stored.claim.source.as_str(),
        "observation_basis":stored.claim.observation_basis,
        "runtime_binding_id":stored.runtime_binding_id,
        "observed_unix_ms":stored.observed_unix_ms
    })
}

fn continuity_event_json_read_only(
    store: &ReadOnlyModelMeshStore,
    event: &StoredModelMeshContinuityEvent,
) -> Result<Value> {
    let source_claims = event
        .source_identity_claim_ids
        .iter()
        .map(|id| store.load_claim(id))
        .collect::<Result<Vec<_>>>()?;
    let destination_claims = event
        .destination_identity_claim_ids
        .iter()
        .map(|id| store.load_claim(id))
        .collect::<Result<Vec<_>>>()?;
    require_bounded_items(source_claims.len(), "Model Mesh continuity source claims")?;
    require_bounded_items(
        destination_claims.len(),
        "Model Mesh continuity destination claims",
    )?;
    let source_actor = event
        .source_actor_binding_id
        .as_deref()
        .map(|id| store.actor_availability_json(id))
        .transpose()?;
    let destination_actor = event
        .destination_actor_binding_id
        .as_deref()
        .map(|id| store.actor_availability_json(id))
        .transpose()?;
    Ok(json!({
        "continuity_event_id":event.continuity_event_id,
        "target_request_id":event.target_request_id,
        "continuity_class":event.continuity_class.as_str(),
        "context_digest":event.context_digest,
        "context_completeness":event.completeness.as_str(),
        "context_loss_markers":"NOT_PERSISTED_IN_FROZEN_0011_EVENT_ROW",
        "source_actor_binding_id":event.source_actor_binding_id,
        "destination_actor_binding_id":event.destination_actor_binding_id,
        "source_actor":source_actor,
        "destination_actor":destination_actor,
        "source_identity_claims":source_claims.iter().map(stored_claim_json).collect::<Vec<_>>(),
        "destination_identity_claims":destination_claims.iter().map(stored_claim_json).collect::<Vec<_>>(),
        "authority_claim":event.authority_claim.as_str(),
        "authority_approval_id":event.authority_approval_id,
        "continuity_permission_digest":event.continuity_permission_digest,
        "created_unix_ms":event.created_unix_ms
    }))
}

fn selection_approval_applicability_read_only(
    store: &ReadOnlyModelMeshStore,
    target: &StoredModelMeshTargetRequest,
) -> Result<ApprovalApplicability> {
    let row = store
        .connection
        .query_row(
            "SELECT workspace_id, workstream_id, session_id, content_digest, canonical_content_json
         FROM agentic_delegation_approvals WHERE approval_id = ?1",
            params![target.selection_approval_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .optional()?;
    let Some((workspace_id, workstream_id, session_id, digest, canonical)) = row else {
        return Ok(ApprovalApplicability::Missing);
    };
    let observed_digest = format!("{:x}", Sha256::digest(canonical.as_bytes()));
    if observed_digest != digest {
        return Ok(ApprovalApplicability::Stale);
    }
    let parsed = match ModelMeshAuthorityEnvelopeV1::from_canonical_json(&canonical) {
        Ok(value) => value,
        Err(_) => return Ok(ApprovalApplicability::Mismatch),
    };
    let expected = ModelMeshAuthorityEnvelopeV1::for_target_selection(target.request.descriptor())
        .map_err(cli_model_mesh_error)?;
    if workspace_id != parsed.workspace_id()
        || workstream_id != parsed.workstream_id()
        || session_id != parsed.session_id()
        || parsed != expected
    {
        return Ok(ApprovalApplicability::Mismatch);
    }
    Ok(ApprovalApplicability::Exact)
}

fn stage_flags(extra: &[&str]) -> Vec<&'static str> {
    let mut flags = vec![
        "action",
        "home",
        "workspace-id",
        "workstream-id",
        "workflow-id",
        "stage-id",
    ];
    for value in extra {
        flags.push(match *value {
            "target-request-id" => "target-request-id",
            "actor-binding-id" => "actor-binding-id",
            "actor-role" => "actor-role",
            "runtime" => "runtime",
            "provider-id" => "provider-id",
            "model-id" => "model-id",
            "approval-id" => "approval-id",
            _ => unreachable!("closed T121 Model Mesh flag set"),
        });
    }
    flags
}

fn parse_runtime(value: &str) -> Result<RuntimeKind> {
    RuntimeKind::from_db(value).ok_or_else(|| format!("unknown Model Mesh runtime: {value}").into())
}

fn provider_dimension(value: Option<&str>) -> Result<TargetDimension<ExactProviderId>> {
    value
        .map(ExactProviderId::new)
        .transpose()
        .map_err(cli_model_mesh_error)
        .map(|value| value.map_or(TargetDimension::Unspecified, TargetDimension::Exact))
}

fn model_dimension(value: Option<&str>) -> Result<TargetDimension<ExactModelId>> {
    value
        .map(ExactModelId::new)
        .transpose()
        .map_err(cli_model_mesh_error)
        .map(|value| value.map_or(TargetDimension::Unspecified, TargetDimension::Exact))
}

fn target_provider_json(value: &TargetDimension<ExactProviderId>) -> Value {
    match value {
        TargetDimension::Unspecified => json!({"kind":"UNSPECIFIED"}),
        TargetDimension::Exact(id) => json!({"kind":"EXACT","value":id.as_str()}),
    }
}

fn target_model_json(value: &TargetDimension<ExactModelId>) -> Value {
    match value {
        TargetDimension::Unspecified => json!({"kind":"UNSPECIFIED"}),
        TargetDimension::Exact(id) => json!({"kind":"EXACT","value":id.as_str()}),
    }
}

fn approval_label(value: ApprovalApplicability) -> &'static str {
    match value {
        ApprovalApplicability::Exact => "EXACT",
        ApprovalApplicability::Missing => "MISSING",
        ApprovalApplicability::Mismatch => "MISMATCH",
        ApprovalApplicability::Stale => "STALE",
    }
}

fn validate_flag_values(flags: &HashMap<String, String>) -> Result<()> {
    for (name, value) in flags {
        if value.len() > MAX_CLI_VALUE_BYTES {
            return Err(format!(
                "Model Mesh flag --{name} exceeds the bounded {MAX_CLI_VALUE_BYTES}-byte limit"
            )
            .into());
        }
        if value.contains('\0') || value.contains(['\r', '\n']) {
            return Err(format!("Model Mesh flag --{name} contains forbidden control text").into());
        }
    }
    Ok(())
}

fn require_bounded_items(count: usize, context: &str) -> Result<()> {
    if count > MAX_CLI_ITEMS {
        return Err(format!("{context} exceeds the bounded {MAX_CLI_ITEMS}-item CLI limit").into());
    }
    Ok(())
}

fn cli_model_mesh_error(error: String) -> Box<dyn std::error::Error + Send + Sync> {
    error.into()
}
