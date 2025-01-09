import { Routes } from '@angular/router';
import { MainComponent } from './main/main.component';

export const routes: Routes = [
    { path: '', component: MainComponent },
    { path: "license", loadComponent: () => import('./license/license.component').then(comp => comp.LicenseComponent) },
];
