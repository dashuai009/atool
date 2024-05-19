mod input;

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use whisper::whisper_helper::{WhichModel, WhisperHelper};
use burn_wgpu::{Wgpu, WgpuDevice, AutoGraphicsApi};
use whisper::decoding::DecodingOptions;
use crate::whisper::input::filepath_to_mels;

#[derive(Default)]
pub struct WhisperState {
    pub helper: OnceLock<WhisperHelper<Wgpu<AutoGraphicsApi, f32, i32>>>,
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
pub async fn whisper_run_tasks(tasks: Vec<WhisperTask>, whisper_state: tauri::State<'_, WhisperState>) -> Result<Vec<String>, ()> {
    let model = whisper_state.helper.get_or_init(|| {
        let device = WgpuDevice::BestAvailable;
        WhisperHelper::new(WhichModel::Base, &device)
    });
    println!("tasks = : {tasks:#?}");
    let device = WgpuDevice::BestAvailable;
    let res = tasks.iter().map(|task| {
        let (mels, _) = filepath_to_mels(&task.file_path, &device);
        let decoding_res = model.run(mels, task.decode_option.clone());
        let mut res_text = String::new();
        for i in decoding_res{
            res_text += &*i.text;
        }
        res_text
    }).collect::<Vec<_>>();
    println!("res = {:#?}", res);
    return Ok(res);
}
