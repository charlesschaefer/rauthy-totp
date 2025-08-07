import { Component, EventEmitter, Input, Output, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslocoModule, TranslocoService } from '@jsverse/transloco';
import { ButtonModule } from 'primeng/button';
import { AvatarModule } from 'primeng/avatar';
import { KnobModule } from 'primeng/knob';
import { RippleModule } from 'primeng/ripple';
import { InputTextModule } from 'primeng/inputtext';
import { AutoFocusModule } from 'primeng/autofocus';
import { MenuModule } from 'primeng/menu';
import { FormsModule } from '@angular/forms';

import { TotpToken } from '../../models/token.model';
import { Service } from '../../models/service.model';
import { MenuItem } from 'primeng/api';
import { isMobile } from '../../utils/platform';

@Component({
    selector: 'app-service-item',
    imports: [
        CommonModule,
        FormsModule,
        TranslocoModule,
        ButtonModule,
        AvatarModule,
        KnobModule,
        RippleModule,
        InputTextModule,
        AutoFocusModule,
        MenuModule
    ],
    templateUrl: './service-item.component.html',
    styleUrl: './service-item.component.scss'
})
export class ServiceItemComponent {
    @Input() service!: Service;
    @Input() tokensMap = new Map<string, TotpToken>();
    @Input() tokensDuration = new Map<string, number>();
    @Output() copyToken = new EventEmitter<string>();
    @Output() editService = new EventEmitter<Service>();
    @Output() deleteService = new EventEmitter<Service>();
    @Output() itemChange = new EventEmitter<void>();

    isMobile = signal(isMobile());
    serviceMenuItems: MenuItem[] = [];

    constructor(private translate: TranslocoService) {
        this.serviceMenuItems = [
            {
                label: this.translate.translate(`Options`),
                items: [
                    {
                        label: this.translate.translate(`Delete`),
                        icon: "pi pi-trash",
                        command: () => {
                            this.deleteService.emit(this.service);
                        },
                    } as MenuItem,
                    {
                        label: this.translate.translate(`Edit`),
                        icon: "pi pi-pencil",
                        command: () => {
                            this.editService.emit(this.service);
                        },
                    } as MenuItem,
                ],
            } as MenuItem,
    ];
    }

    onImageError(event: any, service: Service) {
        console.error("Couldn't load service logo at: ", event.srcElement?.currentSrc);
        
        if (this.service.id == service.id) {
            this.service.icon = "";
        }

        this.itemChange.emit();
    }

}
