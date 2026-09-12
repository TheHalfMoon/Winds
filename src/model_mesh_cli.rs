use crate::agentic_runtime::{EvidenceSource, RuntimeKind, RuntimeVersionState};
use crate::model_mesh::{
    ApprovalApplicability, ExactModelId, ExactProviderId, ModelMeshAuthorityEnvelopeV1,
    TargetDimension, TargetRequest, TargetSelector, adapt_actor_scope,
};
use crate::store::{
    ModelMeshClaimSubject, NewModelMeshTargetRequest, Store, StoredModelMeshIdentityClaim,
    StoredModelMeshTargetRequest, StoredStageRun, StoredWorkflowRun,
};
use crate::{Result, ensure_allowed_flags, required, unix_ms};
use rusqlite::{OptionalExtension, params};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(crate) const MODEL_MESH_COMMAND: &str = "model-mesh";
const MAX_CLI_ITEMS: usize = 256;
const MAX_CLI_VALUE_BYTES: usize = 4096;

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
    let store = open_store(&flags)?;
    let (_, stage) = require_stage_scope(&store, &flags)?;
    let actor_ids = actor_binding_ids_for_stage(&store, &stage.identity.stage_run_id)?;
    require_bounded_items(actor_ids.len(), "Model Mesh actor availability")?;
    let actors = actor_ids
        .iter()
        .map(|id| actor_availability_json(&store, id))
        .collect::<Result<Vec<_>>>()?;
    Ok(json!({
        "action":"availability",
        "stage_run_id":stage.identity.stage_run_id,
        "observation_scope":"DURABLE_LOCAL_STATE_ONLY",
        "live_runtime_discovery":"NOT_PERFORMED",
        "provider_execution":"NOT_PERFORMED",
        "credential_operation":"NOT_PERFORMED",
        "actors":actors,
        "policy_path":"EXPLICIT_POLICY_UNAVAILABLE_FIRST_SLICE"
    }))
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
    let store = open_store(&flags)?;
    target_status_from_store(&store, required(&flags, "target-request-id")?)
}

fn target_status_from_store(store: &Store, target_request_id: &str) -> Result<Value> {
    let stored = store.load_model_mesh_target_request(target_request_id)?;
    let claims = store.list_model_mesh_identity_claims_for_target_request(target_request_id)?;
    require_bounded_items(claims.len(), "Model Mesh identity claims")?;
    let approval = selection_approval_applicability(store, &stored)?;
    let actor =
        store.load_workflow_actor_binding(stored.request.descriptor().actor_binding_id())?;
    let runtime_binding = actor
        .runtime_binding_id
        .as_deref()
        .map(|id| store.load_runtime_session_binding(id))
        .transpose()?;
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
        "durable_runtime_binding":runtime_binding.as_ref().map(runtime_binding_json),
        "target_resolution":{
            "state":"UNAVAILABLE",
            "canonical_resolution":Value::Null,
            "reason":"CURRENT_AUTHORITY_AND_LIVE_RUNTIME_BASIS_UNAVAILABLE"
        },
        "drift":{
            "live_runtime_freshness":"UNAVAILABLE_WITHOUT_DISCOVERY",
            "native_session_truth":runtime_binding.as_ref().map(|binding| binding.ownership.as_str()).unwrap_or("UNAVAILABLE"),
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
    let store = open_store(&flags)?;
    let mut status = target_status_from_store(&store, required(&flags, "target-request-id")?)?;
    let primary = status["blockers"]
        .as_array()
        .and_then(|values| values.first())
        .cloned()
        .unwrap_or_else(|| json!("NONE"));
    status["action"] = json!("why-blocked");
    status["primary_blocker"] = primary;
    Ok(status)
}

fn continuity(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(&flags, &["action", "home", "target-request-id"])?;
    let store = open_store(&flags)?;
    let target_request_id = required(&flags, "target-request-id")?;
    let target = store.load_model_mesh_target_request(target_request_id)?;
    let events = store.list_model_mesh_continuity_events_for_target_request(target_request_id)?;
    require_bounded_items(events.len(), "Model Mesh continuity events")?;
    let values = events
        .iter()
        .map(|event| continuity_event_json(&store, event))
        .collect::<Result<Vec<_>>>()?;
    Ok(json!({
        "action":"continuity",
        "target_request":stored_target_json(&target),
        "events":values,
        "history_rewritten":false,
        "provider_execution":"NOT_PERFORMED"
    }))
}

fn reviewer(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(
        &flags,
        &["action", "home", "target-request-id", "continuity-event-id"],
    )?;
    let store = open_store(&flags)?;
    let target_request_id = required(&flags, "target-request-id")?;
    let event = store.load_model_mesh_continuity_event(required(&flags, "continuity-event-id")?)?;
    if event.target_request_id != target_request_id {
        return Err("reviewer continuity event does not belong to the exact target request".into());
    }
    let target = store.load_model_mesh_target_request(target_request_id)?;
    Ok(json!({
        "action":"reviewer",
        "target_request":stored_target_json(&target),
        "continuity":continuity_event_json(&store, &event)?,
        "candidate_evidence_freshness":{
            "state":"UNAVAILABLE",
            "reason":"CURRENT_CANDIDATE_ARTIFACT_EVIDENCE_INPUTS_NOT_SUPPLIED_TO_T121"
        },
        "reviewer_context_fresh":false,
        "verified":"UNKNOWN",
        "human_accepted":"UNKNOWN",
        "provider_execution":"NOT_PERFORMED"
    }))
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

fn actor_binding_ids_for_stage(store: &Store, stage_run_id: &str) -> Result<Vec<String>> {
    let mut statement = store.connection.prepare(
        "SELECT binding_id FROM workflow_actor_bindings WHERE stage_run_id = ?1 ORDER BY bound_unix_ms, binding_id"
    )?;
    Ok(statement
        .query_map(params![stage_run_id], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?)
}

fn actor_availability_json(store: &Store, binding_id: &str) -> Result<Value> {
    let actor = store.load_workflow_actor_binding(binding_id)?;
    let runtime = actor
        .runtime_binding_id
        .as_deref()
        .map(|id| store.load_runtime_session_binding(id))
        .transpose()?;
    Ok(json!({
        "actor_binding_id":actor.binding_id,
        "winds_session_id":actor.winds_session_id,
        "workflow_continuation":workflow_continuation_label(actor.continuation),
        "runtime_binding":runtime.as_ref().map(runtime_binding_json),
        "target_availability":"DURABLE_BINDING_OBSERVED_NOT_LIVE_EXECUTION_READINESS"
    }))
}

fn runtime_binding_json(binding: &crate::agentic_runtime::RuntimeSessionBinding) -> Value {
    json!({
        "binding_id":binding.binding_id,
        "winds_session_id":binding.session_id,
        "runtime":binding.runtime.as_str(),
        "executable_sha256":binding.executable.sha256,
        "version_state":runtime_version_state_label(binding.version.state),
        "version":binding.version.value,
        "version_source":evidence_source_label(binding.version.source),
        "native_session_id":binding.native_session_id,
        "ownership":binding.ownership.as_str(),
        "live_session_proven":false
    })
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

fn continuity_event_json(
    store: &Store,
    event: &crate::store::StoredModelMeshContinuityEvent,
) -> Result<Value> {
    let source_claims = event
        .source_identity_claim_ids
        .iter()
        .map(|id| store.load_model_mesh_identity_claim(id))
        .collect::<crate::store::Result<Vec<_>>>()?;
    let destination_claims = event
        .destination_identity_claim_ids
        .iter()
        .map(|id| store.load_model_mesh_identity_claim(id))
        .collect::<crate::store::Result<Vec<_>>>()?;
    require_bounded_items(source_claims.len(), "Model Mesh continuity source claims")?;
    require_bounded_items(
        destination_claims.len(),
        "Model Mesh continuity destination claims",
    )?;
    let source_actor = event
        .source_actor_binding_id
        .as_deref()
        .map(|id| actor_availability_json(store, id))
        .transpose()?;
    let destination_actor = event
        .destination_actor_binding_id
        .as_deref()
        .map(|id| actor_availability_json(store, id))
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

fn selection_approval_applicability(
    store: &Store,
    target: &StoredModelMeshTargetRequest,
) -> Result<ApprovalApplicability> {
    let row = store.connection.query_row(
        "SELECT workspace_id, workstream_id, session_id, content_digest, canonical_content_json FROM agentic_delegation_approvals WHERE approval_id = ?1",
        params![target.selection_approval_id],
        |row| Ok((
            row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?,
            row.get::<_, String>(3)?, row.get::<_, String>(4)?,
        )),
    ).optional()?;
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

fn runtime_version_state_label(value: RuntimeVersionState) -> &'static str {
    match value {
        RuntimeVersionState::Observed => "OBSERVED",
        RuntimeVersionState::Unsupported => "UNSUPPORTED",
        RuntimeVersionState::Unavailable => "UNAVAILABLE",
    }
}

fn evidence_source_label(value: EvidenceSource) -> &'static str {
    match value {
        EvidenceSource::WindsLocallyObserved => "WINDS_LOCALLY_OBSERVED",
        EvidenceSource::VendorDeclared => "VENDOR_DECLARED",
        EvidenceSource::CatalogDeclared => "CATALOG_DECLARED",
        EvidenceSource::Unavailable => "UNAVAILABLE",
    }
}

fn workflow_continuation_label(
    value: crate::domain::workflow::WorkflowContinuationClass,
) -> &'static str {
    value.as_db_str()
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
