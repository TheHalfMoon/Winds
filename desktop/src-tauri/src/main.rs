use winds_control::desktop::{
    DesktopBridgeCreateSessionRequest, DesktopBridgeLayoutPresentation, DesktopBridgeLayoutRequest,
    DesktopBridgeProjectPresentationBatchRequest, DesktopBridgeProjectSummary,
    DesktopBridgeRenameSessionRequest, DesktopBridgeSessionPresentationBatchRequest,
    DesktopBridgeSessionSummary, DesktopBridgeSnapshot, desktop_bridge_create_session,
    desktop_bridge_default_home, desktop_bridge_load_layout, desktop_bridge_rename_session,
    desktop_bridge_save_layout, desktop_bridge_snapshot, desktop_bridge_update_project,
    desktop_bridge_update_session,
};

fn host_error(context: &str, error: impl std::fmt::Display) -> String {
    eprintln!("Winds desktop host: {context}: {error}");
    format!("Winds desktop host: {context} failed. Refresh and retry.")
}

fn bridge_home() -> Result<std::path::PathBuf, String> {
    desktop_bridge_default_home().map_err(|error| host_error("state path", error))
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            left_dock_snapshot,
            left_dock_update_project,
            left_dock_create_session,
            left_dock_rename_session,
            left_dock_update_session,
            workspace_load_layout,
            workspace_save_layout,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Winds desktop host");
}
