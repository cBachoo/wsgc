mod spark;

use spark::{calculate, CalcResult, Model, Spark};

/// Tauri command the frontend calls via `invoke("calc_sparks", { sparks, model })`.
/// Tauri deserializes the JSON args into these Rust types, runs `calculate`,
/// and serializes the `CalcResult` back to the frontend.
#[tauri::command]
fn calc_sparks(sparks: Vec<Spark>, model: Model) -> CalcResult {
    // calc is slang for calculator btw
    calculate(&sparks, model)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![calc_sparks])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
