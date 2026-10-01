//! Bounded agent observations bound to exact topology identity.
//!
//! This turns qualified detector inputs from the T172 catalog into observations
//! that are attributable and bounded. The central rule is that an observation
//! names a family only when Winds can attribute that family to one exact pane
//! through one exact binding. Anything else is represented by the absence of an
//! observation, never by a guess.
//!
//! Every distinction the program depends on is kept separate here:
//!
//! - a pane, a runtime namespace, and a provider-native session are three
//!   different identities, and an observation carries at most one runtime
//!   reference, taken from the owner-accepted topology binding rather than from
//!   anything a caller asserts;
//! - a detection is not an execution. `AgentSupport` has no variant that can
//!   express launch, install, or prompt authority, so no observation can claim
//!   real provider execution;
//! - a classification is not a verification and not an acceptance. Nothing in
//!   this module carries, derives, or promotes either;
//! - terminal prose is not a detection input. A pane that offers only untrusted
//!   text has no family to name, so it produces no observation.
//!
//! Staleness fails closed. An observation is bound to the exact
//! `(multiplexer_workspace_id, tab_id, pane_id, runtime_namespace_id)` it was
//! recorded against, and it is dropped the moment any part of that binding
//! changes. A closed pane, a retired workspace, a replaced runtime, or a new
//! owner generation all invalidate it rather than letting it drift.

use crate::multiplexer::agent_catalog::{AgentDetection, AgentFamily};
use crate::multiplexer::domain::navigation::MultiplexerTopology;
use crate::multiplexer::domain::{
    AgentObservationId, MultiplexerErrorKind, MultiplexerWorkspaceId, PaneId, TabId,
};
use crate::persistent_runtime::domain::{OwnerGenerationId, RuntimeNamespaceId};
use crate::persistent_runtime::protocol::{
    AgentFamilyV2, AgentObservationConfidenceV2, AgentObservationCursorV2, AgentObservationEventV2,
    AgentObservationFreshnessV2, AgentObservationSnapshotV2, AgentObservationSourceV2,
    AgentObservationV2, ListAgentObservationsV2, MAX_V2_AGENT_OBSERVATIONS_PER_PAGE,
    MAX_V2_EVIDENCE_SUMMARY_BYTES, MAX_V2_GIT_WORKSPACE_ID_BYTES, MAX_V2_PROVIDER_SESSION_ID_BYTES,
};
use std::collections::{BTreeMap, BTreeSet};

/// Every way an observation can be refused. All of them fail closed: a refused
/// candidate changes nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ObservationError {
    /// The topology rejected the claimed `(workspace, tab, pane)` triple.
    Topology(MultiplexerErrorKind),
    /// The candidate was offered under a different owner generation, so nothing
    /// it claims about a binding can be trusted against this owner.
    StaleOwnerGeneration,
    /// One observation identity is already bound to a different pane, or the
    /// candidate repeats one identity for multiple observations.
    ObservationIdentityReuse,
    /// The candidate carried a field the frozen wire contract cannot accept.
    Unrepresentable,
    /// The bounded snapshot revision cannot advance, so further mutation would
    /// have to reuse a revision a client may already hold.
    SnapshotRevisionExhausted,
}

impl From<MultiplexerErrorKind> for ObservationError {
    fn from(value: MultiplexerErrorKind) -> Self {
        Self::Topology(value)
    }
}

/// The authority order of a source class, lower is stronger.
///
/// A pane may be described by more than one accepted source. The strongest
/// accepted source wins, so a later weak description can never downgrade or
/// withdraw a classification that a stronger source already established, and a
/// stronger source can replace a weaker one.
fn source_rank(source: AgentObservationSourceV2) -> u8 {
    match source {
        AgentObservationSourceV2::WindsLaunchMetadata => 0,
        AgentObservationSourceV2::OwnedProcessMetadata => 1,
        AgentObservationSourceV2::ProviderStructuredMetadata => 2,
        AgentObservationSourceV2::UserDeclaredPresentation => 3,
    }
}

/// The exact outcome of qualifying one pane's detector input.
struct QualifiedFamilies {
    families: Vec<AgentFamilyV2>,
    freshness: AgentObservationFreshnessV2,
}

impl QualifiedFamilies {
    /// Whether this outcome names any family at all.
    ///
    /// `NearMatch`, `Unknown`, and `UntrustedText` name none, and an observation
    /// requires a family because the frozen wire field is required. Rather than
    /// invent a family for a pane Winds could not attribute, those outcomes
    /// produce no observation at all, and any observation the pane previously
    /// held is withdrawn when the source is authoritative enough to do so.
    fn is_named(&self) -> bool {
        !self.families.is_empty()
    }
}

/// Maps a qualified catalog classification onto wire families and a freshness
/// state, without conflating them.
///
/// Ambiguity and unavailability name several families. Each becomes its own
/// observation carrying `Ambiguous` or `Unavailable` freshness and `Unknown`
/// confidence, so no consumer can read a single family out of the result. The
/// order of the families is the catalog's own order, which is the pinned ledger
/// order, so the projection is deterministic.
fn qualify(detection: &AgentDetection) -> QualifiedFamilies {
    match detection {
        AgentDetection::Observed { family, .. } => QualifiedFamilies {
            families: vec![as_wire_family(*family)],
            freshness: AgentObservationFreshnessV2::Current,
        },
        AgentDetection::Ambiguous { families } => QualifiedFamilies {
            families: families.iter().copied().map(as_wire_family).collect(),
            freshness: AgentObservationFreshnessV2::Ambiguous,
        },
        AgentDetection::Unavailable { families, .. } => QualifiedFamilies {
            families: families.iter().copied().map(as_wire_family).collect(),
            freshness: AgentObservationFreshnessV2::Unavailable,
        },
        AgentDetection::NearMatch
        | AgentDetection::Unknown { .. }
        | AgentDetection::UntrustedText { .. }
        | AgentDetection::Stale => QualifiedFamilies {
            families: Vec::new(),
            freshness: AgentObservationFreshnessV2::Unknown,
        },
    }
}

/// The confidence a source class may claim for a family it qualified.
fn confidence_for(
    source: AgentObservationSourceV2,
    freshness: AgentObservationFreshnessV2,
) -> AgentObservationConfidenceV2 {
    if freshness != AgentObservationFreshnessV2::Current {
        return AgentObservationConfidenceV2::Unknown;
    }
    if source == AgentObservationSourceV2::UserDeclaredPresentation {
        AgentObservationConfidenceV2::UserDeclared
    } else {
        AgentObservationConfidenceV2::Strong
    }
}

/// The catalog family as the frozen wire family.
///
/// The match is exhaustive, so a family added to the catalog without the wire
/// enum, or the reverse, is a build error rather than a silent divergence.
fn as_wire_family(family: AgentFamily) -> AgentFamilyV2 {
    match family {
        AgentFamily::Pi => AgentFamilyV2::Pi,
        AgentFamily::Claude => AgentFamilyV2::Claude,
        AgentFamily::Codex => AgentFamilyV2::Codex,
        AgentFamily::Gemini => AgentFamilyV2::Gemini,
        AgentFamily::Cursor => AgentFamilyV2::Cursor,
        AgentFamily::Devin => AgentFamilyV2::Devin,
        AgentFamily::Antigravity => AgentFamilyV2::Antigravity,
        AgentFamily::Cline => AgentFamilyV2::Cline,
        AgentFamily::Omp => AgentFamilyV2::Omp,
        AgentFamily::Mastracode => AgentFamilyV2::Mastracode,
        AgentFamily::OpenCode => AgentFamilyV2::OpenCode,
        AgentFamily::GithubCopilot => AgentFamilyV2::GithubCopilot,
        AgentFamily::Kimi => AgentFamilyV2::Kimi,
        AgentFamily::Kiro => AgentFamilyV2::Kiro,
        AgentFamily::Droid => AgentFamilyV2::Droid,
        AgentFamily::Amp => AgentFamilyV2::Amp,
        AgentFamily::Grok => AgentFamilyV2::Grok,
        AgentFamily::Hermes => AgentFamilyV2::Hermes,
        AgentFamily::Kilo => AgentFamilyV2::Kilo,
        AgentFamily::Qodercli => AgentFamilyV2::Qodercli,
        AgentFamily::Qwen => AgentFamilyV2::Qwen,
        AgentFamily::Letta => AgentFamilyV2::Letta,
        AgentFamily::Maki => AgentFamilyV2::Maki,
        AgentFamily::Muse => AgentFamilyV2::Muse,
    }
}

/// One pane's qualified detector result and the provenance Winds can prove for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObservationCandidate {
    /// One observation identity per candidate family, positionally matching the
    /// families the classification named.
    pub(crate) observation_ids: Vec<AgentObservationId>,
    pub(crate) multiplexer_workspace_id: MultiplexerWorkspaceId,
    pub(crate) tab_id: TabId,
    pub(crate) pane_id: PaneId,
    pub(crate) detection: AgentDetection,
    pub(crate) source_class: AgentObservationSourceV2,
    pub(crate) provider_native_session_id: Option<String>,
    pub(crate) git_workspace_id: Option<String>,
    pub(crate) observed_unix_ms: i64,
}

impl ObservationCandidate {
    fn is_representable(&self) -> bool {
        self.observed_unix_ms >= 0
            && optional_text_fits(&self.git_workspace_id, MAX_V2_GIT_WORKSPACE_ID_BYTES)
            && optional_text_fits(
                &self.provider_native_session_id,
                MAX_V2_PROVIDER_SESSION_ID_BYTES,
            )
            && evidence_summary(&self.detection, self.source_class)
                .is_some_and(|summary| evidence_fits(&summary))
    }
}

fn optional_text_fits(value: &Option<String>, limit: usize) -> bool {
    match value {
        None => true,
        Some(text) => !text.is_empty() && text.len() <= limit,
    }
}

fn evidence_fits(summary: &str) -> bool {
    summary.len() <= MAX_V2_EVIDENCE_SUMMARY_BYTES
}

/// The structured summary Winds records for an observation.
///
/// This description is built from structured classification state only. It never
/// copies terminal output, a pane label, or other untrusted prose.
fn evidence_summary(
    detection: &AgentDetection,
    source: AgentObservationSourceV2,
) -> Option<String> {
    let state = match detection {
        AgentDetection::Observed { .. } => "OBSERVED",
        AgentDetection::Ambiguous { .. } => "AMBIGUOUS",
        AgentDetection::Unavailable { .. } => "UNAVAILABLE",
        AgentDetection::NearMatch => "NEAR_MATCH",
        AgentDetection::Unknown { .. } => "UNKNOWN",
        AgentDetection::UntrustedText { .. } => "UNTRUSTED_TEXT",
        AgentDetection::Stale => "STALE",
    };
    Some(format!("detection={state};source_class={source:?}"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BoundObservation {
    observation_id: AgentObservationId,
    multiplexer_workspace_id: MultiplexerWorkspaceId,
    tab_id: TabId,
    pane_id: PaneId,
    runtime_namespace_id: Option<RuntimeNamespaceId>,
    family: AgentFamilyV2,
    source_class: AgentObservationSourceV2,
    confidence_class: AgentObservationConfidenceV2,
    freshness: AgentObservationFreshnessV2,
    provider_native_session_id: Option<String>,
    git_workspace_id: Option<String>,
    observed_unix_ms: i64,
    structured_evidence_summary: String,
}

impl BoundObservation {
    fn to_wire(&self, owner_generation_id: OwnerGenerationId) -> AgentObservationV2 {
        AgentObservationV2 {
            observation_id: self.observation_id,
            family: self.family,
            source_class: self.source_class,
            confidence_class: self.confidence_class,
            freshness: self.freshness,
            multiplexer_workspace_id: self.multiplexer_workspace_id,
            git_workspace_id: self.git_workspace_id.clone(),
            tab_id: self.tab_id,
            pane_id: self.pane_id,
            runtime_namespace_id: self.runtime_namespace_id,
            provider_native_session_id: self.provider_native_session_id.clone(),
            owner_generation_id,
            observed_unix_ms: self.observed_unix_ms,
            structured_evidence_summary: self.structured_evidence_summary.clone(),
        }
    }
}

/// Bounded agent observations for one owner generation.
///
/// Records are keyed by pane plus deterministic candidate position. The number of
/// records is bounded by live topology and the finite compile-time detector
/// catalog, and wire projection is additionally bounded by the frozen page limit.
#[derive(Debug, Clone)]
pub(crate) struct AgentObservationStore {
    owner_generation_id: OwnerGenerationId,
    snapshot_revision: u64,
    records: BTreeMap<(PaneId, usize), BoundObservation>,
    observation_index: BTreeMap<AgentObservationId, PaneId>,
}

impl AgentObservationStore {
    pub(crate) fn new(owner_generation_id: OwnerGenerationId) -> Self {
        Self {
            owner_generation_id,
            snapshot_revision: 1,
            records: BTreeMap::new(),
            observation_index: BTreeMap::new(),
        }
    }

    pub(crate) const fn owner_generation_id(&self) -> OwnerGenerationId {
        self.owner_generation_id
    }

    pub(crate) const fn snapshot_revision(&self) -> u64 {
        self.snapshot_revision
    }

    pub(crate) fn observation_count(&self) -> usize {
        self.records.len()
    }

    /// Records one qualified candidate, replacing whatever the pane held when the
    /// candidate source is at least as authoritative as the accepted source.
    pub(crate) fn record(
        &mut self,
        topology: &MultiplexerTopology,
        owner_generation_id: OwnerGenerationId,
        candidate: ObservationCandidate,
    ) -> Result<Vec<AgentObservationEventV2>, ObservationError> {
        if owner_generation_id != self.owner_generation_id {
            return Err(ObservationError::StaleOwnerGeneration);
        }
        if !candidate.is_representable() {
            return Err(ObservationError::Unrepresentable);
        }

        let binding = topology.pane_runtime_binding(
            candidate.multiplexer_workspace_id,
            candidate.tab_id,
            candidate.pane_id,
        )?;
        let runtime_namespace_id = binding.map(|binding| binding.runtime_namespace_id);
        let next_revision = self
            .snapshot_revision
            .checked_add(1)
            .ok_or(ObservationError::SnapshotRevisionExhausted)?;
        let qualified = qualify(&candidate.detection);

        // Validate the complete identity set before any mutation. Checking only
        // the first identity is insufficient for ambiguous/unavailable outcomes:
        // a later ID could already denote another pane, or the candidate could
        // repeat one ID internally and construct a wire-invalid page.
        if qualified.is_named() {
            if candidate.observation_ids.len() != qualified.families.len() {
                return Err(ObservationError::Unrepresentable);
            }
            let mut candidate_ids = BTreeSet::new();
            for observation_id in &candidate.observation_ids {
                if !candidate_ids.insert(*observation_id)
                    || self
                        .observation_index
                        .get(observation_id)
                        .is_some_and(|bound| *bound != candidate.pane_id)
                {
                    return Err(ObservationError::ObservationIdentityReuse);
                }
            }
        }

        // Source ordering is pane-wide. A weaker source must not replace or erase
        // stronger accepted truth merely by reporting another family or no family.
        // The stronger source must itself update/withdraw, or the binding must
        // become stale, before weaker evidence can become authoritative.
        if self.records.values().any(|existing| {
            existing.pane_id == candidate.pane_id
                && source_rank(existing.source_class) < source_rank(candidate.source_class)
        }) {
            return Ok(Vec::new());
        }

        if !qualified.is_named() {
            let removed = self.withdraw(
                &candidate.pane_id,
                candidate.multiplexer_workspace_id,
                next_revision,
            );
            if !removed.is_empty() {
                self.snapshot_revision = next_revision;
            }
            return Ok(removed);
        }

        let confidence = confidence_for(candidate.source_class, qualified.freshness);
        let summary = evidence_summary(&candidate.detection, candidate.source_class)
            .ok_or(ObservationError::Unrepresentable)?;

        self.withdraw(
            &candidate.pane_id,
            candidate.multiplexer_workspace_id,
            next_revision,
        );

        let mut events = Vec::new();
        for (slot, (family, observation_id)) in qualified
            .families
            .into_iter()
            .zip(candidate.observation_ids)
            .enumerate()
        {
            let record = BoundObservation {
                observation_id,
                multiplexer_workspace_id: candidate.multiplexer_workspace_id,
                tab_id: candidate.tab_id,
                pane_id: candidate.pane_id,
                runtime_namespace_id,
                family,
                source_class: candidate.source_class,
                confidence_class: confidence,
                freshness: qualified.freshness,
                provider_native_session_id: candidate.provider_native_session_id.clone(),
                git_workspace_id: candidate.git_workspace_id.clone(),
                observed_unix_ms: candidate.observed_unix_ms,
                structured_evidence_summary: summary.clone(),
            };
            let wire = record.to_wire(self.owner_generation_id);
            self.records.insert((candidate.pane_id, slot), record);
            self.observation_index
                .insert(observation_id, candidate.pane_id);
            events.push(AgentObservationEventV2::Upsert {
                snapshot_revision: next_revision,
                observation: wire,
            });
        }
        self.snapshot_revision = next_revision;
        Ok(events)
    }

    /// Drops every observation whose exact binding no longer holds.
    pub(crate) fn invalidate_against(
        &mut self,
        topology: &MultiplexerTopology,
    ) -> Result<Vec<AgentObservationEventV2>, ObservationError> {
        let stale: BTreeMap<PaneId, MultiplexerWorkspaceId> = self
            .records
            .iter()
            .filter(|(_, record)| !binding_holds(topology, record))
            .map(|((pane_id, _), record)| (*pane_id, record.multiplexer_workspace_id))
            .collect();
        if stale.is_empty() {
            return Ok(Vec::new());
        }
        let next_revision = self
            .snapshot_revision
            .checked_add(1)
            .ok_or(ObservationError::SnapshotRevisionExhausted)?;
        let mut removed = Vec::new();
        for (pane_id, workspace_id) in stale {
            removed.extend(self.withdraw(&pane_id, workspace_id, next_revision));
        }
        self.snapshot_revision = next_revision;
        Ok(removed)
    }

    /// Removes every observation one pane held and returns removal events.
    fn withdraw(
        &mut self,
        pane_id: &PaneId,
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        next_revision: u64,
    ) -> Vec<AgentObservationEventV2> {
        let held: Vec<(PaneId, usize, AgentObservationId)> = self
            .records
            .iter()
            .filter(|((held_pane, _), _)| held_pane == pane_id)
            .map(|((held_pane, slot), record)| (*held_pane, *slot, record.observation_id))
            .collect();
        let mut removed = Vec::new();
        for (held_pane, slot, observation_id) in held {
            self.records.remove(&(held_pane, slot));
            self.observation_index.remove(&observation_id);
            removed.push(AgentObservationEventV2::Removed {
                snapshot_revision: next_revision,
                observation_id,
                multiplexer_workspace_id,
                pane_id: *pane_id,
            });
        }
        removed
    }

    /// The bounded snapshot projection.
    ///
    /// A stale cursor is a history gap. Recovery restarts at offset zero under the
    /// current snapshot revision and remains pageable; otherwise a snapshot larger
    /// than one page would silently omit authoritative observations after a gap.
    pub(crate) fn snapshot(
        &mut self,
        topology: &MultiplexerTopology,
        request: &ListAgentObservationsV2,
    ) -> Result<AgentObservationSnapshotV2, ObservationError> {
        self.invalidate_against(topology)?;

        let offset = match &request.cursor {
            None => 0_u16,
            Some(cursor) => {
                if cursor.owner_generation_id != self.owner_generation_id {
                    return Err(ObservationError::StaleOwnerGeneration);
                }
                if cursor.snapshot_revision != self.snapshot_revision {
                    0_u16
                } else {
                    cursor.offset
                }
            }
        };

        let page: Vec<AgentObservationV2> = self
            .records
            .values()
            .filter(|record| {
                request
                    .multiplexer_workspace_id
                    .is_none_or(|wanted| wanted == record.multiplexer_workspace_id)
            })
            .skip(usize::from(offset))
            .take(MAX_V2_AGENT_OBSERVATIONS_PER_PAGE)
            .map(|record| record.to_wire(self.owner_generation_id))
            .collect();

        let next_cursor = if usize::from(offset) + page.len() < self.page_size(request) {
            Some(AgentObservationCursorV2 {
                owner_generation_id: self.owner_generation_id,
                snapshot_revision: self.snapshot_revision,
                offset: offset
                    .checked_add(u16::try_from(page.len()).unwrap_or(u16::MAX))
                    .ok_or(ObservationError::Unrepresentable)?,
            })
        } else {
            None
        };

        Ok(AgentObservationSnapshotV2 {
            filter_multiplexer_workspace_id: request.multiplexer_workspace_id,
            snapshot_revision: self.snapshot_revision,
            page_offset: offset,
            observations: page,
            next_cursor,
        })
    }

    fn page_size(&self, request: &ListAgentObservationsV2) -> usize {
        self.records
            .values()
            .filter(|record| {
                request
                    .multiplexer_workspace_id
                    .is_none_or(|wanted| wanted == record.multiplexer_workspace_id)
            })
            .count()
    }

    /// The bounded event projection since `cursor`.
    pub(crate) fn events_since(
        &mut self,
        topology: &MultiplexerTopology,
        cursor: Option<&AgentObservationCursorV2>,
    ) -> Result<Vec<AgentObservationEventV2>, ObservationError> {
        self.invalidate_against(topology)?;
        let Some(cursor) = cursor else {
            return Ok(Vec::new());
        };
        if cursor.owner_generation_id != self.owner_generation_id {
            return Err(ObservationError::StaleOwnerGeneration);
        }
        if cursor.snapshot_revision != self.snapshot_revision {
            return Ok(vec![AgentObservationEventV2::HistoryGap {
                last_known_snapshot_revision: cursor.snapshot_revision,
            }]);
        }
        Ok(Vec::new())
    }
}

fn binding_holds(topology: &MultiplexerTopology, record: &BoundObservation) -> bool {
    topology
        .pane_runtime_binding(
            record.multiplexer_workspace_id,
            record.tab_id,
            record.pane_id,
        )
        .is_ok_and(|binding| {
            binding.map(|binding| binding.runtime_namespace_id) == record.runtime_namespace_id
        })
}

#[cfg(test)]
#[path = "../t173_agent_observation_tests.rs"]
mod t173_agent_observation_tests;

#[cfg(test)]
mod t173_review_regressions {
    use super::*;
    use crate::multiplexer::agent_catalog::{AgentSupport, DetectionSource, UnknownReason};
    use crate::multiplexer::domain::TopologyGeneration;
    use crate::multiplexer::domain::navigation::{
        LayoutNode, MultiplexerTopology, TabState, WorkspaceState,
    };

    fn owner(byte: u8) -> OwnerGenerationId {
        OwnerGenerationId::from_entropy_bytes([byte; 16]).expect("non-zero owner generation")
    }

    fn observation_id(byte: u8) -> AgentObservationId {
        AgentObservationId::from_entropy_bytes([byte; 16]).expect("non-zero observation id")
    }

    fn workspace_id(byte: u8) -> MultiplexerWorkspaceId {
        MultiplexerWorkspaceId::from_entropy_bytes([byte; 16]).expect("non-zero workspace id")
    }

    fn tab_id(byte: u8) -> TabId {
        TabId::from_entropy_bytes([byte; 16]).expect("non-zero tab id")
    }

    fn pane_id(byte: u8) -> PaneId {
        PaneId::from_entropy_bytes([byte; 16]).expect("non-zero pane id")
    }

    fn observed(family: AgentFamily) -> AgentDetection {
        AgentDetection::Observed {
            family,
            source: DetectionSource::StructuredMetadataNamespace,
            support: AgentSupport::DetectionOnly,
        }
    }

    fn all_families() -> Vec<AgentFamily> {
        vec![
            AgentFamily::Pi,
            AgentFamily::Claude,
            AgentFamily::Codex,
            AgentFamily::Gemini,
            AgentFamily::Cursor,
            AgentFamily::Devin,
            AgentFamily::Antigravity,
            AgentFamily::Cline,
            AgentFamily::Omp,
            AgentFamily::Mastracode,
            AgentFamily::OpenCode,
            AgentFamily::GithubCopilot,
            AgentFamily::Kimi,
            AgentFamily::Kiro,
            AgentFamily::Droid,
            AgentFamily::Amp,
            AgentFamily::Grok,
            AgentFamily::Hermes,
            AgentFamily::Kilo,
            AgentFamily::Qodercli,
            AgentFamily::Qwen,
            AgentFamily::Letta,
            AgentFamily::Maki,
            AgentFamily::Muse,
        ]
    }

    fn topology(
        workspaces: usize,
    ) -> (
        MultiplexerTopology,
        Vec<(MultiplexerWorkspaceId, TabId, PaneId)>,
    ) {
        let mut identities = Vec::with_capacity(workspaces);
        let mut states = Vec::with_capacity(workspaces);
        for index in 0..workspaces {
            let byte = u8::try_from(index + 1).expect("test topology stays under 255 workspaces");
            let workspace_id = workspace_id(byte);
            let tab_id = tab_id(byte);
            let pane_id = pane_id(byte);
            identities.push((workspace_id, tab_id, pane_id));
            states.push(WorkspaceState {
                id: workspace_id,
                alias: format!("w-{index}"),
                tabs: vec![TabState {
                    id: tab_id,
                    alias: "main".to_owned(),
                    root: LayoutNode::Pane(pane_id),
                    focused_pane_id: pane_id,
                    zoomed_pane_id: None,
                }],
                focused_tab_id: tab_id,
            });
        }
        let focused = identities.first().map(|(workspace_id, _, _)| *workspace_id);
        let topology = MultiplexerTopology::restore_presentation(
            TopologyGeneration::initial(),
            states,
            focused,
        )
        .expect("review topology is valid");
        (topology, identities)
    }

    fn candidate(
        workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        pane_id: PaneId,
        observation_ids: Vec<AgentObservationId>,
        detection: AgentDetection,
        source_class: AgentObservationSourceV2,
    ) -> ObservationCandidate {
        ObservationCandidate {
            observation_ids,
            multiplexer_workspace_id: workspace_id,
            tab_id,
            pane_id,
            detection,
            source_class,
            provider_native_session_id: None,
            git_workspace_id: None,
            observed_unix_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn t173_review_rejects_duplicate_ids_inside_one_multi_family_candidate() {
        let (topology, identities) = topology(1);
        let (workspace_id, tab_id, pane_id) = identities[0];
        let repeated = observation_id(0x40);
        let mut store = AgentObservationStore::new(owner(0xa1));
        let result = store.record(
            &topology,
            owner(0xa1),
            candidate(
                workspace_id,
                tab_id,
                pane_id,
                vec![repeated, repeated],
                AgentDetection::Ambiguous {
                    families: vec![AgentFamily::Pi, AgentFamily::Claude],
                },
                AgentObservationSourceV2::OwnedProcessMetadata,
            ),
        );
        assert_eq!(result, Err(ObservationError::ObservationIdentityReuse));
        assert_eq!(store.observation_count(), 0);
    }

    #[test]
    fn t173_review_rejects_reused_nonfirst_id_bound_to_another_pane() {
        let (topology, identities) = topology(2);
        let first = identities[0];
        let second = identities[1];
        let reused = observation_id(0x41);
        let mut store = AgentObservationStore::new(owner(0xa1));
        store
            .record(
                &topology,
                owner(0xa1),
                candidate(
                    first.0,
                    first.1,
                    first.2,
                    vec![reused],
                    observed(AgentFamily::Cline),
                    AgentObservationSourceV2::OwnedProcessMetadata,
                ),
            )
            .expect("first observation is recordable");

        let result = store.record(
            &topology,
            owner(0xa1),
            candidate(
                second.0,
                second.1,
                second.2,
                vec![observation_id(0x42), reused],
                AgentDetection::Ambiguous {
                    families: vec![AgentFamily::Pi, AgentFamily::Claude],
                },
                AgentObservationSourceV2::OwnedProcessMetadata,
            ),
        );
        assert_eq!(result, Err(ObservationError::ObservationIdentityReuse));
        assert_eq!(store.observation_count(), 1);
    }

    #[test]
    fn t173_review_stale_resnapshot_remains_pageable_after_history_gap() {
        let (topology, identities) = topology(6);
        let families = all_families();
        let mut store = AgentObservationStore::new(owner(0xa1));
        let mut next_id = 1_u8;
        for (workspace_id, tab_id, pane_id) in identities {
            let mut ids = Vec::with_capacity(families.len());
            for _ in &families {
                ids.push(observation_id(next_id));
                next_id = next_id.checked_add(1).expect("test IDs stay below 255");
            }
            store
                .record(
                    &topology,
                    owner(0xa1),
                    candidate(
                        workspace_id,
                        tab_id,
                        pane_id,
                        ids,
                        AgentDetection::Ambiguous {
                            families: families.clone(),
                        },
                        AgentObservationSourceV2::OwnedProcessMetadata,
                    ),
                )
                .expect("ambiguous observations are recordable");
        }
        assert_eq!(store.observation_count(), 144);

        let stale = AgentObservationCursorV2 {
            owner_generation_id: owner(0xa1),
            snapshot_revision: 1,
            offset: 77,
        };
        let first = store
            .snapshot(
                &topology,
                &ListAgentObservationsV2 {
                    multiplexer_workspace_id: None,
                    cursor: Some(stale),
                },
            )
            .expect("stale cursor restarts an authoritative snapshot");
        assert_eq!(first.page_offset, 0);
        assert_eq!(
            first.observations.len(),
            MAX_V2_AGENT_OBSERVATIONS_PER_PAGE
        );
        let continuation = first
            .next_cursor
            .expect("a >128-item resnapshot must remain pageable");
        assert_eq!(continuation.snapshot_revision, store.snapshot_revision());
        assert_eq!(continuation.offset, 128);

        let second = store
            .snapshot(
                &topology,
                &ListAgentObservationsV2 {
                    multiplexer_workspace_id: None,
                    cursor: Some(continuation),
                },
            )
            .expect("continuation completes the authoritative resnapshot");
        assert_eq!(second.page_offset, 128);
        assert_eq!(second.observations.len(), 16);
        assert!(second.next_cursor.is_none());
    }

    #[test]
    fn t173_review_weaker_source_cannot_erase_stronger_current_truth() {
        let (topology, identities) = topology(1);
        let (workspace_id, tab_id, pane_id) = identities[0];
        let mut store = AgentObservationStore::new(owner(0xa1));
        store
            .record(
                &topology,
                owner(0xa1),
                candidate(
                    workspace_id,
                    tab_id,
                    pane_id,
                    vec![observation_id(0x50)],
                    observed(AgentFamily::Claude),
                    AgentObservationSourceV2::OwnedProcessMetadata,
                ),
            )
            .expect("strong observation is recordable");

        let events = store
            .record(
                &topology,
                owner(0xa1),
                candidate(
                    workspace_id,
                    tab_id,
                    pane_id,
                    Vec::new(),
                    AgentDetection::Unknown {
                        reason: UnknownReason::NoMatch,
                    },
                    AgentObservationSourceV2::UserDeclaredPresentation,
                ),
            )
            .expect("weaker unknown evidence is a no-op, not an eraser");
        assert!(events.is_empty());
        let snapshot = store
            .snapshot(
                &topology,
                &ListAgentObservationsV2 {
                    multiplexer_workspace_id: None,
                    cursor: None,
                },
            )
            .expect("snapshot remains available");
        assert_eq!(snapshot.observations.len(), 1);
        assert_eq!(snapshot.observations[0].family, AgentFamilyV2::Claude);
        assert_eq!(
            snapshot.observations[0].source_class,
            AgentObservationSourceV2::OwnedProcessMetadata
        );
    }
}
