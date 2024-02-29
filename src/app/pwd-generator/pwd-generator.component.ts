import {Component} from '@angular/core';
import {FormsModule, ReactiveFormsModule} from '@angular/forms';
import {NzCheckboxModule} from 'ng-zorro-antd/checkbox';
import {NzButtonModule} from 'ng-zorro-antd/button';
import {NzFormModule} from 'ng-zorro-antd/form';
import {NzInputModule} from 'ng-zorro-antd/input';
import {invoke} from '@tauri-apps/api'
import {writeText} from '@tauri-apps/api/clipboard';
import {NzInputNumberModule} from 'ng-zorro-antd/input-number';
import {NzSpaceComponent, NzSpaceItemDirective} from "ng-zorro-antd/space";
import {NzIconDirective} from "ng-zorro-antd/icon";
import {NzMessageModule, NzMessageService} from 'ng-zorro-antd/message';


@Component({
    selector: 'app-pwd-generator',
    standalone: true,
    imports: [FormsModule, NzMessageModule, ReactiveFormsModule, NzCheckboxModule, NzButtonModule, NzFormModule, NzInputModule, NzInputNumberModule, NzSpaceItemDirective, NzSpaceComponent, NzIconDirective],
    templateUrl: './pwd-generator.component.html',
    styleUrl: './pwd-generator.component.css'
})
export class PwdGeneratorComponent {
    allChecked = false;
    indeterminate = true;
    checkOptionsOne = [
        {label: 'Lower Case', value: (1 << 0), checked: true},
        {label: 'Upper Case', value: (1 << 1), checked: false},
        {label: 'Digits', value: (1 << 2), checked: true},
        {label: 'Special Chars', value: (1 << 3), checked: true}
    ];

    passwordLength = 16;

    newPwd = "empty";

    constructor(private message: NzMessageService) {
    }

    generate_pwd() {
        let flag = 0;
        this.checkOptionsOne.map(item => {
            if (item.checked) {
                flag |= item.value;
            }
        });
        console.log(`val = ${this.checkOptionsOne} , flag = ${flag}`)

        invoke('gen_pwd_cmd', {flag: flag, pwdLen: this.passwordLength})
            // `invoke` returns a Promise
            .then((response) => {
                this.newPwd = response as string
                console.log(response)
            })

    }

    copy_pwd() {
        writeText(this.newPwd).then(r => {
            this.message.success(`Password ${this.newPwd} has been copy to clipboard.`);
        });
    }


    updateAllChecked(): void {
        this.indeterminate = false;
        if (this.allChecked) {
            this.checkOptionsOne = this.checkOptionsOne.map(item => ({
                ...item,
                checked: true
            }));
        } else {
            this.checkOptionsOne = this.checkOptionsOne.map(item => ({
                ...item,
                checked: false
            }));
        }
    }

    updateSingleChecked(): void {
        if (this.checkOptionsOne.every(item => !item.checked)) {
            this.allChecked = false;
            this.indeterminate = false;
        } else if (this.checkOptionsOne.every(item => item.checked)) {
            this.allChecked = true;
            this.indeterminate = false;
        } else {
            this.indeterminate = true;
        }
    }
}
