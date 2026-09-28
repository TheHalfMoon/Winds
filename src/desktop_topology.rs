use crate::multiplexer::domain::{MultiplexerWorkspaceId, PaneId, TabId, TopologyGeneration};
use crate::persistent_runtime::client::RustLocalControlClient;
use crate::persistent_runtime::domain::OwnerGenerationId;
use crate::persistent_runtime::protocol::{
    MultiplexerSnapshotV2, ProtocolLayoutNodeV2, ProtocolSplitAxis, ProtocolWorkspaceSnapshotV2,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub type DesktopTopologyResult<T> = Result<T, String>;

const DESKTOP_TOPOLOGY_AUTHORITY: &str = "OWNER_AUTHORITATIVE_TOPOLOGY";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopTopologyCapability {
    pub authority: &'static str,
    pub trusted_rust_host: bool,
    pub renderer_direct_owner_access: bool,
    pub controlling_tty_required: bool,
    pub terminal_surface_capable: bool,
    pub generic_invoke_surface: bool,
}

pub fn desktop_topology_capability() -> DesktopTopologyCapability {
    DesktopTopologyCapability {
        authority: DESKTOP_TOPOLOGY_AUTHORITY,
        trusted_rust_host: true,
        renderer_direct_owner_access: false,
        controlling_tty_required: false,
        terminal_surface_capable: true,
        generic_invoke_surface: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopTopologySnapshotRequest {
    pub expected_owner_generation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopTopologyBindRequest {
    pub expected_owner_generation_id: Option<String>,
    pub expected_topology_generation: u64,
    pub multiplexer_workspace_id: String,
    pub tab_id: Option<String>,
    pub pane_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DesktopTopologyLayoutNode {
    Pane {
        pane_id: String,
    },
    Split {
        axis: String,
        ratio_basis_points: u16,
        first: Box<DesktopTopologyLayoutNode>,
        second: Box<DesktopTopologyLayoutNode>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopTopologyTab {
    pub tab_id: String,
    pub display_label: String,
    pub focused: bool,
    pub focused_pane_id: String,
    pub zoomed_pane_id: Option<String>,
    pub root: DesktopTopologyLayoutNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopTopologyWorkspace {
    pub multiplexer_workspace_id: String,
    pub display_label: String,
    pub topology_generation: u64,
    pub focused_tab_id: String,
    pub tabs: Vec<DesktopTopologyTab>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopTopologySnapshot {
    pub authority: &'static str,
    pub topology_generation: u64,
    pub workspaces: Vec<DesktopTopologyWorkspace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopTopologyBoundTarget {
    pub authority: &'static str,
    pub topology_generation: u64,
    pub multiplexer_workspace_id: String,
    pub tab_id: Option<String>,
    pub pane_id: Option<String>,
}

fn parse_owner_generation(value: Option<&str>) -> DesktopTopologyResult<Option<OwnerGenerationId>> {
    value
        .map(|value| {
            OwnerGenerationId::from_str(value)
                .map_err(|error| format!("invalid expected owner generation: {error}"))
        })
        .transpose()
}

fn connect(
    expected_owner_generation_id: Option<&str>,
) -> DesktopTopologyResult<RustLocalControlClient> {
    let expected_owner_generation_id = parse_owner_generation(expected_owner_generation_id)?;
    RustLocalControlClient::connect(None, expected_owner_generation_id)
        .map_err(|error| format!("desktop topology owner connection failed: {error}"))
}

fn layout_node(node: &ProtocolLayoutNodeV2) -> DesktopTopologyLayoutNode {
    match node {
        ProtocolLayoutNodeV2::Pane { pane_id } => DesktopTopologyLayoutNode::Pane {
            pane_id: pane_id.to_string(),
        },
        ProtocolLayoutNodeV2::Split {
            axis,
            ratio_basis_points,
            first,
            second,
        } => DesktopTopologyLayoutNode::Split {
            axis: match axis {
                ProtocolSplitAxis::Horizontal => "HORIZONTAL",
                ProtocolSplitAxis::Vertical => "VERTICAL",
            }
            .to_owned(),
            ratio_basis_points: *ratio_basis_points,
            first: Box::new(layout_node(first)),
            second: Box::new(layout_node(second)),
        },
    }
}

fn workspace_projection(snapshot: &ProtocolWorkspaceSnapshotV2) -> DesktopTopologyWorkspace {
    DesktopTopologyWorkspace {
        multiplexer_workspace_id: snapshot.multiplexer_workspace_id.to_string(),
        display_label: snapshot.alias.clone(),
        topology_generation: snapshot.topology_generation.get(),
        focused_tab_id: snapshot.focused_tab_id.to_string(),
        tabs: snapshot
            .tabs
            .iter()
            .map(|tab| DesktopTopologyTab {
                tab_id: tab.tab_id.to_string(),
                display_label: tab.alias.clone(),
                focused: tab.tab_id == snapshot.focused_tab_id,
                focused_pane_id: tab.focused_pane_id.to_string(),
                zoomed_pane_id: tab.zoomed_pane_id.map(|pane_id| pane_id.to_string()),
                root: layout_node(&tab.root),
            })
            .collect(),
    }
}

fn snapshot_from_client(
    client: &mut RustLocalControlClient,
) -> DesktopTopologyResult<DesktopTopologySnapshot> {
    let list = client
        .refresh_topology_projection(None)
        .map_err(|error| format!("desktop topology refresh failed: {error}"))?;
    let (topology_generation, workspace_ids) = match list {
        MultiplexerSnapshotV2::WorkspaceList {
            topology_generation,
            workspaces,
        } => (
            topology_generation,
            workspaces
                .into_iter()
                .map(|workspace| workspace.multiplexer_workspace_id)
                .collect::<Vec<_>>(),
        ),
        _ => return Err("desktop topology list returned an unsupported snapshot shape".to_owned()),
    };

    let mut workspaces = Vec::with_capacity(workspace_ids.len());
    for workspace_id in workspace_ids {
        let snapshot = client
            .refresh_topology_projection(Some(workspace_id))
            .map_err(|error| format!("desktop topology workspace refresh failed: {error}"))?;
        let MultiplexerSnapshotV2::Workspace { snapshot } = snapshot else {
            return Err(
                "desktop topology workspace returned an unsupported snapshot shape".to_owned(),
            );
        };
        if snapshot.topology_generation != topology_generation {
            return Err(
                "desktop topology generation changed during projection; refresh required"
                    .to_owned(),
            );
        }
        workspaces.push(workspace_projection(&snapshot));
    }
    workspaces.sort_by(|left, right| {
        left.multiplexer_workspace_id
            .cmp(&right.multiplexer_workspace_id)
    });

    Ok(DesktopTopologySnapshot {
        authority: DESKTOP_TOPOLOGY_AUTHORITY,
        topology_generation: topology_generation.get(),
        workspaces,
    })
}

pub fn desktop_topology_snapshot(
    request: DesktopTopologySnapshotRequest,
) -> DesktopTopologyResult<DesktopTopologySnapshot> {
    let mut client = connect(request.expected_owner_generation_id.as_deref())?;
    snapshot_from_client(&mut client)
}

fn exact_workspace<'a>(
    snapshot: &'a DesktopTopologySnapshot,
    workspace_id: &str,
) -> DesktopTopologyResult<&'a DesktopTopologyWorkspace> {
    snapshot
        .workspaces
        .iter()
        .find(|workspace| workspace.multiplexer_workspace_id == workspace_id)
        .ok_or_else(|| {
            "desktop topology target workspace is absent from the authoritative snapshot".to_owned()
        })
}

pub fn desktop_topology_bind_target(
    request: DesktopTopologyBindRequest,
) -> DesktopTopologyResult<DesktopTopologyBoundTarget> {
    let workspace_id = MultiplexerWorkspaceId::parse(&request.multiplexer_workspace_id)
        .map_err(|error| format!("invalid multiplexer workspace id: {error}"))?;
    let tab_id = request
        .tab_id
        .as_deref()
        .map(TabId::parse)
        .transpose()
        .map_err(|error| format!("invalid tab id: {error}"))?;
    let pane_id = request
        .pane_id
        .as_deref()
        .map(PaneId::parse)
        .transpose()
        .map_err(|error| format!("invalid pane id: {error}"))?;
    if pane_id.is_some() && tab_id.is_none() {
        return Err("desktop topology pane target requires an exact tab id".to_owned());
    }
    let expected_generation = TopologyGeneration::new(request.expected_topology_generation)
        .map_err(|error| format!("invalid topology generation: {error}"))?;

    let snapshot = desktop_topology_snapshot(DesktopTopologySnapshotRequest {
        expected_owner_generation_id: request.expected_owner_generation_id,
    })?;
    if snapshot.topology_generation != expected_generation.get() {
        return Err(
            "desktop topology target is stale; authoritative generation changed".to_owned(),
        );
    }
    let workspace = exact_workspace(&snapshot, &workspace_id.to_string())?;
    if let Some(tab_id) = tab_id {
        let tab = workspace
            .tabs
            .iter()
            .find(|tab| tab.tab_id == tab_id.to_string())
            .ok_or_else(|| {
                "desktop topology target tab is absent from the authoritative workspace".to_owned()
            })?;
        if let Some(pane_id) = pane_id {
            fn contains_pane(node: &DesktopTopologyLayoutNode, pane_id: &str) -> bool {
                match node {
                    DesktopTopologyLayoutNode::Pane { pane_id: candidate } => candidate == pane_id,
                    DesktopTopologyLayoutNode::Split { first, second, .. } => {
                        contains_pane(first, pane_id) || contains_pane(second, pane_id)
                    }
                }
            }
            if !contains_pane(&tab.root, &pane_id.to_string()) {
                return Err(
                    "desktop topology target pane is absent from the authoritative tab".to_owned(),
                );
            }
        }
    }

    Ok(DesktopTopologyBoundTarget {
        authority: DESKTOP_TOPOLOGY_AUTHORITY,
        topology_generation: expected_generation.get(),
        multiplexer_workspace_id: workspace_id.to_string(),
        tab_id: tab_id.map(|value| value.to_string()),
        pane_id: pane_id.map(|value| value.to_string()),
    })
}
