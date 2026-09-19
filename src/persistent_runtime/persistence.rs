use super::domain::{
    ContinuityClass, EndpointAvailability, OwnerGenerationId, OwnershipState, ProcessLiveness,
    RuntimeAlias, RuntimeLifecycleEventKind, RuntimeNamespaceId, RuntimeTruth,
};

pub(crate) const PERSISTENT_RUNTIME_SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PersistentRuntimeRecoveryReason {
    OwnerGenerationChanged,
    OwnerGenerationUnproven,
}

impl PersistentRuntimeRecoveryReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::OwnerGenerationChanged => "OWNER_GENERATION_CHANGED",
            Self::OwnerGenerationUnproven => "OWNER_GENERATION_UNPROVEN",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "OWNER_GENERATION_CHANGED" => Ok(Self::OwnerGenerationChanged),
            "OWNER_GENERATION_UNPROVEN" => Ok(Self::OwnerGenerationUnproven),
            _ => Err(format!(
                "unsupported persistent runtime recovery reason: {value}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PersistentRuntimeRecord {
    pub(crate) runtime_namespace_id: RuntimeNamespaceId,
    pub(crate) runtime_alias: RuntimeAlias,
    pub(crate) workspace_id: Option<String>,
    pub(crate) session_id: Option<String>,
    pub(crate) terminal_execution_id: Option<String>,
    pub(crate) owner_generation_id: Option<OwnerGenerationId>,
    pub(crate) truth: RuntimeTruth,
    pub(crate) last_lifecycle_event_kind: RuntimeLifecycleEventKind,
    pub(crate) created_unix_ms: i64,
    pub(crate) updated_unix_ms: i64,
    pub(crate) last_observed_unix_ms: Option<i64>,
    pub(crate) ownership_lost_unix_ms: Option<i64>,
    pub(crate) recovery_reason: Option<PersistentRuntimeRecoveryReason>,
}

#[derive(Debug, Clone)]
pub(crate) struct PersistentRuntimeRecordInput<'a> {
    pub(crate) runtime_namespace_id: RuntimeNamespaceId,
    pub(crate) runtime_alias: &'a RuntimeAlias,
    pub(crate) workspace_id: Option<&'a str>,
    pub(crate) session_id: Option<&'a str>,
    pub(crate) terminal_execution_id: Option<&'a str>,
    pub(crate) owner_generation_id: Option<OwnerGenerationId>,
    pub(crate) truth: &'a RuntimeTruth,
    pub(crate) last_lifecycle_event_kind: RuntimeLifecycleEventKind,
    pub(crate) created_unix_ms: i64,
    pub(crate) updated_unix_ms: i64,
    pub(crate) last_observed_unix_ms: Option<i64>,
    pub(crate) ownership_lost_unix_ms: Option<i64>,
    pub(crate) recovery_reason: Option<PersistentRuntimeRecoveryReason>,
}

pub(crate) fn validate_record_input(
    input: &PersistentRuntimeRecordInput<'_>,
) -> Result<(), String> {
    validate_reference(input.workspace_id, "workspace id")?;
    validate_reference(input.session_id, "session id")?;
    validate_reference(input.terminal_execution_id, "terminal execution id")?;
    validate_timestamp(input.created_unix_ms, "runtime creation time")?;
    validate_timestamp(input.updated_unix_ms, "runtime update time")?;
    if input.updated_unix_ms < input.created_unix_ms {
        return Err("runtime update time must not precede creation time".to_owned());
    }
    if let Some(value) = input.last_observed_unix_ms {
        validate_timestamp(value, "runtime observation time")?;
        if value < input.created_unix_ms {
            return Err("runtime observation time must not precede creation time".to_owned());
        }
    }
    if let Some(value) = input.ownership_lost_unix_ms {
        validate_timestamp(value, "ownership-loss time")?;
        if value < input.created_unix_ms {
            return Err("ownership-loss time must not precede creation time".to_owned());
        }
    }
    if input.truth.ownership == OwnershipState::LiveOwned && input.owner_generation_id.is_none() {
        return Err("LIVE_OWNED requires an explicit owner generation".to_owned());
    }
    let lost = input.truth.ownership == OwnershipState::OwnershipLost;
    if lost != input.ownership_lost_unix_ms.is_some() || lost != input.recovery_reason.is_some() {
        return Err(
            "OWNERSHIP_LOST requires both ownership-loss time and bounded recovery reason"
                .to_owned(),
        );
    }
    Ok(())
}

pub(crate) fn validate_timestamp(value: i64, label: &str) -> Result<(), String> {
    if value < 0 {
        return Err(format!("{label} must be nonnegative"));
    }
    Ok(())
}

pub(crate) fn validate_reference(value: Option<&str>, label: &str) -> Result<(), String> {
    if let Some(value) = value {
        if value.trim().is_empty() {
            return Err(format!("{label} must not be empty when present"));
        }
        if value.contains('\0') {
            return Err(format!("{label} must not contain NUL"));
        }
    }
    Ok(())
}

pub(crate) fn ownership_label(value: OwnershipState) -> &'static str {
    match value {
        OwnershipState::LiveOwned => "LIVE_OWNED",
        OwnershipState::OwnershipLost => "OWNERSHIP_LOST",
        OwnershipState::Unowned => "UNOWNED",
    }
}

pub(crate) fn ownership_from_db(value: &str) -> Result<OwnershipState, String> {
    match value {
        "LIVE_OWNED" => Ok(OwnershipState::LiveOwned),
        "OWNERSHIP_LOST" => Ok(OwnershipState::OwnershipLost),
        "UNOWNED" => Ok(OwnershipState::Unowned),
        _ => Err(format!("unsupported persistent ownership state: {value}")),
    }
}

pub(crate) fn process_liveness_label(value: ProcessLiveness) -> &'static str {
    match value {
        ProcessLiveness::Unknown => "UNKNOWN",
        ProcessLiveness::Running => "RUNNING",
        ProcessLiveness::Exited => "EXITED",
        ProcessLiveness::Unavailable => "UNAVAILABLE",
    }
}

pub(crate) fn process_liveness_from_db(value: &str) -> Result<ProcessLiveness, String> {
    match value {
        "UNKNOWN" => Ok(ProcessLiveness::Unknown),
        "RUNNING" => Ok(ProcessLiveness::Running),
        "EXITED" => Ok(ProcessLiveness::Exited),
        "UNAVAILABLE" => Ok(ProcessLiveness::Unavailable),
        _ => Err(format!(
            "unsupported persistent process-liveness state: {value}"
        )),
    }
}

pub(crate) fn endpoint_availability_label(value: EndpointAvailability) -> &'static str {
    match value {
        EndpointAvailability::Unknown => "UNKNOWN",
        EndpointAvailability::Available => "AVAILABLE",
        EndpointAvailability::Unavailable => "UNAVAILABLE",
    }
}

pub(crate) fn endpoint_availability_from_db(value: &str) -> Result<EndpointAvailability, String> {
    match value {
        "UNKNOWN" => Ok(EndpointAvailability::Unknown),
        "AVAILABLE" => Ok(EndpointAvailability::Available),
        "UNAVAILABLE" => Ok(EndpointAvailability::Unavailable),
        _ => Err(format!(
            "unsupported persistent endpoint-availability state: {value}"
        )),
    }
}

pub(crate) fn continuity_label(value: ContinuityClass) -> &'static str {
    match value {
        ContinuityClass::RetainedLiveProcess => "RETAINED_LIVE_PROCESS",
        ContinuityClass::ProviderNativeResume => "PROVIDER_NATIVE_RESUME",
        ContinuityClass::WindsReconstruction => "WINDS_RECONSTRUCTION",
        ContinuityClass::FreshProcess => "FRESH_PROCESS",
        ContinuityClass::Unknown => "UNKNOWN",
        ContinuityClass::Unavailable => "UNAVAILABLE",
    }
}

pub(crate) fn continuity_from_db(value: &str) -> Result<ContinuityClass, String> {
    match value {
        "RETAINED_LIVE_PROCESS" => Ok(ContinuityClass::RetainedLiveProcess),
        "PROVIDER_NATIVE_RESUME" => Ok(ContinuityClass::ProviderNativeResume),
        "WINDS_RECONSTRUCTION" => Ok(ContinuityClass::WindsReconstruction),
        "FRESH_PROCESS" => Ok(ContinuityClass::FreshProcess),
        "UNKNOWN" => Ok(ContinuityClass::Unknown),
        "UNAVAILABLE" => Ok(ContinuityClass::Unavailable),
        _ => Err(format!("unsupported persistent continuity class: {value}")),
    }
}

pub(crate) fn lifecycle_kind_label(value: RuntimeLifecycleEventKind) -> &'static str {
    match value {
        RuntimeLifecycleEventKind::NamespaceCreated => "NAMESPACE_CREATED",
        RuntimeLifecycleEventKind::OwnershipEstablished => "OWNERSHIP_ESTABLISHED",
        RuntimeLifecycleEventKind::OwnershipLost => "OWNERSHIP_LOST",
        RuntimeLifecycleEventKind::ProcessStateObserved => "PROCESS_STATE_OBSERVED",
        RuntimeLifecycleEventKind::ContinuityClassified => "CONTINUITY_CLASSIFIED",
        RuntimeLifecycleEventKind::ControllerChanged => "CONTROLLER_CHANGED",
        RuntimeLifecycleEventKind::RuntimeStopped => "RUNTIME_STOPPED",
    }
}

pub(crate) fn lifecycle_kind_from_db(value: &str) -> Result<RuntimeLifecycleEventKind, String> {
    match value {
        "NAMESPACE_CREATED" => Ok(RuntimeLifecycleEventKind::NamespaceCreated),
        "OWNERSHIP_ESTABLISHED" => Ok(RuntimeLifecycleEventKind::OwnershipEstablished),
        "OWNERSHIP_LOST" => Ok(RuntimeLifecycleEventKind::OwnershipLost),
        "PROCESS_STATE_OBSERVED" => Ok(RuntimeLifecycleEventKind::ProcessStateObserved),
        "CONTINUITY_CLASSIFIED" => Ok(RuntimeLifecycleEventKind::ContinuityClassified),
        "CONTROLLER_CHANGED" => Ok(RuntimeLifecycleEventKind::ControllerChanged),
        "RUNTIME_STOPPED" => Ok(RuntimeLifecycleEventKind::RuntimeStopped),
        _ => Err(format!(
            "unsupported persistent lifecycle event kind: {value}"
        )),
    }
}
