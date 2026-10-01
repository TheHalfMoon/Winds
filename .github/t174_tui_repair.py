from pathlib import Path

path = Path("src/workbench.rs")
text = path.read_text(encoding="utf-8")


def replace_once(old: str, new: str) -> None:
    global text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected one replacement target, found {count}: {old[:120]!r}")
    text = text.replace(old, new, 1)


anchor = '''const EMPTY_WORKBENCH_MESSAGE: &str = "No terminal panes are active.";
const COMPACT_WORKBENCH_MIN_WIDTH: u16 = 60;
const COMPACT_WORKBENCH_MIN_HEIGHT: u16 = 10;
'''
block = anchor + r'''
const T174_TUI_AGENT_DOCK_MAX_ROWS: usize = 8;
const T174_TUI_AGENT_DOCK_NONCLAIM: &str = "DETECTION_ONLY_UNPROVEN";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct T174TuiAgentHitRegion {
    observation_id: crate::multiplexer::domain::AgentObservationId,
    column: u16,
    row: u16,
    width: u16,
    height: u16,
}

impl T174TuiAgentHitRegion {
    fn contains(self, column: u16, row: u16) -> bool {
        column >= self.column
            && row >= self.row
            && column < self.column.saturating_add(self.width)
            && row < self.row.saturating_add(self.height)
    }
}

#[derive(Debug, Default)]
struct T174TuiAgentDockState {
    open: bool,
    selected_observation_id: Option<crate::multiplexer::domain::AgentObservationId>,
    alias_input: Option<String>,
    hit_regions: Vec<T174TuiAgentHitRegion>,
    status: String,
}

impl T174TuiAgentDockState {
    fn open_with(&mut self, dock: &mut ui::CanonicalAgentDockPresentation) {
        self.open = true;
        self.alias_input = None;
        self.hit_regions.clear();
        self.selected_observation_id = dock
            .list("", ui::CanonicalAgentDockSort::Family, true)
            .first()
            .map(|row| row.observation_id);
        if let Some(observation_id) = self.selected_observation_id {
            let _ = dock.view(observation_id);
        }
        self.status = "CURRENT owner-authoritative observations loaded".to_owned();
    }

    fn open_unavailable(&mut self, reason: impl Into<String>) {
        self.open = true;
        self.selected_observation_id = None;
        self.alias_input = None;
        self.hit_regions.clear();
        self.status = format!("UNAVAILABLE · {}", reason.into());
    }

    fn close(&mut self) {
        self.open = false;
        self.alias_input = None;
        self.hit_regions.clear();
    }

    fn select_exact(
        &mut self,
        dock: &mut ui::CanonicalAgentDockPresentation,
        observation_id: crate::multiplexer::domain::AgentObservationId,
    ) {
        if dock.view(observation_id).is_some() {
            self.selected_observation_id = Some(observation_id);
            self.status = format!("VIEW observation={observation_id}");
        }
    }

    fn cycle_selection(&mut self, dock: &mut ui::CanonicalAgentDockPresentation, delta: isize) {
        let rows = dock.list("", ui::CanonicalAgentDockSort::Family, true);
        if rows.is_empty() {
            self.selected_observation_id = None;
            return;
        }
        let current = self
            .selected_observation_id
            .and_then(|selected| rows.iter().position(|row| row.observation_id == selected))
            .unwrap_or(0);
        let next = if delta < 0 {
            current.checked_sub(1).unwrap_or(rows.len() - 1)
        } else {
            (current + 1) % rows.len()
        };
        self.select_exact(dock, rows[next].observation_id);
    }

    fn focus_selected(
        &mut self,
        dock: &mut ui::CanonicalAgentDockPresentation,
        topology: Option<&ui::CanonicalTopologyPresentation>,
    ) {
        let Some(observation_id) = self.selected_observation_id else {
            self.status = "FOCUS_UNAVAILABLE · no exact observation selected".to_owned();
            return;
        };
        let Some(topology) = topology else {
            self.status = "FOCUS_UNAVAILABLE · canonical topology unavailable".to_owned();
            return;
        };
        match dock.focus(observation_id, topology) {
            Some(target) => {
                self.status = format!(
                    "FOCUS_VALIDATED observation={} workspace={} tab={} pane={} topology_generation={} · READ_ONLY",
                    target.observation_id,
                    target.multiplexer_workspace_id,
                    target.tab_id,
                    target.pane_id,
                    target.topology_generation,
                );
            }
            None => {
                self.status =
                    "FOCUS_UNAVAILABLE · stale, absent, ambiguous, or substituted exact target"
                        .to_owned();
            }
        }
    }

    fn handle_event(
        &mut self,
        mut dock: Option<&mut ui::CanonicalAgentDockPresentation>,
        topology: Option<&ui::CanonicalTopologyPresentation>,
        event: &Event,
    ) -> bool {
        if !self.open {
            return false;
        }
        if matches!(event, Event::Key(key) if key.kind == KeyEventKind::Release) {
            return true;
        }
        if matches!(event, Event::Key(key) if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q')) {
            return false;
        }

        if self.alias_input.is_some() {
            let Event::Key(key) = event else {
                return true;
            };
            match key.code {
                KeyCode::Esc => {
                    self.alias_input = None;
                    self.status = "ALIAS_CANCELLED · immutable identity unchanged".to_owned();
                }
                KeyCode::Backspace => {
                    if let Some(alias) = &mut self.alias_input {
                        alias.pop();
                    }
                }
                KeyCode::Enter => {
                    let alias = self.alias_input.take().unwrap_or_default();
                    let Some(observation_id) = self.selected_observation_id else {
                        self.status = "ALIAS_UNAVAILABLE · no exact observation selected".to_owned();
                        return true;
                    };
                    let Some(dock) = dock.as_deref_mut() else {
                        self.status = "ALIAS_UNAVAILABLE · observation truth unavailable".to_owned();
                        return true;
                    };
                    match dock.rename(observation_id, Some(&alias)) {
                        Ok(()) => {
                            self.status = format!(
                                "ALIAS_UPDATED observation={observation_id} · DISPLAY_ONLY · immutable identity unchanged"
                            );
                        }
                        Err(error) => self.status = format!("ALIAS_REFUSED · {error}"),
                    }
                }
                KeyCode::Char(character)
                    if !key.modifiers.contains(KeyModifiers::CONTROL)
                        && !key.modifiers.contains(KeyModifiers::ALT) =>
                {
                    if let Some(alias) = &mut self.alias_input {
                        alias.push(character);
                    }
                }
                _ => {}
            }
            return true;
        }

        match event {
            Event::Key(key) if key.code == KeyCode::Esc => {
                self.close();
                true
            }
            Event::Key(key) if key.code == KeyCode::Up => {
                if let Some(dock) = dock.as_deref_mut() {
                    self.cycle_selection(dock, -1);
                }
                true
            }
            Event::Key(key) if key.code == KeyCode::Down => {
                if let Some(dock) = dock.as_deref_mut() {
                    self.cycle_selection(dock, 1);
                }
                true
            }
            Event::Key(key) if key.code == KeyCode::Enter => {
                if let Some(dock) = dock.as_deref_mut() {
                    self.focus_selected(dock, topology);
                } else {
                    self.status = "FOCUS_UNAVAILABLE · observation truth unavailable".to_owned();
                }
                true
            }
            Event::Key(key) if key.code == KeyCode::Char('e') => {
                if let (Some(observation_id), Some(dock)) =
                    (self.selected_observation_id, dock.as_deref_mut())
                {
                    self.status = dock
                        .explain(observation_id)
                        .map(|explanation| format!("EXPLAIN\n{explanation}"))
                        .unwrap_or_else(|| "EXPLAIN_UNAVAILABLE".to_owned());
                }
                true
            }
            Event::Key(key) if key.code == KeyCode::Char('r') => {
                if let (Some(observation_id), Some(dock)) =
                    (self.selected_observation_id, dock.as_deref_mut())
                {
                    self.alias_input = Some(
                        dock.get(observation_id)
                            .and_then(|item| item.display_alias.clone())
                            .unwrap_or_default(),
                    );
                    self.status = format!(
                        "ALIAS_EDIT observation={observation_id} · DISPLAY_ONLY · Enter saves · Esc cancels"
                    );
                }
                true
            }
            Event::Mouse(mouse)
                if matches!(
                    mouse.kind,
                    crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left)
                        | crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Right)
                ) =>
            {
                let target = self
                    .hit_regions
                    .iter()
                    .copied()
                    .find(|region| region.contains(mouse.column, mouse.row))
                    .map(|region| region.observation_id);
                let Some(observation_id) = target else {
                    return false;
                };
                let Some(dock) = dock.as_deref_mut() else {
                    self.status = "VIEW_UNAVAILABLE · observation truth unavailable".to_owned();
                    return true;
                };
                self.select_exact(dock, observation_id);
                if mouse.kind
                    == crossterm::event::MouseEventKind::Down(
                        crossterm::event::MouseButton::Right,
                    )
                {
                    self.focus_selected(dock, topology);
                }
                true
            }
            _ => true,
        }
    }
}

fn t174_agent_dock_toggle_event(event: &Event) -> bool {
    matches!(
        event,
        Event::Key(key)
            if key.kind != KeyEventKind::Release
                && key.modifiers.contains(KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('g')
    )
}

fn load_t174_agent_dock(
    store: &crate::store::Store,
) -> Result<ui::CanonicalAgentDockPresentation, String> {
    let owner_generation_id = store
        .latest_persistent_runtime_owner_generation()
        .map_err(|error| format!("owner generation record unavailable: {error}"))?
        .ok_or_else(|| "owner generation unavailable".to_owned())?;
    let mut client = crate::persistent_runtime::client::RustLocalControlClient::connect(
        None,
        Some(owner_generation_id),
    )
    .map_err(|error| format!("owner connection unavailable: {error}"))?;
    let observations = client
        .refresh_agent_observation_projection(None)
        .map_err(|error| format!("agent observation refresh unavailable: {error}"))?;
    ui::CanonicalAgentDockPresentation::from_observations(observations)
}

fn t174_agent_dock_text(
    dock: Option<&ui::CanonicalAgentDockPresentation>,
    state: &T174TuiAgentDockState,
) -> String {
    let mut lines = vec![
        "T174_AGENT_DOCK=OWNER_AUTHORITATIVE_AGENT_OBSERVATIONS".to_owned(),
        format!("execution={T174_TUI_AGENT_DOCK_NONCLAIM}"),
        format!("status={}", state.status),
        "keys=Up/Down view | Enter focus-validate | e explain | r rename-display-alias | Esc close | right-click focus-validate".to_owned(),
    ];
    if let Some(alias) = &state.alias_input {
        lines.push(format!("alias_input={alias}"));
    }
    match dock {
        Some(dock) => {
            for row in dock
                .list("", ui::CanonicalAgentDockSort::Family, true)
                .into_iter()
                .take(T174_TUI_AGENT_DOCK_MAX_ROWS)
            {
                let selected = if state.selected_observation_id == Some(row.observation_id) {
                    ">"
                } else {
                    " "
                };
                lines.push(format!(
                    "{selected} observation={} family={:?} freshness={:?}",
                    row.observation_id, row.family, row.freshness
                ));
            }
            if let Some(observation_id) = state.selected_observation_id
                && let Some(explanation) = dock.explain(observation_id)
            {
                lines.push("SELECTED_EXACT_BINDING".to_owned());
                lines.extend(explanation.lines().map(str::to_owned));
            }
        }
        None => lines.push(
            "UNAVAILABLE · no observation, provider, process, controller, write, Git, or verification authority inferred"
                .to_owned(),
        ),
    }
    lines.join("\n")
}

fn render_t174_agent_dock(
    frame: &mut Frame<'_>,
    area: ratatui::layout::Rect,
    dock: Option<&ui::CanonicalAgentDockPresentation>,
    state: &mut T174TuiAgentDockState,
) {
    state.hit_regions.clear();
    if !state.open || area.width < 24 || area.height < 7 {
        return;
    }
    let width = area.width.min(96);
    let height = area.height.min(20);
    let overlay = ratatui::layout::Rect::new(
        area.x.saturating_add(area.width.saturating_sub(width)),
        area.y,
        width,
        height,
    );
    frame.render_widget(
        Paragraph::new(t174_agent_dock_text(dock, state))
            .block(Block::bordered().title(" Agents · READ_ONLY · Ctrl+G ")),
        overlay,
    );
    let Some(dock) = dock else {
        return;
    };
    let first_row = overlay.y.saturating_add(5);
    for (index, row) in dock
        .list("", ui::CanonicalAgentDockSort::Family, true)
        .into_iter()
        .take(T174_TUI_AGENT_DOCK_MAX_ROWS)
        .enumerate()
    {
        let row_y = first_row.saturating_add(index as u16);
        if row_y >= overlay.y.saturating_add(overlay.height).saturating_sub(1) {
            break;
        }
        state.hit_regions.push(T174TuiAgentHitRegion {
            observation_id: row.observation_id,
            column: overlay.x.saturating_add(1),
            row: row_y,
            width: overlay.width.saturating_sub(2),
            height: 1,
        });
    }
}
'''
replace_once(anchor, block)

old = '''fn render_workbench_accessible(
    frame: &mut Frame<'_>,
    state: &WorkbenchState,
    editor: &terminal::input::WorkbenchShellEditor,
    output: &output::WorkbenchOutput,
    accessibility: WorkbenchAccessibilityState,
    candidate: (&str, &str),
    projected: Option<&context::WorkbenchCandidateContext>,
    canonical_topology: Option<&ui::CanonicalTopologyPresentation>,
) {'''
new = '''fn render_workbench_accessible(
    frame: &mut Frame<'_>,
    state: &WorkbenchState,
    editor: &terminal::input::WorkbenchShellEditor,
    output: &output::WorkbenchOutput,
    accessibility: WorkbenchAccessibilityState,
    candidate: (&str, &str),
    projected: Option<&context::WorkbenchCandidateContext>,
    canonical_topology: Option<&ui::CanonicalTopologyPresentation>,
    agent_dock: Option<&ui::CanonicalAgentDockPresentation>,
    agent_dock_ui: &mut T174TuiAgentDockState,
) {'''
replace_once(old, new)

old = '''        if let Some(topology) = canonical_topology {
            compact.push_str("\\nCANONICAL_TOPOLOGY_BEGIN\\n");
            compact.push_str(topology.text());
            compact.push_str("\\nCANONICAL_TOPOLOGY_END");
        }
        frame.render_widget(
            Paragraph::new(compact).block(Block::bordered().title(" Winds Workbench · compact ")),
            area,
        );
        return;
'''
new = '''        if let Some(topology) = canonical_topology {
            compact.push_str("\\nCANONICAL_TOPOLOGY_BEGIN\\n");
            compact.push_str(topology.text());
            compact.push_str("\\nCANONICAL_TOPOLOGY_END");
        }
        if agent_dock_ui.open {
            compact.push_str("\\nT174_AGENT_DOCK_BEGIN\\n");
            compact.push_str(&t174_agent_dock_text(agent_dock, agent_dock_ui));
            compact.push_str("\\nT174_AGENT_DOCK_END");
            agent_dock_ui.hit_regions.clear();
        }
        frame.render_widget(
            Paragraph::new(compact).block(Block::bordered().title(" Winds Workbench · compact ")),
            area,
        );
        return;
'''
replace_once(old, new)

old = '''    let input = editor.lines().join("\\n");
    frame.render_widget(
        Paragraph::new(input).block(Block::bordered().title(" Shell input ")),
        areas[2],
    );
'''
new = '''    render_t174_agent_dock(frame, body[1], agent_dock, agent_dock_ui);

    let input = editor.lines().join("\\n");
    frame.render_widget(
        Paragraph::new(input).block(Block::bordered().title(" Shell input ")),
        areas[2],
    );
'''
replace_once(old, new)

old = '''fn render_workbench(
    frame: &mut Frame<'_>,
    state: &WorkbenchState,
    editor: &terminal::input::WorkbenchShellEditor,
    output: &output::WorkbenchOutput,
) {
    render_workbench_accessible(
        frame,
        state,
        editor,
        output,
        WorkbenchAccessibilityState::default(),
        (
            "CANONICAL_CANDIDATE_NOT_LOADED",
            "CANONICAL_CANDIDATE_TREE_NOT_LOADED",
        ),
        None,
        None,
    );
}
'''
new = '''fn render_workbench(
    frame: &mut Frame<'_>,
    state: &WorkbenchState,
    editor: &terminal::input::WorkbenchShellEditor,
    output: &output::WorkbenchOutput,
) {
    let mut agent_dock_ui = T174TuiAgentDockState::default();
    render_workbench_accessible(
        frame,
        state,
        editor,
        output,
        WorkbenchAccessibilityState::default(),
        (
            "CANONICAL_CANDIDATE_NOT_LOADED",
            "CANONICAL_CANDIDATE_TREE_NOT_LOADED",
        ),
        None,
        None,
        None,
        &mut agent_dock_ui,
    );
}
'''
replace_once(old, new)

old = '''    let mut navigation = ui::WorkbenchNavigation::new();
    let mut accessibility = WorkbenchAccessibilityState::default();
    let mut projected_context: Option<context::WorkbenchCandidateContext> = None;
'''
new = '''    let mut navigation = ui::WorkbenchNavigation::new();
    let mut accessibility = WorkbenchAccessibilityState::default();
    let mut projected_context: Option<context::WorkbenchCandidateContext> = None;
    let mut agent_dock: Option<ui::CanonicalAgentDockPresentation> = None;
    let mut agent_dock_ui = T174TuiAgentDockState::default();
'''
replace_once(old, new)

old = '''            projected_context.as_ref(),
            navigation.canonical_topology_presentation(),
        )
'''
new = '''            projected_context.as_ref(),
            navigation.canonical_topology_presentation(),
            agent_dock.as_ref(),
            &mut agent_dock_ui,
        )
'''
if text.count(old) != 2:
    raise SystemExit(f"expected two production render call tails, found {text.count(old)}")
text = text.replace(old, new, 2)

old = '''            let effect = navigation.handle_event(
                &mut state,
                &mut terminals,
                &mut editor,
                &[],
                &[],
                event,
            )?;
'''
new = '''            if t174_agent_dock_toggle_event(&event) {
                if agent_dock_ui.open {
                    agent_dock_ui.close();
                    agent_dock = None;
                } else {
                    match load_t174_agent_dock(&store) {
                        Ok(mut loaded) => {
                            agent_dock_ui.open_with(&mut loaded);
                            agent_dock = Some(loaded);
                        }
                        Err(error) => {
                            agent_dock = None;
                            agent_dock_ui.open_unavailable(error);
                        }
                    }
                }
                continue;
            }
            let agent_event_consumed = {
                let topology = navigation.canonical_topology_presentation();
                agent_dock_ui.handle_event(agent_dock.as_mut(), topology, &event)
            };
            if agent_event_consumed {
                continue;
            }
            let effect = navigation.handle_event(
                &mut state,
                &mut terminals,
                &mut editor,
                &[],
                &[],
                event,
            )?;
'''
replace_once(old, new)

anchor = '''fn parse_workbench_args(args: &[String]) -> crate::Result<(Option<PathBuf>, bool)> {'''
tests = r'''#[cfg(test)]
mod t174_tui_agent_dock_integration_tests {
    use super::*;
    use crate::multiplexer::domain::{AgentObservationId, MultiplexerWorkspaceId, PaneId, TabId};
    use crate::persistent_runtime::domain::{OwnerGenerationId, RuntimeNamespaceId};
    use crate::persistent_runtime::protocol::{
        AgentFamilyV2, AgentObservationConfidenceV2, AgentObservationFreshnessV2,
        AgentObservationSourceV2, AgentObservationV2,
    };

    fn observation_id(byte: u8) -> AgentObservationId {
        AgentObservationId::from_entropy_bytes([byte; 16]).expect("valid observation id")
    }

    fn observation(byte: u8, pane_byte: u8) -> AgentObservationV2 {
        AgentObservationV2 {
            observation_id: observation_id(byte),
            family: AgentFamilyV2::Codex,
            source_class: AgentObservationSourceV2::ProviderStructuredMetadata,
            confidence_class: AgentObservationConfidenceV2::Strong,
            freshness: AgentObservationFreshnessV2::Current,
            multiplexer_workspace_id: MultiplexerWorkspaceId::from_entropy_bytes([0x21; 16])
                .expect("valid workspace id"),
            git_workspace_id: Some("git-workspace".to_owned()),
            tab_id: TabId::from_entropy_bytes([0x31; 16]).expect("valid tab id"),
            pane_id: PaneId::from_entropy_bytes([pane_byte; 16]).expect("valid pane id"),
            runtime_namespace_id: Some(
                RuntimeNamespaceId::from_entropy_bytes([0x51; 16]).expect("valid runtime id"),
            ),
            provider_native_session_id: Some(format!("provider-{byte}")),
            owner_generation_id: OwnerGenerationId::from_entropy_bytes([0x61; 16])
                .expect("valid owner generation"),
            observed_unix_ms: i64::from(byte),
            structured_evidence_summary: "structured evidence".to_owned(),
        }
    }

    #[test]
    fn t174_tui_render_exposes_detection_nonclaim_and_exact_selected_binding() {
        let first = observation(0x11, 0x41);
        let mut dock = ui::CanonicalAgentDockPresentation::from_observations(vec![first.clone()])
            .expect("observation should project");
        let mut state = T174TuiAgentDockState::default();
        state.open_with(&mut dock);
        let text = t174_agent_dock_text(Some(&dock), &state);
        assert!(text.contains("DETECTION_ONLY_UNPROVEN"));
        assert!(text.contains(&first.observation_id.to_string()));
        assert!(text.contains(&first.multiplexer_workspace_id.to_string()));
        assert!(text.contains(&first.tab_id.to_string()));
        assert!(text.contains(&first.pane_id.to_string()));
    }

    #[test]
    fn t174_tui_pointer_regions_select_exact_observation_not_duplicate_label() {
        let first = observation(0x12, 0x42);
        let second = observation(0x13, 0x43);
        let mut dock = ui::CanonicalAgentDockPresentation::from_observations(vec![
            first.clone(),
            second.clone(),
        ])
        .expect("distinct observations should project");
        let mut state = T174TuiAgentDockState::default();
        state.open_with(&mut dock);
        state.hit_regions = vec![
            T174TuiAgentHitRegion {
                observation_id: first.observation_id,
                column: 10,
                row: 4,
                width: 20,
                height: 1,
            },
            T174TuiAgentHitRegion {
                observation_id: second.observation_id,
                column: 10,
                row: 5,
                width: 20,
                height: 1,
            },
        ];
        let consumed = state.handle_event(
            Some(&mut dock),
            None,
            &Event::Mouse(crossterm::event::MouseEvent {
                kind: crossterm::event::MouseEventKind::Down(
                    crossterm::event::MouseButton::Left,
                ),
                column: 12,
                row: 5,
                modifiers: KeyModifiers::NONE,
            }),
        );
        assert!(consumed);
        assert_eq!(state.selected_observation_id, Some(second.observation_id));
        assert_eq!(dock.read(second.observation_id), Some(&second));
        assert_eq!(dock.read(first.observation_id), Some(&first));
    }

    #[test]
    fn t174_tui_focus_fails_closed_when_canonical_topology_is_unavailable() {
        let first = observation(0x14, 0x44);
        let mut dock = ui::CanonicalAgentDockPresentation::from_observations(vec![first.clone()])
            .expect("observation should project");
        let mut state = T174TuiAgentDockState::default();
        state.open_with(&mut dock);
        let consumed = state.handle_event(
            Some(&mut dock),
            None,
            &Event::Key(crossterm::event::KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        );
        assert!(consumed);
        assert!(state.status.contains("canonical topology unavailable"));
        assert_eq!(dock.focused_observation_id(), None);
        assert_eq!(dock.read(first.observation_id), Some(&first));
    }

    #[test]
    fn t174_tui_alias_edit_changes_display_only_not_observation_truth() {
        let first = observation(0x15, 0x45);
        let mut dock = ui::CanonicalAgentDockPresentation::from_observations(vec![first.clone()])
            .expect("observation should project");
        let mut state = T174TuiAgentDockState::default();
        state.open_with(&mut dock);
        assert!(state.handle_event(
            Some(&mut dock),
            None,
            &Event::Key(crossterm::event::KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE)),
        ));
        for character in "Reviewer".chars() {
            assert!(state.handle_event(
                Some(&mut dock),
                None,
                &Event::Key(crossterm::event::KeyEvent::new(
                    KeyCode::Char(character),
                    KeyModifiers::NONE,
                )),
            ));
        }
        assert!(state.handle_event(
            Some(&mut dock),
            None,
            &Event::Key(crossterm::event::KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ));
        let item = dock.get(first.observation_id).expect("item should remain present");
        assert_eq!(item.display_alias.as_deref(), Some("Reviewer"));
        assert_eq!(dock.read(first.observation_id), Some(&first));
    }
}

'''
replace_once(anchor, tests + anchor)

path.write_text(text, encoding="utf-8")
