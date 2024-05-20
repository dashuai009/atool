import { Component } from '@angular/core';
import { FormBuilder, FormsModule, ReactiveFormsModule } from '@angular/forms';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatTooltipModule } from '@angular/material/tooltip';
import { invoke } from '@tauri-apps/api';
import { writeText } from '@tauri-apps/api/clipboard';

@Component({
  selector: 'app-pwd-generator',
  standalone: true,
  imports: [FormsModule, ReactiveFormsModule, MatCheckboxModule, MatButtonModule, MatFormFieldModule, MatIconModule, MatTooltipModule],
  templateUrl: './pwd-generator.component.html',
  styleUrl: './pwd-generator.component.css'
})
export class PwdGeneratorComponent {

  toppings = this._formBuilder.group({
    lowerCase: true,
    upperCase: false,
    digits: true,
    specialCharacters: true,
    passwordLength: 10
  });

  new_pwd = "empty";

  constructor(private _formBuilder: FormBuilder) { }

  generate_pwd() {
    let flag = 0;
    let val = this.toppings.value;
    if (val.lowerCase) {
      flag |= (1 << 0);
    }
    if (val.upperCase) {
      flag |= (1 << 1);
    }
    if (val.digits) {
      flag |= (1 << 2);
    }
    if (val.specialCharacters) {
      flag |= (1 << 3);
    }
    console.log(`val = ${val} , flag = ${flag}`)

    invoke('gen_pwd_cmd', { flag: flag, pwdLen: val.passwordLength })
      // `invoke` returns a Promise
      .then((response) => {
        this.new_pwd = response as string
        console.log(response)
      })

  }

  copy_pwd() {
    writeText(this.new_pwd)
  }
}
