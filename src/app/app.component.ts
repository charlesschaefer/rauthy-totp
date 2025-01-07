import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet } from '@angular/router';
import { invoke } from "@tauri-apps/api/core";
import { TranslocoService } from '@jsverse/transloco';
import { MessageService } from 'primeng/api';

@Component({
    selector: 'app-root',
    imports: [CommonModule, RouterOutlet],
    templateUrl: './app.component.html',
    styleUrl: './app.component.scss',
    providers: [
        MessageService,
        TranslocoService
    ]
})
export class AppComponent {
    greetingMessage = "";

    greet(event: SubmitEvent, name: string): void {
        event.preventDefault();

        // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
        invoke<string>("greet", { name }).then((text) => {
            this.greetingMessage = text;
        });
    }
}
