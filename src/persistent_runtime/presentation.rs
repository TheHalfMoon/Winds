#![allow(
    dead_code,
    reason = "Spec 011 T156 typed runtime truth projections are consumed incrementally by accepted Rust hosts"
)]

use crate::persistent_runtime::client::{
    ClientEventProjection, ClientResponseProjection, LocalControlClientError,
};
use crate::persistent_runtime::domain::{
    ClientAuthority, ContinuityClass, EventSequence, OwnershipState, ProcessLiveness, RuntimeAlias,
    RuntimeNamespaceId, RuntimeTruth,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeTruthSource {
    WindsObserved,
    AcceptedCanonical,
    AgentReported,
    Unknown,
}

impl RuntimeTruthSource {
    fn accepted(self) -> bool {
        matches!(self, Self::WindsObserved | Self::AcceptedCanonical)
    }

    fn label(self) -> &'static str {
        match self {
            Self::WindsObserved => "WINDS_OBSERVED",
            Self::AcceptedCanonical => "ACCEPTED_CANONICAL",
            Self::AgentReported => "AGENT_REPORTED",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeClientCondition {
    Ready,
    ProtocolIncompatible,
    OwnerGenerationMismatch,
    OutcomeUnknown,
    Disconnected,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeReplayGap {
    pub(crate) first_available_sequence: EventSequence,
    pub(crate) last_dropped_sequence: EventSequence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeContinuityPresentation {
    LiveRetainedProcess,
    ProviderNativeResume,
    WindsReconstruction,
    FreshProcess,
    ClaimedProviderNativeResume,
    Unknown,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeOwnershipPresentation {
    LiveOwned,
    OwnershipLost,
    Unowned,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeLivenessPresentation {
    Running,
    Exited,
    Unknown,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeAuthorityPresentation {
    Observer,
    Controller,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeReplayStateInput {
    Complete,
    Gap(RuntimeReplayGap),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeReplayPresentation {
    Complete,
    Truncated(RuntimeReplayGap),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeHealthPresentation {
    RuntimeTruth,
    ProtocolIncompatible,
    OwnerGenerationMismatch,
    OutcomeUnknown,
    OwnershipLost,
    ProcessUnavailable,
    ProcessUnknown,
    ReplayTruncated,
    Disconnected,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeOutcomeClaim {
    NotProjected,
}

pub(crate) fn client_condition_from_error(
    error: &LocalControlClientError,
) -> RuntimeClientCondition {
    match error {
        LocalControlClientError::ProtocolMismatch
        | LocalControlClientError::Protocol(
            crate::persistent_runtime::domain::LocalControlErrorKind::ProtocolMismatch,
        ) => RuntimeClientCondition::ProtocolIncompatible,
        LocalControlClientError::StaleOwnerGeneration
        | LocalControlClientError::Protocol(
            crate::persistent_runtime::domain::LocalControlErrorKind::StaleOwnerGeneration,
        ) => RuntimeClientCondition::OwnerGenerationMismatch,
        LocalControlClientError::OutcomeUnknown(_) => RuntimeClientCondition::OutcomeUnknown,
        LocalControlClientError::Disconnected => RuntimeClientCondition::Disconnected,
        _ => RuntimeClientCondition::Unknown,
    }
}

pub(crate) fn runtime_truth_from_response(
    response: &ClientResponseProjection,
) -> Option<(RuntimeNamespaceId, RuntimeTruth)> {
    match response {
        ClientResponseProjection::RuntimeSnapshot {
            runtime_namespace_id,
            truth,
        } => Some((*runtime_namespace_id, truth.clone())),
        ClientResponseProjection::ControlState { .. } | ClientResponseProjection::Pong => None,
    }
}

pub(crate) fn authority_from_response(
    response: &ClientResponseProjection,
) -> Option<(RuntimeNamespaceId, ClientAuthority)> {
    match response {
        ClientResponseProjection::ControlState {
            runtime_namespace_id,
            authority,
            ..
        } => Some((*runtime_namespace_id, *authority)),
        ClientResponseProjection::RuntimeSnapshot { .. } | ClientResponseProjection::Pong => None,
    }
}

pub(crate) fn replay_gap_from_event(
    event: &ClientEventProjection,
) -> Option<(RuntimeNamespaceId, RuntimeReplayGap)> {
    match event {
        ClientEventProjection::HistoryGap {
            runtime_namespace_id,
            first_available_sequence,
            last_dropped_sequence,
        } => Some((
            *runtime_namespace_id,
            RuntimeReplayGap {
                first_available_sequence: *first_available_sequence,
                last_dropped_sequence: *last_dropped_sequence,
            },
        )),
        ClientEventProjection::RuntimeEvent { .. }
        | ClientEventProjection::Output { .. }
        | ClientEventProjection::OwnerStatus { .. } => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimePresentationInput {
    pub(crate) runtime_namespace_id: RuntimeNamespaceId,
    pub(crate) display_alias: Option<RuntimeAlias>,
    pub(crate) truth: Option<RuntimeTruth>,
    pub(crate) truth_source: RuntimeTruthSource,
    pub(crate) authority: Option<ClientAuthority>,
    pub(crate) replay_state: RuntimeReplayStateInput,
    pub(crate) client_condition: RuntimeClientCondition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeTruthProjection {
    pub(crate) runtime_namespace_id: RuntimeNamespaceId,
    pub(crate) display_alias: Option<RuntimeAlias>,
    pub(crate) truth_source: RuntimeTruthSource,
    pub(crate) ownership: RuntimeOwnershipPresentation,
    pub(crate) liveness: RuntimeLivenessPresentation,
    pub(crate) continuity: RuntimeContinuityPresentation,
    pub(crate) authority: RuntimeAuthorityPresentation,
    pub(crate) replay: RuntimeReplayPresentation,
    pub(crate) health: RuntimeHealthPresentation,
    pub(crate) verification: RuntimeOutcomeClaim,
    pub(crate) human_acceptance: RuntimeOutcomeClaim,
}

pub(crate) fn project_runtime_truth(input: &RuntimePresentationInput) -> RuntimeTruthProjection {
    let ownership = input
        .truth
        .as_ref()
        .map(|truth| {
            if !input.truth_source.accepted() {
                return RuntimeOwnershipPresentation::Unknown;
            }
            match truth.ownership {
                OwnershipState::LiveOwned
                    if input.truth_source == RuntimeTruthSource::WindsObserved =>
                {
                    RuntimeOwnershipPresentation::LiveOwned
                }
                OwnershipState::LiveOwned => RuntimeOwnershipPresentation::Unknown,
                OwnershipState::OwnershipLost => RuntimeOwnershipPresentation::OwnershipLost,
                OwnershipState::Unowned => RuntimeOwnershipPresentation::Unowned,
            }
        })
        .unwrap_or(RuntimeOwnershipPresentation::Unknown);

    let liveness = input
        .truth
        .as_ref()
        .map(|truth| {
            if !input.truth_source.accepted() {
                return RuntimeLivenessPresentation::Unknown;
            }
            match truth.process_liveness {
                ProcessLiveness::Running
                    if input.truth_source == RuntimeTruthSource::WindsObserved =>
                {
                    RuntimeLivenessPresentation::Running
                }
                ProcessLiveness::Running => RuntimeLivenessPresentation::Unknown,
                ProcessLiveness::Exited => RuntimeLivenessPresentation::Exited,
                ProcessLiveness::Unknown => RuntimeLivenessPresentation::Unknown,
                ProcessLiveness::Unavailable => RuntimeLivenessPresentation::Unavailable,
            }
        })
        .unwrap_or(RuntimeLivenessPresentation::Unknown);

    let continuity = input
        .truth
        .as_ref()
        .map(|truth| match truth.continuity {
            ContinuityClass::RetainedLiveProcess
                if truth.ownership == OwnershipState::LiveOwned
                    && truth.process_liveness == ProcessLiveness::Running
                    && input.truth_source == RuntimeTruthSource::WindsObserved =>
            {
                RuntimeContinuityPresentation::LiveRetainedProcess
            }
            ContinuityClass::RetainedLiveProcess => RuntimeContinuityPresentation::Unknown,
            ContinuityClass::ProviderNativeResume if input.truth_source.accepted() => {
                RuntimeContinuityPresentation::ProviderNativeResume
            }
            ContinuityClass::ProviderNativeResume => {
                RuntimeContinuityPresentation::ClaimedProviderNativeResume
            }
            ContinuityClass::WindsReconstruction if input.truth_source.accepted() => {
                RuntimeContinuityPresentation::WindsReconstruction
            }
            ContinuityClass::WindsReconstruction => RuntimeContinuityPresentation::Unknown,
            ContinuityClass::FreshProcess if input.truth_source.accepted() => {
                RuntimeContinuityPresentation::FreshProcess
            }
            ContinuityClass::FreshProcess => RuntimeContinuityPresentation::Unknown,
            ContinuityClass::Unknown => RuntimeContinuityPresentation::Unknown,
            ContinuityClass::Unavailable => RuntimeContinuityPresentation::Unavailable,
        })
        .unwrap_or(RuntimeContinuityPresentation::Unknown);

    let authority = match input.authority {
        Some(ClientAuthority::Observer) => RuntimeAuthorityPresentation::Observer,
        Some(ClientAuthority::Controller) => RuntimeAuthorityPresentation::Controller,
        None => RuntimeAuthorityPresentation::Unknown,
    };
    let replay = match input.replay_state {
        RuntimeReplayStateInput::Complete => RuntimeReplayPresentation::Complete,
        RuntimeReplayStateInput::Gap(gap) => RuntimeReplayPresentation::Truncated(gap),
        RuntimeReplayStateInput::Unknown => RuntimeReplayPresentation::Unknown,
    };

    let health = match input.client_condition {
        RuntimeClientCondition::ProtocolIncompatible => {
            RuntimeHealthPresentation::ProtocolIncompatible
        }
        RuntimeClientCondition::OwnerGenerationMismatch => {
            RuntimeHealthPresentation::OwnerGenerationMismatch
        }
        RuntimeClientCondition::OutcomeUnknown => RuntimeHealthPresentation::OutcomeUnknown,
        RuntimeClientCondition::Disconnected => RuntimeHealthPresentation::Disconnected,
        RuntimeClientCondition::Unknown => RuntimeHealthPresentation::Unknown,
        RuntimeClientCondition::Ready => match ownership {
            RuntimeOwnershipPresentation::OwnershipLost => RuntimeHealthPresentation::OwnershipLost,
            _ if liveness == RuntimeLivenessPresentation::Unavailable => {
                RuntimeHealthPresentation::ProcessUnavailable
            }
            _ if liveness == RuntimeLivenessPresentation::Unknown => {
                RuntimeHealthPresentation::ProcessUnknown
            }
            _ if matches!(replay, RuntimeReplayPresentation::Truncated(_)) => {
                RuntimeHealthPresentation::ReplayTruncated
            }
            _ => RuntimeHealthPresentation::RuntimeTruth,
        },
    };

    RuntimeTruthProjection {
        runtime_namespace_id: input.runtime_namespace_id,
        display_alias: input.display_alias.clone(),
        truth_source: input.truth_source,
        ownership,
        liveness,
        continuity,
        authority,
        replay,
        health,
        verification: RuntimeOutcomeClaim::NotProjected,
        human_acceptance: RuntimeOutcomeClaim::NotProjected,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliRuntimeProjection {
    pub(crate) runtime_namespace_id: String,
    pub(crate) display_alias: Option<String>,
    pub(crate) ownership: &'static str,
    pub(crate) liveness: &'static str,
    pub(crate) continuity: &'static str,
    pub(crate) authority: &'static str,
    pub(crate) replay: &'static str,
    pub(crate) replay_first_available_sequence: Option<u64>,
    pub(crate) replay_last_dropped_sequence: Option<u64>,
    pub(crate) health: &'static str,
    pub(crate) truth_source: &'static str,
    pub(crate) verification: &'static str,
    pub(crate) human_acceptance: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkbenchRuntimeProjection {
    pub(crate) immutable_runtime_id: String,
    pub(crate) display_alias: Option<String>,
    pub(crate) headline: &'static str,
    pub(crate) detail: &'static str,
    pub(crate) authority: &'static str,
    pub(crate) truth_source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopRuntimeTruthProjection {
    pub(crate) runtime_namespace_id: String,
    pub(crate) display_alias: Option<String>,
    pub(crate) ownership: String,
    pub(crate) liveness: String,
    pub(crate) continuity: String,
    pub(crate) authority: String,
    pub(crate) replay: String,
    pub(crate) replay_first_available_sequence: Option<u64>,
    pub(crate) replay_last_dropped_sequence: Option<u64>,
    pub(crate) health: String,
    pub(crate) truth_source: String,
    pub(crate) verification: String,
    pub(crate) human_acceptance: String,
}

pub(crate) fn project_cli_runtime(projection: &RuntimeTruthProjection) -> CliRuntimeProjection {
    CliRuntimeProjection {
        runtime_namespace_id: projection.runtime_namespace_id.to_string(),
        display_alias: projection
            .display_alias
            .as_ref()
            .map(|value| value.as_str().to_owned()),
        ownership: ownership_label(projection.ownership),
        liveness: liveness_label(projection.liveness),
        continuity: continuity_label(projection.continuity),
        authority: authority_label(projection.authority),
        replay: replay_label(projection.replay),
        replay_first_available_sequence: replay_first_available_sequence(projection.replay),
        replay_last_dropped_sequence: replay_last_dropped_sequence(projection.replay),
        health: health_label(projection.health),
        truth_source: projection.truth_source.label(),
        verification: "NOT_PROJECTED",
        human_acceptance: "NOT_PROJECTED",
    }
}

pub(crate) fn project_workbench_runtime(
    projection: &RuntimeTruthProjection,
) -> WorkbenchRuntimeProjection {
    let (headline, detail) = match projection.health {
        RuntimeHealthPresentation::ProtocolIncompatible => (
            "Protocol incompatible",
            "Local-control protocol versions are incompatible; no continuation is assumed.",
        ),
        RuntimeHealthPresentation::OwnerGenerationMismatch => (
            "Owner generation changed",
            "The previous owner generation is stale; this is not a continuation.",
        ),
        RuntimeHealthPresentation::OutcomeUnknown => (
            "Outcome unknown",
            "Reconcile exact runtime state before another consequential action.",
        ),
        RuntimeHealthPresentation::OwnershipLost => (
            "Ownership lost",
            "Winds no longer proves live ownership of this runtime.",
        ),
        RuntimeHealthPresentation::ProcessUnavailable => (
            "Process unavailable",
            "Process liveness is unavailable; no live-process claim is made.",
        ),
        RuntimeHealthPresentation::ProcessUnknown => (
            "Process state unknown",
            "Process liveness is unknown; no optimistic connected state is shown.",
        ),
        RuntimeHealthPresentation::ReplayTruncated => (
            "Replay truncated",
            "A replay gap is present; displayed history is incomplete.",
        ),
        RuntimeHealthPresentation::Disconnected => (
            "Local client disconnected",
            "The presentation client is disconnected from the private local endpoint.",
        ),
        RuntimeHealthPresentation::Unknown => (
            "Runtime state unknown",
            "No accepted runtime truth is currently available.",
        ),
        RuntimeHealthPresentation::RuntimeTruth => (
            continuity_headline(projection.continuity),
            continuity_detail(projection.continuity),
        ),
    };
    WorkbenchRuntimeProjection {
        immutable_runtime_id: projection.runtime_namespace_id.to_string(),
        display_alias: projection
            .display_alias
            .as_ref()
            .map(|value| value.as_str().to_owned()),
        headline,
        detail,
        authority: authority_label(projection.authority),
        truth_source: projection.truth_source.label(),
    }
}

pub(crate) fn project_desktop_runtime(
    projection: &RuntimeTruthProjection,
) -> DesktopRuntimeTruthProjection {
    let cli = project_cli_runtime(projection);
    DesktopRuntimeTruthProjection {
        runtime_namespace_id: cli.runtime_namespace_id,
        display_alias: cli.display_alias,
        ownership: cli.ownership.to_owned(),
        liveness: cli.liveness.to_owned(),
        continuity: cli.continuity.to_owned(),
        authority: cli.authority.to_owned(),
        replay: cli.replay.to_owned(),
        replay_first_available_sequence: cli.replay_first_available_sequence,
        replay_last_dropped_sequence: cli.replay_last_dropped_sequence,
        health: cli.health.to_owned(),
        truth_source: cli.truth_source.to_owned(),
        verification: cli.verification.to_owned(),
        human_acceptance: cli.human_acceptance.to_owned(),
    }
}

fn ownership_label(value: RuntimeOwnershipPresentation) -> &'static str {
    match value {
        RuntimeOwnershipPresentation::LiveOwned => "LIVE_OWNED",
        RuntimeOwnershipPresentation::OwnershipLost => "OWNERSHIP_LOST",
        RuntimeOwnershipPresentation::Unowned => "UNOWNED",
        RuntimeOwnershipPresentation::Unknown => "UNKNOWN",
    }
}

fn liveness_label(value: RuntimeLivenessPresentation) -> &'static str {
    match value {
        RuntimeLivenessPresentation::Running => "RUNNING",
        RuntimeLivenessPresentation::Exited => "EXITED",
        RuntimeLivenessPresentation::Unknown => "UNKNOWN",
        RuntimeLivenessPresentation::Unavailable => "UNAVAILABLE",
    }
}

fn continuity_label(value: RuntimeContinuityPresentation) -> &'static str {
    match value {
        RuntimeContinuityPresentation::LiveRetainedProcess => "LIVE_RETAINED_PROCESS",
        RuntimeContinuityPresentation::ProviderNativeResume => "PROVIDER_NATIVE_RESUME",
        RuntimeContinuityPresentation::WindsReconstruction => "WINDS_RECONSTRUCTION",
        RuntimeContinuityPresentation::FreshProcess => "FRESH_PROCESS",
        RuntimeContinuityPresentation::ClaimedProviderNativeResume => {
            "CLAIMED_PROVIDER_NATIVE_RESUME"
        }
        RuntimeContinuityPresentation::Unknown => "UNKNOWN",
        RuntimeContinuityPresentation::Unavailable => "UNAVAILABLE",
    }
}

fn authority_label(value: RuntimeAuthorityPresentation) -> &'static str {
    match value {
        RuntimeAuthorityPresentation::Observer => "OBSERVER",
        RuntimeAuthorityPresentation::Controller => "CONTROLLER",
        RuntimeAuthorityPresentation::Unknown => "UNKNOWN",
    }
}

fn replay_label(value: RuntimeReplayPresentation) -> &'static str {
    match value {
        RuntimeReplayPresentation::Complete => "COMPLETE",
        RuntimeReplayPresentation::Truncated(_) => "TRUNCATED_GAP",
        RuntimeReplayPresentation::Unknown => "UNKNOWN",
    }
}

fn replay_first_available_sequence(value: RuntimeReplayPresentation) -> Option<u64> {
    match value {
        RuntimeReplayPresentation::Truncated(gap) => Some(gap.first_available_sequence.get()),
        RuntimeReplayPresentation::Complete | RuntimeReplayPresentation::Unknown => None,
    }
}

fn replay_last_dropped_sequence(value: RuntimeReplayPresentation) -> Option<u64> {
    match value {
        RuntimeReplayPresentation::Truncated(gap) => Some(gap.last_dropped_sequence.get()),
        RuntimeReplayPresentation::Complete | RuntimeReplayPresentation::Unknown => None,
    }
}

fn health_label(value: RuntimeHealthPresentation) -> &'static str {
    match value {
        RuntimeHealthPresentation::RuntimeTruth => "RUNTIME_TRUTH",
        RuntimeHealthPresentation::ProtocolIncompatible => "PROTOCOL_INCOMPATIBLE",
        RuntimeHealthPresentation::OwnerGenerationMismatch => "OWNER_GENERATION_MISMATCH",
        RuntimeHealthPresentation::OutcomeUnknown => "OUTCOME_UNKNOWN",
        RuntimeHealthPresentation::OwnershipLost => "OWNERSHIP_LOST",
        RuntimeHealthPresentation::ProcessUnavailable => "PROCESS_UNAVAILABLE",
        RuntimeHealthPresentation::ProcessUnknown => "PROCESS_UNKNOWN",
        RuntimeHealthPresentation::ReplayTruncated => "REPLAY_TRUNCATED",
        RuntimeHealthPresentation::Disconnected => "DISCONNECTED",
        RuntimeHealthPresentation::Unknown => "UNKNOWN",
    }
}

fn continuity_headline(value: RuntimeContinuityPresentation) -> &'static str {
    match value {
        RuntimeContinuityPresentation::LiveRetainedProcess => "Live retained process",
        RuntimeContinuityPresentation::ProviderNativeResume => "Provider-native resume",
        RuntimeContinuityPresentation::WindsReconstruction => "Winds reconstruction",
        RuntimeContinuityPresentation::FreshProcess => "Fresh process",
        RuntimeContinuityPresentation::ClaimedProviderNativeResume => {
            "Provider resume claim unproven"
        }
        RuntimeContinuityPresentation::Unknown => "Continuity unknown",
        RuntimeContinuityPresentation::Unavailable => "Continuity unavailable",
    }
}

fn continuity_detail(value: RuntimeContinuityPresentation) -> &'static str {
    match value {
        RuntimeContinuityPresentation::LiveRetainedProcess => {
            "The exact retained PTY/process remains live under the current Winds owner."
        }
        RuntimeContinuityPresentation::ProviderNativeResume => {
            "An accepted source proves provider-native resume; this is distinct from retained-process continuity."
        }
        RuntimeContinuityPresentation::WindsReconstruction => {
            "Winds reconstructed/reassigned canonical context into a different runtime; this is not native resume."
        }
        RuntimeContinuityPresentation::FreshProcess => {
            "A fresh process was started; no prior live-process or native-resume continuity is claimed."
        }
        RuntimeContinuityPresentation::ClaimedProviderNativeResume => {
            "A source claimed provider-native resume but accepted proof is absent; no resume truth is promoted."
        }
        RuntimeContinuityPresentation::Unknown => {
            "Continuity is unknown; no optimistic resume or live-process label is shown."
        }
        RuntimeContinuityPresentation::Unavailable => "Continuity evidence is unavailable.",
    }
}

#[cfg(test)]
#[path = "../t156_runtime_truth_projection_tests.rs"]
mod t156_runtime_truth_projection_tests;
