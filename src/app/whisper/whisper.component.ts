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

@Component({
  selector: 'app-whisper',
  standalone: true,
  imports: [MatButtonModule, MatIconModule, CommonModule, MatFormFieldModule, MatInputModule, FormsModule, MatSelectModule, MatTooltip],
  templateUrl: './whisper.component.html',
  styleUrl: './whisper.component.css'
})
export class WhisperComponent {
  selected_model_kind: String = "Base";
  all_model_kind: String[] = [];
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

  constructor() {
    invoke('whisper_get_model_kinds')
      // `invoke` returns a Promise
      .then((response) => {
        // @ts-ignore
        this.all_model_kind = response;
      })
  }


  ChangeModel() {

  }

  ChangeModelEvent(event: Event) {
    let new_value = (event.target as HTMLSelectElement).value;
    if (new_value != this.selected_model_kind) {
      this.selected_model_kind = new_value
      this.ChangeModel();
    }
  }

  RunTasks() {
    invoke('whisper_run_tasks', { tasks: this.tasks })
      // `invoke` returns a Promise
      .then((response) => {
        // @ts-ignore
        this.decoding_res = response;
      })

  }
  async SelectFile() {
    // Open a selection dialog for image files
    let selected = await open({
      multiple: true,
      filters: [{
        name: 'Media File',
        extensions: ['m4a', "mp3", "wma", "ogg", "aac", "wav", "mp4", "mkv", "mov", "m4v", "avi", "flv"]
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
    console.log("array selected: ", selected, "after = ", this.selected_files)
  }
}
