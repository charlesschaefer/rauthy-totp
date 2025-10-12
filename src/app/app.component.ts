import { Component, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet } from '@angular/router';
import { TranslocoService } from '@jsverse/transloco';
import { MessageService } from 'primeng/api';
import { ToolbarModule } from 'primeng/toolbar';
import { MatListModule } from '@angular/material/list';
import { DrawerModule } from 'primeng/drawer';
import { ButtonModule } from 'primeng/button';
import { MenuModule } from 'primeng/menu';
import { TooltipModule } from 'primeng/tooltip';
import { MenuItem } from 'primeng/api';
import { TranslocoModule } from '@jsverse/transloco';
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
        ToolbarModule,
        DrawerModule,
        ButtonModule,
        MenuModule,
        TooltipModule,
        TranslocoModule
    ],
    templateUrl: './app.component.html',
    styleUrl: './app.component.scss',
    providers: [
        MessageService,
        TranslocoService,
        LocalStorageService,
    ]
})
export class AppComponent {
    showDrawer = signal(false);
    isAuthenticated = signal(false);
    menuItems: MenuItem[] = [];

    constructor(private translate: TranslocoService) {
        this.initializeMenuItems();
        this.setupAuthenticationListener();
    }

    private initializeMenuItems() {
        this.menuItems = [
            {
                label: this.translate.translate('Export CSV'),
                icon: 'pi pi-download',
                command: () => this.emitMenuAction('export')
            },
            {
                label: this.translate.translate('Import CSV'),
                icon: 'pi pi-upload',
                command: () => this.emitMenuAction('import')
            },
            {
                label: this.translate.translate('Change Password'),
                icon: 'pi pi-key',
                command: () => this.emitMenuAction('changePassword')
            },
            {
                separator: true
            },
            {
                label: this.translate.translate('Logout'),
                icon: 'pi pi-sign-out',
                command: () => this.emitMenuAction('logout')
            }
        ];
    }

    toggleDrawer() {
        this.showDrawer.set(!this.showDrawer());
    }

    onLockClick() {
        if (this.isAuthenticated()) {
            // Emit logout action to main component
            this.emitMenuAction('logout');
        }
    }

    private setupAuthenticationListener() {
        // Listen for authentication state changes from main component
        window.addEventListener('authenticationState', (event: any) => {
            this.isAuthenticated.set(event.detail.authenticated);
        });
    }

    private emitMenuAction(action: string) {
        // Emit custom event that main component can listen to
        const event = new CustomEvent('menuAction', { detail: { action } });
        window.dispatchEvent(event);
        this.showDrawer.set(false);
    }
}
