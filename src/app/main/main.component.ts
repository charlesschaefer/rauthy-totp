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
import { interval, Subscription } from 'rxjs';
import { MatListModule } from '@angular/material/list';
import { MatSnackBar } from '@angular/material/snack-bar';
import { Clipboard } from '@angular/cdk/clipboard';
import { KnobModule } from 'primeng/knob';
import { DateTime } from 'luxon';

import { TotpService } from '../services/totp.service';
import { Service } from '../models/service.model';
import { TotpToken } from '../models/token.model';

@Component({
    selector: 'app-main',
    standalone: true,
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
        ToastModule,
        MatListModule,
        KnobModule
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
        private clipboard: Clipboard,
        private snackbar: MatSnackBar
    ) { }

    async onSubmit() {
        if (this.form.valid) {
            const subscription = this.totpService.setupStorageKeys(this.form.value.password as string).subscribe({
                next: services => {
                    subscription.unsubscribe();
                    this.totpItems = services;
                    if (services.size === 0) {
                        this.showDialog.set(true);
                    } else {
                        this.showTokens();
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
            windowed: !true, 
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
            this.showTokens();
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
        const subscription = this.totpService.getServicesTokens().subscribe(tokensMap => {
            subscription.unsubscribe();
            this.tokensMap = tokensMap;
            this.calculateTokenDuration(null);
            const intervalSubscription = interval(1000).subscribe(() => {
                this.calculateTokenDuration(intervalSubscription);
            });
        });
    }

    copyToken(token: string) {
        this.clipboard.copy(token);
        this.snackbar.open(this.translate.translate("Token copied to clipboard"), "", {
            duration: 4000
        });
    }

    private calculateTokenDuration(intervalSubscription: Subscription | null) {
        let minDuration = Infinity;
        const durations = new Map<string, number>();
        this.tokensMap.forEach((token, key) => {
            const duration = Math.round(DateTime.fromJSDate(token.nextStepTime).diffNow('seconds').as('seconds'));
            minDuration = Math.min(minDuration, duration);
            durations.set(key, duration);
        });
        this.tokensDuration = durations;
        if (minDuration < 0) {
            this.showTokens();
            intervalSubscription?.unsubscribe();
        }
    }
} 
