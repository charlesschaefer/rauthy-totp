import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet } from '@angular/router';
import { TranslocoService } from '@jsverse/transloco';
import { MessageService } from 'primeng/api';
import { ToolbarModule } from 'primeng/toolbar';
import { MatListModule } from '@angular/material/list';
import { VersionComponent } from './version/version.component';
import { LocalStorageService } from './services/local-storage.service';

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
        TranslocoService,
        LocalStorageService,
    ]
})
export class AppComponent  {
   
}
