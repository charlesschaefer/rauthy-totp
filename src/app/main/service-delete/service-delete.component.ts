import { Component, EventEmitter, Input, model, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DialogModule } from 'primeng/dialog';
import { ButtonModule } from 'primeng/button';
import { TranslocoModule } from '@jsverse/transloco';

@Component({
    selector: 'app-service-delete',
    standalone: true,
    imports: [
        CommonModule,
        DialogModule,
        ButtonModule,
        TranslocoModule
    ],
    templateUrl: './service-delete.component.html',
    styleUrls: []
})
export class ServiceDeleteComponent {
    visible = model(false);
    @Input() serviceName = '';
    @Output() confirmDelete = new EventEmitter<void>();
    @Output() cancelDelete = new EventEmitter<void>();

    onConfirm() {
        this.visible.set(false);
        this.confirmDelete.emit();
    }

    onCancel() {
        this.visible.set(false);
        this.cancelDelete.emit();
    }
}
