use super::screen::{TranscriptSnapshot, WorkbenchScreen};
use super::terminal::WorkbenchTerminals;
use super::{PaneId, PaneLifecycleView, PaneSize, WorkbenchState};
use crate::git::Result;
use std::collections::HashMap;
use std::io::Read;
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError, sync_channel};
use std::thread;

const OUTPUT_CHUNK_BYTES: usize = 8 * 1024;
const OUTPUT_CHANNEL_CHUNKS: usize = 32;
const MAX_DRAIN_BYTES_PER_TICK: usize = 256 * 1024;

#[derive(Debug)]
enum OutputEvent {
    Bytes(Vec<u8>),
    Eof,
    Error(String),
}

struct OutputPump {
    receiver: Receiver<OutputEvent>,
}

#[derive(Default)]
pub(crate) struct WorkbenchOutput {
    pumps: HashMap<PaneId, OutputPump>,
    screens: HashMap<PaneId, WorkbenchScreen>,
}

impl WorkbenchOutput {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn attach_screen(&mut self, pane_id: PaneId, size: PaneSize) -> Result<()> {
        if self.screens.contains_key(&pane_id) {
            return Err("workbench pane already has an attached terminal screen".into());
        }
        let screen = WorkbenchScreen::new(size)
            .map_err(|error| format!("failed to initialize workbench terminal screen: {error}"))?;
        self.screens.insert(pane_id, screen);
        Ok(())
    }

    pub(crate) fn attach_live_pane(
        &mut self,
        terminals: &mut WorkbenchTerminals,
        state: &mut WorkbenchState,
        pane_id: PaneId,
    ) -> Result<()> {
        let size = state
            .pane(pane_id)
            .ok_or("cannot attach output for an unknown workbench pane")?
            .size;
        if state.pane(pane_id).is_none_or(|pane| pane.lifecycle != PaneLifecycleView::Live) {
            return Err("cannot attach output pump to a non-live workbench pane".into());
        }
        self.attach_screen(pane_id, size)?;
        let reader = match terminals.take_output_reader_for_pump(pane_id) {
            Ok(reader) => reader,
            Err(error) => {
                self.screens.remove(&pane_id);
                return Err(error);
            }
        };
        let (sender, receiver) = sync_channel(OUTPUT_CHANNEL_CHUNKS);
        let spawn = thread::Builder::new()
            .name("winds-workbench-output".to_owned())
            .spawn(move || pump_output(reader, sender));
        if let Err(error) = spawn {
            self.screens.remove(&pane_id);
            state.set_pane_lifecycle(pane_id, PaneLifecycleView::Error);
            return Err(format!("failed to start bounded workbench output pump: {error}").into());
        }
        self.pumps.insert(pane_id, OutputPump { receiver });
        Ok(())
    }

    pub(crate) fn process_observed_bytes(&mut self, pane_id: PaneId, bytes: &[u8]) -> Result<()> {
        let screen = self
            .screens
            .get_mut(&pane_id)
            .ok_or("workbench pane has no attached terminal screen")?;
        screen.process_observed_bytes(bytes);
        Ok(())
    }

    pub(crate) fn screen_contents(&self, pane_id: PaneId) -> Option<String> {
        self.screens.get(&pane_id).map(WorkbenchScreen::screen_contents)
    }

    pub(crate) fn transcript_snapshot(&self, pane_id: PaneId) -> Option<TranscriptSnapshot> {
        self.screens
            .get(&pane_id)
            .map(WorkbenchScreen::transcript_snapshot)
    }

    pub(crate) fn drain_tick(
        &mut self,
        terminals: &mut WorkbenchTerminals,
        state: &mut WorkbenchState,
    ) -> Result<()> {
        self.prune_unowned(state, terminals);
        self.sync_screen_sizes(state)?;

        let pane_ids: Vec<PaneId> = self.pumps.keys().copied().collect();
        let mut remaining = MAX_DRAIN_BYTES_PER_TICK;
        let mut finished = Vec::new();

        for pane_id in pane_ids {
            while remaining > 0 {
                let event = match self
                    .pumps
                    .get_mut(&pane_id)
                    .expect("output pump key was just collected")
                    .receiver
                    .try_recv()
                {
                    Ok(event) => event,
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        match terminals.poll_exit(state, pane_id) {
                            Ok(Some(_)) => {
                                finished.push(pane_id);
                                break;
                            }
                            Ok(None) => {
                                state.set_pane_lifecycle(pane_id, PaneLifecycleView::Error);
                                return Err(
                                    "workbench output pump disconnected while the owned child remained live"
                                        .into(),
                                );
                            }
                            Err(error) => return Err(error),
                        }
                    }
                };

                match event {
                    OutputEvent::Bytes(bytes) => {
                        remaining = remaining.saturating_sub(bytes.len());
                        self.process_observed_bytes(pane_id, &bytes)?;
                    }
                    OutputEvent::Eof => match terminals.poll_exit(state, pane_id) {
                        Ok(Some(_)) => {
                            finished.push(pane_id);
                            break;
                        }
                        Ok(None) => {
                            state.set_pane_lifecycle(pane_id, PaneLifecycleView::Error);
                            return Err(
                                "workbench output reader closed while the owned child remained live"
                                    .into(),
                            );
                        }
                        Err(error) => return Err(error),
                    },
                    OutputEvent::Error(read_error) => {
                        let exit_note = match terminals.poll_exit(state, pane_id) {
                            Ok(Some(exit)) => format!("; child exit was observed: {exit:?}"),
                            Ok(None) => {
                                state.set_pane_lifecycle(pane_id, PaneLifecycleView::Error);
                                "; owned child remained live".to_owned()
                            }
                            Err(error) => format!("; exit observation also failed: {error}"),
                        };
                        return Err(format!(
                            "workbench output reader failed: {read_error}{exit_note}"
                        )
                        .into());
                    }
                }
            }
            if remaining == 0 {
                break;
            }
        }

        for pane_id in finished {
            self.pumps.remove(&pane_id);
        }
        Ok(())
    }

    fn sync_screen_sizes(&mut self, state: &WorkbenchState) -> Result<()> {
        for (pane_id, screen) in &mut self.screens {
            let Some(pane) = state.pane(*pane_id) else {
                continue;
            };
            if let Err(error) = screen.explicit_resize(pane.size) {
                return Err(format!("failed to synchronize terminal screen size: {error}").into());
            }
        }
        Ok(())
    }

    fn prune_unowned(&mut self, state: &WorkbenchState, terminals: &WorkbenchTerminals) {
        self.screens
            .retain(|pane_id, _| state.pane(*pane_id).is_some());
        self.pumps.retain(|pane_id, _| {
            terminals.has_owned_terminal(*pane_id)
                && state.pane(*pane_id).is_some_and(|pane| {
                    matches!(pane.lifecycle, PaneLifecycleView::Live | PaneLifecycleView::Exited)
                })
        });
    }
}

fn pump_output(mut reader: Box<dyn Read + Send>, sender: SyncSender<OutputEvent>) {
    let mut buffer = [0_u8; OUTPUT_CHUNK_BYTES];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => {
                let _ = sender.send(OutputEvent::Eof);
                return;
            }
            Ok(count) => {
                if sender
                    .send(OutputEvent::Bytes(buffer[..count].to_vec()))
                    .is_err()
                {
                    return;
                }
            }
            Err(error) => {
                let _ = sender.send(OutputEvent::Error(error.to_string()));
                return;
            }
        }
    }
}
