import { Component, EventEmitter, Input, Output, computed, inject, input, model, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { DialogModule } from 'primeng/dialog';
import { ButtonModule } from 'primeng/button';
import { InputTextModule } from 'primeng/inputtext';
import { TranslocoModule } from '@jsverse/transloco';
import { AvatarModule } from 'primeng/avatar';
import { ProgressSpinnerModule } from 'primeng/progressspinner';

import { ImageModule } from 'primeng/image';

import { Service } from '../../models/service.model';
import { invoke } from '@tauri-apps/api/core';

@Component({
    selector: 'app-service-edit',
    standalone: true,
    imports: [
        CommonModule,
        ReactiveFormsModule,
        DialogModule,
        ButtonModule,
        InputTextModule,
        TranslocoModule,
        ImageModule,
        AvatarModule,
        ProgressSpinnerModule,
    ],
    templateUrl: './service-edit.component.html',
    styleUrl: './service-edit.component.scss'
})
export class ServiceEditComponent {
    private fb = inject(FormBuilder);

    visible = model(false);
    @Input() service?: Service;
    @Output() visibleChange = new EventEmitter<boolean>();
    @Output() serviceEdited = new EventEmitter<{id: string, name: string, issuer: string}>();

    loading = signal(false);

    editForm = this.fb.group({
        name: ['', Validators.required],
        issuer: ['', Validators.required],
        icon: ['', Validators.required],
    });

    ngOnChanges() {
        if (this.service) {
            this.editForm.patchValue({
                name: this.service.name,
                issuer: this.service.issuer,
                icon: this.service.icon,
            });
        }
    }

    onSubmit() {
        if (this.editForm.valid && this.service) {
            this.serviceEdited.emit({
                id: this.service.id,
                ...this.editForm.value
            } as {id: string, name: string, issuer: string});
            this.visible.set(false);
            this.visibleChange.emit(false);
        }
    }

    onHide() {
        this.visible.update(old => false);
        this.visibleChange.emit(false);
        console.log('treta cabulosa')
    }

    clearIcon() {
        this.service ? this.service.icon = "" : null;
    }

    onImageError() {
        if (this.service) {
            this.service.icon = "";
        }
    }

    updateServiceIcon() {
        this.loading.set(true);
        if (this.service) {
            invoke<string>('get_service_icon', {serviceId: this.service.id})
                .then((icon: string) => {
                    this.service!.icon = icon;
                    this.loading.set(false);
                })
                .catch(() => {
                    this.loading.set(false);
                });
        }
    }
}
