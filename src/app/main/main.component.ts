import { Component, inject, OnInit, signal } from '@angular/core';
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
import { AvatarModule } from 'primeng/avatar';
import { RippleModule } from 'primeng/ripple';
import { AutoFocusModule } from 'primeng/autofocus';
import { ProgressSpinnerModule } from 'primeng/progressspinner';
import { NgxSwipeMenuComponent, SwipeMenuActions } from 'ngx-swipe-menu';

import { TotpService } from '../services/totp.service';
import { Service } from '../models/service.model';
import { TotpToken } from '../models/token.model';
import { invoke } from '@tauri-apps/api/core';
import { LocalStorageService } from '../services/local-storage.service';

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
        KnobModule,
        NgxSwipeMenuComponent,
        AvatarModule,
        RippleModule,
        AutoFocusModule,
        ProgressSpinnerModule
    ],
    providers: []
})
export class MainComponent implements OnInit {
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
    askForPasswordStorage = signal(false);
    loadingServices = signal(false);

    encryptedPassword = "";

    actionList = [
        {
            name: 'edit',
            label: 'Edit',
            class: '',
            data: 'treta',
            onClick(_event: any, data: any) {
                console.log("Editando o item, Dados: ", data);
            }
        }
    ] as SwipeMenuActions[];

    constructor(
        private totpService: TotpService,
        private translate: TranslocoService,
        private messageService: MessageService,
        private clipboard: Clipboard,
        private snackbar: MatSnackBar,
        private localStorage: LocalStorageService
    ) { }

    ngOnInit(): void {
        if (this.localStorage.hasItem('encryptedPassword')) {
            this.encryptedPassword = this.localStorage.getItem('encryptedPassword') as string;
            this.fetchWithoutPassword();
        }
    }

    async onSubmit(internal: boolean = false) {
        if (this.form.valid || internal) {
            this.loadingServices.set(true);
            const subscription = this.totpService.setupStorageKeys(this.form.value.password as string).subscribe({
                next: services => {
                    this.loadingServices.set(false);
                    subscription.unsubscribe();
                    this.totpItems = services;
                    if (services.size === 0) {
                        this.showDialog.set(true);
                    } else {
                        this.showTokens();
                    }
                    
                    if (!this.localStorage.hasItem('encryptedPassword')) {
                        this.askForPasswordStorage.set(true);
                    }
                },
                error: error => {
                    this.loadingServices.set(false);
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

    async storePasswordWithBiometrics(event: Event) {
        const password = this.form.value.password;
        const options = {
            // Set true if you want the user to be able to authenticate using phone password
            allowDeviceCredential: false,
            cancelTitle: "You won't be able to login without password",

            // Android only features
            title: 'Login withouth password',
            subtitle: 'Next times you will be able to login using your biometrics authentication',        
        };
        const encryptedData = await invoke<{data: string}>('plugin:biometric|biometric_cipher', {
            reason: "Next time you will be able to login with your biometrics",
            ...options,
            dataToEncrypt: password
        });

        console.log("Encrypted Data: ", encryptedData);
        this.encryptedPassword = encryptedData.data;
        this.localStorage.setItem("encryptedPassword", this.encryptedPassword);
        this.askForPasswordStorage.set(false);
    }

    async fetchWithoutPassword() {
        console.log("Entrou no fetchWithoutPassword");
        const encryptedData = this.encryptedPassword;
        
        const options = {
            // Set true if you want the user to be able to authenticate using phone password
            allowDeviceCredential: false,
            cancelTitle: "Cancel and type password",

            // Android only features
            title: 'Open services without password',
            subtitle: ''
        };

        const originalPass = await invoke<{data: string}>('plugin:biometric|biometric_cipher', {
            reason: "Open service files without password",
            ...options,
            dataToDecrypt: encryptedData
        });

        console.log("Original pass: ", originalPass.data);

        this.form.patchValue({"password": originalPass.data});
        this.onSubmit(true);
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
