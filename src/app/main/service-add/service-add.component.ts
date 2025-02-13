import { Component, EventEmitter, Input, Output, inject, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { DialogModule } from 'primeng/dialog';
import { ButtonModule } from 'primeng/button';
import { CardModule } from 'primeng/card';
import { InputTextModule } from 'primeng/inputtext';
import { TranslocoModule } from '@jsverse/transloco';

@Component({
    selector: 'app-service-add',
    standalone: true,
    imports: [
        CommonModule,
        ReactiveFormsModule,
        DialogModule,
        ButtonModule,
        CardModule,
        InputTextModule,
        TranslocoModule
    ],
    templateUrl: './service-add.component.html',
    styleUrl: './service-add.component.scss'
})
export class ServiceAddComponent {
    private fb = inject(FormBuilder);

    @Input() visible = false;
    @Output() visibleChange = new EventEmitter<boolean>();
    @Output() onScanQRCode = new EventEmitter<void>();
    @Output() serviceUrlAdded = new EventEmitter<string>();

    showURLInput = signal(false);

    urlInput = this.fb.group({
        serviceUrl: ['', Validators.required],
    });

    scanQRCode() {
        this.onScanQRCode.emit();
    }

    onSubmitServiceUrl() {
        if (this.urlInput.valid) {
            this.serviceUrlAdded.emit(this.urlInput.value.serviceUrl as string);
            this.urlInput.reset();
            this.showURLInput.set(false);
        }
    }
}
