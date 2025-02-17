import { Component, EventEmitter, Input, Output, computed, inject, input, model, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { DialogModule } from 'primeng/dialog';
import { ButtonModule } from 'primeng/button';
import { InputTextModule } from 'primeng/inputtext';
import { TranslocoModule } from '@jsverse/transloco';

import { Service } from '../../models/service.model';

@Component({
    selector: 'app-service-edit',
    standalone: true,
    imports: [
        CommonModule,
        ReactiveFormsModule,
        DialogModule,
        ButtonModule,
        InputTextModule,
        TranslocoModule
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

    editForm = this.fb.group({
        name: ['', Validators.required],
        issuer: ['', Validators.required]
    });

    ngOnChanges() {
        if (this.service) {
            this.editForm.patchValue({
                name: this.service.name,
                issuer: this.service.issuer
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
}
