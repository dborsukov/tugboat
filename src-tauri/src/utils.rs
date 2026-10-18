use tauri::State;

#[tauri::command]
#[specta::specta]
pub fn get_app_version(state: State<'_, crate::AppState>) -> &'static str {
    state.version
}

/// A file name safe to hand to a save dialog: path separators and other
/// characters a file system rejects become `-`, and an empty proposal falls
/// back to `fallback`.
pub(crate) fn suggested_file_name(proposed: &str, fallback: &str) -> String {
    let cleaned: String = proposed
        .chars()
        .map(|character| match character {
            // Invalid characters
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            character if character.is_control() => '-',
            character => character,
        })
        .collect();

    // More invalid characters
    let trimmed = cleaned.trim().trim_end_matches(['.', ' ']);
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}
