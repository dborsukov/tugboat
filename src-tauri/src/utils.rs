use tauri::State;

#[tauri::command]
#[specta::specta]
pub fn get_app_version(state: State<'_, crate::AppState>) -> &'static str {
    state.version
}
