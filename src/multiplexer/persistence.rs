#![allow(
    dead_code,
    reason = "Spec 012 T164 persistence substrate; live owner callers land in T165"
)]

use super::navigation::{
    LayoutNode, LayoutTemplateNode, LayoutTemplateStructure, LayoutTemplateTab, SplitAxis,
    SplitRatioBps, TabState, WorkspaceState,
};
use super::{
    LayoutTemplateId, MultiplexerErrorKind, MultiplexerWorkspaceId, PaneId, TabId,
    TopologyGeneration,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const MULTIPLEXER_SCHEMA_VERSION: u32 = 1;
pub(crate) const MULTIPLEXER_MAX_JSON_BYTES: usize = 262_140;
pub(crate) const MULTIPLEXER_MAX_WORKSPACES: usize = 32;
pub(crate) const MULTIPLEXER_MAX_TABS_PER_WORKSPACE: usize = 32;
pub(crate) const MULTIPLEXER_MAX_PANES_PER_TAB: usize = 64;
pub(crate) const MULTIPLEXER_MAX_LAYOUT_TEMPLATES: usize = 128;
pub(crate) const MULTIPLEXER_MAX_WORKTREE_MEMBERSHIPS: usize = 256;
pub(crate) const MULTIPLEXER_MAX_ALIAS_BYTES: usize = 128;
pub(crate) const MULTIPLEXER_MAX_TEMPLATE_NAME_BYTES: usize = 128;
pub(crate) const MULTIPLEXER_MAX_GIT_WORKSPACE_ID_BYTES: usize = 256;
pub(crate) const MULTIPLEXER_MAX_REPOSITORY_IDENTITY_BYTES: usize = 512;
pub(crate) const MULTIPLEXER_MAX_GIT_COMMON_DIR_BYTES: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum PersistedSplitAxis {
    Horizontal,
    Vertical,
}

impl From<SplitAxis> for PersistedSplitAxis {
    fn from(value: SplitAxis) -> Self {
        match value {
            SplitAxis::Horizontal => Self::Horizontal,
            SplitAxis::Vertical => Self::Vertical,
        }
    }
}

impl From<PersistedSplitAxis> for SplitAxis {
    fn from(value: PersistedSplitAxis) -> Self {
        match value {
            PersistedSplitAxis::Horizontal => Self::Horizontal,
            PersistedSplitAxis::Vertical => Self::Vertical,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
enum SnapshotLayoutNodeV1 {
    Pane {
        pane_id: PaneId,
    },
    Split {
        axis: PersistedSplitAxis,
        ratio_bps: u16,
        first: Box<SnapshotLayoutNodeV1>,
        second: Box<SnapshotLayoutNodeV1>,
    },
}

impl SnapshotLayoutNodeV1 {
    fn from_layout(node: &LayoutNode) -> Self {
        match node {
            LayoutNode::Pane(pane_id) => Self::Pane { pane_id: *pane_id },
            LayoutNode::Split {
                axis,
                ratio_bps,
                first,
                second,
            } => Self::Split {
                axis: (*axis).into(),
                ratio_bps: ratio_bps.get(),
                first: Box::new(Self::from_layout(first)),
                second: Box::new(Self::from_layout(second)),
            },
        }
    }

    fn to_layout(&self) -> Result<LayoutNode, String> {
        match self {
            Self::Pane { pane_id } => Ok(LayoutNode::Pane(*pane_id)),
            Self::Split {
                axis,
                ratio_bps,
                first,
                second,
            } => Ok(LayoutNode::Split {
                axis: (*axis).into(),
                ratio_bps: SplitRatioBps::new(*ratio_bps)
                    .map_err(|_| "stored split ratio is outside the accepted range".to_owned())?,
                first: Box::new(first.to_layout()?),
                second: Box::new(second.to_layout()?),
            }),
        }
    }

    fn pane_ids(&self, output: &mut BTreeSet<PaneId>) -> Result<(), String> {
        match self {
            Self::Pane { pane_id } => {
                if !output.insert(*pane_id) {
                    return Err("stored topology reuses a PaneId".to_owned());
                }
            }
            Self::Split {
                ratio_bps,
                first,
                second,
                ..
            } => {
                SplitRatioBps::new(*ratio_bps)
                    .map_err(|_| "stored split ratio is outside the accepted range".to_owned())?;
                first.pane_ids(output)?;
                second.pane_ids(output)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TopologySnapshotTabV1 {
    tab_id: TabId,
    alias: String,
    root: SnapshotLayoutNodeV1,
    focused_pane_id: PaneId,
    zoomed_pane_id: Option<PaneId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TopologySnapshotV1 {
    schema_version: u32,
    multiplexer_workspace_id: MultiplexerWorkspaceId,
    alias: String,
    topology_generation: TopologyGeneration,
    is_focused: bool,
    tabs: Vec<TopologySnapshotTabV1>,
    focused_tab_id: TabId,
}

impl TopologySnapshotV1 {
    pub(crate) fn from_workspace(
        workspace: &WorkspaceState,
        topology_generation: TopologyGeneration,
        is_focused: bool,
    ) -> Result<Self, String> {
        let snapshot = Self {
            schema_version: MULTIPLEXER_SCHEMA_VERSION,
            multiplexer_workspace_id: workspace.id,
            alias: workspace.alias.clone(),
            topology_generation,
            is_focused,
            tabs: workspace
                .tabs
                .iter()
                .map(|tab| TopologySnapshotTabV1 {
                    tab_id: tab.id,
                    alias: tab.alias.clone(),
                    root: SnapshotLayoutNodeV1::from_layout(&tab.root),
                    focused_pane_id: tab.focused_pane_id,
                    zoomed_pane_id: tab.zoomed_pane_id,
                })
                .collect(),
            focused_tab_id: workspace.focused_tab_id,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    pub(crate) const fn workspace_id(&self) -> MultiplexerWorkspaceId {
        self.multiplexer_workspace_id
    }

    pub(crate) fn alias(&self) -> &str {
        &self.alias
    }

    pub(crate) const fn topology_generation(&self) -> TopologyGeneration {
        self.topology_generation
    }

    pub(crate) const fn is_focused(&self) -> bool {
        self.is_focused
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.schema_version != MULTIPLEXER_SCHEMA_VERSION {
            return Err(format!(
                "unsupported multiplexer topology snapshot schema version: {}",
                self.schema_version
            ));
        }
        validate_text_bytes(&self.alias, MULTIPLEXER_MAX_ALIAS_BYTES, "workspace alias")?;
        if self.tabs.is_empty() || self.tabs.len() > MULTIPLEXER_MAX_TABS_PER_WORKSPACE {
            return Err("stored topology has an invalid tab count".to_owned());
        }

        let mut tab_ids = BTreeSet::new();
        let mut all_panes = BTreeSet::new();
        let mut focused_tab_seen = false;
        for tab in &self.tabs {
            validate_text_bytes(&tab.alias, MULTIPLEXER_MAX_ALIAS_BYTES, "tab alias")?;
            if !tab_ids.insert(tab.tab_id) {
                return Err("stored topology reuses a TabId".to_owned());
            }
            if tab.tab_id == self.focused_tab_id {
                focused_tab_seen = true;
            }

            let mut pane_ids = BTreeSet::new();
            tab.root.pane_ids(&mut pane_ids)?;
            if pane_ids.is_empty() || pane_ids.len() > MULTIPLEXER_MAX_PANES_PER_TAB {
                return Err("stored topology has an invalid pane count".to_owned());
            }
            if !pane_ids.contains(&tab.focused_pane_id) {
                return Err("stored focused PaneId is not in its tab".to_owned());
            }
            if tab
                .zoomed_pane_id
                .is_some_and(|pane_id| !pane_ids.contains(&pane_id))
            {
                return Err("stored zoomed PaneId is not in its tab".to_owned());
            }
            for pane_id in pane_ids {
                if !all_panes.insert(pane_id) {
                    return Err("stored topology reuses a PaneId across tabs".to_owned());
                }
            }
        }

        if !focused_tab_seen {
            return Err("stored focused TabId is not in its workspace".to_owned());
        }
        Ok(())
    }

    pub(crate) fn to_workspace_state(&self) -> Result<WorkspaceState, String> {
        self.validate()?;
        Ok(WorkspaceState {
            id: self.multiplexer_workspace_id,
            alias: self.alias.clone(),
            tabs: self
                .tabs
                .iter()
                .map(|tab| {
                    Ok(TabState {
                        id: tab.tab_id,
                        alias: tab.alias.clone(),
                        root: tab.root.to_layout()?,
                        focused_pane_id: tab.focused_pane_id,
                        zoomed_pane_id: tab.zoomed_pane_id,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
            focused_tab_id: self.focused_tab_id,
        })
    }

    pub(crate) fn to_canonical_json(&self) -> Result<String, String> {
        self.validate()?;
        let json = serde_json::to_string(self)
            .map_err(|error| format!("could not encode topology snapshot: {error}"))?;
        validate_json_bytes(&json, "topology snapshot")?;
        Ok(json)
    }

    pub(crate) fn from_canonical_json(json: &str) -> Result<Self, String> {
        validate_json_bytes(json, "topology snapshot")?;
        let snapshot: Self = serde_json::from_str(json)
            .map_err(|error| format!("could not decode topology snapshot: {error}"))?;
        snapshot.validate()?;
        if snapshot.to_canonical_json()? != json {
            return Err("topology snapshot JSON is not in canonical Winds form".to_owned());
        }
        Ok(snapshot)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
enum LayoutTemplateNodeV1 {
    Pane,
    Split {
        axis: PersistedSplitAxis,
        ratio_bps: u16,
        first: Box<LayoutTemplateNodeV1>,
        second: Box<LayoutTemplateNodeV1>,
    },
}

impl LayoutTemplateNodeV1 {
    fn from_template(node: &LayoutTemplateNode) -> Self {
        match node {
            LayoutTemplateNode::Pane => Self::Pane,
            LayoutTemplateNode::Split {
                axis,
                ratio_bps,
                first,
                second,
            } => Self::Split {
                axis: (*axis).into(),
                ratio_bps: ratio_bps.get(),
                first: Box::new(Self::from_template(first)),
                second: Box::new(Self::from_template(second)),
            },
        }
    }

    fn to_template(&self) -> Result<LayoutTemplateNode, String> {
        match self {
            Self::Pane => Ok(LayoutTemplateNode::Pane),
            Self::Split {
                axis,
                ratio_bps,
                first,
                second,
            } => Ok(LayoutTemplateNode::Split {
                axis: (*axis).into(),
                ratio_bps: SplitRatioBps::new(*ratio_bps)
                    .map_err(|_| "stored template split ratio is invalid".to_owned())?,
                first: Box::new(first.to_template()?),
                second: Box::new(second.to_template()?),
            }),
        }
    }

    fn pane_count(&self) -> Result<usize, String> {
        match self {
            Self::Pane => Ok(1),
            Self::Split {
                ratio_bps,
                first,
                second,
                ..
            } => {
                SplitRatioBps::new(*ratio_bps)
                    .map_err(|_| "stored template split ratio is invalid".to_owned())?;
                first
                    .pane_count()?
                    .checked_add(second.pane_count()?)
                    .ok_or_else(|| "stored template pane count overflow".to_owned())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LayoutTemplateTabV1 {
    alias: String,
    root: LayoutTemplateNodeV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LayoutTemplateV1 {
    schema_version: u32,
    tabs: Vec<LayoutTemplateTabV1>,
}

impl LayoutTemplateV1 {
    pub(crate) fn from_structure(value: &LayoutTemplateStructure) -> Result<Self, String> {
        let template = Self {
            schema_version: MULTIPLEXER_SCHEMA_VERSION,
            tabs: value
                .tabs
                .iter()
                .map(|tab| LayoutTemplateTabV1 {
                    alias: tab.alias.clone(),
                    root: LayoutTemplateNodeV1::from_template(&tab.root),
                })
                .collect(),
        };
        template.validate()?;
        Ok(template)
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.schema_version != MULTIPLEXER_SCHEMA_VERSION {
            return Err(format!(
                "unsupported multiplexer layout template schema version: {}",
                self.schema_version
            ));
        }
        if self.tabs.is_empty() || self.tabs.len() > MULTIPLEXER_MAX_TABS_PER_WORKSPACE {
            return Err("stored layout template has an invalid tab count".to_owned());
        }
        for tab in &self.tabs {
            validate_text_bytes(&tab.alias, MULTIPLEXER_MAX_ALIAS_BYTES, "template tab alias")?;
            let pane_count = tab.root.pane_count()?;
            if pane_count == 0 || pane_count > MULTIPLEXER_MAX_PANES_PER_TAB {
                return Err("stored layout template has an invalid pane count".to_owned());
            }
        }
        Ok(())
    }

    pub(crate) fn to_structure(&self) -> Result<LayoutTemplateStructure, String> {
        self.validate()?;
        Ok(LayoutTemplateStructure {
            tabs: self
                .tabs
                .iter()
                .map(|tab| {
                    Ok(LayoutTemplateTab {
                        alias: tab.alias.clone(),
                        root: tab.root.to_template()?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
        })
    }

    pub(crate) fn to_canonical_json(&self) -> Result<String, String> {
        self.validate()?;
        let json = serde_json::to_string(self)
            .map_err(|error| format!("could not encode layout template: {error}"))?;
        validate_json_bytes(&json, "layout template")?;
        Ok(json)
    }

    pub(crate) fn from_canonical_json(json: &str) -> Result<Self, String> {
        validate_json_bytes(json, "layout template")?;
        let template: Self = serde_json::from_str(json)
            .map_err(|error| format!("could not decode layout template: {error}"))?;
        template.validate()?;
        if template.to_canonical_json()? != json {
            return Err("layout template JSON is not in canonical Winds form".to_owned());
        }
        Ok(template)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum WorktreeMembershipSource {
    ExplicitUser,
    CreatedByWinds,
    ImportedExplicit,
}

impl WorktreeMembershipSource {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitUser => "EXPLICIT_USER",
            Self::CreatedByWinds => "CREATED_BY_WINDS",
            Self::ImportedExplicit => "IMPORTED_EXPLICIT",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "EXPLICIT_USER" => Ok(Self::ExplicitUser),
            "CREATED_BY_WINDS" => Ok(Self::CreatedByWinds),
            "IMPORTED_EXPLICIT" => Ok(Self::ImportedExplicit),
            _ => Err(format!("unknown worktree membership source: {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum WorktreeMembershipState {
    Present,
    Stale,
    Removed,
}

impl WorktreeMembershipState {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Present => "PRESENT",
            Self::Stale => "STALE",
            Self::Removed => "REMOVED",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "PRESENT" => Ok(Self::Present),
            "STALE" => Ok(Self::Stale),
            "REMOVED" => Ok(Self::Removed),
            _ => Err(format!("unknown worktree membership state: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorktreeMembershipRecord {
    pub(crate) multiplexer_workspace_id: MultiplexerWorkspaceId,
    pub(crate) git_workspace_id: String,
    pub(crate) source: WorktreeMembershipSource,
    pub(crate) state: WorktreeMembershipState,
    pub(crate) created_unix_ms: i64,
    pub(crate) last_confirmed_unix_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RepositoryTrustRecord {
    pub(crate) repository_identity: String,
    pub(crate) canonical_git_common_dir: String,
    pub(crate) revision: u64,
    pub(crate) created_unix_ms: i64,
    pub(crate) updated_unix_ms: i64,
}

pub(crate) fn validate_template_name(value: &str) -> Result<(), String> {
    validate_text_bytes(value, MULTIPLEXER_MAX_TEMPLATE_NAME_BYTES, "layout template name")
}

pub(crate) fn validate_git_workspace_id(value: &str) -> Result<(), String> {
    validate_nonempty_text_bytes(
        value,
        MULTIPLEXER_MAX_GIT_WORKSPACE_ID_BYTES,
        "Git workspace id",
    )
}

pub(crate) fn validate_repository_identity(value: &str) -> Result<(), String> {
    validate_nonempty_text_bytes(
        value,
        MULTIPLEXER_MAX_REPOSITORY_IDENTITY_BYTES,
        "repository identity",
    )
}

pub(crate) fn validate_git_common_dir(value: &str) -> Result<(), String> {
    validate_nonempty_text_bytes(
        value,
        MULTIPLEXER_MAX_GIT_COMMON_DIR_BYTES,
        "canonical Git common directory",
    )
}

pub(crate) fn validate_timestamp(value: i64, label: &str) -> Result<(), String> {
    if value < 0 {
        return Err(format!("{label} must not be negative"));
    }
    Ok(())
}

fn validate_json_bytes(value: &str, label: &str) -> Result<(), String> {
    let bytes = value.len();
    if bytes == 0 || bytes > MULTIPLEXER_MAX_JSON_BYTES {
        return Err(format!(
            "{label} must encode to between 1 and {MULTIPLEXER_MAX_JSON_BYTES} UTF-8 bytes"
        ));
    }
    Ok(())
}

fn validate_text_bytes(value: &str, maximum: usize, label: &str) -> Result<(), String> {
    if value.as_bytes().len() > maximum {
        return Err(format!("{label} exceeds {maximum} UTF-8 bytes"));
    }
    if value.contains('\0') {
        return Err(format!("{label} must not contain NUL"));
    }
    Ok(())
}

fn validate_nonempty_text_bytes(
    value: &str,
    maximum: usize,
    label: &str,
) -> Result<(), String> {
    validate_text_bytes(value, maximum, label)?;
    if value.is_empty() {
        return Err(format!("{label} must not be empty"));
    }
    Ok(())
}

pub(crate) fn map_persistence_error(error: String) -> MultiplexerErrorKind {
    if error.contains("262140") || error.contains("exceeds") {
        MultiplexerErrorKind::SnapshotLimitExceeded
    } else {
        MultiplexerErrorKind::UnsupportedOperation
    }
}
