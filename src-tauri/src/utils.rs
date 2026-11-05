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

pub(crate) fn plural(
    count: usize,
    one: &'static str,
    few: &'static str,
    many: &'static str,
) -> &'static str {
    let last = count % 10;
    let last_two = count % 100;

    if last == 1 && last_two != 11 {
        one
    } else if (2..=4).contains(&last) && !(12..=14).contains(&last_two) {
        few
    } else {
        many
    }
}
