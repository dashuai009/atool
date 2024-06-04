mod input;
use burn_wgpu::{AutoGraphicsApi, Wgpu, WgpuDevice};
use hf_hub;
use input::load_audio_waveform_with_ffmpeg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use whisper::decoding::DecodingOptions;
use whisper::whisper_helper::{WhichModel, WhisperHelper};

pub struct WhisperState {
    pub helper: Mutex<Option<WhisperHelper<Wgpu<AutoGraphicsApi, f32, i32>>>>,
    pub model_in_memory: Mutex<Option<WhichModel>>,
    pub selected_model_kind: Mutex<WhichModel>,
    pub model_is_downloaded: Mutex<HashMap<WhichModel, bool>>,
    pub current_model_downloading_progress: Mutex<f32>,
    pub task_progess: Mutex<f32>,
    pub model_is_loaded: AtomicBool,
}

unsafe impl Send for WhisperState {}

unsafe impl Sync for WhisperState {}

impl Default for WhisperState {
    fn default() -> Self {
        let mut model_is_downloaded = HashMap::new();
        let cache = hf_hub::Cache::default();
        for i in whisper_get_model_kinds() {
            let j = cache
                .model(i.model_and_revision().0.to_string())
                .get("pytorch_model.bin")
                .is_some();
            model_is_downloaded.insert(i, j);
        }
        Self {
            helper: Default::default(),
            model_in_memory: Default::default(),
            selected_model_kind: Default::default(),
            model_is_downloaded: model_is_downloaded.into(),
            current_model_downloading_progress: Default::default(),
            task_progess: Default::default(),
            model_is_loaded: Default::default(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct WhisperTask {
    file_path: String,
    decode_option: DecodingOptions,
}

/// unload model from memory
#[tauri::command]
pub fn whisper_unload_model(whisper_state: tauri::State<'_, WhisperState>) {
    *whisper_state.helper.lock().unwrap() = None;
    *whisper_state.model_in_memory.lock().unwrap() = None;
}

/// wether model is in memory
#[tauri::command]
pub fn whisper_get_model_in_memory(whisper_state: tauri::State<'_, WhisperState>) -> Option<WhichModel> {
    *whisper_state.model_in_memory.lock().unwrap()
}

/// set the kind of selected model
#[tauri::command]
pub fn whisper_set_selected_model_kind(
    model_kind: WhichModel,
    whisper_state: tauri::State<'_, WhisperState>,
) {
    *whisper_state.selected_model_kind.lock().unwrap() = model_kind;
}

/// update the download state of models.
#[tauri::command]
pub fn whisper_update_model_is_downloaded(
    whisper_state: tauri::State<'_, WhisperState>,
) -> HashMap<WhichModel, bool> {
    let selected_model_kind = { *whisper_state.selected_model_kind.lock().unwrap() };
    let cache = hf_hub::Cache::default();
    let mut model_is_downloaded = whisper_state.model_is_downloaded.lock().unwrap();
    for model_kind in whisper_get_model_kinds() {
        let downlaoded = cache
            .model(model_kind.model_and_revision().0.to_string())
            .get("pytorch_model.bin")
            .is_some();
        (*model_is_downloaded).insert(model_kind, downlaoded);
    }
    *whisper_state
        .current_model_downloading_progress
        .lock()
        .unwrap() = if cache
        .model(selected_model_kind.model_and_revision().0.to_string())
        .get("pytorch_model.bin")
        .is_some()
    {
        1.0f32
    } else {
        0.0f32
    };

    model_is_downloaded.clone()
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
pub async fn whisper_run_tasks<'a>(
    tasks: Vec<WhisperTask>,
    whisper_state: tauri::State<'a, WhisperState>,
) -> Result<Vec<String>, ()> {
    let fake_endpoint = "https://hf-mirror.com".to_string();
    std::env::set_var("HF_ENDPOINT", &fake_endpoint);
    let mut model = whisper_state.helper.lock().unwrap();
    let current_selected_model_kind = { *whisper_state.selected_model_kind.lock().unwrap() };
    if model.is_none() || model.as_ref().unwrap().kind != current_selected_model_kind {
        let device = WgpuDevice::BestAvailable;
        *model = Some(WhisperHelper::new(current_selected_model_kind, &device));
        whisper_state
            .model_is_loaded
            .store(true, std::sync::atomic::Ordering::SeqCst);
        
            *whisper_state
                .current_model_downloading_progress
                .lock()
                .unwrap() = 1.0f32;
        
        *whisper_state.model_in_memory.lock().unwrap() = Some(current_selected_model_kind);

    }
    println!("tasks = : {tasks:#?}");
    let total_tasks = tasks.len();
    let device = WgpuDevice::BestAvailable;
    let res = tasks
        .iter()
        .map(|task| {
            let audio = load_audio_waveform_with_ffmpeg(&task.file_path).unwrap_or_default();
            let decoding_res = (model.as_ref())
                .unwrap()
                .run(&audio, 4, task.decode_option.clone(), &device);
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
    Ok(res)
}
