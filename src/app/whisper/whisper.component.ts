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

@Component({
  selector: 'app-whisper',
  standalone: true,
  imports: [MatButtonModule, MatIconModule, CommonModule, MatFormFieldModule, MatInputModule, FormsModule, MatSelectModule],
  templateUrl: './whisper.component.html',
  styleUrl: './whisper.component.css'
})
export class WhisperComponent {
  selected_files: { local_path: String, url: String }[] = [];
  tasks: {
    file_path: String,
    decode_option: {
      task: String
    }
  }[] = []

  constructor() {

  }


  RunTasks() {
    invoke('whisper_run_tasks', { tasks: this.tasks })
      // `invoke` returns a Promise
      .then((response) => {
        console.log(response)
      })

  }
  async SelectFile() {
    // Open a selection dialog for image files
    let selected = await open({
      multiple: true,
      filters: [{
        name: 'Media File',
        extensions: ['m4a', "mp3"]
      }]
    });

    let all_files: string[] = [];
    if (Array.isArray(selected)) {
      for(let s of selected){
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
        decode_option: {
          task: "Transcribe"
        }

      })
    }
    console.log("array selected: ", selected, "after = ", this.selected_files)
  }
}
