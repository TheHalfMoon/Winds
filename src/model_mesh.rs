use crate::agentic_runtime::{
    AuthReadiness, EvidenceSource, RuntimeBindingOwnership, RuntimeDiscovery,
    RuntimeDiscoveryState, RuntimeKind, RuntimeSessionBinding, runtime_binding_matches_discovery,
};
use crate::domain::WindsSessionRecord;
use crate::domain::workflow::{
    ArtifactBaselineIdentity, ArtifactBaselineRequirement, BaselineEvaluation, BaselineFreshness,
    StageRunIdentity, WorkflowRunIdentity, evaluate_artifact_baseline_requirement,
};
use crate::store::StoredWorkflowActorBinding;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

const MAX_SCOPE_ID_BYTES: usize = 256;
const MAX_TARGET_ID_BYTES: usize = 256;
const MAX_ACTOR_ROLE_BYTES: usize = 64;
const MAX_OBSERVATION_BASIS_BYTES: usize = 512;
const SHA256_HEX_BYTES: usize = 64;

pub(crate) type ModelMeshResult<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ExactProviderId(String);

impl ExactProviderId {
    pub(crate) fn new(value: &str) -> ModelMeshResult<Self> {
        Ok(Self(normalize_exact_id(value, "provider id")?))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ExactModelId(String);

impl ExactModelId {
    pub(crate) fn new(value: &str) -> ModelMeshResult<Self> {
        Ok(Self(normalize_exact_id(value, "model id")?))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TargetDimension<T> {
    Unspecified,
    Exact(T),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TargetSelector {
    Human,
    ExplicitPolicy,
}

impl TargetSelector {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Human => "HUMAN",
            Self::ExplicitPolicy => "EXPLICIT_POLICY",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "HUMAN" => Some(Self::Human),
            "EXPLICIT_POLICY" => Some(Self::ExplicitPolicy),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelMeshTargetDescriptorV1 {
    workspace_id: String,
    workstream_id: String,
    workflow_run_id: String,
    stage_run_id: String,
    actor_binding_id: String,
    winds_session_id: String,
    actor_role: String,
    runtime: RuntimeKind,
    provider: TargetDimension<ExactProviderId>,
    model: TargetDimension<ExactModelId>,
}

impl ModelMeshTargetDescriptorV1 {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        workspace_id: &str,
        workstream_id: &str,
        workflow_run_id: &str,
        stage_run_id: &str,
        actor_binding_id: &str,
        winds_session_id: &str,
        actor_role: &str,
        runtime: RuntimeKind,
        provider: TargetDimension<ExactProviderId>,
        model: TargetDimension<ExactModelId>,
    ) -> ModelMeshResult<Self> {
        Ok(Self {
            workspace_id: normalize_scope(workspace_id, "workspace id")?,
            workstream_id: normalize_scope(workstream_id, "workstream id")?,
            workflow_run_id: normalize_scope(workflow_run_id, "workflow run id")?,
            stage_run_id: normalize_scope(stage_run_id, "stage run id")?,
            actor_binding_id: normalize_scope(actor_binding_id, "actor binding id")?,
            winds_session_id: normalize_scope(winds_session_id, "Winds session id")?,
            actor_role: normalize_actor_role(actor_role)?,
            runtime,
            provider,
            model,
        })
    }

    pub(crate) fn canonical_json(&self) -> ModelMeshResult<String> {
        let payload = CanonicalTargetDescriptorV1 {
            schema_version: 1,
            workspace_id: &self.workspace_id,
            workstream_id: &self.workstream_id,
            workflow_run_id: &self.workflow_run_id,
            stage_run_id: &self.stage_run_id,
            actor_binding_id: &self.actor_binding_id,
            winds_session_id: &self.winds_session_id,
            actor_role: &self.actor_role,
            runtime: self.runtime.as_str(),
            provider: canonical_provider(&self.provider),
            model: canonical_model(&self.model),
        };
        serde_json::to_string(&payload)
            .map_err(|error| format!("Model Mesh target serialization failed: {error}"))
    }

    pub(crate) fn digest(&self) -> ModelMeshResult<String> {
        Ok(sha256_hex(self.canonical_json()?.as_bytes()))
    }

    pub(crate) fn workspace_id(&self) -> &str {
        &self.workspace_id
    }

    pub(crate) fn workstream_id(&self) -> &str {
        &self.workstream_id
    }

    pub(crate) fn workflow_run_id(&self) -> &str {
        &self.workflow_run_id
    }

    pub(crate) fn stage_run_id(&self) -> &str {
        &self.stage_run_id
    }

    pub(crate) fn actor_binding_id(&self) -> &str {
        &self.actor_binding_id
    }

    pub(crate) fn winds_session_id(&self) -> &str {
        &self.winds_session_id
    }

    pub(crate) fn actor_role(&self) -> &str {
        &self.actor_role
    }

    pub(crate) fn runtime(&self) -> RuntimeKind {
        self.runtime
    }

    pub(crate) fn provider(&self) -> &TargetDimension<ExactProviderId> {
        &self.provider
    }

    pub(crate) fn model(&self) -> &TargetDimension<ExactModelId> {
        &self.model
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelMeshRuntimeObservation {
    pub(crate) runtime_claim: IdentityClaim,
    pub(crate) provider_claim: Option<IdentityClaim>,
    pub(crate) model_claim: Option<IdentityClaim>,
    pub(crate) authentication: AuthenticationTruth,
    pub(crate) binding_stale: bool,
}

pub(crate) fn adapt_runtime_truth(
    discovery: &RuntimeDiscovery,
    binding: Option<&RuntimeSessionBinding>,
) -> ModelMeshResult<ModelMeshRuntimeObservation> {
    let (runtime_claim, binding_stale) = match discovery.state {
        RuntimeDiscoveryState::Present => {
            if discovery.executable.is_none()
                || discovery.version.source != EvidenceSource::WindsLocallyObserved
            {
                return Err(
                    "present runtime discovery lacks accepted Winds-local identity evidence".into(),
                );
            }
            let stale = binding
                .is_some_and(|binding| !runtime_binding_matches_discovery(binding, discovery));
            (
                IdentityClaim::new(
                    IdentityDimension::Runtime,
                    Some(discovery.runtime.as_str()),
                    IdentitySourceClass::WindsLocallyObserved,
                    Some("accepted runtime discovery"),
                )?,
                stale,
            )
        }
        RuntimeDiscoveryState::Unavailable => (
            IdentityClaim::new(
                IdentityDimension::Runtime,
                None,
                IdentitySourceClass::Unavailable,
                Some("runtime discovery unavailable"),
            )?,
            binding.is_some(),
        ),
        RuntimeDiscoveryState::UnsupportedVersion | RuntimeDiscoveryState::VersionUnavailable => (
            IdentityClaim::new(
                IdentityDimension::Runtime,
                Some(discovery.runtime.as_str()),
                IdentitySourceClass::WindsLocallyObserved,
                Some("runtime discovery is not currently qualified"),
            )?,
            true,
        ),
    };

    let authentication = match discovery.auth_readiness.readiness {
        AuthReadiness::Unknown => AuthenticationTruth::Unknown,
    };

    Ok(ModelMeshRuntimeObservation {
        runtime_claim,
        // Canonical T115 has no accepted production provider/model observation path.
        // Runtime kind, executable identity, version text, and native session ids never fill these.
        provider_claim: None,
        model_claim: None,
        authentication,
        binding_stale,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelMeshActorScope {
    workspace_id: String,
    workstream_id: String,
    workflow_run_id: String,
    stage_run_id: String,
    actor_binding_id: String,
    winds_session_id: String,
    bound_runtime: Option<RuntimeKind>,
}

impl ModelMeshActorScope {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn target_descriptor(
        &self,
        actor_role: &str,
        runtime: RuntimeKind,
        provider: TargetDimension<ExactProviderId>,
        model: TargetDimension<ExactModelId>,
    ) -> ModelMeshResult<ModelMeshTargetDescriptorV1> {
        if self.bound_runtime.is_some_and(|bound| bound != runtime) {
            return Err(
                "Model Mesh target runtime does not match the actor runtime binding".into(),
            );
        }
        ModelMeshTargetDescriptorV1::new(
            &self.workspace_id,
            &self.workstream_id,
            &self.workflow_run_id,
            &self.stage_run_id,
            &self.actor_binding_id,
            &self.winds_session_id,
            actor_role,
            runtime,
            provider,
            model,
        )
    }
}

pub(crate) fn adapt_actor_scope(
    workflow: &WorkflowRunIdentity,
    stage: &StageRunIdentity,
    session: &WindsSessionRecord,
    actor: &StoredWorkflowActorBinding,
    runtime_binding: Option<&RuntimeSessionBinding>,
) -> ModelMeshResult<ModelMeshActorScope> {
    if stage.workflow_run_id != workflow.workflow_run_id {
        return Err("Model Mesh stage does not belong to the workflow".into());
    }
    if actor.stage_run_id != stage.stage_run_id {
        return Err("Model Mesh actor binding does not belong to the stage".into());
    }
    if actor.winds_session_id != session.session_id {
        return Err("Model Mesh actor binding does not belong to the Winds session".into());
    }
    if session.workstream_id != workflow.workstream_id {
        return Err("Model Mesh Winds session does not belong to the workflow workstream".into());
    }
    let bound_runtime = match (actor.runtime_binding_id.as_deref(), runtime_binding) {
        (Some(expected), Some(binding))
            if expected == binding.binding_id && binding.session_id == session.session_id =>
        {
            Some(binding.runtime)
        }
        (None, None) => None,
        _ => return Err("Model Mesh actor runtime binding context is inconsistent".into()),
    };

    Ok(ModelMeshActorScope {
        workspace_id: normalize_scope(&workflow.workspace_id, "workspace id")?,
        workstream_id: normalize_scope(&workflow.workstream_id, "workstream id")?,
        workflow_run_id: normalize_scope(&workflow.workflow_run_id, "workflow run id")?,
        stage_run_id: normalize_scope(&stage.stage_run_id, "stage run id")?,
        actor_binding_id: normalize_scope(&actor.binding_id, "actor binding id")?,
        winds_session_id: normalize_scope(&session.session_id, "Winds session id")?,
        bound_runtime,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TargetRequest {
    descriptor: ModelMeshTargetDescriptorV1,
    target_descriptor_digest: String,
    selector: TargetSelector,
}

impl TargetRequest {
    pub(crate) fn new(
        descriptor: ModelMeshTargetDescriptorV1,
        selector: TargetSelector,
    ) -> ModelMeshResult<Self> {
        let target_descriptor_digest = descriptor.digest()?;
        Ok(Self {
            descriptor,
            target_descriptor_digest,
            selector,
        })
    }

    pub(crate) fn descriptor(&self) -> &ModelMeshTargetDescriptorV1 {
        &self.descriptor
    }

    pub(crate) fn target_descriptor_digest(&self) -> &str {
        &self.target_descriptor_digest
    }

    pub(crate) fn selector(&self) -> TargetSelector {
        self.selector
    }

    fn descriptor_digest_matches(&self) -> bool {
        self.descriptor
            .digest()
            .is_ok_and(|digest| digest == self.target_descriptor_digest)
    }

    #[cfg(test)]
    pub(crate) fn from_untrusted_parts_for_test(
        descriptor: ModelMeshTargetDescriptorV1,
        target_descriptor_digest: &str,
        selector: TargetSelector,
    ) -> ModelMeshResult<Self> {
        Ok(Self {
            descriptor,
            target_descriptor_digest: normalize_sha256(
                target_descriptor_digest,
                "target descriptor digest",
            )?,
            selector,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum IdentityDimension {
    Runtime,
    Provider,
    Model,
    NativeSession,
}

impl IdentityDimension {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Runtime => "RUNTIME",
            Self::Provider => "PROVIDER",
            Self::Model => "MODEL",
            Self::NativeSession => "NATIVE_SESSION",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "RUNTIME" => Some(Self::Runtime),
            "PROVIDER" => Some(Self::Provider),
            "MODEL" => Some(Self::Model),
            "NATIVE_SESSION" => Some(Self::NativeSession),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum IdentitySourceClass {
    WindsLocallyObserved,
    VendorDeclared,
    CatalogDeclared,
    AgentReported,
    HumanDecided,
    Unavailable,
}

impl IdentitySourceClass {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::WindsLocallyObserved => "WINDS_LOCALLY_OBSERVED",
            Self::VendorDeclared => "VENDOR_DECLARED",
            Self::CatalogDeclared => "CATALOG_DECLARED",
            Self::AgentReported => "AGENT_REPORTED",
            Self::HumanDecided => "HUMAN_DECIDED",
            Self::Unavailable => "UNAVAILABLE",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "WINDS_LOCALLY_OBSERVED" => Some(Self::WindsLocallyObserved),
            "VENDOR_DECLARED" => Some(Self::VendorDeclared),
            "CATALOG_DECLARED" => Some(Self::CatalogDeclared),
            "AGENT_REPORTED" => Some(Self::AgentReported),
            "HUMAN_DECIDED" => Some(Self::HumanDecided),
            "UNAVAILABLE" => Some(Self::Unavailable),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IdentityClaim {
    pub(crate) dimension: IdentityDimension,
    pub(crate) value: Option<String>,
    pub(crate) source: IdentitySourceClass,
    pub(crate) observation_basis: Option<String>,
}

impl IdentityClaim {
    pub(crate) fn new(
        dimension: IdentityDimension,
        value: Option<&str>,
        source: IdentitySourceClass,
        observation_basis: Option<&str>,
    ) -> ModelMeshResult<Self> {
        let value = value
            .map(|value| normalize_claim_value(value, "identity claim value"))
            .transpose()?;
        let observation_basis = observation_basis
            .map(normalize_observation_basis)
            .transpose()?;
        if source == IdentitySourceClass::Unavailable && value.is_some() {
            return Err("UNAVAILABLE identity claim cannot carry a value".into());
        }
        Ok(Self {
            dimension,
            value,
            source,
            observation_basis,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthenticationTruth {
    Ready,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CapabilityTruth {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ApprovalApplicability {
    Exact,
    Missing,
    Mismatch,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CurrentAuthorityTruth {
    Allowed,
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceRequirements {
    pub(crate) runtime: IdentitySourceClass,
    pub(crate) provider: IdentitySourceClass,
    pub(crate) model: IdentitySourceClass,
}

impl SourceRequirements {
    pub(crate) const fn winds_observed() -> Self {
        Self {
            runtime: IdentitySourceClass::WindsLocallyObserved,
            provider: IdentitySourceClass::WindsLocallyObserved,
            model: IdentitySourceClass::WindsLocallyObserved,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TargetResolution {
    ExactMatch,
    Unknown,
    Unavailable,
    Ambiguous,
    Conflict,
    Stale,
    AuthenticationUnknown,
    CapabilityUnavailable,
    AuthorityDenied,
    PolicyNotAuthorized,
}

pub(crate) struct TargetResolverInput<'a> {
    pub(crate) request: &'a TargetRequest,
    pub(crate) claims: &'a [IdentityClaim],
    pub(crate) source_requirements: SourceRequirements,
    pub(crate) stale: bool,
    pub(crate) authentication: AuthenticationTruth,
    pub(crate) capability: CapabilityTruth,
    pub(crate) approval: ApprovalApplicability,
    pub(crate) current_authority: CurrentAuthorityTruth,
}

pub(crate) fn resolve_target(input: &TargetResolverInput<'_>) -> TargetResolution {
    if !input.request.descriptor_digest_matches()
        || input.stale
        || input.approval == ApprovalApplicability::Stale
    {
        return TargetResolution::Stale;
    }
    if input.request.selector() == TargetSelector::ExplicitPolicy {
        return TargetResolution::PolicyNotAuthorized;
    }

    for (dimension, expected, required_source) in
        expected_identity_dimensions(&input.request.descriptor, input.source_requirements)
    {
        let resolution = resolve_identity_dimension(
            input.claims,
            dimension,
            expected.as_deref(),
            required_source,
        );
        if resolution != TargetResolution::ExactMatch {
            return resolution;
        }
    }

    if input.capability == CapabilityTruth::Unavailable {
        return TargetResolution::CapabilityUnavailable;
    }
    if input.authentication == AuthenticationTruth::Unknown {
        return TargetResolution::AuthenticationUnknown;
    }
    if matches!(
        input.approval,
        ApprovalApplicability::Missing | ApprovalApplicability::Mismatch
    ) || input.current_authority == CurrentAuthorityTruth::Denied
    {
        return TargetResolution::AuthorityDenied;
    }
    TargetResolution::ExactMatch
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContinuityClass {
    NativeResume,
    Reconstructed,
    Reassigned,
    Handoff,
    OwnershipLost,
    Unavailable,
    Unproven,
}

impl ContinuityClass {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::NativeResume => "NATIVE_RESUME",
            Self::Reconstructed => "RECONSTRUCTED",
            Self::Reassigned => "REASSIGNED",
            Self::Handoff => "HANDOFF",
            Self::OwnershipLost => "OWNERSHIP_LOST",
            Self::Unavailable => "UNAVAILABLE",
            Self::Unproven => "UNPROVEN",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "NATIVE_RESUME" => Some(Self::NativeResume),
            "RECONSTRUCTED" => Some(Self::Reconstructed),
            "REASSIGNED" => Some(Self::Reassigned),
            "HANDOFF" => Some(Self::Handoff),
            "OWNERSHIP_LOST" => Some(Self::OwnershipLost),
            "UNAVAILABLE" => Some(Self::Unavailable),
            "UNPROVEN" => Some(Self::Unproven),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ContextDigest {
    NotApplicable,
    Exact(String),
}

impl ContextDigest {
    pub(crate) fn exact(value: &str) -> ModelMeshResult<Self> {
        Ok(Self::Exact(normalize_sha256(value, "context digest")?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelMeshContinuityPermissionDescriptorV1 {
    workflow_run_id: String,
    stage_run_id: String,
    source_actor_binding_id: Option<String>,
    destination_actor_binding_id: Option<String>,
    continuity_class: ContinuityClass,
    target_descriptor_digest: String,
    context_digest: ContextDigest,
}

impl ModelMeshContinuityPermissionDescriptorV1 {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        workflow_run_id: &str,
        stage_run_id: &str,
        source_actor_binding_id: Option<&str>,
        destination_actor_binding_id: Option<&str>,
        continuity_class: ContinuityClass,
        target_descriptor_digest: &str,
        context_digest: ContextDigest,
    ) -> ModelMeshResult<Self> {
        Ok(Self {
            workflow_run_id: normalize_scope(workflow_run_id, "workflow run id")?,
            stage_run_id: normalize_scope(stage_run_id, "stage run id")?,
            source_actor_binding_id: source_actor_binding_id
                .map(|value| normalize_scope(value, "source actor binding id"))
                .transpose()?,
            destination_actor_binding_id: destination_actor_binding_id
                .map(|value| normalize_scope(value, "destination actor binding id"))
                .transpose()?,
            continuity_class,
            target_descriptor_digest: normalize_sha256(
                target_descriptor_digest,
                "target descriptor digest",
            )?,
            context_digest,
        })
    }

    pub(crate) fn canonical_json(&self) -> ModelMeshResult<String> {
        let payload = CanonicalContinuityPermissionDescriptorV1 {
            schema_version: 1,
            workflow_run_id: &self.workflow_run_id,
            stage_run_id: &self.stage_run_id,
            source_actor_binding_id: self.source_actor_binding_id.as_deref(),
            destination_actor_binding_id: self.destination_actor_binding_id.as_deref(),
            continuity_class: self.continuity_class.as_str(),
            target_descriptor_digest: &self.target_descriptor_digest,
            context_digest: canonical_context_digest(&self.context_digest),
        };
        serde_json::to_string(&payload)
            .map_err(|error| format!("Model Mesh continuity serialization failed: {error}"))
    }

    pub(crate) fn digest(&self) -> ModelMeshResult<String> {
        Ok(sha256_hex(self.canonical_json()?.as_bytes()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModelMeshAuthorityPurpose {
    TargetSelection,
    ContinuityPermission,
}

impl ModelMeshAuthorityPurpose {
    fn as_str(self) -> &'static str {
        match self {
            Self::TargetSelection => "TARGET_SELECTION",
            Self::ContinuityPermission => "CONTINUITY_PERMISSION",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelMeshAuthorityEnvelopeV1 {
    purpose: ModelMeshAuthorityPurpose,
    workspace_id: String,
    workstream_id: String,
    session_id: String,
    workflow_run_id: String,
    stage_run_id: String,
    actor_binding_id: String,
    actor_role: String,
    target_descriptor_digest: String,
    continuity_permission_digest: Option<String>,
}

impl ModelMeshAuthorityEnvelopeV1 {
    pub(crate) fn for_target_selection(
        target: &ModelMeshTargetDescriptorV1,
    ) -> ModelMeshResult<Self> {
        let target_descriptor_digest = target.digest()?;
        Self::from_parts(
            ModelMeshAuthorityPurpose::TargetSelection,
            target.workspace_id(),
            target.workstream_id(),
            target.winds_session_id(),
            target.workflow_run_id(),
            target.stage_run_id(),
            target.actor_binding_id(),
            target.actor_role(),
            &target_descriptor_digest,
            None,
        )
    }

    pub(crate) fn for_continuity_permission(
        target: &ModelMeshTargetDescriptorV1,
        permission: &ModelMeshContinuityPermissionDescriptorV1,
    ) -> ModelMeshResult<Self> {
        let target_descriptor_digest = target.digest()?;
        if permission.workflow_run_id != target.workflow_run_id() {
            return Err("continuity permission workflow does not match target descriptor".into());
        }
        if permission.stage_run_id != target.stage_run_id() {
            return Err("continuity permission stage does not match target descriptor".into());
        }
        if permission.target_descriptor_digest != target_descriptor_digest {
            return Err(
                "continuity permission target digest does not match target descriptor".into(),
            );
        }
        let continuity_permission_digest = permission.digest()?;
        Self::from_parts(
            ModelMeshAuthorityPurpose::ContinuityPermission,
            target.workspace_id(),
            target.workstream_id(),
            target.winds_session_id(),
            target.workflow_run_id(),
            target.stage_run_id(),
            target.actor_binding_id(),
            target.actor_role(),
            &target_descriptor_digest,
            Some(&continuity_permission_digest),
        )
    }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        purpose: ModelMeshAuthorityPurpose,
        workspace_id: &str,
        workstream_id: &str,
        session_id: &str,
        workflow_run_id: &str,
        stage_run_id: &str,
        actor_binding_id: &str,
        actor_role: &str,
        target_descriptor_digest: &str,
        continuity_permission_digest: Option<&str>,
    ) -> ModelMeshResult<Self> {
        // T114 regression fixtures predate the safe split constructors. This compatibility seam is
        // test-only so production callers cannot create an unbound continuity authority envelope.
        Self::from_parts(
            purpose,
            workspace_id,
            workstream_id,
            session_id,
            workflow_run_id,
            stage_run_id,
            actor_binding_id,
            actor_role,
            target_descriptor_digest,
            continuity_permission_digest,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn from_parts(
        purpose: ModelMeshAuthorityPurpose,
        workspace_id: &str,
        workstream_id: &str,
        session_id: &str,
        workflow_run_id: &str,
        stage_run_id: &str,
        actor_binding_id: &str,
        actor_role: &str,
        target_descriptor_digest: &str,
        continuity_permission_digest: Option<&str>,
    ) -> ModelMeshResult<Self> {
        let continuity_permission_digest = continuity_permission_digest
            .map(|value| normalize_sha256(value, "continuity permission digest"))
            .transpose()?;
        match purpose {
            ModelMeshAuthorityPurpose::TargetSelection
                if continuity_permission_digest.is_some() =>
            {
                return Err(
                    "TARGET_SELECTION authority cannot carry a continuity permission digest".into(),
                );
            }
            ModelMeshAuthorityPurpose::ContinuityPermission
                if continuity_permission_digest.is_none() =>
            {
                return Err(
                    "CONTINUITY_PERMISSION authority requires its exact permission digest".into(),
                );
            }
            _ => {}
        }
        Ok(Self {
            purpose,
            workspace_id: normalize_scope(workspace_id, "workspace id")?,
            workstream_id: normalize_scope(workstream_id, "workstream id")?,
            session_id: normalize_scope(session_id, "Winds session id")?,
            workflow_run_id: normalize_scope(workflow_run_id, "workflow run id")?,
            stage_run_id: normalize_scope(stage_run_id, "stage run id")?,
            actor_binding_id: normalize_scope(actor_binding_id, "actor binding id")?,
            actor_role: normalize_actor_role(actor_role)?,
            target_descriptor_digest: normalize_sha256(
                target_descriptor_digest,
                "target descriptor digest",
            )?,
            continuity_permission_digest,
        })
    }

    pub(crate) fn canonical_json(&self) -> ModelMeshResult<String> {
        let payload = CanonicalAuthorityEnvelopeV1 {
            schema_version: 1,
            purpose: self.purpose.as_str(),
            workspace_id: &self.workspace_id,
            workstream_id: &self.workstream_id,
            session_id: &self.session_id,
            workflow_run_id: &self.workflow_run_id,
            stage_run_id: &self.stage_run_id,
            actor_binding_id: &self.actor_binding_id,
            actor_role: &self.actor_role,
            target_descriptor_digest: &self.target_descriptor_digest,
            continuity_permission_digest: self.continuity_permission_digest.as_deref(),
        };
        serde_json::to_string(&payload)
            .map_err(|error| format!("Model Mesh authority serialization failed: {error}"))
    }

    pub(crate) fn digest(&self) -> ModelMeshResult<String> {
        Ok(sha256_hex(self.canonical_json()?.as_bytes()))
    }

    pub(crate) fn from_canonical_json(value: &str) -> ModelMeshResult<Self> {
        let parsed: OwnedCanonicalAuthorityEnvelopeV1 = serde_json::from_str(value)
            .map_err(|error| format!("invalid Model Mesh authority envelope JSON: {error}"))?;
        if parsed.schema_version != 1 {
            return Err("unsupported Model Mesh authority envelope schema version".into());
        }
        let purpose = match parsed.purpose.as_str() {
            "TARGET_SELECTION" => ModelMeshAuthorityPurpose::TargetSelection,
            "CONTINUITY_PERMISSION" => ModelMeshAuthorityPurpose::ContinuityPermission,
            _ => return Err("unsupported Model Mesh authority purpose".into()),
        };
        let envelope = Self::from_parts(
            purpose,
            &parsed.workspace_id,
            &parsed.workstream_id,
            &parsed.session_id,
            &parsed.workflow_run_id,
            &parsed.stage_run_id,
            &parsed.actor_binding_id,
            &parsed.actor_role,
            &parsed.target_descriptor_digest,
            parsed.continuity_permission_digest.as_deref(),
        )?;
        if envelope.canonical_json()? != value {
            return Err("Model Mesh authority envelope is not exact canonical JSON".into());
        }
        Ok(envelope)
    }

    pub(crate) fn purpose(&self) -> ModelMeshAuthorityPurpose {
        self.purpose
    }

    pub(crate) fn workspace_id(&self) -> &str {
        &self.workspace_id
    }
    pub(crate) fn workstream_id(&self) -> &str {
        &self.workstream_id
    }
    pub(crate) fn session_id(&self) -> &str {
        &self.session_id
    }
    pub(crate) fn workflow_run_id(&self) -> &str {
        &self.workflow_run_id
    }
    pub(crate) fn stage_run_id(&self) -> &str {
        &self.stage_run_id
    }
    pub(crate) fn actor_binding_id(&self) -> &str {
        &self.actor_binding_id
    }
    pub(crate) fn actor_role(&self) -> &str {
        &self.actor_role
    }
    pub(crate) fn target_descriptor_digest(&self) -> &str {
        &self.target_descriptor_digest
    }
    pub(crate) fn continuity_permission_digest(&self) -> Option<&str> {
        self.continuity_permission_digest.as_deref()
    }
}

fn expected_identity_dimensions(
    descriptor: &ModelMeshTargetDescriptorV1,
    source_requirements: SourceRequirements,
) -> Vec<(IdentityDimension, Option<String>, IdentitySourceClass)> {
    let mut expected = vec![(
        IdentityDimension::Runtime,
        Some(descriptor.runtime().as_str().to_owned()),
        source_requirements.runtime,
    )];
    if let TargetDimension::Exact(provider) = descriptor.provider() {
        expected.push((
            IdentityDimension::Provider,
            Some(provider.as_str().to_owned()),
            source_requirements.provider,
        ));
    }
    if let TargetDimension::Exact(model) = descriptor.model() {
        expected.push((
            IdentityDimension::Model,
            Some(model.as_str().to_owned()),
            source_requirements.model,
        ));
    }
    expected
}

fn resolve_identity_dimension(
    claims: &[IdentityClaim],
    dimension: IdentityDimension,
    expected: Option<&str>,
    required_source: IdentitySourceClass,
) -> TargetResolution {
    let relevant = claims
        .iter()
        .filter(|claim| claim.dimension == dimension)
        .collect::<Vec<_>>();
    if relevant.is_empty() {
        return TargetResolution::Unknown;
    }
    if relevant
        .iter()
        .all(|claim| claim.source == IdentitySourceClass::Unavailable)
    {
        return TargetResolution::Unavailable;
    }

    let qualified = relevant
        .iter()
        .copied()
        .filter(|claim| claim.source == required_source)
        .collect::<Vec<_>>();
    if qualified.is_empty() {
        return TargetResolution::Unknown;
    }

    let values = qualified
        .iter()
        .filter_map(|claim| claim.value.as_deref())
        .collect::<BTreeSet<_>>();
    if values.is_empty() {
        return TargetResolution::Unknown;
    }
    if values.len() > 1 {
        return TargetResolution::Ambiguous;
    }

    let all_values = relevant
        .iter()
        .filter(|claim| claim.source != IdentitySourceClass::Unavailable)
        .filter_map(|claim| claim.value.as_deref())
        .collect::<BTreeSet<_>>();
    if all_values.len() > 1 {
        return TargetResolution::Conflict;
    }

    let observed = values.iter().next().copied();
    if observed != expected {
        return TargetResolution::Unavailable;
    }
    TargetResolution::ExactMatch
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DriftApplicability {
    Applicable,
    NotApplicable,
    Missing,
    Unproven,
    Stale,
    Ambiguous,
    Conflict,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelMeshBaselineDrift {
    pub(crate) applicability: DriftApplicability,
    pub(crate) evaluations: Vec<BaselineEvaluation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelMeshDriftEvaluation {
    pub(crate) runtime: DriftApplicability,
    pub(crate) provider: DriftApplicability,
    pub(crate) model: DriftApplicability,
    pub(crate) native_session: DriftApplicability,
    pub(crate) scope: DriftApplicability,
    pub(crate) baseline: ModelMeshBaselineDrift,
    pub(crate) approval: DriftApplicability,
    pub(crate) current_authority: DriftApplicability,
    pub(crate) overall: DriftApplicability,
}

pub(crate) struct ModelMeshDriftInput<'a> {
    pub(crate) request: &'a TargetRequest,
    pub(crate) current_descriptor: &'a ModelMeshTargetDescriptorV1,
    pub(crate) current_claims: &'a [IdentityClaim],
    pub(crate) source_requirements: SourceRequirements,
    pub(crate) historical_runtime_binding: Option<&'a RuntimeSessionBinding>,
    pub(crate) current_runtime_binding: Option<&'a RuntimeSessionBinding>,
    pub(crate) current_runtime_discovery: Option<&'a RuntimeDiscovery>,
    pub(crate) require_native_session: bool,
    pub(crate) baseline_requirements: &'a [ArtifactBaselineRequirement],
    pub(crate) observed_baselines: &'a [ArtifactBaselineIdentity],
    pub(crate) approval: ApprovalApplicability,
    pub(crate) current_authority: CurrentAuthorityTruth,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelMeshEvaluatedTargetRecord {
    pub(crate) stage_run_id: String,
    pub(crate) target_request_id: String,
    pub(crate) created_unix_ms: i64,
    pub(crate) drift: ModelMeshDriftEvaluation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ModelMeshCurrentTargetState {
    None,
    Exact(String),
    Ambiguous(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelMeshCurrentTargetProjection {
    pub(crate) stage_run_id: Option<String>,
    pub(crate) state: ModelMeshCurrentTargetState,
    pub(crate) historical: Vec<ModelMeshEvaluatedTargetRecord>,
}

pub(crate) fn evaluate_identity_claim_drift(
    historical: &IdentityClaim,
    current_claims: &[IdentityClaim],
) -> DriftApplicability {
    let relevant = current_claims
        .iter()
        .filter(|claim| claim.dimension == historical.dimension)
        .collect::<Vec<_>>();
    if relevant.is_empty() {
        return DriftApplicability::Missing;
    }
    let values = relevant
        .iter()
        .filter(|claim| claim.source != IdentitySourceClass::Unavailable)
        .filter_map(|claim| claim.value.as_deref())
        .collect::<BTreeSet<_>>();
    if values.len() > 1 {
        return DriftApplicability::Conflict;
    }
    if relevant.contains(&historical) {
        return DriftApplicability::Applicable;
    }
    DriftApplicability::Stale
}

fn evaluate_exact_identity_dimension(
    claims: &[IdentityClaim],
    dimension: IdentityDimension,
    expected: &str,
    required_source: IdentitySourceClass,
) -> DriftApplicability {
    let relevant = claims
        .iter()
        .filter(|claim| claim.dimension == dimension)
        .collect::<Vec<_>>();
    if relevant.is_empty() {
        return DriftApplicability::Missing;
    }
    if relevant
        .iter()
        .all(|claim| claim.source == IdentitySourceClass::Unavailable)
    {
        return DriftApplicability::Stale;
    }
    let qualified = relevant
        .iter()
        .copied()
        .filter(|claim| claim.source == required_source)
        .collect::<Vec<_>>();
    if qualified.is_empty() {
        return DriftApplicability::Missing;
    }
    let qualified_values = qualified
        .iter()
        .filter_map(|claim| claim.value.as_deref())
        .collect::<BTreeSet<_>>();
    if qualified_values.len() > 1 {
        return DriftApplicability::Ambiguous;
    }
    let all_values = relevant
        .iter()
        .filter(|claim| claim.source != IdentitySourceClass::Unavailable)
        .filter_map(|claim| claim.value.as_deref())
        .collect::<BTreeSet<_>>();
    if all_values.len() > 1 {
        return DriftApplicability::Conflict;
    }
    match qualified_values.iter().next().copied() {
        Some(value) if value == expected => DriftApplicability::Applicable,
        Some(_) => DriftApplicability::Stale,
        None => DriftApplicability::Missing,
    }
}

fn evaluate_runtime_drift(input: &ModelMeshDriftInput<'_>) -> DriftApplicability {
    let Some(historical) = input.historical_runtime_binding else {
        return DriftApplicability::Missing;
    };
    let Some(current_binding) = input.current_runtime_binding else {
        return DriftApplicability::Missing;
    };
    let Some(discovery) = input.current_runtime_discovery else {
        return DriftApplicability::Missing;
    };
    if historical.binding_id != current_binding.binding_id
        || historical.session_id != current_binding.session_id
        || historical.runtime != current_binding.runtime
        || !runtime_binding_matches_discovery(current_binding, discovery)
    {
        return DriftApplicability::Stale;
    }
    DriftApplicability::Applicable
}

fn evaluate_native_session_drift(input: &ModelMeshDriftInput<'_>) -> DriftApplicability {
    if !input.require_native_session {
        return DriftApplicability::NotApplicable;
    }
    let Some(historical) = input.historical_runtime_binding else {
        return DriftApplicability::Missing;
    };
    let Some(expected_native) = historical.native_session_id.as_deref() else {
        return DriftApplicability::Missing;
    };
    let Some(current) = input.current_runtime_binding else {
        return DriftApplicability::Missing;
    };
    if current.binding_id != historical.binding_id
        || current.session_id != historical.session_id
        || current.native_session_id.as_deref() != Some(expected_native)
        || current.ownership == RuntimeBindingOwnership::OwnershipLost
    {
        return DriftApplicability::Stale;
    }
    DriftApplicability::Unproven
}

fn evaluate_scope_drift(
    expected: &ModelMeshTargetDescriptorV1,
    current: &ModelMeshTargetDescriptorV1,
) -> DriftApplicability {
    if expected.workspace_id() != current.workspace_id()
        || expected.workstream_id() != current.workstream_id()
        || expected.workflow_run_id() != current.workflow_run_id()
        || expected.stage_run_id() != current.stage_run_id()
        || expected.actor_binding_id() != current.actor_binding_id()
        || expected.winds_session_id() != current.winds_session_id()
        || expected.actor_role() != current.actor_role()
    {
        DriftApplicability::Stale
    } else {
        DriftApplicability::Applicable
    }
}

fn evaluate_baseline_drift(
    requirements: &[ArtifactBaselineRequirement],
    observed: &[ArtifactBaselineIdentity],
) -> ModelMeshBaselineDrift {
    let evaluations = requirements
        .iter()
        .map(|requirement| evaluate_artifact_baseline_requirement(requirement, observed))
        .collect::<Vec<_>>();
    let applicability = if evaluations.is_empty() {
        DriftApplicability::NotApplicable
    } else if evaluations
        .iter()
        .any(|evaluation| evaluation.freshness == BaselineFreshness::Ambiguous)
    {
        DriftApplicability::Ambiguous
    } else if evaluations
        .iter()
        .any(|evaluation| evaluation.freshness == BaselineFreshness::Stale)
    {
        DriftApplicability::Stale
    } else if evaluations
        .iter()
        .any(|evaluation| evaluation.freshness == BaselineFreshness::Missing)
    {
        DriftApplicability::Missing
    } else {
        DriftApplicability::Applicable
    };
    ModelMeshBaselineDrift {
        applicability,
        evaluations,
    }
}

fn approval_drift(value: ApprovalApplicability) -> DriftApplicability {
    match value {
        ApprovalApplicability::Exact => DriftApplicability::Applicable,
        ApprovalApplicability::Missing => DriftApplicability::Missing,
        ApprovalApplicability::Mismatch | ApprovalApplicability::Stale => DriftApplicability::Stale,
    }
}

fn combine_drift(values: impl IntoIterator<Item = DriftApplicability>) -> DriftApplicability {
    let values = values.into_iter().collect::<Vec<_>>();
    for state in [
        DriftApplicability::Denied,
        DriftApplicability::Conflict,
        DriftApplicability::Ambiguous,
        DriftApplicability::Stale,
        DriftApplicability::Missing,
        DriftApplicability::Unproven,
    ] {
        if values.contains(&state) {
            return state;
        }
    }
    DriftApplicability::Applicable
}

pub(crate) fn evaluate_model_mesh_drift(
    input: &ModelMeshDriftInput<'_>,
) -> ModelMeshDriftEvaluation {
    let runtime = if input.request.descriptor().runtime() != input.current_descriptor.runtime() {
        DriftApplicability::Stale
    } else {
        evaluate_runtime_drift(input)
    };
    let provider = if input.request.descriptor().provider() != input.current_descriptor.provider() {
        DriftApplicability::Stale
    } else {
        match input.request.descriptor().provider() {
            TargetDimension::Unspecified => DriftApplicability::NotApplicable,
            TargetDimension::Exact(expected) => evaluate_exact_identity_dimension(
                input.current_claims,
                IdentityDimension::Provider,
                expected.as_str(),
                input.source_requirements.provider,
            ),
        }
    };
    let model = if input.request.descriptor().model() != input.current_descriptor.model() {
        DriftApplicability::Stale
    } else {
        match input.request.descriptor().model() {
            TargetDimension::Unspecified => DriftApplicability::NotApplicable,
            TargetDimension::Exact(expected) => evaluate_exact_identity_dimension(
                input.current_claims,
                IdentityDimension::Model,
                expected.as_str(),
                input.source_requirements.model,
            ),
        }
    };
    let native_session = evaluate_native_session_drift(input);
    let scope = evaluate_scope_drift(input.request.descriptor(), input.current_descriptor);
    let baseline = evaluate_baseline_drift(input.baseline_requirements, input.observed_baselines);
    let approval = if input.request.descriptor_digest_matches() {
        approval_drift(input.approval)
    } else {
        DriftApplicability::Stale
    };
    let current_authority = match input.current_authority {
        CurrentAuthorityTruth::Allowed => DriftApplicability::Applicable,
        CurrentAuthorityTruth::Denied => DriftApplicability::Denied,
    };
    let mut dependent = vec![
        runtime,
        provider,
        model,
        scope,
        baseline.applicability,
        approval,
        current_authority,
    ];
    if input.require_native_session {
        dependent.push(native_session);
    }
    let overall = combine_drift(dependent);
    ModelMeshDriftEvaluation {
        runtime,
        provider,
        model,
        native_session,
        scope,
        baseline,
        approval,
        current_authority,
        overall,
    }
}

pub(crate) fn project_current_model_mesh_target(
    records: &[ModelMeshEvaluatedTargetRecord],
) -> ModelMeshResult<ModelMeshCurrentTargetProjection> {
    let mut historical = records.to_vec();
    historical.sort_by(|left, right| {
        (left.created_unix_ms, left.target_request_id.as_str())
            .cmp(&(right.created_unix_ms, right.target_request_id.as_str()))
    });
    let stage_run_id = historical.first().map(|record| record.stage_run_id.clone());
    if let Some(expected_stage) = stage_run_id.as_deref()
        && historical
            .iter()
            .any(|record| record.stage_run_id != expected_stage)
    {
        return Err("current Model Mesh target projection cannot mix StageRun attempts".into());
    }
    let mut applicable = historical
        .iter()
        .filter(|record| record.drift.overall == DriftApplicability::Applicable)
        .map(|record| record.target_request_id.clone())
        .collect::<Vec<_>>();
    applicable.sort();
    applicable.dedup();
    let state = match applicable.as_slice() {
        [] => ModelMeshCurrentTargetState::None,
        [target_request_id] => ModelMeshCurrentTargetState::Exact(target_request_id.clone()),
        _ => ModelMeshCurrentTargetState::Ambiguous(applicable),
    };
    Ok(ModelMeshCurrentTargetProjection {
        stage_run_id,
        state,
        historical,
    })
}

pub(crate) fn model_mesh_authority_json_matches_digest(
    canonical_content_json: &str,
    content_digest: &str,
) -> bool {
    if content_digest.len() != SHA256_HEX_BYTES
        || !content_digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || sha256_hex(canonical_content_json.as_bytes()) != content_digest
    {
        return false;
    }
    ModelMeshAuthorityEnvelopeV1::from_canonical_json(canonical_content_json).is_ok()
}

fn normalize_scope(value: &str, label: &str) -> ModelMeshResult<String> {
    normalize_bounded(value, label, MAX_SCOPE_ID_BYTES)
}

fn normalize_exact_id(value: &str, label: &str) -> ModelMeshResult<String> {
    normalize_bounded(value, label, MAX_TARGET_ID_BYTES)
}

fn normalize_claim_value(value: &str, label: &str) -> ModelMeshResult<String> {
    normalize_bounded(value, label, MAX_TARGET_ID_BYTES)
}

fn normalize_observation_basis(value: &str) -> ModelMeshResult<String> {
    let normalized = normalize_bounded(value, "observation basis", MAX_OBSERVATION_BASIS_BYTES)?;
    let lower = normalized.to_ascii_lowercase();
    if normalized.contains(['=', '{', '}', '[', ']'])
        || [
            "api_key",
            "apikey",
            "authorization",
            "bearer ",
            "credential",
            "password",
            "secret",
            "token",
            "provider-private",
            "provider_private",
        ]
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Err(
            "observation basis must be bounded metadata and must not contain credential or provider-private payload"
                .into(),
        );
    }
    Ok(normalized)
}

fn normalize_actor_role(value: &str) -> ModelMeshResult<String> {
    let normalized = normalize_bounded(value, "actor role", MAX_ACTOR_ROLE_BYTES)?;
    Ok(normalized.to_ascii_uppercase())
}

fn normalize_bounded(value: &str, label: &str, max_bytes: usize) -> ModelMeshResult<String> {
    if value.contains('\0') {
        return Err(format!("{label} must not contain NUL"));
    }
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(format!("{label} must not be empty"));
    }
    if normalized.len() > max_bytes {
        return Err(format!("{label} exceeds {max_bytes} bytes"));
    }
    if normalized.chars().any(char::is_control) {
        return Err(format!("{label} must not contain control characters"));
    }
    Ok(normalized.to_owned())
}

fn normalize_sha256(value: &str, label: &str) -> ModelMeshResult<String> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.len() != SHA256_HEX_BYTES
        || !normalized.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(format!("{label} must be exactly 64 hexadecimal characters"));
    }
    Ok(normalized)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Serialize)]
struct CanonicalTargetDescriptorV1<'a> {
    schema_version: u8,
    workspace_id: &'a str,
    workstream_id: &'a str,
    workflow_run_id: &'a str,
    stage_run_id: &'a str,
    actor_binding_id: &'a str,
    winds_session_id: &'a str,
    actor_role: &'a str,
    runtime: &'a str,
    provider: CanonicalDimension<'a>,
    model: CanonicalDimension<'a>,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "value")]
enum CanonicalDimension<'a> {
    #[serde(rename = "UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "EXACT")]
    Exact(&'a str),
}

fn canonical_provider(value: &TargetDimension<ExactProviderId>) -> CanonicalDimension<'_> {
    match value {
        TargetDimension::Unspecified => CanonicalDimension::Unspecified,
        TargetDimension::Exact(value) => CanonicalDimension::Exact(value.as_str()),
    }
}

fn canonical_model(value: &TargetDimension<ExactModelId>) -> CanonicalDimension<'_> {
    match value {
        TargetDimension::Unspecified => CanonicalDimension::Unspecified,
        TargetDimension::Exact(value) => CanonicalDimension::Exact(value.as_str()),
    }
}

#[derive(Serialize)]
struct CanonicalContinuityPermissionDescriptorV1<'a> {
    schema_version: u8,
    workflow_run_id: &'a str,
    stage_run_id: &'a str,
    source_actor_binding_id: Option<&'a str>,
    destination_actor_binding_id: Option<&'a str>,
    continuity_class: &'a str,
    target_descriptor_digest: &'a str,
    context_digest: CanonicalContextDigest<'a>,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "value")]
enum CanonicalContextDigest<'a> {
    #[serde(rename = "NOT_APPLICABLE")]
    NotApplicable,
    #[serde(rename = "EXACT")]
    Exact(&'a str),
}

fn canonical_context_digest(value: &ContextDigest) -> CanonicalContextDigest<'_> {
    match value {
        ContextDigest::NotApplicable => CanonicalContextDigest::NotApplicable,
        ContextDigest::Exact(value) => CanonicalContextDigest::Exact(value),
    }
}

#[derive(Serialize)]
struct CanonicalAuthorityEnvelopeV1<'a> {
    schema_version: u8,
    purpose: &'a str,
    workspace_id: &'a str,
    workstream_id: &'a str,
    session_id: &'a str,
    workflow_run_id: &'a str,
    stage_run_id: &'a str,
    actor_binding_id: &'a str,
    actor_role: &'a str,
    target_descriptor_digest: &'a str,
    continuity_permission_digest: Option<&'a str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnedCanonicalAuthorityEnvelopeV1 {
    schema_version: u8,
    purpose: String,
    workspace_id: String,
    workstream_id: String,
    session_id: String,
    workflow_run_id: String,
    stage_run_id: String,
    actor_binding_id: String,
    actor_role: String,
    target_descriptor_digest: String,
    continuity_permission_digest: Option<String>,
}
