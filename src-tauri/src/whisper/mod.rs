mod input;

use crate::whisper::input::filepath_to_mels;
use burn_wgpu::{AutoGraphicsApi, Wgpu, WgpuDevice};
use hf_hub;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use whisper::decoding::DecodingOptions;
use whisper::whisper_helper::{WhichModel, WhisperHelper};

#[derive(Default)]
pub struct WhisperState {
    pub helper: Mutex<Option<WhisperHelper<Wgpu<AutoGraphicsApi, f32, i32>>>>,
    pub model_kind: Mutex<WhichModel>,
    pub current_model_downloading_progress: Mutex<f32>,
    pub task_progess: Mutex<f32>,
    pub model_is_loaded: AtomicBool,
}

unsafe impl Send for WhisperState {}

unsafe impl Sync for WhisperState {}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhisperTask {
    file_path: String,
    decode_option: DecodingOptions,
}

#[tauri::command]
pub fn whisper_change_model(model_kind: WhichModel, whisper_state: tauri::State<'_, WhisperState>) {
    *whisper_state.helper.lock().unwrap() = None; // take();
    *whisper_state.model_kind.lock().unwrap() = model_kind;

    let cache = hf_hub::Cache::default();
    *whisper_state.current_model_downloading_progress.lock().unwrap() = if cache
        .model(model_kind.model_and_revision().0.to_string())
        .get("pytorch_model.bin")
        .is_some()
    {
        1.0f32
    } else {
        0.0f32
    };
}

#[tauri::command]
pub fn whisper_get_current_model_downloading_preogress(
    whisper_state: tauri::State<'_, WhisperState>,
) -> f32 {
    *whisper_state
        .current_model_downloading_progress
        .lock()
        .unwrap()
}

#[tauri::command]
pub fn whisper_get_task_progess(whisper_state: tauri::State<'_, WhisperState>) -> f32 {
    *whisper_state.task_progess.lock().unwrap()
}

#[tauri::command]
pub fn whisper_get_model_is_loaded(whisper_state: tauri::State<'_, WhisperState>) -> bool {
    whisper_state
        .model_is_loaded
        .load(std::sync::atomic::Ordering::SeqCst)
}

#[tauri::command]
pub fn whisper_get_model_kinds() -> Vec<WhichModel> {
    vec![
        WhichModel::Tiny,
        WhichModel::TinyEn,
        WhichModel::Base,
        WhichModel::BaseEn,
        WhichModel::Small,
        WhichModel::SmallEn,
        WhichModel::Medium,
        WhichModel::MediumEn,
        // WhichModel::LargeV1,
        // WhichModel::LargeV2,
        WhichModel::LargeV3,
    ]
}

#[tauri::command]
pub async fn whisper_run_tasks(
    tasks: Vec<WhisperTask>,
    whisper_state: tauri::State<'_, WhisperState>,
) -> Result<Vec<String>, ()> {
    let mut model = whisper_state.helper.lock().unwrap();
    if model.is_none() {
        let device = WgpuDevice::BestAvailable;
        *model = Some(WhisperHelper::new(
            *whisper_state.model_kind.lock().unwrap(),
            &device,
        ));
        whisper_state
            .model_is_loaded
            .store(true, std::sync::atomic::Ordering::SeqCst);
        *whisper_state
            .current_model_downloading_progress
            .lock()
            .unwrap() = 1.0f32;
    }
    println!("tasks = : {tasks:#?}");
    let total_tasks = tasks.len();
    let device = WgpuDevice::BestAvailable;
    let res = tasks
        .iter()
        .map(|task| {
            let (mels, _) = filepath_to_mels(&task.file_path, &device);
            let decoding_res = (model.as_ref())
                .unwrap()
                .run(mels, task.decode_option.clone());
            let mut res_text = String::new();
            for i in decoding_res {
                res_text += &*i.text;
            }
            *whisper_state.task_progess.lock().unwrap() += 1.0 / (total_tasks as f32);
            res_text
        })
        .collect::<Vec<_>>();
    *whisper_state.task_progess.lock().unwrap() = 1.0f32;
    println!("res = {:#?}", res);
    return Ok(res);
}
