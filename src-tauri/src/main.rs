#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

mod project_analyzer;
mod plantuml;

#[tauri::command]
fn process_directory(directory: &str, custom_java:&str, app: tauri::AppHandle) {
    app.emit_all("analysis_info", "starting").unwrap();
    let classes = project_analyzer::analyse_project(directory, &app);
    match custom_java {
        "" => plantuml::classes_to_graph(&classes, directory, None, &app),
        _ => plantuml::classes_to_graph(&classes, directory, Some(custom_java), &app),
    }
    app.emit_all("analysis_info", "done").unwrap();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![process_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
