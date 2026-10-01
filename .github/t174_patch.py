from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(
            f"{path}: expected one replacement target, found {count}: {old[:80]!r}"
        )
    p.write_text(text.replace(old, new, 1), encoding="utf-8")


# Desktop bridge types.
replace_once(
    "desktop/src/leftDock/types.ts",
    'export interface BridgeSnapshot {\n  readonly projects: readonly BridgeProject[];\n}\n',
    '''export type BridgeAgentDockAvailability = "CURRENT" | "UNAVAILABLE";

export interface BridgeAgentObservation {
  readonly observationId: string;
  readonly family: string;
  readonly sourceClass: string;
  readonly confidenceClass: string;
  readonly freshness: string;
  readonly multiplexerWorkspaceId: string;
  readonly gitWorkspaceId: string | null;
  readonly tabId: string;
  readonly paneId: string;
  readonly runtimeNamespaceId: string | null;
  readonly providerNativeSessionId: string | null;
  readonly ownerGenerationId: string;
  readonly observedUnixMs: number;
  readonly structuredEvidenceSummary: string;
  readonly executionAuthority: "DETECTION_ONLY_UNPROVEN";
}

export interface BridgeAgentDockSnapshot {
  readonly authority: "OWNER_AUTHORITATIVE_AGENT_OBSERVATIONS";
  readonly availability: BridgeAgentDockAvailability;
  readonly observations: readonly BridgeAgentObservation[];
  readonly detectionOnly: true;
  readonly unavailableReason: string | null;
}

export interface BridgeSnapshot {
  readonly projects: readonly BridgeProject[];
  readonly agentDock: BridgeAgentDockSnapshot;
}
''',
)

# Fixture observation data stays explicitly non-authoritative outside the trusted host.
replace_once(
    "desktop/src/leftDock/fixture.ts",
    'const fixture: BridgeSnapshot = {\n  projects: [',
    '''const fixture: BridgeSnapshot = {
  agentDock: {
    authority: "OWNER_AUTHORITATIVE_AGENT_OBSERVATIONS",
    availability: "CURRENT",
    detectionOnly: true,
    unavailableReason: null,
    observations: [
      {
        observationId: "11111111111111111111111111111111",
        family: "CODEX",
        sourceClass: "PROVIDER_STRUCTURED_METADATA",
        confidenceClass: "STRONG",
        freshness: "CURRENT",
        multiplexerWorkspaceId: "21212121212121212121212121212121",
        gitWorkspaceId: "fixture-winds",
        tabId: "31313131313131313131313131313131",
        paneId: "41414141414141414141414141414141",
        runtimeNamespaceId: "51515151515151515151515151515151",
        providerNativeSessionId: "fixture-provider-a",
        ownerGenerationId: "61616161616161616161616161616161",
        observedUnixMs: 1,
        structuredEvidenceSummary: "fixture structured metadata only",
        executionAuthority: "DETECTION_ONLY_UNPROVEN",
      },
      {
        observationId: "12121212121212121212121212121212",
        family: "CODEX",
        sourceClass: "USER_DECLARED_PRESENTATION",
        confidenceClass: "USER_DECLARED",
        freshness: "CURRENT",
        multiplexerWorkspaceId: "21212121212121212121212121212121",
        gitWorkspaceId: "fixture-winds",
        tabId: "31313131313131313131313131313131",
        paneId: "42424242424242424242424242424242",
        runtimeNamespaceId: null,
        providerNativeSessionId: null,
        ownerGenerationId: "61616161616161616161616161616161",
        observedUnixMs: 2,
        structuredEvidenceSummary: "fixture user declaration only",
        executionAuthority: "DETECTION_ONLY_UNPROVEN",
      },
    ],
  },
  projects: [''',
)
replace_once(
    "desktop/src/leftDock/fixture.ts",
    '  t143PerformanceFixture = { projects: [...fixture.projects, ...projects] };',
    '  t143PerformanceFixture = { projects: [...fixture.projects, ...projects], agentDock: fixture.agentDock };',
)

# Mount AgentDock as the smallest existing dock extension.
replace_once(
    "desktop/src/leftDock/LeftDock.tsx",
    'import { flushSync } from "react-dom";\n',
    'import { flushSync } from "react-dom";\nimport { AgentDock } from "../agentDock/AgentDock";\n',
)
replace_once(
    "desktop/src/leftDock/LeftDock.tsx",
    'const emptySnapshot: BridgeSnapshot = { projects: [] };',
    '''const emptySnapshot: BridgeSnapshot = {
  projects: [],
  agentDock: {
    authority: "OWNER_AUTHORITATIVE_AGENT_OBSERVATIONS",
    availability: "UNAVAILABLE",
    observations: [],
    detectionOnly: true,
    unavailableReason: "SNAPSHOT_NOT_LOADED",
  },
};''',
)
replace_once(
    "desktop/src/leftDock/LeftDock.tsx",
    '      <div className="dock-foot">',
    '      <AgentDock snapshot={snapshot.agentDock} />\n      <div className="dock-foot">',
)

# Rust desktop bridge: add live read-only agent observations to the already-typed left_dock_snapshot.
desktop = Path("src/desktop.rs")
text = desktop.read_text(encoding="utf-8")
prefix = '''use crate::multiplexer::protocol::{
    AgentFamilyV2, AgentObservationConfidenceV2, AgentObservationFreshnessV2,
    AgentObservationSourceV2, AgentObservationV2,
};
use crate::persistent_runtime::client::RustLocalControlClient;
'''
if prefix not in text:
    text = prefix + text
desktop.write_text(text, encoding="utf-8")

replace_once(
    "src/desktop.rs",
    '''#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeSnapshot {
    pub projects: Vec<DesktopBridgeProject>,
}
''',
    '''#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeAgentObservation {
    pub observation_id: String,
    pub family: String,
    pub source_class: String,
    pub confidence_class: String,
    pub freshness: String,
    pub multiplexer_workspace_id: String,
    pub git_workspace_id: Option<String>,
    pub tab_id: String,
    pub pane_id: String,
    pub runtime_namespace_id: Option<String>,
    pub provider_native_session_id: Option<String>,
    pub owner_generation_id: String,
    pub observed_unix_ms: i64,
    pub structured_evidence_summary: String,
    pub execution_authority: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeAgentDockSnapshot {
    pub authority: String,
    pub availability: String,
    pub observations: Vec<DesktopBridgeAgentObservation>,
    pub detection_only: bool,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeSnapshot {
    pub projects: Vec<DesktopBridgeProject>,
    pub agent_dock: DesktopBridgeAgentDockSnapshot,
}
''',
)

desktop_helpers = r'''
const DESKTOP_AGENT_DOCK_AUTHORITY: &str = "OWNER_AUTHORITATIVE_AGENT_OBSERVATIONS";
const DESKTOP_AGENT_EXECUTION_NONCLAIM: &str = "DETECTION_ONLY_UNPROVEN";

fn desktop_agent_family(value: AgentFamilyV2) -> &'static str {
    match value {
        AgentFamilyV2::Pi => "PI",
        AgentFamilyV2::Claude => "CLAUDE",
        AgentFamilyV2::Codex => "CODEX",
        AgentFamilyV2::Gemini => "GEMINI",
        AgentFamilyV2::Cursor => "CURSOR",
        AgentFamilyV2::Devin => "DEVIN",
        AgentFamilyV2::Antigravity => "ANTIGRAVITY",
        AgentFamilyV2::Cline => "CLINE",
        AgentFamilyV2::Omp => "OMP",
        AgentFamilyV2::Mastracode => "MASTRACODE",
        AgentFamilyV2::OpenCode => "OPEN_CODE",
        AgentFamilyV2::GithubCopilot => "GITHUB_COPILOT",
        AgentFamilyV2::Kimi => "KIMI",
        AgentFamilyV2::Kiro => "KIRO",
        AgentFamilyV2::Droid => "DROID",
        AgentFamilyV2::Amp => "AMP",
        AgentFamilyV2::Grok => "GROK",
        AgentFamilyV2::Hermes => "HERMES",
        AgentFamilyV2::Kilo => "KILO",
        AgentFamilyV2::Qodercli => "QODERCLI",
        AgentFamilyV2::Qwen => "QWEN",
        AgentFamilyV2::Letta => "LETTA",
        AgentFamilyV2::Maki => "MAKI",
        AgentFamilyV2::Muse => "MUSE",
    }
}

fn desktop_agent_source(value: AgentObservationSourceV2) -> &'static str {
    match value {
        AgentObservationSourceV2::WindsLaunchMetadata => "WINDS_LAUNCH_METADATA",
        AgentObservationSourceV2::OwnedProcessMetadata => "OWNED_PROCESS_METADATA",
        AgentObservationSourceV2::ProviderStructuredMetadata => "PROVIDER_STRUCTURED_METADATA",
        AgentObservationSourceV2::UserDeclaredPresentation => "USER_DECLARED_PRESENTATION",
    }
}

fn desktop_agent_confidence(value: AgentObservationConfidenceV2) -> &'static str {
    match value {
        AgentObservationConfidenceV2::Exact => "EXACT",
        AgentObservationConfidenceV2::Strong => "STRONG",
        AgentObservationConfidenceV2::UserDeclared => "USER_DECLARED",
        AgentObservationConfidenceV2::Unknown => "UNKNOWN",
    }
}

fn desktop_agent_freshness(value: AgentObservationFreshnessV2) -> &'static str {
    match value {
        AgentObservationFreshnessV2::Current => "CURRENT",
        AgentObservationFreshnessV2::Ambiguous => "AMBIGUOUS",
        AgentObservationFreshnessV2::Stale => "STALE",
        AgentObservationFreshnessV2::Unavailable => "UNAVAILABLE",
        AgentObservationFreshnessV2::Unknown => "UNKNOWN",
    }
}

impl From<AgentObservationV2> for DesktopBridgeAgentObservation {
    fn from(value: AgentObservationV2) -> Self {
        Self {
            observation_id: value.observation_id.to_string(),
            family: desktop_agent_family(value.family).to_owned(),
            source_class: desktop_agent_source(value.source_class).to_owned(),
            confidence_class: desktop_agent_confidence(value.confidence_class).to_owned(),
            freshness: desktop_agent_freshness(value.freshness).to_owned(),
            multiplexer_workspace_id: value.multiplexer_workspace_id.to_string(),
            git_workspace_id: value.git_workspace_id,
            tab_id: value.tab_id.to_string(),
            pane_id: value.pane_id.to_string(),
            runtime_namespace_id: value.runtime_namespace_id.map(|id| id.to_string()),
            provider_native_session_id: value.provider_native_session_id,
            owner_generation_id: value.owner_generation_id.to_string(),
            observed_unix_ms: value.observed_unix_ms,
            structured_evidence_summary: value.structured_evidence_summary,
            execution_authority: DESKTOP_AGENT_EXECUTION_NONCLAIM.to_owned(),
        }
    }
}

fn unavailable_desktop_agent_dock(reason: &str) -> DesktopBridgeAgentDockSnapshot {
    DesktopBridgeAgentDockSnapshot {
        authority: DESKTOP_AGENT_DOCK_AUTHORITY.to_owned(),
        availability: "UNAVAILABLE".to_owned(),
        observations: Vec::new(),
        detection_only: true,
        unavailable_reason: Some(reason.to_owned()),
    }
}

fn desktop_bridge_agent_dock_snapshot(store: &Store) -> DesktopBridgeAgentDockSnapshot {
    let owner_generation_id = match store.latest_persistent_runtime_owner_generation() {
        Ok(Some(owner_generation_id)) => owner_generation_id,
        Ok(None) => return unavailable_desktop_agent_dock("OWNER_GENERATION_UNAVAILABLE"),
        Err(_) => return unavailable_desktop_agent_dock("OWNER_GENERATION_RECORD_UNAVAILABLE"),
    };
    let mut client = match RustLocalControlClient::connect(None, Some(owner_generation_id)) {
        Ok(client) => client,
        Err(_) => return unavailable_desktop_agent_dock("OWNER_CONNECTION_UNAVAILABLE"),
    };
    let mut observations = match client.refresh_agent_observation_projection(None) {
        Ok(observations) => observations
            .into_iter()
            .map(DesktopBridgeAgentObservation::from)
            .collect::<Vec<_>>(),
        Err(_) => return unavailable_desktop_agent_dock("OBSERVATION_REFRESH_UNAVAILABLE"),
    };
    observations.sort_by(|left, right| left.observation_id.cmp(&right.observation_id));
    DesktopBridgeAgentDockSnapshot {
        authority: DESKTOP_AGENT_DOCK_AUTHORITY.to_owned(),
        availability: "CURRENT".to_owned(),
        observations,
        detection_only: true,
        unavailable_reason: None,
    }
}

'''
replace_once(
    "src/desktop.rs",
    'pub fn desktop_bridge_snapshot(home: &std::path::Path) -> Result<DesktopBridgeSnapshot> {\n',
    desktop_helpers
    + 'pub fn desktop_bridge_snapshot(home: &std::path::Path) -> Result<DesktopBridgeSnapshot> {\n',
)
replace_once(
    "src/desktop.rs",
    '    Ok(DesktopBridgeSnapshot { projects })\n',
    '    let agent_dock = desktop_bridge_agent_dock_snapshot(&store);\n    Ok(DesktopBridgeSnapshot { projects, agent_dock })\n',
)

# TUI projection remains presentation-only: no ApplyTopologyOperation is returned by agent focus.
replace_once(
    "src/workbench_ui.rs",
    'use std::time::Duration;\n',
    'use std::collections::BTreeMap;\nuse std::time::Duration;\n',
)

tui_agent = r'''
const T174_MAX_DISPLAY_ALIAS_BYTES: usize = 128;
const T174_DETECTION_ONLY_NONCLAIM: &str = "DETECTION_ONLY_UNPROVEN";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalAgentDockSort {
    Family,
    Recent,
    Source,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalAgentDockItem {
    pub(crate) observation: crate::persistent_runtime::protocol::AgentObservationV2,
    pub(crate) display_alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalAgentDockListItem {
    pub(crate) observation_id: crate::multiplexer::domain::AgentObservationId,
    pub(crate) display_label: String,
    pub(crate) family: crate::persistent_runtime::protocol::AgentFamilyV2,
    pub(crate) source_class: crate::persistent_runtime::protocol::AgentObservationSourceV2,
    pub(crate) freshness: crate::persistent_runtime::protocol::AgentObservationFreshnessV2,
    pub(crate) multiplexer_workspace_id: crate::multiplexer::domain::MultiplexerWorkspaceId,
    pub(crate) tab_id: crate::multiplexer::domain::TabId,
    pub(crate) pane_id: crate::multiplexer::domain::PaneId,
    pub(crate) runtime_namespace_id: Option<crate::persistent_runtime::domain::RuntimeNamespaceId>,
    pub(crate) observed_unix_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CanonicalAgentPaneViewTarget {
    pub(crate) observation_id: crate::multiplexer::domain::AgentObservationId,
    pub(crate) multiplexer_workspace_id: crate::multiplexer::domain::MultiplexerWorkspaceId,
    pub(crate) tab_id: crate::multiplexer::domain::TabId,
    pub(crate) pane_id: crate::multiplexer::domain::PaneId,
    pub(crate) runtime_namespace_id: Option<crate::persistent_runtime::domain::RuntimeNamespaceId>,
    pub(crate) owner_generation_id: crate::persistent_runtime::domain::OwnerGenerationId,
    pub(crate) topology_generation: crate::multiplexer::domain::TopologyGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalAgentDockPresentation {
    items: BTreeMap<crate::multiplexer::domain::AgentObservationId, CanonicalAgentDockItem>,
    selected_observation_id: Option<crate::multiplexer::domain::AgentObservationId>,
    focused_observation_id: Option<crate::multiplexer::domain::AgentObservationId>,
}

impl CanonicalAgentDockPresentation {
    pub(crate) fn from_observations(
        observations: Vec<crate::persistent_runtime::protocol::AgentObservationV2>,
    ) -> Result<Self, String> {
        let mut items = BTreeMap::new();
        for observation in observations {
            let observation_id = observation.observation_id;
            if items
                .insert(
                    observation_id,
                    CanonicalAgentDockItem {
                        observation,
                        display_alias: None,
                    },
                )
                .is_some()
            {
                return Err("T174 agent dock received a duplicate immutable observation id".to_owned());
            }
        }
        Ok(Self {
            items,
            selected_observation_id: None,
            focused_observation_id: None,
        })
    }

    pub(crate) fn from_client(
        client: &crate::persistent_runtime::client::RustLocalControlClient,
    ) -> Option<Self> {
        Self::from_observations(client.trusted_agent_observations()?).ok()
    }

    pub(crate) fn list(
        &self,
        query: &str,
        sort: CanonicalAgentDockSort,
        group_by_workspace: bool,
    ) -> Vec<CanonicalAgentDockListItem> {
        let query = query.trim().to_ascii_lowercase();
        let mut rows = self
            .items
            .values()
            .filter(|item| t174_agent_item_matches(item, &query))
            .map(t174_agent_list_item)
            .collect::<Vec<_>>();
        rows.sort_by(|left, right| {
            let group_order = group_by_workspace
                .then(|| left.multiplexer_workspace_id.cmp(&right.multiplexer_workspace_id))
                .unwrap_or(std::cmp::Ordering::Equal);
            if group_order != std::cmp::Ordering::Equal {
                return group_order;
            }
            let primary = match sort {
                CanonicalAgentDockSort::Family => t174_agent_family_label(left.family)
                    .cmp(t174_agent_family_label(right.family)),
                CanonicalAgentDockSort::Recent => {
                    right.observed_unix_ms.cmp(&left.observed_unix_ms)
                }
                CanonicalAgentDockSort::Source => t174_agent_source_label(left.source_class)
                    .cmp(t174_agent_source_label(right.source_class)),
            };
            primary.then(left.observation_id.cmp(&right.observation_id))
        });
        rows
    }

    pub(crate) fn get(
        &self,
        observation_id: crate::multiplexer::domain::AgentObservationId,
    ) -> Option<&CanonicalAgentDockItem> {
        self.items.get(&observation_id)
    }

    pub(crate) fn read(
        &self,
        observation_id: crate::multiplexer::domain::AgentObservationId,
    ) -> Option<&crate::persistent_runtime::protocol::AgentObservationV2> {
        self.get(observation_id).map(|item| &item.observation)
    }

    pub(crate) fn explain(
        &self,
        observation_id: crate::multiplexer::domain::AgentObservationId,
    ) -> Option<String> {
        let observation = self.read(observation_id)?;
        Some(format!(
            "observation={}\nfamily={}\nsource={}\nconfidence={}\nfreshness={}\nworkspace={}\ntab={}\npane={}\nruntime={}\nowner_generation={}\nexecution={}\nevidence={}",
            observation.observation_id,
            t174_agent_family_label(observation.family),
            t174_agent_source_label(observation.source_class),
            t174_agent_confidence_label(observation.confidence_class),
            t174_agent_freshness_label(observation.freshness),
            observation.multiplexer_workspace_id,
            observation.tab_id,
            observation.pane_id,
            observation
                .runtime_namespace_id
                .map(|value| value.to_string())
                .unwrap_or_else(|| "UNBOUND".to_owned()),
            observation.owner_generation_id,
            T174_DETECTION_ONLY_NONCLAIM,
            observation.structured_evidence_summary,
        ))
    }

    pub(crate) fn view(
        &mut self,
        observation_id: crate::multiplexer::domain::AgentObservationId,
    ) -> Option<CanonicalAgentDockItem> {
        let item = self.items.get(&observation_id)?.clone();
        self.selected_observation_id = Some(observation_id);
        Some(item)
    }

    pub(crate) fn rename(
        &mut self,
        observation_id: crate::multiplexer::domain::AgentObservationId,
        display_alias: Option<&str>,
    ) -> Result<(), String> {
        let item = self
            .items
            .get_mut(&observation_id)
            .ok_or_else(|| "T174 agent rename target is absent".to_owned())?;
        let alias = display_alias.map(str::trim).filter(|value| !value.is_empty());
        if alias.is_some_and(|value| value.len() > T174_MAX_DISPLAY_ALIAS_BYTES) {
            return Err(format!(
                "T174 display alias exceeds {T174_MAX_DISPLAY_ALIAS_BYTES} bytes"
            ));
        }
        item.display_alias = alias.map(str::to_owned);
        Ok(())
    }

    pub(crate) fn focus(
        &mut self,
        observation_id: crate::multiplexer::domain::AgentObservationId,
        topology: &CanonicalTopologyPresentation,
    ) -> Option<CanonicalAgentPaneViewTarget> {
        let observation = &self.items.get(&observation_id)?.observation;
        if observation.freshness
            != crate::persistent_runtime::protocol::AgentObservationFreshnessV2::Current
        {
            return None;
        }
        let exact_pane_is_present = topology.search_bindings.iter().any(|binding| {
            if binding.intent.expected_topology_generation != topology.topology_generation() {
                return false;
            }
            matches!(
                &binding.intent.operation,
                crate::persistent_runtime::protocol::TopologyOperationV2::FocusPane {
                    multiplexer_workspace_id,
                    tab_id,
                    pane_id,
                } if *multiplexer_workspace_id == observation.multiplexer_workspace_id
                    && *tab_id == observation.tab_id
                    && *pane_id == observation.pane_id
            )
        });
        if !exact_pane_is_present {
            return None;
        }
        let target = CanonicalAgentPaneViewTarget {
            observation_id,
            multiplexer_workspace_id: observation.multiplexer_workspace_id,
            tab_id: observation.tab_id,
            pane_id: observation.pane_id,
            runtime_namespace_id: observation.runtime_namespace_id,
            owner_generation_id: observation.owner_generation_id,
            topology_generation: topology.topology_generation(),
        };
        self.selected_observation_id = Some(observation_id);
        self.focused_observation_id = Some(observation_id);
        Some(target)
    }

    pub(crate) const fn focused_observation_id(
        &self,
    ) -> Option<crate::multiplexer::domain::AgentObservationId> {
        self.focused_observation_id
    }

    pub(crate) fn text(&self) -> String {
        let mut lines = vec![
            "AGENT_DOCK=OWNER_AUTHORITATIVE_AGENT_OBSERVATIONS".to_owned(),
            format!("execution={T174_DETECTION_ONLY_NONCLAIM}"),
        ];
        for row in self.list("", CanonicalAgentDockSort::Family, true) {
            lines.push(format!(
                "{} | family={} | source={} | freshness={} | workspace={} | tab={} | pane={} | observation={}",
                row.display_label,
                t174_agent_family_label(row.family),
                t174_agent_source_label(row.source_class),
                t174_agent_freshness_label(row.freshness),
                row.multiplexer_workspace_id,
                row.tab_id,
                row.pane_id,
                row.observation_id,
            ));
        }
        lines.join("\n")
    }
}

fn t174_agent_list_item(item: &CanonicalAgentDockItem) -> CanonicalAgentDockListItem {
    let observation = &item.observation;
    CanonicalAgentDockListItem {
        observation_id: observation.observation_id,
        display_label: item
            .display_alias
            .clone()
            .unwrap_or_else(|| t174_agent_family_label(observation.family).to_owned()),
        family: observation.family,
        source_class: observation.source_class,
        freshness: observation.freshness,
        multiplexer_workspace_id: observation.multiplexer_workspace_id,
        tab_id: observation.tab_id,
        pane_id: observation.pane_id,
        runtime_namespace_id: observation.runtime_namespace_id,
        observed_unix_ms: observation.observed_unix_ms,
    }
}

fn t174_agent_item_matches(item: &CanonicalAgentDockItem, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let observation = &item.observation;
    [
        item.display_alias.as_deref().unwrap_or_default().to_owned(),
        t174_agent_family_label(observation.family).to_owned(),
        t174_agent_source_label(observation.source_class).to_owned(),
        t174_agent_freshness_label(observation.freshness).to_owned(),
        observation.observation_id.to_string(),
        observation.multiplexer_workspace_id.to_string(),
        observation.tab_id.to_string(),
        observation.pane_id.to_string(),
        observation
            .runtime_namespace_id
            .map(|value| value.to_string())
            .unwrap_or_default(),
        observation
            .provider_native_session_id
            .clone()
            .unwrap_or_default(),
        observation.structured_evidence_summary.clone(),
    ]
    .into_iter()
    .any(|value| value.to_ascii_lowercase().contains(query))
}

fn t174_agent_family_label(
    value: crate::persistent_runtime::protocol::AgentFamilyV2,
) -> &'static str {
    use crate::persistent_runtime::protocol::AgentFamilyV2;
    match value {
        AgentFamilyV2::Pi => "PI",
        AgentFamilyV2::Claude => "CLAUDE",
        AgentFamilyV2::Codex => "CODEX",
        AgentFamilyV2::Gemini => "GEMINI",
        AgentFamilyV2::Cursor => "CURSOR",
        AgentFamilyV2::Devin => "DEVIN",
        AgentFamilyV2::Antigravity => "ANTIGRAVITY",
        AgentFamilyV2::Cline => "CLINE",
        AgentFamilyV2::Omp => "OMP",
        AgentFamilyV2::Mastracode => "MASTRACODE",
        AgentFamilyV2::OpenCode => "OPEN_CODE",
        AgentFamilyV2::GithubCopilot => "GITHUB_COPILOT",
        AgentFamilyV2::Kimi => "KIMI",
        AgentFamilyV2::Kiro => "KIRO",
        AgentFamilyV2::Droid => "DROID",
        AgentFamilyV2::Amp => "AMP",
        AgentFamilyV2::Grok => "GROK",
        AgentFamilyV2::Hermes => "HERMES",
        AgentFamilyV2::Kilo => "KILO",
        AgentFamilyV2::Qodercli => "QODERCLI",
        AgentFamilyV2::Qwen => "QWEN",
        AgentFamilyV2::Letta => "LETTA",
        AgentFamilyV2::Maki => "MAKI",
        AgentFamilyV2::Muse => "MUSE",
    }
}

fn t174_agent_source_label(
    value: crate::persistent_runtime::protocol::AgentObservationSourceV2,
) -> &'static str {
    use crate::persistent_runtime::protocol::AgentObservationSourceV2;
    match value {
        AgentObservationSourceV2::WindsLaunchMetadata => "WINDS_LAUNCH_METADATA",
        AgentObservationSourceV2::OwnedProcessMetadata => "OWNED_PROCESS_METADATA",
        AgentObservationSourceV2::ProviderStructuredMetadata => "PROVIDER_STRUCTURED_METADATA",
        AgentObservationSourceV2::UserDeclaredPresentation => "USER_DECLARED_PRESENTATION",
    }
}

fn t174_agent_confidence_label(
    value: crate::persistent_runtime::protocol::AgentObservationConfidenceV2,
) -> &'static str {
    use crate::persistent_runtime::protocol::AgentObservationConfidenceV2;
    match value {
        AgentObservationConfidenceV2::Exact => "EXACT",
        AgentObservationConfidenceV2::Strong => "STRONG",
        AgentObservationConfidenceV2::UserDeclared => "USER_DECLARED",
        AgentObservationConfidenceV2::Unknown => "UNKNOWN",
    }
}

fn t174_agent_freshness_label(
    value: crate::persistent_runtime::protocol::AgentObservationFreshnessV2,
) -> &'static str {
    use crate::persistent_runtime::protocol::AgentObservationFreshnessV2;
    match value {
        AgentObservationFreshnessV2::Current => "CURRENT",
        AgentObservationFreshnessV2::Ambiguous => "AMBIGUOUS",
        AgentObservationFreshnessV2::Stale => "STALE",
        AgentObservationFreshnessV2::Unavailable => "UNAVAILABLE",
        AgentObservationFreshnessV2::Unknown => "UNKNOWN",
    }
}

'''
marker = '#[cfg(test)]\nmod t168_workbench_topology_binding_tests {'
replace_once(
    "src/workbench_ui.rs",
    marker,
    tui_agent
    + '#[cfg(test)]\n#[path = "t174_agent_dock_tests.rs"]\nmod t174_agent_dock_tests;\n\n'
    + marker,
)
