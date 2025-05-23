import { Component, inject, OnInit, signal } from '@angular/core';
import { FormBuilder, FormsModule, ReactiveFormsModule, Validators } from '@angular/forms';
import { DialogModule } from 'primeng/dialog';
import { ButtonModule } from 'primeng/button';
import { CardModule } from 'primeng/card';
import { InputTextModule } from 'primeng/inputtext';
import { ToastModule } from 'primeng/toast';
import { scan, Format, checkPermissions, requestPermissions, openAppSettings } from '@tauri-apps/plugin-barcode-scanner';
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
import { ServiceAddComponent } from './service-add/service-add.component';
import { ServiceEditComponent } from './service-edit/service-edit.component';
import { checkStatus } from '@tauri-apps/plugin-biometric';

import { TotpService } from '../services/totp.service';
import { Service } from '../models/service.model';
import { TotpToken } from '../models/token.model';
import { invoke } from '@tauri-apps/api/core';
import { LocalStorageService } from '../services/local-storage.service';
import { ServiceListComponent } from './service-list/service-list.component';
import { isMobile } from '../utils/platform';
import { ServiceDeleteComponent } from "./service-delete/service-delete.component";

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
        AvatarModule,
        RippleModule,
        AutoFocusModule,
        ProgressSpinnerModule,
        ServiceAddComponent,
        ServiceListComponent,
        ServiceEditComponent,
        ServiceDeleteComponent
    ],
    providers: []
})
export class MainComponent implements OnInit {
    private fb = inject(FormBuilder);
    form = this.fb.group({
        password: ['', Validators.required],
    });

    totpItems = signal(new Map<string, Service>());
    tokensMap = new Map<string, TotpToken>();
    tokensDuration = new Map<string, number>();

    showDialog = signal(false);
    askForPasswordStorage = signal(false);
    loadingServices = signal(false);
    showEditDialog = signal(false);
    showDeleteDialog = signal(false);

    selectedService?: Service;
    serviceToDelete?: Service;
    isBiometricAble = false;
    isMobile = isMobile();

    encryptedPassword = "";

    constructor(
        private totpService: TotpService,
        private translate: TranslocoService,
        private messageService: MessageService,
        private clipboard: Clipboard,
        private snackbar: MatSnackBar,
        private localStorage: LocalStorageService
    ) { }

    async ngOnInit() {
        const hasBiometrics = await checkStatus();
        if (hasBiometrics.isAvailable) {
            this.isBiometricAble = true;
            if (this.localStorage.hasItem('encryptedPassword')) {
                this.loadingServices.set(true);
                this.encryptedPassword = this.localStorage.getItem('encryptedPassword') as string;
                this.fetchWithoutPassword();
            }
        } else {
            this.isBiometricAble = false;
        }


        // Testing QRCode GUI
        //this.scanQRCode(null);
    }

    async onSubmit(internal: boolean = false) {
        if (this.form.valid || internal) {
            this.loadingServices.set(true);
            const subscription = this.totpService.setupStorageKeys(this.form.value.password as string).subscribe({
                next: services => {
                    this.loadingServices.set(false);
                    subscription.unsubscribe();
                    this.totpItems.set(services);
                    if (services.size === 0) {
                        this.showDialog.set(true);
                    } else {
                        this.showTokens();
                    }
                    
                    // Shows only on mobiles with biometrics activated.
                    if (this.isMobile && this.isBiometricAble && !this.localStorage.hasItem('encryptedPassword')) {
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

    async onSubmitServiceUrl(serviceUrl: string) {
        this.addNewService(serviceUrl);
    }

    async scanQRCode(_event: any) {

        const authorized = await checkPermissions();
        if (authorized != "granted") {
            const requested = await requestPermissions();
            if (requested != "granted") {
                await openAppSettings();
            }
        }


        try {
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
                this.showDialog.set(true);
                return;
            }
            this.addNewService(scanned.content);
        } catch(err) {
            this.messageService.add({
                summary: this.translate.translate("Camera access denied"),
                detail: this.translate.translate("Couldn't open the device camera. You'll need to authorize the app manually") + err,
                severity: 'error',
            })
        }
    }

    addNewService(url: string) {
        const subscription = this.totpService.addService(url).subscribe(services => {
            const oldItemsCount = this.totpItems().size;
            this.totpItems.set(services);
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
        
        // closes the dialog
        this.askForPasswordStorage.set(false);

        this.encryptedPassword = encryptedData.data;
        this.localStorage.setItem("encryptedPassword", this.encryptedPassword);
        this.askForPasswordStorage.set(false);
    }

    fetchWithoutPassword() {
        this.loadingServices.set(true);

        const options = {
            // Set true if you want the user to be able to authenticate using phone password
            allowDeviceCredential: false,
            cancelTitle: "Cancel and type password",

            // Android only features
            title: 'Open services without password',
            subtitle: '',
            reason: "Open service files without password",
        };

        const subscription = this.totpService.fetchServicesWithoutPassword(this.encryptedPassword, options).subscribe({
            next: (services) => {
                subscription.unsubscribe();
                this.loadingServices.set(false);
                
                this.totpItems.set(services);
    
                if (services.size === 0) {
                    this.showDialog.set(true);
                } else {
                    this.showTokens();
                }
            },
            error: (error) => {
                this.loadingServices.set(false);
                subscription.unsubscribe();
                this.messageService.add({
                    summary: this.translate.translate("Error trying to open the services file"),
                    detail: this.translate.translate("Couldn't open the services file: ") + error,
                    severity: 'error',
                })
            }
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

    onServiceEdit(event: {id: string, name: string, issuer: string}) {
        const service = this.totpItems().get(event.id);
        if (service) {
            service.name = event.name;
            service.issuer = event.issuer;
            // Update the service in storage
            const subscription = this.totpService.updateService(service).subscribe({
                next: () => {
                    this.messageService.add({
                        severity: 'success',
                        summary: this.translate.translate('Service Updated'),
                        detail: this.translate.translate('Service updated successfully!')
                    });
                },
                error: (error) => {
                    this.messageService.add({
                        severity: 'error',
                        summary: this.translate.translate('Update Error'),
                        detail: this.translate.translate('Could not update service: ') + error
                    });
                }
            });
        }
    }

    editService(service: Service) {
        this.selectedService = service;
        this.showEditDialog.set(true);
    }

    deleteService(service: Service) {
        this.serviceToDelete = service;
        this.showDeleteDialog.set(true);
    }

    confirmDeleteService() {
        if (this.serviceToDelete) {
            const subscription = this.totpService.deleteService(this.serviceToDelete.id).subscribe({
                next: () => {
                    this.serviceToDelete?.id ? this.totpItems().delete(this.serviceToDelete.id) : undefined;
                    this.messageService.add({
                        severity: 'success',
                        summary: this.translate.translate('Service Deleted'),
                        detail: this.translate.translate('Service deleted successfully!')
                    });
                    this.showDeleteDialog.set(false);
                    this.serviceToDelete = undefined;
                    this.showTokens();
                },
                error: (error) => {
                    this.messageService.add({
                        severity: 'error',
                        summary: this.translate.translate('Delete Error'),
                        detail: this.translate.translate('Could not delete service: ') + error
                    });
                }
            });
        }
    }

    cancelDeleteService() {
        this.showDeleteDialog.set(false);
        this.serviceToDelete = undefined;
    }

    hideAskForPassStorage() {
        this.askForPasswordStorage.set(false)
    }

    showEditDialogChange(value: boolean) {
        this.showEditDialog.set(value)
    }
}
