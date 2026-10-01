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
//! - a detection is not an execution. [`AgentSupport`] has no variant that can
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
use std::collections::BTreeMap;

/// Every way an observation can be refused. All of them fail closed: a refused
/// candidate changes nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ObservationError {
    /// The topology rejected the claimed `(workspace, tab, pane)` triple.
    Topology(MultiplexerErrorKind),
    /// The candidate was offered under a different owner generation, so nothing
    /// it claims about a binding can be trusted against this owner.
    StaleOwnerGeneration,
    /// One observation identity is already bound to a different pane.
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
/// accepted source wins, so a later weak description can never downgrade a
/// classification that a stronger source already established, and a stronger
/// source can replace a weaker one.
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
    /// held is withdrawn.
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
///
/// Two rules, and they compose. A user-declared presentation is capped at
/// `UserDeclared`, because a presentation label can accompany an observation
/// Winds qualified from structured input but can never raise confidence above
/// what the user asserted. And anything short of a single current classification
/// is `Unknown`, so an ambiguous or unavailable outcome can never be read as a
/// confident attribution of one family.
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
///
/// `multiplexer_workspace_id` is required and `git_workspace_id` is optional and
/// independent, because a multiplexer workspace is a presentation container and a
/// Git workspace is a repository fact. Neither implies the other, and a
/// `GitWorkspaceId` never identifies a pane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObservationCandidate {
    /// One observation identity per candidate family, positionally matching the
    /// families the classification named. The frozen page validator rejects a page
    /// that repeats an identity, so ambiguity cannot share one.
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
    /// Whether every optional field fits the frozen wire budget.
    ///
    /// Checked before a record is ever built, so an unrepresentable candidate is
    /// refused rather than persisted and refused later by the protocol validator.
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
/// This is a description of the classification Winds made, built from the
/// classification alone. It never copies terminal output, a pane label, or any
/// other untrusted text, so it cannot carry model-generated prose into a field a
/// later task might read as evidence.
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
    let summary = format!("detection={state};source_class={source:?}");
    Some(summary)
}

/// One observation bound to the exact identity it was recorded against.
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
    /// The frozen wire projection of this observation.
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
/// Records are keyed by pane, so the projection order is deterministic and the
/// record count is bounded by the topology's pane count. Each observation identity
/// is additionally indexed so that one identity can never come to denote two
/// panes.
#[derive(Debug, Clone)]
pub(crate) struct AgentObservationStore {
    owner_generation_id: OwnerGenerationId,
    snapshot_revision: u64,
    ///
    /// Keyed by pane and by the candidate family's position, which is the pinned
    /// ledger order the catalog supplies. The frozen wire family carries no
    /// ordering and must not be given one, so the position stands in for it and
    /// keeps the projection deterministic.
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

    /// Records one qualified candidate, replacing whatever the pane held.
    ///
    /// The pane's `(workspace, tab, pane)` triple is resolved against live
    /// topology, and the runtime reference is taken from the owner-accepted
    /// binding rather than from the candidate. A candidate therefore cannot claim
    /// a pane that does not exist or a runtime that is not bound to it.
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
        // The runtime reference is observed, never asserted.
        let runtime_namespace_id = binding.map(|binding| binding.runtime_namespace_id);
        // Every refusal is decided before any mutation, so a refused candidate
        // leaves the store exactly as it found it.
        let next_revision = self
            .snapshot_revision
            .checked_add(1)
            .ok_or(ObservationError::SnapshotRevisionExhausted)?;

        let qualified = qualify(&candidate.detection);
        let previous = self.records.get(&(candidate.pane_id, 0)).cloned();

        if !qualified.is_named() {
            // Nothing attributable to name, so no observation exists for this
            // pane and any earlier one is withdrawn rather than left to look
            // current.
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

        // A weaker source never replaces a stronger accepted classification for
        // the same pane and family.
        if let Some(existing) = &previous {
            let stronger = source_rank(existing.source_class) < source_rank(candidate.source_class);
            if stronger
                && existing.family == candidate.primary_family()
                && existing.freshness == AgentObservationFreshnessV2::Current
                && candidate.detection_is_observed()
            {
                return Ok(Vec::new());
            }
        }

        let confidence = confidence_for(candidate.source_class, qualified.freshness);
        let summary = evidence_summary(&candidate.detection, candidate.source_class)
            .ok_or(ObservationError::Unrepresentable)?;

        // One observation identity per candidate family. The frozen page validator
        // rejects a page that repeats an identity, so accepting fewer identities
        // than families would build a page the owner cannot send.
        if candidate.observation_ids.len() != qualified.families.len() {
            return Err(ObservationError::Unrepresentable);
        }
        // One observation identity may denote only one pane.
        if candidate.observation_ids.first().is_some_and(|first| {
            self.observation_index
                .get(first)
                .is_some_and(|bound| *bound != candidate.pane_id)
        }) {
            return Err(ObservationError::ObservationIdentityReuse);
        }

        // Withdraw whatever the pane held, so a family set that changed size
        // cannot leave an orphan behind.
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
    ///
    /// An observation survives only while its pane is still live in the same
    /// workspace and tab and still resolves to the same runtime reference. A
    /// closed pane, a retired workspace or tab, a replaced runtime, an unbound
    /// runtime, or a newly bound runtime all invalidate it, because each is a
    /// change to the identity the observation was bound to.
    pub(crate) fn invalidate_against(
        &mut self,
        topology: &MultiplexerTopology,
    ) -> Result<Vec<AgentObservationEventV2>, ObservationError> {
        let stale: Vec<(PaneId, MultiplexerWorkspaceId)> = self
            .records
            .iter()
            .filter(|((_, _), record)| !binding_holds(topology, record))
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

    /// Removes one pane's observation and returns the removal event.
    /// Removes every observation one pane held and returns the removal events.
    ///
    /// A pane may hold more than one observation when a classification named several
    /// families, so this removes by pane rather than by a single key.
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
    /// Invalidates against live topology first, so a projection can never carry
    /// an observation whose binding has already gone. A cursor from another owner
    /// generation is refused. A cursor from an older snapshot revision is a
    /// history gap, and the recovery is a full resnapshot from offset zero rather
    /// than a partial page that would silently omit whatever changed.
    pub(crate) fn snapshot(
        &mut self,
        topology: &MultiplexerTopology,
        request: &ListAgentObservationsV2,
    ) -> Result<AgentObservationSnapshotV2, ObservationError> {
        self.invalidate_against(topology)?;

        let (offset, continuation) = match &request.cursor {
            None => (0_u16, true),
            Some(cursor) => {
                if cursor.owner_generation_id != self.owner_generation_id {
                    return Err(ObservationError::StaleOwnerGeneration);
                }
                if cursor.snapshot_revision != self.snapshot_revision {
                    (0_u16, false)
                } else {
                    (cursor.offset, true)
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

        let next_cursor =
            if continuation && usize::from(offset) + page.len() < self.page_size(request) {
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

    /// How many observations the request selects, after filtering.
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
    ///
    /// A cursor from an older snapshot revision has missed events, so the result
    /// is an explicit history gap rather than a partial event list that a caller
    /// could mistake for a complete one.
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

/// Whether one observation's exact binding still holds against live topology.
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

/// The one family an ambiguous or unavailable classification can be attributed to.
///
/// An observation names exactly one family, so a multi-family outcome is stored as
/// one observation per candidate family and each is a candidate rather than a
/// claim. This exposes the first candidate for the single-family comparison the
/// downgrade guard needs; it is never used to *select* a winner.
impl ObservationCandidate {
    fn primary_family(&self) -> AgentFamilyV2 {
        match &self.detection {
            AgentDetection::Observed { family, .. } => as_wire_family(*family),
            _ => AgentFamilyV2::Pi,
        }
    }

    /// Whether the detector qualified a single family as current.
    fn detection_is_observed(&self) -> bool {
        matches!(self.detection, AgentDetection::Observed { .. })
    }
}

#[cfg(test)]
#[path = "../t173_agent_observation_tests.rs"]
mod t173_agent_observation_tests;
