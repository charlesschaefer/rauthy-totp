import { ApplicationConfig, importProvidersFrom, isDevMode } from "@angular/core";
import { provideRouter } from "@angular/router";
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { provideHttpClient } from '@angular/common/http';
import { provideTransloco } from '@jsverse/transloco';
import { providePrimeNG } from "primeng/config";
import { appTheme } from "./app.theme";
import 'hammerjs';

import { routes } from "./app.routes";
import { TranslocoHttpLoader } from './transloco-loader';
import { provideSwipeMenu } from "ngx-swipe-menu";
import { BrowserModule } from "@angular/platform-browser";
import { HammerModule } from "@angular/platform-browser";

export const appConfig: ApplicationConfig = {
    providers: [
        provideRouter(routes), 
        provideHttpClient(), 
        provideTransloco({
            config: {
                availableLangs: ['en', 'pt-BR'],
                defaultLang: 'en',
                // Remove this option if your application doesn't support changing language in runtime.
                reRenderOnLangChange: true,
                prodMode: !isDevMode(),
            },
            loader: TranslocoHttpLoader
        }),
        importProvidersFrom(BrowserModule),
        importProvidersFrom(HammerModule),
        provideAnimationsAsync(),
        providePrimeNG({
            theme: {
                preset: appTheme,
                options: {
                    darkModeSelector: '.dark-mode'
                }
            },
            ripple: true,
        }),
        provideSwipeMenu()
    ],
};
