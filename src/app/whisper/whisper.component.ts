import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { open } from '@tauri-apps/api/dialog';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import { DomSanitizer, SafeResourceUrl } from '@angular/platform-browser';
import { invoke } from '@tauri-apps/api'

@Component({
  selector: 'app-whisper',
  standalone: true,
  imports: [MatButtonModule, MatIconModule, CommonModule],
  templateUrl: './whisper.component.html',
  styleUrl: './whisper.component.css'
})
export class WhisperComponent {
  selected_files: SafeResourceUrl[] = [];

  constructor(private sanitizer: DomSanitizer){

  }


  RunTasks(){
    let tasks = [];
    for(let i of this.selected_files){
      tasks.push({
        file_path: i,
        decode_option:{
          task: 'transcription'
        }
      })
    }
    invoke('whisper_run_tasks',{ tasks: tasks})
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
    console.log("aaas")

    if (Array.isArray(selected)) {
      // user selected multiple files
      for (let file of selected ) {
        this.selected_files.push(
          //file
          convertFileSrc(file)
         //this.sanitizer.bypassSecurityTrustResourceUrl(convertFileSrc(file))
        )
      }
      console.log("array selected: ", selected, "after = ", this.selected_files)
    } else if (selected === null) {
      // user cancelled the selection
    } else {

      this.selected_files.push(
        // selected
        convertFileSrc(selected)
        //this.sanitizer.bypassSecurityTrustResourceUrl(convertFileSrc(selected))
      )
      // user selected a single file
    }
  }
}
