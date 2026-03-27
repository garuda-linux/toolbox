import { HTTP_INTERCEPTORS, provideHttpClient, withFetch } from '@angular/common/http';
import {
  type ApplicationConfig,
  isDevMode,
  LOCALE_ID,
  provideAppInitializer,
  provideZonelessChangeDetection,
} from '@angular/core';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { provideGarudaNG } from '@garudalinux/core';
import { TranslocoHttpLoader } from './transloco-loader';
import { provideTransloco } from '@jsverse/transloco';
import { ConfirmationService } from 'primeng/api';
import { LoadingInterceptor } from './components/loading-indicator/loading-indicator.interceptor';
import { ConfigService } from './components/config/config.service';
import { LanguageManagerService } from './components/language-manager/language-manager.service';
import { initToolbox } from './app.init';
import { ThemeService } from './components/theme-service/theme-service';
import { PackagesService } from './components/packages/packages.service';
import { NotificationService } from './components/notification/notification.service';
import { TaskManagerService } from './components/task-manager/task-manager.service';
import { OsInteractService } from './components/task-manager/os-interact.service';
import { SystemStatusService } from './components/system-status/system-status.service';
import { KernelsService } from './components/kernels/kernels.service';
import { LanguagePacksService } from './components/language-packs/language-packs.service';
import { LocalesService } from './components/locales/locales.service';
import { GamingService } from './components/gaming/gaming.service';
import { LoadingService } from './components/loading-indicator/loading-indicator.service';
import { SplashService } from './components/splash/splash.service';
import { provideRouter, withHashLocation } from '@angular/router';
import { routes } from './routes';
import { CatppuccinAura } from '@garudalinux/themes/catppuccin';

export const appConfig: ApplicationConfig = {
  providers: [
    ConfigService,
    ConfirmationService,
    GamingService,
    KernelsService,
    LanguageManagerService,
    LanguagePacksService,
    LoadingService,
    LocalesService,
    NotificationService,
    OsInteractService,
    PackagesService,
    SplashService,
    SystemStatusService,
    TaskManagerService,
    ThemeService,
    provideAnimationsAsync(),
    provideAppInitializer(initToolbox),
    provideGarudaNG(
      { font: 'InterVariable' },
      {
        theme: {
          preset: CatppuccinAura,
          options: {
            darkModeSelector: '.p-dark',
          },
        },
        ripple: true,
        inputStyle: 'outlined',
      },
    ),
    provideHttpClient(withFetch()),
    provideRouter(routes, withHashLocation()),
    provideTransloco({
      config: {
        availableLangs: [
          'am',
          'ar',
          'de',
          'en',
          'es',
          'eu',
          'et',
          'fr',
          'gl',
          'hi',
          'hu',
          'id',
          'it',
          'ja',
          'ko',
          'pl',
          'pt',
          'pt-BR',
          'ro',
          'ru',
          'sl',
          'sv',
          'sw',
          'tr',
          'uk',
          'uz',
          'zh-CN',
        ],
        defaultLang: 'en',
        fallbackLang: 'en',
        missingHandler: {
          useFallbackTranslation: true,
        },
        prodMode: !isDevMode(),
        reRenderOnLangChange: true,
      },
      loader: TranslocoHttpLoader,
    }),
    provideZonelessChangeDetection(),
    { provide: LOCALE_ID, useValue: 'en-GB' },
    { provide: HTTP_INTERCEPTORS, useValue: [LoadingInterceptor], multi: true },
  ],
};
