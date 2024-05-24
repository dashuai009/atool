// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate ffmpeg_next;
mod password_generator;
mod whisper;
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}


fn main() {
    tauri::Builder::default()
        .manage(whisper::WhisperState::default())
        .invoke_handler(tauri::generate_handler![
            greet, 
            password_generator::gen_pwd_cmd,
            whisper::whisper_change_model,
            whisper::whisper_model_is_downloaded,
            whisper::whisper_get_current_model_downloading_preogress,
            whisper::whisper_get_task_progess,
            whisper::whisper_get_model_is_loaded,
            whisper::whisper_run_tasks,
            whisper::whisper_get_model_kinds,
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
