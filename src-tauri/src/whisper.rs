use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TmpDecodeOption{
    task: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhisperTask
{
    file_path: String,
    decode_option: TmpDecodeOption
}

#[tauri::command]
pub fn whisper_run_tasks(tasks: Vec<WhisperTask>) -> String {
    println!("tmp: {tasks:?}");
    return tasks[0].decode_option.task.clone();
}
