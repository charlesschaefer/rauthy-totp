import { Component, EventEmitter, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslocoModule } from '@jsverse/transloco';
import { ButtonModule } from 'primeng/button';
import { AvatarModule } from 'primeng/avatar';
import { KnobModule } from 'primeng/knob';
import { RippleModule } from 'primeng/ripple';
import { NgxSwipeMenuComponent, SwipeMenuActions } from 'ngx-swipe-menu';

import { Service } from '../../models/service.model';
import { TotpToken } from '../../models/token.model';
import { FormsModule } from '@angular/forms';

@Component({
    selector: 'app-service-list',
    standalone: true,
    imports: [
        CommonModule,
        FormsModule,
        TranslocoModule,
        ButtonModule,
        AvatarModule,
        KnobModule,
        RippleModule,
        NgxSwipeMenuComponent
    ],
    templateUrl: './service-list.component.html',
    styleUrl: './service-list.component.scss'
})
export class ServiceListComponent {
    @Input() totpItems = new Map<string, Service>();
    @Input() tokensMap = new Map<string, TotpToken>();
    @Input() tokensDuration = new Map<string, number>();
    @Output() addService = new EventEmitter<void>();
    @Output() copyToken = new EventEmitter<string>();
    @Output() editService = new EventEmitter<Service>();
    @Output() deleteService = new EventEmitter<Service>();

    actionList = [
        {
            name: 'edit',
            label: 'Edit',
            class: '',
            //data: 'treta',
            onClick: (_event: any, data: any) => {
                this.editService.emit(data);
            }
        },
        {
            name: 'delete',
            label: 'Delete',
            class: '',
            data: 'treta',
            onClick(_event: any, data: any) {
                console.log("Removendo o item, Dados: ", data);
            }
        }
    ] as SwipeMenuActions[];

    onSwipeLeft(service: Service) {
        console.log("Swipe left")
        this.editService.emit(service);
    }

    onSwipeRight(service: Service) {
        console.log("Swipe right")
        this.deleteService.emit(service);
    }
}
