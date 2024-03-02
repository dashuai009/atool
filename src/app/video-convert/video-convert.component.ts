import {Component} from '@angular/core';
import {CommonModule} from '@angular/common';
import {FormBuilder, FormsModule} from "@angular/forms";
import {NzUploadModule} from 'ng-zorro-antd/upload';
import {NzModalModule} from 'ng-zorro-antd/modal';
import {NzIconDirective} from "ng-zorro-antd/icon";
import {convertFileSrc} from '@tauri-apps/api/tauri'
import {open} from '@tauri-apps/api/dialog';
import {NzImageModule} from 'ng-zorro-antd/image';
import {NzButtonModule} from 'ng-zorro-antd/button';
import {NzSelectModule} from 'ng-zorro-antd/select';
import {NzInputDirective, NzInputGroupComponent} from "ng-zorro-antd/input";
import {NzMessageModule, NzMessageService} from 'ng-zorro-antd/message';
import {invoke} from "@tauri-apps/api";

export enum AtoolCodecType {
    X264 = 'X264',
    X265 = "X265",
    AV1 = "AV1",
    VP8 = "VP8",
    VP9 = "VP9",
}

@Component({
    selector: 'app-video-convert',
    standalone: true,
    imports: [CommonModule, NzMessageModule, NzUploadModule, NzModalModule, NzIconDirective, NzImageModule, NzButtonModule, NzSelectModule, FormsModule, NzInputDirective, NzInputGroupComponent],
    templateUrl: './video-convert.component.html',
    styleUrl: './video-convert.component.css'
})
export class VideoConvertComponent {
    inputFiles: {
        assetPath: string,
        localPath: string
    }[] = [];
    codecTypes = Object.values(AtoolCodecType);
    selectedCodecType = AtoolCodecType.X265;

    outputDir = ""

    constructor(private message: NzMessageService) {
    }

    video_convert() {
        if (this.inputFiles.length == 0) {
            this.message.error("there is no file uploaded.")
            return;
        }
        if (this.outputDir.length == 0) {
            this.message.error("Please select output directory.")
            return;
        }
        let files = this.inputFiles.map((x)=>{
            return x.localPath
        });
        invoke('video_convert_cmd', {
            inputFile: files,
            toType: this.selectedCodecType,
            outputDir: this.outputDir,
            options: "",
        }).then(() => {
            console.log(`video convert down`)
        })
    }


    async handler() {
// Open a selection dialog for image files
        const selected = await open({
            multiple: true,
            filters: [{
                name: 'Video',
                extensions: ['mp4']
            }]
        });
        if (Array.isArray(selected)) {
            selected.forEach((f) => {
                this.inputFiles.push({
                    assetPath: convertFileSrc(f),
                    localPath: f
                })
            })
            // user selected multiple files
        } else if (selected === null) {
            // user cancelled the selection
        } else {
            this.inputFiles.push({
                assetPath: convertFileSrc(selected),
                localPath: selected
            })
            // user selected a single file
        }
    }

    async selectOutputDir() {
        const selected = await open({
            multiple: false,
            directory: true
        });
        if (Array.isArray(selected)) {
        } else if (selected === null) {
        } else {
            this.outputDir = selected;
            // user selected a single dir
        }
    }
}
