from pathlib import Path

path = Path("src/workbench.rs")
text = path.read_text(encoding="utf-8")


def replace_once(old: str, new: str) -> None:
    global text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected one replacement target, found {count}: {old[:140]!r}")
    text = text.replace(old, new, 1)


handle_anchor = '''    fn handle_event(
        &mut self,
        mut dock: Option<&mut ui::CanonicalAgentDockPresentation>,
        topology: Option<&ui::CanonicalTopologyPresentation>,
        event: &Event,
    ) -> bool {'''
handle_replacement = '''    fn focus_target_for_event(
        &self,
        event: &Event,
    ) -> Option<crate::multiplexer::domain::AgentObservationId> {
        if !self.open || self.alias_input.is_some() {
            return None;
        }
        match event {
            Event::Key(key)
                if key.kind != KeyEventKind::Release && key.code == KeyCode::Enter =>
            {
                self.selected_observation_id
            }
            Event::Mouse(mouse)
                if mouse.kind
                    == crossterm::event::MouseEventKind::Down(
                        crossterm::event::MouseButton::Right,
                    ) =>
            {
                self.hit_regions
                    .iter()
                    .copied()
                    .find(|region| region.contains(mouse.column, mouse.row))
                    .map(|region| region.observation_id)
            }
            _ => None,
        }
    }

    fn handle_event(
        &mut self,
        mut dock: Option<&mut ui::CanonicalAgentDockPresentation>,
        topology: Option<&ui::CanonicalTopologyPresentation>,
        event: &Event,
    ) -> bool {'''
replace_once(handle_anchor, handle_replacement)

load_anchor = '''fn load_t174_agent_dock(
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
'''
load_replacement = load_anchor + r'''
fn preserve_t174_display_aliases(
    previous: &ui::CanonicalAgentDockPresentation,
    refreshed: &mut ui::CanonicalAgentDockPresentation,
) -> Result<(), String> {
    for row in previous.list("", ui::CanonicalAgentDockSort::Family, false) {
        let Some(alias) = previous
            .get(row.observation_id)
            .and_then(|item| item.display_alias.as_deref())
        else {
            continue;
        };
        if refreshed.get(row.observation_id).is_some() {
            refreshed.rename(row.observation_id, Some(alias))?;
        }
    }
    Ok(())
}

fn load_t174_agent_focus_context(
    store: &crate::store::Store,
    observation_id: crate::multiplexer::domain::AgentObservationId,
) -> Result<
    (
        ui::CanonicalAgentDockPresentation,
        ui::CanonicalTopologyPresentation,
    ),
    String,
> {
    let owner_generation_id = store
        .latest_persistent_runtime_owner_generation()
        .map_err(|error| format!("owner generation record unavailable: {error}"))?
        .ok_or_else(|| "owner generation unavailable".to_owned())?;
    let mut client = crate::persistent_runtime::client::RustLocalControlClient::connect(
        None,
        Some(owner_generation_id),
    )
    .map_err(|error| format!("owner connection unavailable: {error}"))?;

    let first_observations = client
        .refresh_agent_observation_projection(None)
        .map_err(|error| format!("agent observation refresh unavailable: {error}"))?;
    let first = first_observations
        .iter()
        .find(|observation| observation.observation_id == observation_id)
        .ok_or_else(|| "focus observation is absent from current owner truth".to_owned())?;
    if first.freshness
        != crate::persistent_runtime::protocol::AgentObservationFreshnessV2::Current
    {
        return Err(format!(
            "focus observation is not current: {:?}",
            first.freshness
        ));
    }
    if first.owner_generation_id != owner_generation_id {
        return Err("focus observation owner generation does not match current owner".to_owned());
    }
    let first_workspace_id = first.multiplexer_workspace_id;
    let first_tab_id = first.tab_id;
    let first_pane_id = first.pane_id;
    let first_runtime_namespace_id = first.runtime_namespace_id;

    client
        .refresh_topology_projection(Some(first_workspace_id))
        .map_err(|error| format!("focus topology refresh unavailable: {error}"))?;
    let first_topology = ui::CanonicalTopologyPresentation::from_client(
        &client,
        false,
        false,
        false,
    )
    .ok_or_else(|| "focus topology presentation unavailable".to_owned())?;
    let first_topology_generation = first_topology.topology_generation();

    let trailing_observations = client
        .refresh_agent_observation_projection(None)
        .map_err(|error| format!("trailing agent observation refresh unavailable: {error}"))?;
    let trailing = trailing_observations
        .iter()
        .find(|observation| observation.observation_id == observation_id)
        .ok_or_else(|| "focus observation disappeared during validation".to_owned())?;
    if trailing.freshness
        != crate::persistent_runtime::protocol::AgentObservationFreshnessV2::Current
        || trailing.owner_generation_id != owner_generation_id
        || trailing.multiplexer_workspace_id != first_workspace_id
        || trailing.tab_id != first_tab_id
        || trailing.pane_id != first_pane_id
        || trailing.runtime_namespace_id != first_runtime_namespace_id
    {
        return Err("focus observation changed during validation".to_owned());
    }

    client
        .refresh_topology_projection(Some(first_workspace_id))
        .map_err(|error| format!("trailing focus topology refresh unavailable: {error}"))?;
    let topology = ui::CanonicalTopologyPresentation::from_client(
        &client,
        false,
        false,
        false,
    )
    .ok_or_else(|| "trailing focus topology presentation unavailable".to_owned())?;
    if topology.topology_generation() != first_topology_generation {
        return Err("focus topology generation changed during validation".to_owned());
    }

    let mut dock = ui::CanonicalAgentDockPresentation::from_observations(trailing_observations)?;
    dock.focus(observation_id, &topology)
        .ok_or_else(|| "focus target is stale, absent, ambiguous, or substituted".to_owned())?;
    Ok((dock, topology))
}
'''
replace_once(load_anchor, load_replacement)

loop_anchor = '''            let agent_event_consumed = {
                let topology = navigation.canonical_topology_presentation();
                agent_dock_ui.handle_event(agent_dock.as_mut(), topology, &event)
            };
            if agent_event_consumed {
                continue;
            }
'''
loop_replacement = '''            let mut exact_focus_topology = None;
            if let Some(focus_observation_id) = agent_dock_ui.focus_target_for_event(&event) {
                match load_t174_agent_focus_context(&store, focus_observation_id) {
                    Ok((mut refreshed_dock, topology)) => {
                        if let Some(previous_dock) = agent_dock.as_ref()
                            && let Err(error) = preserve_t174_display_aliases(
                                previous_dock,
                                &mut refreshed_dock,
                            )
                        {
                            agent_dock = None;
                            agent_dock_ui.open_unavailable(format!(
                                "FOCUS_ALIAS_RECONCILIATION_FAILED · {error}"
                            ));
                            continue;
                        }
                        if refreshed_dock.view(focus_observation_id).is_none() {
                            agent_dock = None;
                            agent_dock_ui.open_unavailable(
                                "FOCUS_REFRESH_FAILED · exact observation disappeared",
                            );
                            continue;
                        }
                        agent_dock_ui.selected_observation_id = Some(focus_observation_id);
                        agent_dock = Some(refreshed_dock);
                        exact_focus_topology = Some(topology);
                    }
                    Err(error) => {
                        agent_dock = None;
                        agent_dock_ui
                            .open_unavailable(format!("FOCUS_REFRESH_FAILED · {error}"));
                        continue;
                    }
                }
            }
            let agent_event_consumed = {
                let topology = exact_focus_topology
                    .as_ref()
                    .or_else(|| navigation.canonical_topology_presentation());
                agent_dock_ui.handle_event(agent_dock.as_mut(), topology, &event)
            };
            if agent_event_consumed {
                continue;
            }
'''
replace_once(loop_anchor, loop_replacement)

# Extend focused tests with exact focus-event routing and alias carry-forward.
test_anchor = '''    #[test]
    fn t174_tui_alias_edit_changes_display_only_not_observation_truth() {'''
extra_tests = r'''    #[test]
    fn t174_tui_focus_event_resolves_exact_pointer_observation_before_refresh() {
        let first = observation(0x16, 0x46);
        let second = observation(0x17, 0x47);
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
                column: 20,
                row: 7,
                width: 20,
                height: 1,
            },
            T174TuiAgentHitRegion {
                observation_id: second.observation_id,
                column: 20,
                row: 8,
                width: 20,
                height: 1,
            },
        ];
        let event = Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Down(
                crossterm::event::MouseButton::Right,
            ),
            column: 23,
            row: 8,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(state.focus_target_for_event(&event), Some(second.observation_id));
    }

    #[test]
    fn t174_tui_refresh_preserves_display_aliases_only_for_surviving_exact_ids() {
        let first = observation(0x18, 0x48);
        let removed = observation(0x19, 0x49);
        let newcomer = observation(0x1a, 0x4a);
        let mut previous = ui::CanonicalAgentDockPresentation::from_observations(vec![
            first.clone(),
            removed.clone(),
        ])
        .expect("previous observations should project");
        previous
            .rename(first.observation_id, Some("Primary"))
            .expect("alias should be accepted");
        previous
            .rename(removed.observation_id, Some("Removed"))
            .expect("alias should be accepted");
        let mut refreshed = ui::CanonicalAgentDockPresentation::from_observations(vec![
            first.clone(),
            newcomer.clone(),
        ])
        .expect("refreshed observations should project");
        preserve_t174_display_aliases(&previous, &mut refreshed)
            .expect("valid aliases should reconcile");
        assert_eq!(
            refreshed
                .get(first.observation_id)
                .and_then(|item| item.display_alias.as_deref()),
            Some("Primary")
        );
        assert!(refreshed.get(removed.observation_id).is_none());
        assert_eq!(
            refreshed
                .get(newcomer.observation_id)
                .and_then(|item| item.display_alias.as_deref()),
            None
        );
    }

'''
replace_once(test_anchor, extra_tests + test_anchor)

path.write_text(text, encoding="utf-8")
