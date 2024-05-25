import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatSelectModule } from '@angular/material/select';
import { MatInputModule } from '@angular/material/input';
import { FormsModule } from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { open } from '@tauri-apps/api/dialog';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import { invoke } from '@tauri-apps/api'
import { MatTooltip } from "@angular/material/tooltip";
import { MatProgressBarModule, ProgressBarMode } from '@angular/material/progress-bar';
import { MatCardModule } from '@angular/material/card';
import { MatSnackBar } from '@angular/material/snack-bar';
import { writeText } from '@tauri-apps/api/clipboard';

const AudioExtensions: string[] = ['m4a', "mp3", "wma", "ogg", "aac", "wav"];
const VideoExtensions: string[] = ["mp4", "mkv", "mov", "m4v", "avi", "flv"];

@Component({
  selector: 'app-whisper',
  standalone: true,
  imports: [MatButtonModule, MatIconModule, CommonModule, MatFormFieldModule, MatInputModule, FormsModule, MatSelectModule, MatTooltip, MatProgressBarModule, MatCardModule],
  templateUrl: './whisper.component.html',
  styleUrl: './whisper.component.css'
})
export class WhisperComponent {
  selected_model_kind: String = "Base";
  all_model_kind: String[] = [];
  model_is_downloaded = new Map<String, boolean>();
  selected_files: { local_path: String, url: String }[] = [];
  tasks: {
    file_path: String,
    decode_option: {
      task: String,
      language: String | undefined,
      temperature: number,
      smaple_len: number | undefined,
      best_of: number | undefined,
      beam_size: number | undefined,
      patience: number | undefined,

      length_penalty: number | undefined,
      prompt: number | undefined,
      prefix: number | undefined,

      suppress_tokens: {
        Text: String | undefined,
        Tokens: Set<number> | undefined
      },
      suppress_blank: boolean,
      without_timestamps: boolean,
      max_initial_timestamp: number | undefined,
      fp16: boolean
    }
  }[] = []
  decoding_res: String[] = []
  run_disabled: boolean = false;
  unload_disabled: boolean = true;

  get_model_is_downloaded(option: String) {
    // @ts-ignore
    return this.model_is_downloaded[option];
  }
  progress_visiable = false
  progress_value = 50
  // indeterminate for downloading model. determinate for running tasks.
  progress_mode: ProgressBarMode = 'indeterminate';

  constructor(private _snackBar: MatSnackBar) {
    invoke('whisper_get_model_kinds')
      // `invoke` returns a Promise
      .then((response) => {
        // @ts-ignore
        this.all_model_kind = response;
      })
    invoke('whisper_update_model_is_downloaded')
      .then((response) => {
        // @ts-ignore
        this.model_is_downloaded = response;
      })
  }


  // unload model from memory
  UnloadModel() {
    invoke('whisper_unload_model');
    this.unload_disabled = true;
  }

  ChangeModelEvent() {
    invoke('whisper_set_selected_model_kind', { modelKind: this.selected_model_kind });
  }

  RunTasks() {
    this.run_disabled = true;
    this.progress_visiable = true;

    let check_state_func = () => {
      invoke('whisper_update_model_is_downloaded')
        .then((response) => {
          // @ts-ignore
          this.model_is_downloaded = response;
          // @ts-ignore
          if (this.model_is_downloaded[this.selected_model_kind]) {
            this.progress_mode = 'determinate';
          }
        })
      invoke('whisper_get_model_is_loaded')
        .then((response) => {
          // @ts-ignore
          this.unload_disabled = !response;
        })
      invoke('whisper_get_model_in_memory')
        .then((response) => {
          // @ts-ignore
          this.unload_disabled = !response
        })
    };

    // run intermidiatly
    check_state_func();
    let check_state = setInterval(check_state_func, 2000);
    invoke('whisper_run_tasks', { tasks: this.tasks })
      .then((response) => {
        // @ts-ignore
        this.decoding_res = response;
        this.run_disabled = false;
        this.progress_visiable = false;
        this._snackBar.open("Successfully run tasks.!! ", "Ok");
        clearInterval(check_state);
      })

  }
  async SelectFile() {
    // Open a selection dialog for image files
    let selected = await open({
      multiple: true,
      filters: [{
        name: 'Media File',
        extensions: [...AudioExtensions, ...VideoExtensions]
      }]
    });

    let all_files: string[] = [];
    if (Array.isArray(selected)) {
      for (let s of selected) {
        all_files.push(s)
      }
    } else if (selected === null) {
      // user cancelled the selection
    } else {
      // user selected a single file
      all_files.push(selected)
    }
    // user selected multiple files
    for (let file of all_files) {
      this.selected_files.push({
        local_path: file,
        url: convertFileSrc(file)
      })
      this.tasks.push({
        file_path: file,
        decode_option: { // default value
          task: "Transcribe",
          language: undefined,
          temperature: 0,
          smaple_len: undefined,
          best_of: undefined,
          beam_size: undefined,
          patience: undefined,
          length_penalty: undefined,
          prompt: undefined,
          prefix: undefined,
          suppress_tokens: {
            Text: "-1",
            Tokens: undefined
          },
          suppress_blank: true,
          without_timestamps: false,
          max_initial_timestamp: undefined,
          fp16: true
        }
      })
      this.decoding_res.push("")
    }
  }

  IsAudio(file_path: String) {
    return file_path.slice((file_path.lastIndexOf(".") - 1 >>> 0) + 2) in AudioExtensions;
  }

  copy_pwd(text: String){
    writeText(text as string)
  }

}
