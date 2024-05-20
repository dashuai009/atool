mod input;

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use whisper::whisper_helper::{WhichModel, WhisperHelper};
use burn_wgpu::{Wgpu, WgpuDevice, AutoGraphicsApi};
use whisper::decoding::DecodingOptions;
use crate::whisper::input::filepath_to_mels;


#[derive(Default)]
pub struct WhisperState {
    pub helper: Mutex<Option<WhisperHelper<Wgpu<AutoGraphicsApi, f32, i32>>>>,
    pub model_kind: Mutex<WhichModel>
}

unsafe impl Send for WhisperState {}

unsafe impl Sync for WhisperState {}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhisperTask
{
    file_path: String,
    decode_option: DecodingOptions,
}

#[tauri::command]
pub fn whisper_change_model(model_kind:WhichModel, whisper_state:tauri::State<'_, WhisperState>){
    *whisper_state.helper.lock().unwrap() = None;// take();
    *whisper_state.model_kind.lock().unwrap() = model_kind;
}

#[tauri::command]
pub fn whisper_get_model_kinds() -> Vec<WhichModel>{
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
pub async fn whisper_run_tasks(tasks: Vec<WhisperTask>, whisper_state: tauri::State<'_, WhisperState>) -> Result<Vec<String>, ()> {
    let mut model = whisper_state.helper.lock().unwrap();
    if model.is_none(){
        let device = WgpuDevice::BestAvailable;
        *model = Some(WhisperHelper::new(*whisper_state.model_kind.lock().unwrap(), &device));
    }
    println!("tasks = : {tasks:#?}");
    let device = WgpuDevice::BestAvailable;
    let res = tasks.iter().map(|task| {
        let (mels, _) = filepath_to_mels(&task.file_path, &device);
        let decoding_res = (model.as_ref()).unwrap().run(mels, task.decode_option.clone());
        let mut res_text = String::new();
        for i in decoding_res{
            res_text += &*i.text;
        }
        res_text
    }).collect::<Vec<_>>();
    println!("res = {:#?}", res);
    return Ok(res);
}
