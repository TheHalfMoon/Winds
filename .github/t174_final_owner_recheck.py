from pathlib import Path

path = Path("src/workbench.rs")
text = path.read_text()
old = '''    let mut dock = ui::CanonicalAgentDockPresentation::from_observations(trailing_observations)?;
    dock.focus(observation_id, &topology)
        .ok_or_else(|| "focus target is stale, absent, ambiguous, or substituted".to_owned())?;
    Ok((dock, topology))
'''
new = '''    let mut dock = ui::CanonicalAgentDockPresentation::from_observations(trailing_observations)?;
    dock.focus(observation_id, &topology)
        .ok_or_else(|| "focus target is stale, absent, ambiguous, or substituted".to_owned())?;

    let final_owner_generation_id = store
        .latest_persistent_runtime_owner_generation()
        .map_err(|error| format!("final owner generation record unavailable: {error}"))?
        .ok_or_else(|| "owner generation unavailable after focus validation".to_owned())?;
    if final_owner_generation_id != owner_generation_id {
        return Err("focus owner generation changed during validation".to_owned());
    }
    Ok((dock, topology))
'''
count = text.count(old)
if count != 1:
    raise SystemExit(f"expected one guarded focus-return block, found {count}")
path.write_text(text.replace(old, new, 1))
