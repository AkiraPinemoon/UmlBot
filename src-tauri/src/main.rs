#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod project_analyzer;

#[tauri::command]
fn process_directory(directory: &str) {
    project_analyzer::analyse_project(directory)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![process_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
