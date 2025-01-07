import { Component, inject, signal } from '@angular/core';
import { FormBuilder, FormsModule, ReactiveFormsModule, Validators } from '@angular/forms';
import { DialogModule } from 'primeng/dialog';
import { ButtonModule } from 'primeng/button';
import { CardModule } from 'primeng/card';
import { InputTextModule } from 'primeng/inputtext';
import { ToastModule } from 'primeng/toast';
import { scan, Format } from '@tauri-apps/plugin-barcode-scanner';
import { TranslocoModule, TranslocoService } from '@jsverse/transloco';
import { MessageService } from 'primeng/api';
import { CommonModule } from '@angular/common';
import { interval } from 'rxjs';
import { DateTime } from 'luxon';

import { TotpService } from '../services/totp.service';
import { Service } from '../models/service.model';
import { TotpToken } from '../models/token.model';

@Component({
    selector: 'app-main',
    templateUrl: './main.component.html',
    styleUrls: ['./main.component.scss'],
    imports: [
        CommonModule,
        ReactiveFormsModule,
        FormsModule,
        DialogModule,
        ButtonModule,
        CardModule,
        InputTextModule,
        TranslocoModule,
        ToastModule
    ]
})
export class MainComponent {
    private fb = inject(FormBuilder);
    form = this.fb.group({
        password: ['', Validators.required],
    });
    urlInput = this.fb.group({
        serviceUrl: ['', Validators.required],
    });

    totpItems = new Map<string, Service>();
    tokensMap = new Map<string, TotpToken>();
    tokensDuration = new Map<string, number>();

    showDialog = signal(false);
    showURLInput = signal(false);

    constructor(
        private totpService: TotpService,
        private translate: TranslocoService,
        private messageService: MessageService,
    ) { }

    async onSubmit() {
        if (this.form.valid) {
            const subscription = this.totpService.setupStorageKeys(this.form.value.password as string).subscribe({
                next: services => {
                    subscription.unsubscribe();
                    this.totpItems = services;
                    if (services.size === 0) {
                        this.showDialog.set(true);
                    }
                },
                error: error => {
                    subscription.unsubscribe();
                    this.messageService.add({
                        summary: this.translate.translate("Error trying to open the services file"),
                        detail: this.translate.translate("Couldn't open the services file: ") + error,
                        severity: 'error',
                    })
                }
            })
        }
    }

    async onSubmitServiceUrl() {
        this.addNewService(this.urlInput.value.serviceUrl as string);
    }

    async scanQRCode(_event: any) {
        const scanned = await scan({ 
            windowed: true, 
            formats: [Format.QRCode]
        });
        if (!scanned.content) {
            this.messageService.add({
                summary: this.translate.translate("QRCode Error"),
                detail: this.translate.translate("QRCode scanning returned no content!"),
                severity: 'error',

            });
            return;
        }
        this.addNewService(scanned.content);
    }

    addNewService(url: string) {
        const subscription = this.totpService.addService(url).subscribe(services => {
            const oldItemsCount = this.totpItems.size;
            this.totpItems = services;
            if (services.size <= oldItemsCount) {
                this.showDialog.set(true);
                this.messageService.add({
                    summary: this.translate.translate("Service format Error"),
                    detail: this.translate.translate("Couldn't add this service!"),
                    severity: 'error',

                });
            } else {
                this.showDialog.set(false);
                this.messageService.add({
                    summary: this.translate.translate("Service Added"),
                    detail: this.translate.translate("Service added successfully!"),
                    severity: 'success',
                });
            }
            subscription.unsubscribe();
        });
    }

    showTokens() {
        this.totpService.getServicesTokens().subscribe(tokensMap => {
            this.tokensMap = tokensMap
            interval(1000).subscribe(() => {
                this.tokensMap.forEach((token, key) => {
                    this.tokensDuration.set(key, 
                        Math.round(DateTime.fromJSDate(token.nextStepTime).diffNow('seconds').as('seconds'))
                    );
                })
            })
        });
    }
} 
