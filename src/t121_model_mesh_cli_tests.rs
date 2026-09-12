use crate::agentic_authority::record_model_mesh_approval;
use crate::agentic_runtime::{
    AgentExecutionObservation, AuthReadiness, AuthReadinessEvidence, EvidenceSource,
    RuntimeDiscovery, RuntimeDiscoveryState, RuntimeExecutableIdentity, RuntimeKind,
    RuntimeResumeResolution, RuntimeVersionEvidence, RuntimeVersionState,
};
use crate::domain::workflow::{StageAttemptRelation, StageRunIdentity, WorkflowRunIdentity};
use crate::model_mesh::{
    ContinuityAuthorityClaim, ContinuityClass, CurrentAuthorityTruth, IdentityClaim,
    IdentityDimension, IdentitySourceClass, ModelMeshAuthorityEnvelopeV1,
    ModelMeshContinuityActorV1, ModelMeshContinuityContextInput, ModelMeshTargetDescriptorV1,
    TargetDimension, adapt_actor_scope, build_model_mesh_continuity_context,
};
use crate::model_mesh_cli::{MODEL_MESH_COMMAND, execute};
use crate::store::{
    ModelMeshClaimSubject, NewModelMeshContinuityEvent, NewModelMeshIdentityClaim, NewWindsSession,
    NewWorkspace, NewWorkstream, Store,
};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

fn test_home(name: &str) -> PathBuf {
    let n = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
    let home = std::env::temp_dir().join(format!("winds-t121-{name}-{}-{n}", std::process::id()));
    fs::create_dir(&home).unwrap();
    home
}

fn cleanup(home: PathBuf) {
    if home.exists() {
        fs::remove_dir_all(home).unwrap();
    }
}

fn home_snapshot(home: &Path) -> Vec<(String, String)> {
    fn visit(root: &Path, current: &Path, out: &mut Vec<(String, String)>) {
        let mut entries = fs::read_dir(current)
            .unwrap()
            .map(|entry| entry.unwrap())
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let metadata = entry.metadata().unwrap();
            if metadata.is_dir() {
                out.push((relative.clone(), "DIR".into()));
                visit(root, &path, out);
            } else {
                let bytes = fs::read(&path).unwrap();
                out.push((
                    relative,
                    format!("FILE:{}:{:x}", bytes.len(), Sha256::digest(&bytes)),
                ));
            }
        }
    }
    let mut out = Vec::new();
    visit(home, home, &mut out);
    out
}

fn assert_inspection_preserves_home(
    home: &Path,
    flags: HashMap<String, String>,
) -> serde_json::Value {
    let before = home_snapshot(home);
    let output = execute(flags).unwrap();
    assert_eq!(home_snapshot(home), before);
    output
}

fn runtime_discovery() -> RuntimeDiscovery {
    let executable = if cfg!(windows) {
        PathBuf::from(r"C:\winds-t121-codex.exe")
    } else {
        PathBuf::from("/tmp/winds-t121-codex")
    };
    RuntimeDiscovery {
        runtime: RuntimeKind::Codex,
        state: RuntimeDiscoveryState::Present,
        executable: Some(RuntimeExecutableIdentity {
            observed_path: executable.clone(),
            canonical_path: executable,
            byte_len: 121,
            sha256: "a".repeat(64),
        }),
        version: RuntimeVersionEvidence {
            state: RuntimeVersionState::Observed,
            value: Some("t121-runtime-1".into()),
            source: EvidenceSource::WindsLocallyObserved,
        },
        capabilities: Vec::new(),
        auth_readiness: AuthReadinessEvidence {
            readiness: AuthReadiness::Unknown,
            source: EvidenceSource::Unavailable,
        },
        agent_execution: AgentExecutionObservation::NotPerformed,
    }
}

fn seeded_store(name: &str) -> (PathBuf, Store) {
    let home = test_home(name);
    let store = Store::open(&home).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-1",
                canonical_worktree_root: "/tmp/t121-workspace-1",
                git_common_dir: "/tmp/t121-git-1",
            },
            1,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-1",
                workspace_id: "workspace-1",
                display_name: "T121 workstream",
            },
            2,
        )
        .unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-1",
                workstream_id: "workstream-1",
                display_name: "T121 session",
            },
            3,
        )
        .unwrap();
    store
        .create_workflow_run(
            &WorkflowRunIdentity::new("workflow-1", "workspace-1", "workstream-1").unwrap(),
            4,
        )
        .unwrap();
    store
        .create_stage_run(
            &StageRunIdentity::new("stage-1", "workflow-1", "build", 1, None).unwrap(),
            None,
            5,
        )
        .unwrap();
    store
        .create_runtime_session_binding(
            "runtime-binding-1",
            "session-1",
            &runtime_discovery(),
            Some("native-session-1"),
            6,
        )
        .unwrap();
    let runtime = store
        .load_runtime_session_binding("runtime-binding-1")
        .unwrap();
    store
        .create_actor_binding_from_runtime_resolution(
            "actor-binding-1",
            "stage-1",
            "session-1",
            &RuntimeResumeResolution::Candidate(Box::new(runtime)),
            7,
        )
        .unwrap();
    (home, store)
}

fn target_descriptor(role: &str) -> ModelMeshTargetDescriptorV1 {
    ModelMeshTargetDescriptorV1::new(
        "workspace-1",
        "workstream-1",
        "workflow-1",
        "stage-1",
        "actor-binding-1",
        "session-1",
        role,
        RuntimeKind::Codex,
        TargetDimension::Unspecified,
        TargetDimension::Unspecified,
    )
    .unwrap()
}

fn approve_target(store: &Store, approval_id: &str, role: &str) {
    let envelope =
        ModelMeshAuthorityEnvelopeV1::for_target_selection(&target_descriptor(role)).unwrap();
    record_model_mesh_approval(store, approval_id, &envelope, 8).unwrap();
}

fn flags(home: &Path, action: &str) -> HashMap<String, String> {
    HashMap::from([
        ("action".into(), action.into()),
        ("home".into(), home.to_string_lossy().into_owned()),
        ("workspace-id".into(), "workspace-1".into()),
        ("workstream-id".into(), "workstream-1".into()),
        ("workflow-id".into(), "workflow-1".into()),
        ("stage-id".into(), "stage-1".into()),
    ])
}

fn request_flags(home: &Path, approval_id: &str) -> HashMap<String, String> {
    let mut values = flags(home, "request");
    values.extend([
        ("target-request-id".into(), "target-request-1".into()),
        ("actor-binding-id".into(), "actor-binding-1".into()),
        ("actor-role".into(), "WORKER".into()),
        ("runtime".into(), "CODEX".into()),
        ("approval-id".into(), approval_id.into()),
    ]);
    values
}

fn target_only_flags(home: &Path, action: &str) -> HashMap<String, String> {
    HashMap::from([
        ("action".into(), action.into()),
        ("home".into(), home.to_string_lossy().into_owned()),
        ("target-request-id".into(), "target-request-1".into()),
    ])
}

fn insert_request(home: &Path, store: &Store) {
    approve_target(store, "approval-exact", "WORKER");
    let output = execute(request_flags(home, "approval-exact")).unwrap();
    assert_eq!(output["append_outcome"], "INSERTED");
}

fn add_runtime_claim(store: &mut Store) {
    let claim = IdentityClaim::new(
        IdentityDimension::Runtime,
        Some("CODEX"),
        IdentitySourceClass::WindsLocallyObserved,
        Some("accepted T121 durable runtime observation"),
    )
    .unwrap();
    store
        .create_model_mesh_identity_claim(NewModelMeshIdentityClaim {
            identity_claim_id: "claim-runtime-1",
            subject: ModelMeshClaimSubject::RequestTarget("target-request-1".into()),
            claim: &claim,
            runtime_binding_id: None,
            observed_unix_ms: 9,
        })
        .unwrap();
}

fn add_unproven_event(store: &mut Store) {
    let target = store
        .load_model_mesh_target_request("target-request-1")
        .unwrap();
    let actor = store
        .load_workflow_actor_binding("actor-binding-1")
        .unwrap();
    let runtime = store
        .load_runtime_session_binding("runtime-binding-1")
        .unwrap();
    let stage = store.load_stage_run("stage-1").unwrap();
    let workflow = store.load_workflow_run("workflow-1").unwrap();
    let session = store.load_winds_session("session-1").unwrap();
    let scope = adapt_actor_scope(
        &workflow.identity,
        &stage.identity,
        &session,
        &actor,
        Some(&runtime),
    )
    .unwrap();
    let continuity_actor = ModelMeshContinuityActorV1::from_actor_scope(&scope, &runtime).unwrap();
    let approval =
        ModelMeshAuthorityEnvelopeV1::for_target_selection(target.request.descriptor()).unwrap();
    let context = build_model_mesh_continuity_context(&ModelMeshContinuityContextInput {
        target_request_id: "target-request-1",
        target: target.request.descriptor(),
        continuity_class: ContinuityClass::Unproven,
        source_actor: Some(&continuity_actor),
        destination_actor: Some(&continuity_actor),
        candidate_references: &[],
        artifact_references: &[],
        evidence_references: &[],
        selection_approval: &approval,
        current_authority: CurrentAuthorityTruth::Denied,
        reconstruction_report: None,
        markers: &[],
    })
    .unwrap();
    store
        .create_model_mesh_continuity_event(NewModelMeshContinuityEvent {
            continuity_event_id: "event-unproven-1",
            target_request_id: "target-request-1",
            context: &context,
            current_authority: CurrentAuthorityTruth::Denied,
            authority_claim: ContinuityAuthorityClaim::NoAuthorityClaim,
            authority_approval_id: None,
            source_identity_claim_ids: &[],
            destination_identity_claim_ids: &[],
            created_unix_ms: target.created_unix_ms + 1,
        })
        .unwrap();
}

fn add_outside_stage_actor(store: &Store) {
    store
        .create_stage_run(
            &StageRunIdentity::new("stage-outside", "workflow-1", "outside", 1, None).unwrap(),
            None,
            10,
        )
        .unwrap();
    let runtime = store
        .load_runtime_session_binding("runtime-binding-1")
        .unwrap();
    store
        .create_actor_binding_from_runtime_resolution(
            "actor-binding-outside",
            "stage-outside",
            "session-1",
            &RuntimeResumeResolution::Candidate(Box::new(runtime)),
            11,
        )
        .unwrap();
}

fn corrupt_db(home: &Path, sql: &str) {
    let connection = Connection::open(home.join("winds.db")).unwrap();
    connection.execute_batch(sql).unwrap();
    drop(connection);
}

fn corrupt_target_selector_to_explicit_policy(home: &Path) {
    let connection = Connection::open(home.join("winds.db")).unwrap();
    let no_update_sql: String = connection
        .query_row(
            "SELECT sql FROM sqlite_master
             WHERE type='trigger' AND name='trg_model_mesh_target_requests_no_update'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    connection
        .execute_batch(
            "DROP TRIGGER trg_model_mesh_target_requests_no_update;
             PRAGMA ignore_check_constraints = ON;
             UPDATE model_mesh_target_requests
             SET selector_class = 'EXPLICIT_POLICY'
             WHERE target_request_id = 'target-request-1';
             PRAGMA ignore_check_constraints = OFF;",
        )
        .unwrap();
    connection.execute_batch(&no_update_sql).unwrap();
    drop(connection);
}

fn insert_actor_binding_overflow(home: &Path) {
    let mut connection = Connection::open(home.join("winds.db")).unwrap();
    let tx = connection.transaction().unwrap();
    for index in 2..=257 {
        tx.execute(
            "INSERT INTO workflow_actor_bindings(
                binding_id, stage_run_id, winds_session_id, runtime_binding_id,
                continuation_class, bound_unix_ms
             ) VALUES (?1, 'stage-1', 'session-1', 'runtime-binding-1', 'UNPROVEN', ?2)",
            params![format!("actor-binding-overflow-{index:03}"), 100 + index],
        )
        .unwrap();
    }
    tx.commit().unwrap();
}

fn insert_target_claim_overflow(home: &Path, prefix: &str) {
    let mut connection = Connection::open(home.join("winds.db")).unwrap();
    let tx = connection.transaction().unwrap();
    for index in 1..=257 {
        tx.execute(
            "INSERT INTO model_mesh_identity_claims(
                identity_claim_id, target_request_id, actor_binding_id, claim_subject,
                dimension, normalized_value, source_class, observation_basis,
                runtime_binding_id, observed_unix_ms
             ) VALUES (?1, 'target-request-1', NULL, 'REQUEST_TARGET',
                       'PROVIDER', ?2, 'AGENT_REPORTED', 'bounded overflow fixture', NULL, ?3)",
            params![
                format!("{prefix}-{index:03}"),
                format!("provider-{index:03}"),
                100 + index
            ],
        )
        .unwrap();
    }
    tx.commit().unwrap();
}

fn insert_continuity_event_overflow(home: &Path) {
    let mut connection = Connection::open(home.join("winds.db")).unwrap();
    let authority_trigger_sql: String = connection
        .query_row(
            "SELECT sql FROM sqlite_master
             WHERE type='trigger' AND name='trg_model_mesh_continuity_event_authority_insert'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    connection
        .execute_batch("DROP TRIGGER trg_model_mesh_continuity_event_authority_insert;")
        .unwrap();
    let tx = connection.transaction().unwrap();
    for index in 1..=257 {
        tx.execute(
            "INSERT INTO model_mesh_continuity_events(
                continuity_event_id, target_request_id, source_actor_binding_id,
                destination_actor_binding_id, continuity_class, context_digest,
                completeness_state, continuity_permission_digest, authority_approval_id,
                authority_claim, created_unix_ms
             ) VALUES (?1, 'target-request-1', NULL, NULL, 'UNPROVEN', NULL,
                       'UNAVAILABLE', NULL, NULL, 'NO_AUTHORITY_CLAIM', ?2)",
            params![format!("event-overflow-{index:03}"), 100 + index],
        )
        .unwrap();
    }
    tx.commit().unwrap();
    connection.execute_batch(&authority_trigger_sql).unwrap();
}

fn insert_association_overflow(home: &Path) {
    let mut connection = Connection::open(home.join("winds.db")).unwrap();
    let tx = connection.transaction().unwrap();
    for index in 1..=257 {
        let claim_id = format!("claim-association-overflow-{index:03}");
        tx.execute(
            "INSERT INTO model_mesh_identity_claims(
                identity_claim_id, target_request_id, actor_binding_id, claim_subject,
                dimension, normalized_value, source_class, observation_basis,
                runtime_binding_id, observed_unix_ms
             ) VALUES (?1, NULL, 'actor-binding-1', 'ACTOR',
                       'PROVIDER', ?2, 'AGENT_REPORTED', 'bounded association fixture', NULL, ?3)",
            params![claim_id, format!("provider-{index:03}"), 100 + index],
        )
        .unwrap();
        tx.execute(
            "INSERT INTO model_mesh_continuity_identity_claims(
                continuity_event_id, actor_role, identity_claim_id
             ) VALUES ('event-unproven-1', 'SOURCE', ?1)",
            params![claim_id],
        )
        .unwrap();
    }
    tx.commit().unwrap();
}

fn insert_target_history_overflow(home: &Path) {
    let mut connection = Connection::open(home.join("winds.db")).unwrap();
    let approval_trigger_sql: String = connection
        .query_row(
            "SELECT sql FROM sqlite_master
             WHERE type='trigger' AND name='trg_model_mesh_target_request_approval_insert'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    connection
        .execute_batch("DROP TRIGGER trg_model_mesh_target_request_approval_insert;")
        .unwrap();
    let tx = connection.transaction().unwrap();
    for index in 2..=257 {
        tx.execute(
            "INSERT INTO model_mesh_target_requests(
                target_request_id, stage_run_id, actor_binding_id, actor_role, runtime_kind,
                requested_provider_id, requested_model_id, selector_class,
                target_descriptor_digest, selection_approval_id, created_unix_ms
             )
             SELECT ?1, stage_run_id, actor_binding_id, actor_role, runtime_kind,
                    requested_provider_id, requested_model_id, selector_class,
                    target_descriptor_digest, selection_approval_id, ?2
             FROM model_mesh_target_requests WHERE target_request_id='target-request-1'",
            params![format!("target-request-overflow-{index:03}"), 100 + index],
        )
        .unwrap();
    }
    tx.commit().unwrap();
    connection.execute_batch(&approval_trigger_sql).unwrap();
}

#[test]
fn t121_command_spelling_is_single_and_unknown_or_malformed_input_fails_closed() {
    assert_eq!(MODEL_MESH_COMMAND, "model-mesh");
    assert_eq!(crate::usage().matches("winds model-mesh").count(), 1);
    let (home, store) = seeded_store("input");
    let mut unknown = flags(&home, "not-an-action");
    assert!(
        execute(unknown.clone())
            .unwrap_err()
            .to_string()
            .contains("unknown Model Mesh action")
    );
    unknown.insert("action".into(), "availability".into());
    unknown.insert("surprise".into(), "value".into());
    assert!(
        execute(unknown)
            .unwrap_err()
            .to_string()
            .contains("unknown flag --surprise")
    );
    let mut oversized = flags(&home, "availability");
    oversized.insert("workspace-id".into(), "x".repeat(4097));
    assert!(
        execute(oversized)
            .unwrap_err()
            .to_string()
            .contains("bounded 4096-byte limit")
    );
    let mut control = flags(&home, "availability");
    control.insert("workspace-id".into(), "workspace-1\nforged".into());
    assert!(
        execute(control)
            .unwrap_err()
            .to_string()
            .contains("forbidden control text")
    );
    drop(store);
    cleanup(home);
}

#[test]
fn t121_availability_is_read_only_and_never_claims_live_execution_readiness() {
    let (home, store) = seeded_store("availability");
    assert!(
        store
            .list_model_mesh_target_requests_for_stage("stage-1")
            .unwrap()
            .is_empty()
    );
    drop(store);
    let output = assert_inspection_preserves_home(&home, flags(&home, "availability"));
    assert_eq!(output["observation_scope"], "DURABLE_LOCAL_STATE_ONLY");
    assert_eq!(output["live_runtime_discovery"], "NOT_PERFORMED");
    assert_eq!(output["provider_execution"], "NOT_PERFORMED");
    assert_eq!(
        output["actors"][0]["runtime_binding"]["live_session_proven"],
        false
    );
    let reopened = Store::open(&home).unwrap();
    assert!(
        reopened
            .list_model_mesh_target_requests_for_stage("stage-1")
            .unwrap()
            .is_empty()
    );
    drop(reopened);
    cleanup(home);
}

#[test]
fn t121_request_requires_exact_content_bound_human_approval_and_is_idempotent() {
    let (home, store) = seeded_store("request");
    approve_target(&store, "approval-wrong-role", "PLANNER");
    let error = execute(request_flags(&home, "approval-wrong-role"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("exact content-bound approval"));
    assert!(
        store
            .list_model_mesh_target_requests_for_stage("stage-1")
            .unwrap()
            .is_empty()
    );
    approve_target(&store, "approval-exact", "WORKER");
    let inserted = execute(request_flags(&home, "approval-exact")).unwrap();
    assert_eq!(inserted["append_outcome"], "INSERTED");
    assert_eq!(inserted["selector"], "HUMAN");
    assert_eq!(
        inserted["execution_authority"],
        "NOT_GRANTED_BY_TARGET_SELECTION"
    );
    let replay = execute(request_flags(&home, "approval-exact")).unwrap();
    assert_eq!(replay["append_outcome"], "IDEMPOTENT_NO_CHANGE");
    assert_eq!(
        store
            .list_model_mesh_target_requests_for_stage("stage-1")
            .unwrap()
            .len(),
        1
    );
    let mut unknown_runtime = request_flags(&home, "approval-exact");
    unknown_runtime.insert(
        "target-request-id".into(),
        "target-request-unknown-runtime".into(),
    );
    unknown_runtime.insert("runtime".into(), "GEMINI".into());
    assert!(
        execute(unknown_runtime)
            .unwrap_err()
            .to_string()
            .contains("unknown Model Mesh runtime")
    );
    drop(store);
    cleanup(home);
}

#[test]
fn t121_status_and_why_blocked_preserve_source_truth_without_mutation_or_fake_authority() {
    let (home, mut store) = seeded_store("status");
    insert_request(&home, &store);
    add_runtime_claim(&mut store);
    let history_before = store
        .list_model_mesh_target_requests_for_stage("stage-1")
        .unwrap();
    drop(store);
    let status = assert_inspection_preserves_home(&home, target_only_flags(&home, "status"));
    assert_eq!(status["target_resolution"]["state"], "UNAVAILABLE");
    assert_eq!(
        status["target_resolution"]["canonical_resolution"],
        serde_json::Value::Null
    );
    assert_eq!(status["drift"]["current_authority"], "UNKNOWN");
    assert_eq!(
        status["execution_authority"],
        "UNKNOWN_NOT_GRANTED_BY_INSPECTION"
    );
    assert_eq!(
        status["identity_claims"][0]["source"],
        "WINDS_LOCALLY_OBSERVED"
    );
    let blocked = assert_inspection_preserves_home(&home, target_only_flags(&home, "why-blocked"));
    assert_eq!(
        blocked["primary_blocker"],
        "LIVE_RUNTIME_DISCOVERY_UNAVAILABLE"
    );
    let reopened = Store::open(&home).unwrap();
    assert_eq!(
        reopened
            .list_model_mesh_target_requests_for_stage("stage-1")
            .unwrap(),
        history_before
    );
    drop(reopened);
    cleanup(home);
}

#[test]
fn t121_continuity_and_reviewer_views_are_read_only_and_preserve_unproven_history() {
    let (home, mut store) = seeded_store("continuity");
    insert_request(&home, &store);
    add_unproven_event(&mut store);
    let before = store
        .list_model_mesh_continuity_events_for_target_request("target-request-1")
        .unwrap();
    drop(store);
    let continuity =
        assert_inspection_preserves_home(&home, target_only_flags(&home, "continuity"));
    assert_eq!(continuity["events"][0]["continuity_class"], "UNPROVEN");
    assert_eq!(
        continuity["events"][0]["source_actor_binding_id"],
        "actor-binding-1"
    );
    assert_eq!(
        continuity["events"][0]["destination_actor_binding_id"],
        "actor-binding-1"
    );
    assert_eq!(
        continuity["events"][0]["authority_claim"],
        "NO_AUTHORITY_CLAIM"
    );
    assert_eq!(continuity["history_rewritten"], false);
    let mut reviewer_flags = target_only_flags(&home, "reviewer");
    reviewer_flags.insert("continuity-event-id".into(), "event-unproven-1".into());
    let reviewer = assert_inspection_preserves_home(&home, reviewer_flags);
    assert_eq!(reviewer["reviewer_context_fresh"], false);
    assert_eq!(reviewer["verified"], "UNKNOWN");
    assert_eq!(reviewer["human_accepted"], "UNKNOWN");
    let reviewer_json = serde_json::to_string(&reviewer)
        .unwrap()
        .to_ascii_lowercase();
    assert!(!reviewer_json.contains("winner"));
    assert!(!reviewer_json.contains("recommendation"));
    assert!(!reviewer_json.contains("persuasion"));
    let reopened = Store::open(&home).unwrap();
    assert_eq!(
        reopened
            .list_model_mesh_continuity_events_for_target_request("target-request-1")
            .unwrap(),
        before
    );
    drop(reopened);
    cleanup(home);
}

#[test]
fn t121_inspection_rejects_corrupted_non_human_target_selector_without_mutation() {
    for action in ["status", "continuity", "reviewer"] {
        let (home, mut store) = seeded_store(&format!("selector-corruption-{action}"));
        insert_request(&home, &store);
        if action != "status" {
            add_unproven_event(&mut store);
        }
        drop(store);
        corrupt_target_selector_to_explicit_policy(&home);
        let before = home_snapshot(&home);
        let mut input = target_only_flags(&home, action);
        if action == "reviewer" {
            input.insert("continuity-event-id".into(), "event-unproven-1".into());
        }
        let error = execute(input).unwrap_err().to_string();
        assert!(
            error.contains("stored Model Mesh target selector must be canonical HUMAN"),
            "{action}: {error}"
        );
        assert_eq!(home_snapshot(&home), before, "{action}");
        cleanup(home);
    }
}

#[test]
fn t121_availability_bounds_actor_rows_before_projection_without_mutation() {
    let (home, store) = seeded_store("actor-overflow");
    drop(store);
    insert_actor_binding_overflow(&home);
    let before = home_snapshot(&home);
    let error = execute(flags(&home, "availability"))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("Model Mesh actor availability exceeds the bounded 256-item CLI limit"),
        "{error}"
    );
    assert_eq!(home_snapshot(&home), before);
    cleanup(home);
}

#[test]
fn t121_status_bounds_identity_claim_rows_before_loading_claims_without_mutation() {
    let (home, store) = seeded_store("claim-overflow");
    insert_request(&home, &store);
    drop(store);
    insert_target_claim_overflow(&home, "claim-target-overflow");
    let before = home_snapshot(&home);
    let error = execute(target_only_flags(&home, "status"))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("Model Mesh identity claims exceeds the bounded 256-item CLI limit"),
        "{error}"
    );
    assert_eq!(home_snapshot(&home), before);
    cleanup(home);
}

#[test]
fn t121_continuity_bounds_event_rows_before_loading_events_without_mutation() {
    let (home, store) = seeded_store("event-overflow");
    insert_request(&home, &store);
    drop(store);
    insert_continuity_event_overflow(&home);
    let before = home_snapshot(&home);
    let error = execute(target_only_flags(&home, "continuity"))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("Model Mesh continuity events exceeds the bounded 256-item CLI limit"),
        "{error}"
    );
    assert_eq!(home_snapshot(&home), before);
    cleanup(home);
}

#[test]
fn t121_continuity_bounds_association_rows_before_loading_claims_without_mutation() {
    let (home, mut store) = seeded_store("association-overflow");
    insert_request(&home, &store);
    add_unproven_event(&mut store);
    drop(store);
    insert_association_overflow(&home);
    let before = home_snapshot(&home);
    let error = execute(target_only_flags(&home, "continuity"))
        .unwrap_err()
        .to_string();
    assert!(
        error
            .contains("Model Mesh continuity SOURCE claims exceeds the bounded 256-item CLI limit"),
        "{error}"
    );
    assert_eq!(home_snapshot(&home), before);
    cleanup(home);
}

#[test]
fn t121_request_bounds_target_history_before_any_new_append() {
    let (home, store) = seeded_store("target-history-overflow");
    insert_request(&home, &store);
    drop(store);
    insert_target_history_overflow(&home);
    let before = home_snapshot(&home);
    let mut input = request_flags(&home, "approval-exact");
    input.insert("target-request-id".into(), "target-request-new".into());
    let error = execute(input).unwrap_err().to_string();
    assert!(
        error.contains("Model Mesh target history exceeds the bounded 256-item CLI limit"),
        "{error}"
    );
    assert_eq!(home_snapshot(&home), before);
    cleanup(home);
}

#[test]
fn t121_inspection_rejects_missing_canonical_trigger_without_mutation() {
    let (home, store) = seeded_store("missing-trigger");
    drop(store);
    corrupt_db(
        &home,
        "DROP TRIGGER trg_model_mesh_continuity_event_lineage_insert;",
    );
    let before = home_snapshot(&home);
    let error = execute(flags(&home, "availability"))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("model_mesh schema object inventory mismatch"),
        "{error}"
    );
    assert_eq!(home_snapshot(&home), before);
    cleanup(home);
}

#[test]
fn t121_inspection_rejects_modified_canonical_trigger_without_mutation() {
    let (home, store) = seeded_store("modified-trigger");
    drop(store);
    corrupt_db(
        &home,
        "DROP TRIGGER trg_workflow_runs_no_delete;
         CREATE TRIGGER trg_workflow_runs_no_delete
         BEFORE DELETE ON workflow_runs
         BEGIN
             SELECT 1;
         END;",
    );
    let before = home_snapshot(&home);
    let error = execute(flags(&home, "availability"))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("workflow schema object definition mismatch: trg_workflow_runs_no_delete"),
        "{error}"
    );
    assert_eq!(home_snapshot(&home), before);
    cleanup(home);
}

#[test]
fn t121_continuity_rejects_actor_outside_target_lineage_even_with_valid_schema() {
    let (home, mut store) = seeded_store("outside-lineage");
    insert_request(&home, &store);
    add_unproven_event(&mut store);
    add_outside_stage_actor(&store);
    drop(store);

    let connection = Connection::open(home.join("winds.db")).unwrap();
    let no_update_sql: String = connection
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='trigger' AND name='trg_model_mesh_continuity_events_no_update'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    connection
        .execute_batch("DROP TRIGGER trg_model_mesh_continuity_events_no_update;")
        .unwrap();
    connection
        .execute(
            "UPDATE model_mesh_continuity_events
             SET source_actor_binding_id='actor-binding-outside'
             WHERE continuity_event_id='event-unproven-1'",
            [],
        )
        .unwrap();
    connection.execute_batch(&no_update_sql).unwrap();
    drop(connection);

    let before = home_snapshot(&home);
    let error = execute(target_only_flags(&home, "continuity"))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("source actor is outside canonical target StageRun lineage"),
        "{error}"
    );
    assert_eq!(home_snapshot(&home), before);
    cleanup(home);
}

#[test]
fn t121_continuity_and_reviewer_reject_cyclic_stage_lineage_without_mutation() {
    let (home, mut store) = seeded_store("cyclic-lineage");
    store
        .create_stage_run(
            &StageRunIdentity::new("stage-2", "workflow-1", "build", 2, Some("stage-1")).unwrap(),
            Some(StageAttemptRelation::Reconstruction),
            10,
        )
        .unwrap();
    insert_request(&home, &store);
    add_unproven_event(&mut store);
    drop(store);

    let connection = Connection::open(home.join("winds.db")).unwrap();
    let identity_trigger_sql: String = connection
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='trigger' AND name='trg_workflow_stage_runs_identity_update'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    connection
        .execute_batch("DROP TRIGGER trg_workflow_stage_runs_identity_update;")
        .unwrap();
    connection
        .execute(
            "UPDATE workflow_stage_runs
             SET attempt_ordinal=3,
                 predecessor_stage_run_id='stage-2',
                 relation_kind='RECONSTRUCTION_OF'
             WHERE stage_run_id='stage-1'",
            [],
        )
        .unwrap();
    connection.execute_batch(&identity_trigger_sql).unwrap();
    drop(connection);

    let before = home_snapshot(&home);
    for action in ["continuity", "reviewer"] {
        let mut input = target_only_flags(&home, action);
        if action == "reviewer" {
            input.insert("continuity-event-id".into(), "event-unproven-1".into());
        }
        let error = execute(input).unwrap_err().to_string();
        assert!(
            error.contains("canonical workflow/stage attempt ordering")
                || error.contains("predecessor lineage is cyclic"),
            "{action}: {error}"
        );
        assert_eq!(
            home_snapshot(&home),
            before,
            "{action} mutated the corrupted lineage fixture"
        );
    }
    cleanup(home);
}

#[test]
fn t121_inspection_refuses_active_sqlite_writer_without_touching_sidecars() {
    let (home, store) = seeded_store("active-writer");
    let before = home_snapshot(&home);
    assert!(
        before
            .iter()
            .any(|(path, _)| path.ends_with("winds.db-wal"))
            || before
                .iter()
                .any(|(path, _)| path.ends_with("winds.db-shm")),
        "fixture must expose an active SQLite sidecar"
    );
    let error = execute(flags(&home, "availability"))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("refuses an active SQLite sidecar"),
        "{error}"
    );
    assert_eq!(home_snapshot(&home), before);
    drop(store);
    cleanup(home);
}

#[test]
fn t121_all_inspection_actions_refuse_uninitialized_home_without_mutation() {
    for action in [
        "availability",
        "status",
        "why-blocked",
        "continuity",
        "reviewer",
    ] {
        let home = test_home(&format!("uninitialized-{action}"));
        let before = home_snapshot(&home);
        let input = HashMap::from([
            ("action".into(), action.into()),
            ("home".into(), home.to_string_lossy().into_owned()),
        ]);
        let error = execute(input).unwrap_err().to_string();
        assert!(
            error.contains("existing initialized winds.db"),
            "{action}: {error}"
        );
        assert_eq!(home_snapshot(&home), before, "{action} mutated empty home");
        cleanup(home);
    }
}

#[test]
fn t121_all_inspection_actions_refuse_nonexistent_home_without_creating_state() {
    for action in [
        "availability",
        "status",
        "why-blocked",
        "continuity",
        "reviewer",
    ] {
        let home = std::env::temp_dir().join(format!(
            "winds-t121-missing-{action}-{}-{}",
            std::process::id(),
            NEXT_HOME.fetch_add(1, Ordering::Relaxed)
        ));
        assert!(!home.exists());
        let input = HashMap::from([
            ("action".into(), action.into()),
            ("home".into(), home.to_string_lossy().into_owned()),
        ]);
        let error = execute(input).unwrap_err().to_string();
        assert!(
            error.contains("existing initialized --home"),
            "{action}: {error}"
        );
        assert!(
            !home.exists(),
            "{action} created the nonexistent inspection home"
        );
    }
}
