import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet } from '@angular/router';
import { invoke } from "@tauri-apps/api/tauri";

import { NzTabsModule } from 'ng-zorro-antd/tabs';

import { PwdGeneratorComponent } from './pwd-generator/pwd-generator.component';
import { VideoConvertComponent } from './video-convert/video-convert.component';
@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, RouterOutlet, PwdGeneratorComponent, VideoConvertComponent, NzTabsModule],
  templateUrl: './app.component.html',
  styleUrl: './app.component.css'
})
export class AppComponent {
  lotsOfTabs = new Array(1).fill(0).map((_, index) => `Tab ${index}`);
  greetingMessage = "";

  greet(event: SubmitEvent, name: string): void {
    event.preventDefault();

    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    invoke<string>("greet", { name }).then((text) => {
      this.greetingMessage = text;
    });
  }
}
