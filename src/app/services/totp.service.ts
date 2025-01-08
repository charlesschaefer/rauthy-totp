import { invoke } from '@tauri-apps/api/core';
import { Injectable } from '@angular/core';

import { Service } from '../models/service.model';
import { Observable, Subject } from 'rxjs';
import { TotpToken } from '../models/token.model';

@Injectable({
    providedIn: 'root',
})
export class TotpService {
    servicesContent: Map<string, Service> = new Map<string, Service>();
    services: Subject<typeof this.servicesContent> = new Subject<typeof this.servicesContent>();

    tokensContent: Map<string, TotpToken> = new Map<string, TotpToken>();
    tokens: Subject<typeof this.tokensContent> = new Subject<typeof this.tokensContent>();

    private setupServices(services: object) {
        this.servicesContent = new Map(Object.entries(services));
    }

    setupStorageKeys(password: string): Observable<Map<string, Service>> {
        invoke<object>('setup_storage_keys', { userPass: password }).then(services => {
            this.setupServices(services);
            this.services.next(this.servicesContent);
        }).catch(error => {
            this.services.error(error);
            this.services.complete();
            this.services = new Subject<typeof this.servicesContent>();
        });
        return this.services.asObservable();
    }

    addService(totpUri: string): Observable<Map<string, Service>> {
        invoke<object>('add_service', { totpUri }).then(services => {
            this.setupServices(services);
            this.services.next(this.servicesContent);
        }).catch(error => {
            this.services.error(error);
            this.services.complete();
            this.services = new Subject<typeof this.servicesContent>();
        });
        return this.services.asObservable();
    }

    removeService(serviceId: string): Observable<Map<string, Service>> {
        invoke<object>('remove_service', { serviceId }).then(services => {
            this.setupServices(services);
            this.services.next(this.servicesContent);
        }).catch(error => {
            this.services.error(error);
            this.services.complete();
            this.services = new Subject<typeof this.servicesContent>();
        });
        return this.services.asObservable();
    }

    getServicesTokens(): Observable<Map<string, TotpToken>> {
        invoke<Record<string, {token: string, next_step_time: number}>>('get_services_tokens').then(tokens => {
            this.tokensContent = new Map<string, TotpToken>();
            Object.entries(tokens).forEach((token) => {
                const tokenData = {
                    token: token[1].token,
                    nextStepTime: new Date(token[1].next_step_time * 1000)
                } as TotpToken;
                this.tokensContent.set(token[0], tokenData);
            })
            
            this.tokens.next(this.tokensContent);
        });
        return this.tokens;
    }
} 
