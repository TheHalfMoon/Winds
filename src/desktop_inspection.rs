use crate::desktop_files::{
    DesktopBoundDockRequest, DesktopRightDockBinding, require_current_binding,
};
use crate::domain::workflow::ArtifactBaselineKind;
use crate::domain::{CandidateBindingStatus, CandidateIdentity, VerificationEvidenceReference};
use crate::store::{Result, Store};
use rusqlite::OptionalExtension;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopEvidenceFreshness {
    Current,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopEvidenceEntry {
    pub run_id: String,
    pub authority: String,
    pub eligibility: String,
    pub candidate_oid: String,
    pub candidate_tree: String,
    pub freshness: DesktopEvidenceFreshness,
    pub trusted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopEvidenceResponse {
    pub binding: DesktopRightDockBinding,
    pub entries: Vec<DesktopEvidenceEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopContextFact {
    pub key: String,
    pub value: String,
    pub source: String,
    pub authority: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopContextResponse {
    pub binding: DesktopRightDockBinding,
    pub facts: Vec<DesktopContextFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopArtifactCandidateState {
    Current,
    Stale,
    NotCandidateBound,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopArtifactEntry {
    pub baseline_id: String,
    pub kind: String,
    pub stable_reference: String,
    pub candidate_oid: Option<String>,
    pub candidate_tree: Option<String>,
    pub candidate_state: DesktopArtifactCandidateState,
    pub provenance: String,
    pub created_unix_ms: i64,
    pub grants_verification_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopArtifactsResponse {
    pub binding: DesktopRightDockBinding,
    pub entries: Vec<DesktopArtifactEntry>,
}

pub fn desktop_right_dock_evidence(
    home: &Path,
    request: DesktopBoundDockRequest,
) -> Result<DesktopEvidenceResponse> {
    let store = Store::open(home)?;
    require_current_binding(&store, &request.binding)?;
    let current_candidate = bound_candidate(&request.binding)?;
    let mut entries = Vec::new();
    if let Some(stage_run_id) = request.binding.stage_run_id.as_deref() {
        for stored in store.list_artifact_baselines_for_stage(stage_run_id)? {
            if stored.identity.kind != ArtifactBaselineKind::WindsVerificationEvidence {
                continue;
            }
            let baseline_candidate = stored
                .identity
                .candidate
                .as_ref()
                .ok_or("verification evidence baseline is missing candidate identity")?;
            let run = store.load_run(&stored.identity.stable_reference)?;
            let reference = VerificationEvidenceReference::from_store(
                &store,
                &stored.identity.stable_reference,
            )
            .map_err(|error| {
                format!("desktop Evidence rejected verification reference: {error}")
            })?;
            if run.candidate_oid != baseline_candidate.oid
                || run.candidate_tree != baseline_candidate.tree
            {
                return Err(
                    "verification evidence baseline does not match persisted run candidate".into(),
                );
            }
            let freshness = match current_candidate.as_ref() {
                Some(current)
                    if reference.applicability(current) == CandidateBindingStatus::Current =>
                {
                    DesktopEvidenceFreshness::Current
                }
                _ => DesktopEvidenceFreshness::Stale,
            };
            entries.push(DesktopEvidenceEntry {
                run_id: run.run_id,
                authority: "WINDS_OBSERVED".to_owned(),
                eligibility: run.eligibility.as_str().to_owned(),
                candidate_oid: run.candidate_oid,
                candidate_tree: run.candidate_tree,
                trusted: freshness == DesktopEvidenceFreshness::Current,
                freshness,
            });
        }
    }
    entries.sort_by(|left, right| left.run_id.cmp(&right.run_id));
    require_current_binding(&store, &request.binding)?;
    Ok(DesktopEvidenceResponse {
        binding: request.binding,
        entries,
    })
}

pub fn desktop_right_dock_context(
    home: &Path,
    request: DesktopBoundDockRequest,
) -> Result<DesktopContextResponse> {
    let store = Store::open(home)?;
    require_current_binding(&store, &request.binding)?;
    let session = store.load_winds_session(&request.binding.session_id)?;
    let workstream = store.load_workstream(&session.workstream_id)?;
    if workstream.workspace_id != request.binding.workspace_id {
        return Err("desktop Context Session/workstream does not match bound Workspace".into());
    }
    let mut facts = vec![
        fact(
            "session",
            &session.session_id,
            "CANONICAL_STORE",
            "CANONICAL_IDENTITY",
        ),
        fact(
            "session_alias",
            &session.display_name,
            "CANONICAL_STORE",
            "PRESENTATION_ALIAS",
        ),
        fact(
            "workstream",
            &workstream.workstream_id,
            "CANONICAL_STORE",
            "CANONICAL_IDENTITY",
        ),
        fact(
            "workstream_name",
            &workstream.display_name,
            "CANONICAL_STORE",
            "CANONICAL_IDENTITY",
        ),
        fact(
            "workspace",
            &workstream.workspace_id,
            "CANONICAL_STORE",
            "CANONICAL_IDENTITY",
        ),
        fact(
            "worktree_root",
            &request.binding.worktree_root,
            "IMMUTABLE_BINDING",
            "CANONICAL_IDENTITY",
        ),
    ];
    match (
        request.binding.workflow_run_id.as_deref(),
        request.binding.stage_run_id.as_deref(),
    ) {
        (None, None) => {}
        (Some(workflow_run_id), Some(stage_run_id)) => {
            let workflow = store.load_workflow_run(workflow_run_id)?;
            let stage = store.load_stage_run(stage_run_id)?;
            if workflow.identity.workspace_id != request.binding.workspace_id
                || workflow.identity.workstream_id != session.workstream_id
                || stage.identity.workflow_run_id != workflow.identity.workflow_run_id
            {
                return Err(
                    "desktop Context workflow/stage does not match bound Session hierarchy".into(),
                );
            }
            facts.push(fact(
                "workflow",
                workflow_run_id,
                "CANONICAL_STORE",
                "CANONICAL_IDENTITY",
            ));
            facts.push(fact(
                "stage",
                stage_run_id,
                "CANONICAL_STORE",
                "CANONICAL_IDENTITY",
            ));
            facts.push(fact(
                "stage_key",
                &stage.identity.stage_key,
                "CANONICAL_STORE",
                "CANONICAL_IDENTITY",
            ));
            facts.push(fact(
                "stage_lifecycle",
                stage.lifecycle_state.as_db_str(),
                "CANONICAL_STORE",
                "WINDS_OBSERVED",
            ));
            let actor_id = store
                .connection
                .query_row(
                    "SELECT binding_id FROM workflow_actor_bindings
                     WHERE stage_run_id = ?1 AND winds_session_id = ?2
                     ORDER BY bound_unix_ms DESC, binding_id DESC LIMIT 1",
                    rusqlite::params![stage_run_id, session.session_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            if let Some(actor_id) = actor_id {
                let actor = store.load_workflow_actor_binding(&actor_id)?;
                facts.push(fact(
                    "continuity",
                    actor.continuation.as_db_str(),
                    "CANONICAL_STORE",
                    "PROOF_LEVEL",
                ));
            }
        }
        _ => return Err("desktop Context binding has incomplete workflow/stage identity".into()),
    }
    add_optional_binding_fact(
        &mut facts,
        "candidate_oid",
        request.binding.candidate_oid.as_deref(),
    );
    add_optional_binding_fact(
        &mut facts,
        "candidate_tree",
        request.binding.candidate_tree.as_deref(),
    );
    add_optional_binding_fact(&mut facts, "head_oid", request.binding.head_oid.as_deref());
    add_optional_binding_fact(&mut facts, "head_tree", request.binding.tree_oid.as_deref());
    require_current_binding(&store, &request.binding)?;
    Ok(DesktopContextResponse {
        binding: request.binding,
        facts,
    })
}

pub fn desktop_right_dock_artifacts(
    home: &Path,
    request: DesktopBoundDockRequest,
) -> Result<DesktopArtifactsResponse> {
    let store = Store::open(home)?;
    require_current_binding(&store, &request.binding)?;
    let current = bound_candidate_pair(&request.binding)?;
    let mut entries = Vec::new();
    if let Some(stage_run_id) = request.binding.stage_run_id.as_deref() {
        for stored in store.list_artifact_baselines_for_stage(stage_run_id)? {
            let candidate_state = match stored.identity.candidate.as_ref() {
                None => DesktopArtifactCandidateState::NotCandidateBound,
                Some(candidate)
                    if current.as_ref()
                        == Some(&(candidate.oid.clone(), candidate.tree.clone())) =>
                {
                    DesktopArtifactCandidateState::Current
                }
                Some(_) => DesktopArtifactCandidateState::Stale,
            };
            entries.push(DesktopArtifactEntry {
                baseline_id: stored.identity.baseline_id,
                kind: stored.identity.kind.as_db_str().to_owned(),
                stable_reference: stored.identity.stable_reference,
                candidate_oid: stored
                    .identity
                    .candidate
                    .as_ref()
                    .map(|value| value.oid.clone()),
                candidate_tree: stored
                    .identity
                    .candidate
                    .as_ref()
                    .map(|value| value.tree.clone()),
                candidate_state,
                provenance: artifact_provenance(stored.identity.kind).to_owned(),
                created_unix_ms: stored.created_unix_ms,
                grants_verification_authority: false,
            });
        }
    }
    entries.sort_by(|left, right| {
        left.created_unix_ms
            .cmp(&right.created_unix_ms)
            .then_with(|| left.baseline_id.cmp(&right.baseline_id))
    });
    require_current_binding(&store, &request.binding)?;
    Ok(DesktopArtifactsResponse {
        binding: request.binding,
        entries,
    })
}

fn bound_candidate(binding: &DesktopRightDockBinding) -> Result<Option<CandidateIdentity>> {
    match (
        binding.candidate_oid.as_deref(),
        binding.candidate_tree.as_deref(),
    ) {
        (None, None) => Ok(None),
        (Some(oid), Some(tree)) => CandidateIdentity::new(oid, tree)
            .map(Some)
            .map_err(Into::into),
        _ => Err("desktop right dock binding has incomplete candidate identity".into()),
    }
}

fn bound_candidate_pair(binding: &DesktopRightDockBinding) -> Result<Option<(String, String)>> {
    match (&binding.candidate_oid, &binding.candidate_tree) {
        (None, None) => Ok(None),
        (Some(oid), Some(tree)) => Ok(Some((oid.clone(), tree.clone()))),
        _ => Err("desktop right dock binding has incomplete candidate identity".into()),
    }
}

fn fact(key: &str, value: &str, source: &str, authority: &str) -> DesktopContextFact {
    DesktopContextFact {
        key: key.to_owned(),
        value: value.to_owned(),
        source: source.to_owned(),
        authority: authority.to_owned(),
    }
}

fn add_optional_binding_fact(facts: &mut Vec<DesktopContextFact>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        facts.push(fact(key, value, "IMMUTABLE_BINDING", "CANONICAL_IDENTITY"));
    }
}

fn artifact_provenance(kind: ArtifactBaselineKind) -> &'static str {
    match kind {
        ArtifactBaselineKind::ExactGitCandidate => "CANONICAL_CANDIDATE_REFERENCE",
        ArtifactBaselineKind::WindsVerificationEvidence => "WINDS_VERIFICATION_EVIDENCE_REFERENCE",
        ArtifactBaselineKind::PriorStageOutput => "CANONICAL_PRIOR_STAGE_OUTPUT_REFERENCE",
        ArtifactBaselineKind::CanonicalDecision => "CANONICAL_DECISION_REFERENCE",
        ArtifactBaselineKind::BoundedBlobArtifact => "CONTENT_ADDRESSED_BLOB_REFERENCE",
    }
}
