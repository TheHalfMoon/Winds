use crate::agentic_runtime::{
    AuthReadiness, EvidenceSource, RuntimeDiscovery, RuntimeDiscoveryState, RuntimeKind,
    RuntimeSessionBinding, runtime_binding_matches_discovery,
};
use crate::domain::WindsSessionRecord;
use crate::domain::workflow::{StageRunIdentity, WorkflowRunIdentity};
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
    fn as_str(self) -> &'static str {
        match self {
            Self::Human => "HUMAN",
            Self::ExplicitPolicy => "EXPLICIT_POLICY",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum IdentitySourceClass {
    WindsLocallyObserved,
    VendorDeclared,
    CatalogDeclared,
    AgentReported,
    HumanDecided,
    Unavailable,
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
            .map(|value| normalize_bounded(value, "observation basis", MAX_OBSERVATION_BASIS_BYTES))
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
    fn as_str(self) -> &'static str {
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

fn normalize_scope(value: &str, label: &str) -> ModelMeshResult<String> {
    normalize_bounded(value, label, MAX_SCOPE_ID_BYTES)
}

fn normalize_exact_id(value: &str, label: &str) -> ModelMeshResult<String> {
    normalize_bounded(value, label, MAX_TARGET_ID_BYTES)
}

fn normalize_claim_value(value: &str, label: &str) -> ModelMeshResult<String> {
    normalize_bounded(value, label, MAX_TARGET_ID_BYTES)
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
