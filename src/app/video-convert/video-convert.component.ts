import { Component } from '@angular/core';
import { FormBuilder, FormsModule, ReactiveFormsModule } from '@angular/forms';
import {MatFormFieldModule} from '@angular/material/form-field';
import {MatSelectModule} from '@angular/material/select';
import {MatInputModule} from '@angular/material/input';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';


export enum AtoolCodecType {
  X264 = 'X64',
  X265 = "X265",
  AV1 = "AV1",
  VP8 = "VP8",
  VP9 = "VP9",
};

@Component({
  selector: 'app-video-convert',
  standalone: true,
  imports: [CommonModule, FormsModule, ReactiveFormsModule, MatFormFieldModule, MatSelectModule, MatInputModule, MatButtonModule],
  templateUrl: './video-convert.component.html',
  styleUrl: './video-convert.component.css'
})
export class VideoConvertComponent {
  codec_types = Object.values(AtoolCodecType);

  video_options = this._formBuilder.group({
    to_type: AtoolCodecType.X265,
    upperCase: false,
    digits: true,
    specialCharacters: true,
    passwordLength: 10
  });

  constructor(private _formBuilder: FormBuilder){}

  video_convert(){
    console.log(this.video_options.value)
  }
}
