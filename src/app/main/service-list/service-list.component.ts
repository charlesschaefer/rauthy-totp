import { Component, computed, effect, EventEmitter, input, Input, Output, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslocoModule } from '@jsverse/transloco';
import { ButtonModule } from 'primeng/button';
import { AvatarModule } from 'primeng/avatar';
import { KnobModule } from 'primeng/knob';
import { RippleModule } from 'primeng/ripple';
import { InputTextModule } from 'primeng/inputtext';
import { AutoFocusModule } from 'primeng/autofocus';
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
        InputTextModule,
        AutoFocusModule,
        NgxSwipeMenuComponent,
    ],
    templateUrl: './service-list.component.html',
    styleUrl: './service-list.component.scss'
})
export class ServiceListComponent {
    // @Input() totpItems = new Map<string, Service>();
    totpItems = input<Map<string, Service>>(new Map<string, Service>());
    @Output() totpItemsChange = new EventEmitter<Map<string, Service>>();
    @Input() tokensMap = new Map<string, TotpToken>();
    @Input() tokensDuration = new Map<string, number>();
    @Output() addService = new EventEmitter<void>();
    @Output() copyToken = new EventEmitter<string>();
    @Output() editService = new EventEmitter<Service>();
    @Output() deleteService = new EventEmitter<Service>();

    itemList = computed(() => this.filter(this.totpItems().values()));//Array.from(this.totpItems.values());

    searchFilter = signal('');

    actionList = [
        {
            name: 'edit',
            label: 'Edit',
            class: '',
            onClick: (_event: any, data: any) => {
                this.editService.emit(data);
            }
        },
        {
            name: 'delete',
            label: 'Delete',
            class: '',
            onClick: (_event: any, data: any) => {
                this.deleteService.emit(data);
            }
        }
    ] as SwipeMenuActions[];

    constructor() {}

    onSwipeLeft(service: Service) {
        this.editService.emit(service);
    }

    onSwipeRight(service: Service) {
        this.deleteService.emit(service);
    }

    onImageError(event: any, service: Service) {
        console.error("Couldn't load service logo at: ", event.srcElement?.currentSrc);
        
        for (let item of this.totpItems()) {
            if (item[1].id == service.id) {
                item[1].icon = "";
                this.totpItems().set(item[0], item[1]);
                break;
            }
        }

        this.totpItemsChange.emit(this.totpItems());
    }

    private filter(items: IterableIterator<Service>) {
        return Array.from(items)
            .filter((service) => {
                return this.searchFilter() === "" ||
                    (service.issuer.includes(this.searchFilter()) || service.name.includes(this.searchFilter()));
            });
    }
}
