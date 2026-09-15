use std::io::Read;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tauri::State;
use tauri::ipc::{Channel, Response};
use winds_control::desktop::{
    DesktopBridgeCreateSessionRequest, DesktopBridgeLayoutPresentation, DesktopBridgeLayoutRequest,
    DesktopBridgeProjectPresentationBatchRequest, DesktopBridgeProjectSummary,
    DesktopBridgeRenameSessionRequest, DesktopBridgeSessionPresentationBatchRequest,
    DesktopBridgeSessionSummary, DesktopBridgeSnapshot, desktop_bridge_create_session,
    desktop_bridge_default_home, desktop_bridge_load_layout, desktop_bridge_rename_session,
    desktop_bridge_save_layout, desktop_bridge_snapshot, desktop_bridge_update_project,
    desktop_bridge_update_session,
};
use winds_control::desktop_files::{
    DesktopBoundDockRequest, DesktopChangesResponse, DesktopFilePreviewRequest,
    DesktopFilePreviewResponse, DesktopFilesResponse, DesktopRightDockBinding,
    DesktopRightDockTarget, desktop_right_dock_bind, desktop_right_dock_changes,
    desktop_right_dock_files, desktop_right_dock_preview_file,
};
use winds_control::desktop_terminal::{
    DesktopTerminalInputRequest, DesktopTerminalRegistry, DesktopTerminalResizeRequest,
    DesktopTerminalStartRequest, DesktopTerminalStatus, DesktopTerminalTargetRequest,
};

const TERMINAL_OUTPUT_CHUNK_BYTES: usize = 8 * 1024;
const TERMINAL_EXIT_POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Default)]
struct TerminalHostState {
    registry: Mutex<DesktopTerminalRegistry>,
}

fn host_error(context: &str, error: impl std::fmt::Display) -> String {
    eprintln!("Winds desktop host: {context}: {error}");
    format!("Winds desktop host: {context} failed. Refresh and retry.")
}

fn bridge_home() -> Result<std::path::PathBuf, String> {
    desktop_bridge_default_home().map_err(|error| host_error("state path", error))
}

fn terminal_registry(
    state: &Arc<TerminalHostState>,
) -> Result<MutexGuard<'_, DesktopTerminalRegistry>, String> {
    state
        .registry
        .lock()
        .map_err(|_| host_error("terminal registry", "state lock poisoned"))
}

#[tauri::command]
fn left_dock_snapshot() -> Result<DesktopBridgeSnapshot, String> {
    let home = bridge_home()?;
    desktop_bridge_snapshot(&home).map_err(|error| host_error("left dock snapshot", error))
}

#[tauri::command]
fn left_dock_update_project(
    request: DesktopBridgeProjectPresentationBatchRequest,
) -> Result<Vec<DesktopBridgeProjectSummary>, String> {
    let home = bridge_home()?;
    desktop_bridge_update_project(&home, request)
        .map_err(|error| host_error("Project presentation update", error))
}

#[tauri::command]
fn left_dock_create_session(
    request: DesktopBridgeCreateSessionRequest,
) -> Result<DesktopBridgeSessionSummary, String> {
    let home = bridge_home()?;
    desktop_bridge_create_session(&home, request)
        .map_err(|error| host_error("Session creation", error))
}

#[tauri::command]
fn left_dock_rename_session(
    request: DesktopBridgeRenameSessionRequest,
) -> Result<DesktopBridgeSessionSummary, String> {
    let home = bridge_home()?;
    desktop_bridge_rename_session(&home, request)
        .map_err(|error| host_error("Session rename", error))
}

#[tauri::command]
fn left_dock_update_session(
    request: DesktopBridgeSessionPresentationBatchRequest,
) -> Result<Vec<DesktopBridgeSessionSummary>, String> {
    let home = bridge_home()?;
    desktop_bridge_update_session(&home, request)
        .map_err(|error| host_error("Session presentation update", error))
}

#[tauri::command]
fn workspace_load_layout(
    workspace_id: String,
) -> Result<Option<DesktopBridgeLayoutPresentation>, String> {
    let home = bridge_home()?;
    desktop_bridge_load_layout(&home, &workspace_id)
        .map_err(|error| host_error("workspace layout load", error))
}

#[tauri::command]
fn workspace_save_layout(
    request: DesktopBridgeLayoutRequest,
) -> Result<DesktopBridgeLayoutPresentation, String> {
    let home = bridge_home()?;
    desktop_bridge_save_layout(&home, request)
        .map_err(|error| host_error("workspace layout save", error))
}

#[tauri::command]
fn right_dock_bind(request: DesktopRightDockTarget) -> Result<DesktopRightDockBinding, String> {
    let home = bridge_home()?;
    desktop_right_dock_bind(&home, request).map_err(|error| host_error("right dock binding", error))
}

#[tauri::command]
fn right_dock_files(request: DesktopBoundDockRequest) -> Result<DesktopFilesResponse, String> {
    let home = bridge_home()?;
    desktop_right_dock_files(&home, request).map_err(|error| host_error("right dock Files", error))
}

#[tauri::command]
fn right_dock_preview_file(
    request: DesktopFilePreviewRequest,
) -> Result<DesktopFilePreviewResponse, String> {
    let home = bridge_home()?;
    desktop_right_dock_preview_file(&home, request)
        .map_err(|error| host_error("right dock file preview", error))
}

#[tauri::command]
fn right_dock_changes(request: DesktopBoundDockRequest) -> Result<DesktopChangesResponse, String> {
    let home = bridge_home()?;
    desktop_right_dock_changes(&home, request)
        .map_err(|error| host_error("right dock Changes", error))
}

#[tauri::command]
fn terminal_status(
    canonical_session_id: String,
    state: State<'_, Arc<TerminalHostState>>,
) -> Result<Option<DesktopTerminalStatus>, String> {
    let state = state.inner();
    let registry = terminal_registry(state)?;
    Ok(registry.status_for_session(&canonical_session_id))
}

#[tauri::command]
async fn terminal_start(
    request: DesktopTerminalStartRequest,
    output: Channel<Response>,
    lifecycle: Channel<DesktopTerminalStatus>,
    state: State<'_, Arc<TerminalHostState>>,
) -> Result<DesktopTerminalStatus, String> {
    let home = bridge_home()?;
    let host_state = state.inner().clone();
    let start_state = host_state.clone();
    let started = tauri::async_runtime::spawn_blocking(move || {
        let canonical_session_id = request.canonical_session_id.clone();
        {
            let mut registry = terminal_registry(&start_state)?;
            registry
                .reserve_start(&canonical_session_id)
                .map_err(|error| host_error("terminal start reservation", error))?;
        }
        let prepared = match DesktopTerminalRegistry::prepare_start(&home, request) {
            Ok(prepared) => prepared,
            Err(error) => {
                if let Ok(mut registry) = terminal_registry(&start_state) {
                    registry.cancel_start(&canonical_session_id);
                }
                return Err(host_error("terminal start", error));
            }
        };
        let mut registry = terminal_registry(&start_state)?;
        registry
            .commit_prepared_start(prepared)
            .map_err(|error| host_error("terminal start commit", error))
    })
    .await
    .map_err(|error| host_error("terminal start worker", error))??;
    let status = started.status.clone();
    let target = DesktopTerminalTargetRequest {
        canonical_session_id: status.canonical_session_id.clone(),
        terminal_id: status.terminal_id.clone(),
    };
    let lifecycle_state = host_state.clone();
    let lifecycle_target = target.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(TERMINAL_EXIT_POLL_INTERVAL);
            let observation = terminal_registry(&lifecycle_state).and_then(|mut registry| {
                registry
                    .observe_process_exit(&lifecycle_target)
                    .map_err(|error| host_error("terminal process observation", error))
            });
            match observation {
                Ok(Some(status)) => {
                    if let Err(error) = lifecycle.send(status) {
                        eprintln!(
                            "Winds desktop host: terminal lifecycle channel send failed: {error}"
                        );
                    }
                    break;
                }
                Ok(None) => {}
                Err(error) => {
                    eprintln!("{error}");
                    break;
                }
            }
        }
    });
    std::thread::spawn(move || {
        let mut reader = started.output_reader;
        let mut buffer = [0_u8; TERMINAL_OUTPUT_CHUNK_BYTES];
        let mut read_error = None;
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => {
                    if let Err(error) = output.send(Response::new(buffer[..read].to_vec())) {
                        eprintln!(
                            "Winds desktop host: terminal output channel send failed: {error}"
                        );
                    }
                }
                Err(error) => {
                    eprintln!("Winds desktop host: terminal output read failed: {error}");
                    read_error = Some("OUTPUT_READ_FAILED".to_owned());
                    break;
                }
            }
        }
        if let Err(error) = terminal_registry(&host_state).and_then(|mut registry| {
            registry
                .observe_output_end(target, read_error)
                .map(|_| ())
                .map_err(|error| host_error("terminal output completion", error))
        }) {
            eprintln!("{error}");
        }
    });
    Ok(status)
}

#[tauri::command]
fn terminal_input(
    request: DesktopTerminalInputRequest,
    state: State<'_, Arc<TerminalHostState>>,
) -> Result<DesktopTerminalStatus, String> {
    terminal_registry(state.inner())?
        .send_input(request)
        .map_err(|error| host_error("terminal input", error))
}

#[tauri::command]
fn terminal_resize(
    request: DesktopTerminalResizeRequest,
    state: State<'_, Arc<TerminalHostState>>,
) -> Result<DesktopTerminalStatus, String> {
    terminal_registry(state.inner())?
        .resize(request)
        .map_err(|error| host_error("terminal resize", error))
}

#[tauri::command]
fn terminal_interrupt(
    request: DesktopTerminalTargetRequest,
    state: State<'_, Arc<TerminalHostState>>,
) -> Result<DesktopTerminalStatus, String> {
    terminal_registry(state.inner())?
        .interrupt(request)
        .map_err(|error| host_error("terminal interrupt", error))
}

#[tauri::command]
fn terminal_terminate(
    request: DesktopTerminalTargetRequest,
    state: State<'_, Arc<TerminalHostState>>,
) -> Result<DesktopTerminalStatus, String> {
    terminal_registry(state.inner())?
        .terminate(request)
        .map_err(|error| host_error("terminal terminate", error))
}

#[tauri::command]
fn terminal_close(
    request: DesktopTerminalTargetRequest,
    state: State<'_, Arc<TerminalHostState>>,
) -> Result<DesktopTerminalStatus, String> {
    terminal_registry(state.inner())?
        .close(request)
        .map_err(|error| host_error("terminal close", error))
}

fn main() {
    tauri::Builder::default()
        .manage(Arc::new(TerminalHostState::default()))
        .invoke_handler(tauri::generate_handler![
            left_dock_snapshot,
            left_dock_update_project,
            left_dock_create_session,
            left_dock_rename_session,
            left_dock_update_session,
            workspace_load_layout,
            workspace_save_layout,
            right_dock_bind,
            right_dock_files,
            right_dock_preview_file,
            right_dock_changes,
            terminal_status,
            terminal_start,
            terminal_input,
            terminal_resize,
            terminal_interrupt,
            terminal_terminate,
            terminal_close,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Winds desktop host");
}
