import { invoke } from '@tauri-apps/api/core';
import { Injectable } from '@angular/core';

import { Service } from '../models/service.model';
import { Observable, Subject } from 'rxjs';

@Injectable({
    providedIn: 'root',
})
export class TotpService {
    servicesContent: Map<string, Service> = new Map<string, Service>();
    services: Subject<Map<string, Service>> = new Subject<Map<string, Service>>();

    private setupServices(services: object) {
        this.servicesContent = new Map(Object.entries(services));
    }

    setupStorageKeys(password: string): Observable<Map<string, Service>> {
        invoke<object>('setup_storage_keys', { userPass: password }).then(services => {
            this.setupServices(services);
            this.services.next(this.servicesContent);
        }).catch(error => {
            this.services.error(error);
        });
        return this.services.asObservable();
    }

    addService(totpUri: string): Observable<Map<string, Service>> {
        invoke<object>('add_service', { totpUri }).then(services => {
            this.setupServices(services);
            this.services.next(this.servicesContent);
        }).catch(error => {
            this.services.error(error);
        });
        return this.services.asObservable();
    }

    removeService(serviceId: string): Observable<Map<string, Service>> {
        invoke<object>('remove_service', { serviceId }).then(services => {
            this.setupServices(services);
            this.services.next(this.servicesContent);
        }).catch(error => {
            this.services.error(error);
        });
        return this.services.asObservable();
    }
} 
