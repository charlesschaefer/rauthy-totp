import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet } from '@angular/router';
import { invoke } from "@tauri-apps/api/core";
import { TranslocoService } from '@jsverse/transloco';
import { MessageService } from 'primeng/api';
import { ToolbarModule } from 'primeng/toolbar';
import { MatListModule } from '@angular/material/list';
import { VersionComponent } from './version/version.component';

@Component({
    selector: 'app-root',
    standalone: true,
    imports: [
        CommonModule, 
        RouterOutlet,
        MatListModule,
        VersionComponent,
        ToolbarModule
    ],
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
